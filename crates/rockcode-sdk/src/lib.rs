use rockcode_protocol::{CapabilitiesResp, ErrorResp, HealthResp, ReadyResp};

pub struct Client {
    base: String,
}

impl Client {
    pub fn new(base: impl Into<String>) -> Self {
        Self { base: base.into() }
    }

    pub async fn health(&self) -> anyhow::Result<HealthResp> {
        let url = format!("{}/health/live", self.base);
        let resp = reqwest::get(&url).await?;
        Ok(resp.json().await?)
    }

    pub async fn ready(&self) -> anyhow::Result<ReadyResp> {
        let url = format!("{}/health/ready", self.base);
        let resp = reqwest::get(&url).await?;
        Ok(resp.json().await?)
    }

    pub async fn capabilities(&self) -> anyhow::Result<CapabilitiesResp> {
        let url = format!("{}/capabilities", self.base);
        let resp = reqwest::get(&url).await?;
        if resp.status().is_client_error() {
            let err: ErrorResp = resp.json().await?;
            anyhow::bail!(err.error.code);
        }
        Ok(resp.json().await?)
    }
}
