use std::net::SocketAddr;

use alloy_sol_types::SolType;
use axum::{
    http::StatusCode,
    response::Json,
    routing::{get, post},
    Router,
};
use fibonacci_lib::PublicValuesStruct;
use serde::{Deserialize, Serialize};
use sp1_sdk::{include_elf, ProverClient, SP1Stdin};
use tower::ServiceBuilder;
use tower_http::cors::{Any, CorsLayer};
use tracing::{info, warn};

/// The ELF (executable and linkable format) file for the Succinct RISC-V zkVM.
pub const FIBONACCI_ELF: &[u8] = include_elf!("fibonacci-program");

/// Request structure for proof generation
#[derive(Deserialize, Debug)]
pub struct ProofRequest {
    /// Transaction hash
    pub tx_hash: String,
    /// Raw transaction hex string
    pub tx: String,
    /// Merkle siblings (array of hex strings)
    pub merkle_siblings: Vec<String>,
    /// Position in the merkle tree
    pub position: u32,
    /// Merkle root (hex string)
    pub merkle_root: String,
}

/// Response structure for proof generation
#[derive(Serialize, Debug)]
pub struct ProofResponse {
    /// Success status
    pub success: bool,
    /// Error message if any
    pub error: Option<String>,
    /// Public values as hex string
    pub public_values: Option<String>,
    /// Proof as hex string
    pub proof: Option<String>,
    /// Execution time in milliseconds
    pub execution_time_ms: Option<u64>,
}

/// Health check response
#[derive(Serialize)]
pub struct HealthResponse {
    pub status: String,
    pub version: String,
}

/// Convert hex string to reversed 32-byte array
fn hex_rev32(s: &str) -> Result<[u8; 32], String> {
    let bytes = hex::decode(s).map_err(|e| format!("Invalid hex: {}", e))?;
    if bytes.len() != 32 {
        return Err("Hex string must be 64 characters (32 bytes)".to_string());
    }
    let mut result: [u8; 32] = bytes.try_into().unwrap();
    result.reverse(); // flip from RPC little-endian to internal big-endian
    Ok(result)
}

/// Generate proof for Bitcoin transaction
async fn generate_proof(
    Json(request): Json<ProofRequest>,
) -> Result<Json<ProofResponse>, StatusCode> {
    let start_time = std::time::Instant::now();

    info!("Received proof request for tx_hash: {}", request.tx_hash);

    // Validate and convert merkle siblings
    let merkle_siblings: Result<Vec<[u8; 32]>, String> = request
        .merkle_siblings
        .into_iter()
        .map(|s| hex_rev32(&s))
        .collect();

    let merkle_siblings = match merkle_siblings {
        Ok(siblings) => siblings,
        Err(e) => {
            warn!("Invalid merkle siblings: {}", e);
            return Ok(Json(ProofResponse {
                success: false,
                error: Some(format!("Invalid merkle siblings: {}", e)),
                public_values: None,
                proof: None,
                execution_time_ms: None,
            }));
        }
    };

    // Convert merkle root
    let merkle_root = match hex_rev32(&request.merkle_root) {
        Ok(root) => root,
        Err(e) => {
            warn!("Invalid merkle root: {}", e);
            return Ok(Json(ProofResponse {
                success: false,
                error: Some(format!("Invalid merkle root: {}", e)),
                public_values: None,
                proof: None,
                execution_time_ms: None,
            }));
        }
    };

    // Setup input
    let mut stdin = SP1Stdin::new();
    stdin.write(&request.tx_hash);
    stdin.write(&request.tx);
    stdin.write(&merkle_siblings);
    stdin.write(&request.position);
    stdin.write(&merkle_root);

    // Generate proof
    match generate_proof_internal(&stdin).await {
        Ok((public_values, proof)) => {
            let execution_time = start_time.elapsed().as_millis() as u64;

            // Decode and check validation results from the program
            match PublicValuesStruct::abi_decode(&public_values) {
                Ok(validation_result) => {
                    if validation_result.valid {
                        info!("Proof generated successfully in {}ms", execution_time);
                        Ok(Json(ProofResponse {
                            success: true,
                            error: None,
                            public_values: Some(hex::encode(public_values)),
                            proof: Some(hex::encode(proof)),
                            execution_time_ms: Some(execution_time),
                        }))
                    } else {
                        warn!(
                            "Proof generated but validation failed in {}ms",
                            execution_time
                        );
                        Ok(Json(ProofResponse {
                            success: false,
                            error: Some(
                                "Transaction validation failed: invalid hash or merkle proof"
                                    .to_string(),
                            ),
                            public_values: Some(hex::encode(public_values)),
                            proof: Some(hex::encode(proof)),
                            execution_time_ms: Some(execution_time),
                        }))
                    }
                }
                Err(e) => {
                    warn!("Failed to decode validation results: {}", e);
                    Ok(Json(ProofResponse {
                        success: false,
                        error: Some(format!("Failed to decode validation results: {}", e)),
                        public_values: Some(hex::encode(public_values)),
                        proof: Some(hex::encode(proof)),
                        execution_time_ms: Some(execution_time),
                    }))
                }
            }
        }
        Err(e) => {
            let execution_time = start_time.elapsed().as_millis() as u64;
            warn!("Proof generation failed: {}", e);

            Ok(Json(ProofResponse {
                success: false,
                error: Some(e.to_string()),
                public_values: None,
                proof: None,
                execution_time_ms: Some(execution_time),
            }))
        }
    }
}

/// Internal proof generation logic
async fn generate_proof_internal(stdin: &SP1Stdin) -> Result<(Vec<u8>, Vec<u8>), anyhow::Error> {
    // Setup the prover client
    let client = ProverClient::from_env();

    // Setup the program for proving
    let (pk, vk) = client.setup(FIBONACCI_ELF);

    // Generate the proof
    let proof = client
        .prove(&pk, stdin)
        .compressed()
        .run()
        .map_err(|e| anyhow::anyhow!("Failed to generate proof: {}", e))?;

    // Get the public values as bytes
    let public_values = proof.public_values.as_slice().to_vec();

    // // Get the proof as bytes
    // let solidity_proof = proof.bytes();

    // Verify the proof
    client
        .verify(&proof, &vk)
        .map_err(|e| anyhow::anyhow!("Failed to verify proof: {}", e))?;

    Ok((public_values, Vec::new()))
}

/// Health check endpoint
async fn health_check() -> Json<HealthResponse> {
    Json(HealthResponse {
        status: "healthy".to_string(),
        version: env!("CARGO_PKG_VERSION").to_string(),
    })
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    dotenv::dotenv().ok();

    tracing_subscriber::fmt().pretty().init();

    // Build the router
    let app = Router::new()
        .route("/health", get(health_check))
        .route("/prove", post(generate_proof))
        .layer(
            ServiceBuilder::new().layer(
                CorsLayer::new()
                    .allow_origin(Any)
                    .allow_methods(Any)
                    .allow_headers(Any),
            ),
        );

    let addr = SocketAddr::from(([0, 0, 0, 0], 4455));
    dbg!("Server running on http://0.0.0.0:4455");
    dbg!("Available endpoints:");
    dbg!("  GET  /health   - Health check");
    dbg!("  POST /prove    - Generate proof for Bitcoin transaction");

    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();

    Ok(())
}
