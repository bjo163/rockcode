use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum KernelError {
    #[error("gateway error")]
    Gateway,
    #[error("tool error")]
    Tool,
    #[error("cancelled")]
    Cancelled,
    #[error("agent turn limit must be between 1 and 16")]
    InvalidTurnLimit,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum MessageRole {
    System,
    User,
    Assistant,
    Tool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ToolCall {
    pub id: String,
    pub name: String,
    pub arguments: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ToolDefinition {
    pub name: String,
    pub description: String,
    pub parameters: Value,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ChatMessage {
    pub role: MessageRole,
    pub content: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tool_call_id: Option<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub tool_calls: Vec<ToolCall>,
}

impl ChatMessage {
    pub fn new(role: MessageRole, content: impl Into<String>) -> Self {
        Self {
            role,
            content: content.into(),
            name: None,
            tool_call_id: None,
            tool_calls: Vec::new(),
        }
    }

    pub fn system(content: impl Into<String>) -> Self {
        Self::new(MessageRole::System, content)
    }

    pub fn user(content: impl Into<String>) -> Self {
        Self::new(MessageRole::User, content)
    }

    pub fn assistant(content: impl Into<String>) -> Self {
        Self::new(MessageRole::Assistant, content)
    }

    pub fn assistant_tool_calls(content: impl Into<String>, tool_calls: Vec<ToolCall>) -> Self {
        let mut message = Self::assistant(content);
        message.tool_calls = tool_calls;
        message
    }

    pub fn tool_result(
        tool_call_id: impl Into<String>,
        name: impl Into<String>,
        content: impl Into<String>,
    ) -> Self {
        let mut message = Self::new(MessageRole::Tool, content);
        message.tool_call_id = Some(tool_call_id.into());
        message.name = Some(name.into());
        message
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CompletionRequest {
    pub messages: Vec<ChatMessage>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub tools: Vec<ToolDefinition>,
    pub temperature: Option<f32>,
    pub max_output_tokens: Option<u32>,
}

impl CompletionRequest {
    pub fn new(messages: Vec<ChatMessage>) -> Self {
        Self {
            messages,
            tools: Vec::new(),
            temperature: None,
            max_output_tokens: None,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CompletionResponse {
    pub content: String,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub tool_calls: Vec<ToolCall>,
    pub finish_reason: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ToolDecision {
    Allow,
    ApprovalRequired,
    Deny,
}

#[async_trait]
pub trait ModelGateway: Send + Sync {
    async fn complete(&self, request: CompletionRequest)
        -> Result<CompletionResponse, KernelError>;
}

#[async_trait]
pub trait ToolInvoker: Send + Sync {
    async fn invoke(&self, name: &str, args: &str) -> Result<String, KernelError>;
}

#[async_trait]
pub trait ToolPolicy: Send + Sync {
    async fn authorize(&self, call: &ToolCall) -> Result<ToolDecision, KernelError>;
}

#[async_trait]
pub trait EventSink: Send + Sync {
    async fn emit(&self, event: &str) -> Result<(), KernelError>;
}

pub trait Clock: Send + Sync {
    fn now_millis(&self) -> u64;
}

#[derive(Debug, Clone)]
pub struct CancellationToken {
    cancelled: std::sync::Arc<std::sync::atomic::AtomicBool>,
}

impl CancellationToken {
    pub fn new() -> Self {
        Self {
            cancelled: std::sync::Arc::new(std::sync::atomic::AtomicBool::new(false)),
        }
    }

    pub fn cancel(&self) {
        self.cancelled
            .store(true, std::sync::atomic::Ordering::SeqCst);
    }

    pub fn is_cancelled(&self) -> bool {
        self.cancelled.load(std::sync::atomic::Ordering::SeqCst)
    }
}

impl Default for CancellationToken {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RunStatus {
    Pending,
    Running,
    WaitingApproval,
    Succeeded,
    Failed,
    Cancelled,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AgentRun {
    pub status: RunStatus,
    pub final_text: String,
    pub executed_tools: usize,
    pub pending_approvals: Vec<String>,
}

pub fn transition(run_status: RunStatus, action: &str) -> Result<RunStatus, KernelError> {
    match (run_status, action) {
        (RunStatus::Pending, "start") => Ok(RunStatus::Running),
        (RunStatus::Running, "wait_approval") => Ok(RunStatus::WaitingApproval),
        (RunStatus::Running, "succeed") => Ok(RunStatus::Succeeded),
        (RunStatus::Running, "fail") => Ok(RunStatus::Failed),
        (RunStatus::Running, "cancel") => Ok(RunStatus::Cancelled),
        _ => Err(KernelError::Cancelled),
    }
}

#[cfg(feature = "test-util")]
pub mod testing {
    use super::*;
    use std::collections::VecDeque;
    use std::sync::atomic::{AtomicBool, Ordering};
    use std::sync::{Arc, Mutex};

    pub struct FixedClock {
        now: u64,
    }

    impl FixedClock {
        pub fn new(now: u64) -> Self {
            Self { now }
        }
    }

    impl Clock for FixedClock {
        fn now_millis(&self) -> u64 {
            self.now
        }
    }

    pub struct FakeGateway {
        responses: Arc<Mutex<VecDeque<CompletionResponse>>>,
        called: Arc<AtomicBool>,
    }

    impl FakeGateway {
        pub fn new(responses: Vec<CompletionResponse>) -> Self {
            Self {
                responses: Arc::new(Mutex::new(responses.into())),
                called: Arc::new(AtomicBool::new(false)),
            }
        }

        pub fn was_called(&self) -> bool {
            self.called.load(Ordering::SeqCst)
        }
    }

    #[async_trait]
    impl ModelGateway for FakeGateway {
        async fn complete(
            &self,
            _request: CompletionRequest,
        ) -> Result<CompletionResponse, KernelError> {
            self.called.store(true, Ordering::SeqCst);
            self.responses
                .lock()
                .unwrap()
                .pop_front()
                .ok_or(KernelError::Gateway)
        }
    }
}
