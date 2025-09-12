use axum::{http::StatusCode, response::Json};

use serde::{Deserialize, Serialize};
use sp1_sdk::{include_elf, ProverClient, SP1Stdin};
use tracing::warn;

use crate::TARGET_ADDRESS;

/// The ELF (executable and linkable format) file for the Succinct RISC-V zkVM.
pub const BITCOIN_PROOF_ELF: &[u8] = include_elf!("fibonacci-program");

/// Request structure for Bitcoin transaction proof generation
#[derive(Deserialize, Debug)]
pub struct ProofRequest {
    /// Raw Bitcoin transaction hex string
    pub tx: String,
    /// Expected Bitcoin transaction ID (hex string)
    pub tx_hash: String,
    /// Merkle siblings (array of hex strings)
    pub merkle: Vec<String>,
    /// Position in the merkle tree
    pub position: usize,
    /// Block header (hex string)
    pub block_header: String,
}

/// Response structure for proof generation
#[derive(Serialize, Debug)]
pub struct ProofResponse {
    /// Success status
    pub success: bool,
    /// Error message if any
    pub error: Option<String>,
    /// Total amount sent to target address
    pub total_amount: Option<u64>,
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

/// Generate proof for Bitcoin transaction verification
pub async fn generate_bitcoin_proof(
    Json(request): Json<ProofRequest>,
) -> Result<Json<ProofResponse>, StatusCode> {
    let start_time = std::time::Instant::now();

    // Setup input for the zkVM
    let mut stdin = SP1Stdin::new();
    stdin.write(&request.tx);
    stdin.write(&request.tx_hash);
    stdin.write(&request.merkle);
    stdin.write(&request.position);
    stdin.write(&request.block_header);
    stdin.write(&String::from(TARGET_ADDRESS));

    // Generate proof using the zkVM
    match generate_proof_internal(&stdin).await {
        Ok((_, total_amount)) => {
            let execution_time = start_time.elapsed().as_millis() as u64;

            Ok(Json(ProofResponse {
                success: true,
                error: None,
                total_amount: Some(total_amount),
                execution_time_ms: Some(execution_time),
            }))
        }
        Err(e) => {
            let execution_time = start_time.elapsed().as_millis() as u64;
            warn!("Proof generation failed: {}", e);

            Ok(Json(ProofResponse {
                success: false,
                error: Some(ProofError::ProofGenerationFailed(e.to_string()).to_string()),
                total_amount: None,
                execution_time_ms: Some(execution_time),
            }))
        }
    }
}

/// Internal proof generation logic using SP1 zkVM
async fn generate_proof_internal(stdin: &SP1Stdin) -> Result<(String, u64), anyhow::Error> {
    // Initialize the SP1 prover client
    let client = ProverClient::from_env();

    // Setup the program for proving (generate proving key and verification key)
    let (proving_key, verification_key) = client.setup(BITCOIN_PROOF_ELF);

    // Generate the zero-knowledge proof
    let proof = client
        .prove(&proving_key, stdin)
        .run()
        .map_err(|e| anyhow::anyhow!("Failed to generate proof: {}", e))?;

    let public_values = proof.public_values.as_slice();

    // Decode the public values
    // Format: [8-byte length][block_hash string][8-byte total_amount]
    if public_values.len() < 8 {
        return Err(anyhow::anyhow!("Invalid public values: too short"));
    }

    // Read the length of the block_hash string (first 8 bytes as u64)
    let block_hash_len = u64::from_le_bytes([
        public_values[0],
        public_values[1],
        public_values[2],
        public_values[3],
        public_values[4],
        public_values[5],
        public_values[6],
        public_values[7],
    ]) as usize;

    if public_values.len() < 8 + block_hash_len + 8 {
        return Err(anyhow::anyhow!("Invalid public values: insufficient data"));
    }

    // Extract the block_hash string
    let block_hash_bytes = &public_values[8..8 + block_hash_len];
    let block_hash = String::from_utf8(block_hash_bytes.to_vec())
        .map_err(|e| anyhow::anyhow!("Invalid UTF-8 in block_hash: {}", e))?;

    // Extract the total_amount (last 8 bytes as u64)
    let amount_start = 8 + block_hash_len;
    let total_amount = u64::from_le_bytes([
        public_values[amount_start],
        public_values[amount_start + 1],
        public_values[amount_start + 2],
        public_values[amount_start + 3],
        public_values[amount_start + 4],
        public_values[amount_start + 5],
        public_values[amount_start + 6],
        public_values[amount_start + 7],
    ]);

    // Verify the generated proof locally
    client
        .verify(&proof, &verification_key)
        .map_err(|e| anyhow::anyhow!("Failed to verify proof: {}", e))?;

    Ok((block_hash, total_amount))
}
