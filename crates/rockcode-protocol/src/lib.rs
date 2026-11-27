use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthResp {
    pub status: String,
    pub version: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReadinessCheck {
    pub name: String,
    pub status: String,
    pub detail: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReadyResp {
    pub status: String,
    pub checks: Vec<ReadinessCheck>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutorCapability {
    pub kind: String,
    pub available: bool,
    pub limitations: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CapabilitiesResp {
    pub server_version: String,
    pub minimum_client_version: String,
    pub protocol_versions: Vec<String>,
    pub event_schema_versions: Vec<u32>,
    pub features: Vec<String>,
    pub executors: Vec<ExecutorCapability>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ErrorResp {
    pub error: ErrorBody,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ErrorBody {
    pub code: String,
    pub message: String,
    pub retryable: bool,
    pub request_id: String,
}
