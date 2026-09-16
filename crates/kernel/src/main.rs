use ironcage_api::ApiServer;
use ironcage_kernel::ResearchMCTS;
use std::sync::Arc;
use tokio::sync::Mutex;
use tracing::info;

#[tokio::main]
async fn main() {
    // Initialize tracing
    tracing_subscriber::fmt::init();

    info!("Starting Ironcage Research Kernel...");

    // Initialize MCTS engine
    let _mcts = Arc::new(Mutex::new(ResearchMCTS::new(
        "Prove the Riemann Hypothesis".to_string(),
    )));
    info!("MCTS engine initialized");

    // Initialize API server with WebSocket support
    let api = ApiServer::new(1000);
    info!("API server initialized on :3000");

    // Start API server
    let listener = tokio::net::TcpListener::bind("127.0.0.1:3000")
        .await
        .expect("Failed to bind to 127.0.0.1:3000");

    let app = api.router();

    info!("Research kernel listening on http://127.0.0.1:3000");
    info!("WebSocket endpoint: ws://127.0.0.1:3000/ws");

    axum::serve(listener, app)
        .await
        .expect("Server failed");
}
