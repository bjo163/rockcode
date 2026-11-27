use std::{env, net::SocketAddr};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt::init();

    let token = env::var("ROCKCODE_API_TOKEN")
        .map_err(|_| "ROCKCODE_API_TOKEN is required (use at least 32 characters)")?;
    if token.len() < 32 {
        return Err("ROCKCODE_API_TOKEN must contain at least 32 characters".into());
    }

    let bind_address =
        env::var("ROCKCODE_BIND_ADDR").unwrap_or_else(|_| "127.0.0.1:43170".to_string());
    let address: SocketAddr = bind_address.parse()?;
    if !address.ip().is_loopback() {
        return Err("agentd only supports loopback binding until TLS is implemented".into());
    }

    let listener = tokio::net::TcpListener::bind(address).await?;
    let local_address = listener.local_addr()?;
    tracing::info!(address = %local_address, "RockCode agent server started");

    let app = rockcode_server::app_with_token(token);
    axum::serve(listener, app).await?;

    Ok(())
}
