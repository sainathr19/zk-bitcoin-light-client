use alloy_sol_types::SolType;
use axum::{http::StatusCode, response::Json};
use fibonacci_lib::PublicValuesStruct;
use serde::{Deserialize, Serialize};
use sp1_sdk::{include_elf, ProverClient, SP1Stdin};
use tracing::{info, warn};

/// The ELF (executable and linkable format) file for the Succinct RISC-V zkVM.
pub const BITCOIN_PROOF_ELF: &[u8] = include_elf!("fibonacci-program");

/// Request structure for Bitcoin transaction proof generation
#[derive(Deserialize, Debug)]
pub struct ProofRequest {
    /// Bitcoin transaction hash (hex string)
    pub tx_hash: String,
    /// Raw Bitcoin transaction hex string
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

/// Error types for better error handling
#[derive(Debug)]
pub enum ProofError {
    InvalidHex(String),
    InvalidMerkleSiblings(String),
    InvalidMerkleRoot(String),
    ProofGenerationFailed(String),
    ValidationFailed(String),
    DecodeError(String),
}

impl std::fmt::Display for ProofError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ProofError::InvalidHex(msg) => write!(f, "Invalid hex: {}", msg),
            ProofError::InvalidMerkleSiblings(msg) => write!(f, "Invalid merkle siblings: {}", msg),
            ProofError::InvalidMerkleRoot(msg) => write!(f, "Invalid merkle root: {}", msg),
            ProofError::ProofGenerationFailed(msg) => write!(f, "Proof generation failed: {}", msg),
            ProofError::ValidationFailed(msg) => write!(f, "Validation failed: {}", msg),
            ProofError::DecodeError(msg) => write!(f, "Decode error: {}", msg),
        }
    }
}

/// Health check endpoint for monitoring service status
pub async fn health_check() -> Json<HealthResponse> {
    Json(HealthResponse {
        status: "healthy".to_string(),
        version: env!("CARGO_PKG_VERSION").to_string(),
    })
}

/// Convert hex string to reversed 32-byte array for Bitcoin compatibility
///
/// Bitcoin uses little-endian format for display but big-endian internally
/// This function converts from RPC format to internal format
fn hex_to_reversed_bytes(s: &str) -> Result<[u8; 32], ProofError> {
    let bytes =
        hex::decode(s).map_err(|e| ProofError::InvalidHex(format!("Invalid hex string: {}", e)))?;

    if bytes.len() != 32 {
        return Err(ProofError::InvalidHex(
            "Hex string must be exactly 64 characters (32 bytes)".to_string(),
        ));
    }

    let mut result: [u8; 32] = bytes
        .try_into()
        .map_err(|_| ProofError::InvalidHex("Failed to convert to 32-byte array".to_string()))?;

    // Convert from RPC little-endian to internal big-endian
    result.reverse();
    Ok(result)
}

/// Validate and convert merkle siblings from hex strings to byte arrays
fn validate_merkle_siblings(siblings: Vec<String>) -> Result<Vec<[u8; 32]>, ProofError> {
    siblings
        .into_iter()
        .enumerate()
        .map(|(i, s)| {
            hex_to_reversed_bytes(&s)
                .map_err(|e| ProofError::InvalidMerkleSiblings(format!("Sibling {}: {}", i, e)))
        })
        .collect()
}

/// Generate proof for Bitcoin transaction verification
pub async fn generate_bitcoin_proof(
    Json(request): Json<ProofRequest>,
) -> Result<Json<ProofResponse>, StatusCode> {
    let start_time = std::time::Instant::now();

    // Validate and convert merkle siblings
    let merkle_siblings = match validate_merkle_siblings(request.merkle_siblings) {
        Ok(siblings) => siblings,
        Err(e) => {
            warn!("Merkle siblings validation failed: {}", e);
            return Ok(Json(ProofResponse {
                success: false,
                error: Some(e.to_string()),
                public_values: None,
                proof: None,
                execution_time_ms: Some(start_time.elapsed().as_millis() as u64),
            }));
        }
    };

    // Validate and convert merkle root
    let merkle_root = match hex_to_reversed_bytes(&request.merkle_root) {
        Ok(root) => root,
        Err(e) => {
            warn!("Merkle root validation failed: {}", e);
            return Ok(Json(ProofResponse {
                success: false,
                error: Some(e.to_string()),
                public_values: None,
                proof: None,
                execution_time_ms: Some(start_time.elapsed().as_millis() as u64),
            }));
        }
    };

    // Setup input for the zkVM
    let mut stdin = SP1Stdin::new();
    stdin.write(&request.tx_hash);
    stdin.write(&request.tx);
    stdin.write(&merkle_siblings);
    stdin.write(&request.position);
    stdin.write(&merkle_root);

    // Generate proof using the zkVM
    match generate_proof_internal(&stdin).await {
        Ok((public_values, proof)) => {
            let execution_time = start_time.elapsed().as_millis() as u64;

            // Decode and validate the proof results
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
                                ProofError::ValidationFailed(
                                    "Validation failed: invalid hash or merkle proof".to_string(),
                                )
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
                        error: Some(ProofError::DecodeError(e.to_string()).to_string()),
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
                error: Some(ProofError::ProofGenerationFailed(e.to_string()).to_string()),
                public_values: None,
                proof: None,
                execution_time_ms: Some(execution_time),
            }))
        }
    }
}

/// Internal proof generation logic using SP1 zkVM
async fn generate_proof_internal(stdin: &SP1Stdin) -> Result<(Vec<u8>, Vec<u8>), anyhow::Error> {
    // Initialize the SP1 prover client
    let client = ProverClient::from_env();

    // Setup the program for proving (generate proving key and verification key)
    let (proving_key, verification_key) = client.setup(BITCOIN_PROOF_ELF);

    // Generate the zero-knowledge proof
    let proof = client
        .prove(&proving_key, stdin)
        .run()
        .map_err(|e| anyhow::anyhow!("Failed to generate proof: {}", e))?;

    // Extract public values from the proof
    let public_values = proof.public_values.as_slice().to_vec();

    // Verify the generated proof locally
    client
        .verify(&proof, &verification_key)
        .map_err(|e| anyhow::anyhow!("Failed to verify proof: {}", e))?;

    // Return public values and empty proof bytes (proof verification is done above)
    Ok((public_values, Vec::new()))
}
