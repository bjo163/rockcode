use std::env;

use rockcode_sdk::Client;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let base =
        env::var("ROCKCODE_URL").unwrap_or_else(|_| "http://127.0.0.1:43170".to_string());
    let token = env::var("ROCKCODE_API_TOKEN").unwrap_or_default();

    let client = if token.is_empty() {
        Client::new(base)
    } else {
        Client::with_token(base, token)
    };

    let health = client.health().await?;
    let readiness = client.ready().await?;
    let capabilities = client.capabilities().await?;

    println!("RockCode server: {}", health.status);
    println!("Version: {}", health.version);
    println!("Readiness: {}", readiness.status);
    for check in readiness.checks {
        println!("  {}: {}", check.name, check.status);
    }
    println!("Protocol: {}", capabilities.protocol_versions.join(", "));
    println!("Features: {}", capabilities.features.join(", "));

    Ok(())
}
