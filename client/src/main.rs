use std::net::SocketAddr;

use axum::{
    routing::{get, post},
    Router,
};
use sp1_sdk::include_elf;
use tower::ServiceBuilder;
use tower_http::cors::{Any, CorsLayer};
use tracing::info;

use crate::server::handlers::{generate_bitcoin_proof, health_check};

pub mod server;

/// The ELF (executable and linkable format) file for the Succinct RISC-V zkVM.
pub const BITCOIN_PROOF_ELF: &[u8] = include_elf!("fibonacci-program");

/// Main server entry point
#[tokio::main]
async fn main() {
    // Load environment variables
    dotenv::dotenv().ok();

    // Initialize structured logging
    tracing_subscriber::fmt()
        .with_target(false)
        .with_thread_ids(true)
        .with_thread_names(true)
        .pretty()
        .init();

    // Build the HTTP router with CORS support
    let app = Router::new()
        .route("/health", get(health_check))
        .route("/prove", post(generate_bitcoin_proof))
        .layer(
            ServiceBuilder::new().layer(
                CorsLayer::new()
                    .allow_origin(Any)
                    .allow_methods(Any)
                    .allow_headers(Any),
            ),
        );

    // Configure server address
    let addr = SocketAddr::from(([0, 0, 0, 0], 4455));

    // Log server startup information
    info!("Server starting on http://0.0.0.0:4455");
    // Start the server
    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}
