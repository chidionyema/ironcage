use ironcage_api::ApiServer;

#[tokio::main]
async fn main() {
    let api = ApiServer::new(1000);
    let app = api.router();

    let listener = tokio::net::TcpListener::bind("127.0.0.1:3001")
        .await
        .expect("Failed to bind");

    println!("API server running on http://127.0.0.1:3001");
    axum::serve(listener, app).await.expect("Server error");
}
