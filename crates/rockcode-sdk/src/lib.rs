use serde::de::DeserializeOwned;

use rockcode_protocol::{CapabilitiesResp, ErrorResp, HealthResp, ReadyResp};

pub struct Client {
    base: String,
    token: Option<String>,
    http: reqwest::Client,
}

impl Client {
    pub fn new(base: impl Into<String>) -> Self {
        Self {
            base: base.into().trim_end_matches('/').to_string(),
            token: None,
            http: reqwest::Client::new(),
        }
    }

    pub fn with_token(base: impl Into<String>, token: impl Into<String>) -> Self {
        Self {
            base: base.into().trim_end_matches('/').to_string(),
            token: Some(token.into()),
            http: reqwest::Client::new(),
        }
    }

    async fn get<T: DeserializeOwned>(&self, path: &str) -> anyhow::Result<T> {
        let mut request = self.http.get(format!("{}{path}", self.base));
        if let Some(token) = &self.token {
            request = request.bearer_auth(token);
        }

        let response = request.send().await?;
        let status = response.status();
        if !status.is_success() {
            let error = response.json::<ErrorResp>().await;
            if let Ok(error) = error {
                anyhow::bail!(
                    "server returned {status}: {} — {}",
                    error.error.code,
                    error.error.message
                );
            }
            anyhow::bail!("server returned {status}");
        }

        Ok(response.json().await?)
    }

    pub async fn health(&self) -> anyhow::Result<HealthResp> {
        self.get("/health/live").await
    }

    pub async fn ready(&self) -> anyhow::Result<ReadyResp> {
        self.get("/health/ready").await
    }

    pub async fn capabilities(&self) -> anyhow::Result<CapabilitiesResp> {
        self.get("/capabilities").await
    }
}
