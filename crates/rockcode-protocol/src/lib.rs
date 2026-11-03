use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthResp {
    pub status: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReadyResp {
    pub status: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CapabilitiesResp {
    pub capabilities: Vec<String>,
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
}
