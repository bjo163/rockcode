#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();
    let app = rockcode_server::app();
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    tracing::info!("listening on {}", addr);
    axum::serve(listener, app).await.unwrap();
}
