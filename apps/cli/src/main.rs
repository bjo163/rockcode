#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let base = std::env::var("AGENTD_URL").unwrap_or_else(|_| "http://127.0.0.1:8080".to_string());
    let client = rockcode_sdk::Client::new(base);
    if let Ok(h) = client.health().await {
        println!("{}", h.status);
    }
    if let Ok(c) = client.capabilities().await {
        println!("{:?}", c.capabilities);
    }
    Ok(())
}
