use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum KernelError {
    #[error("gateway error")]
    Gateway,
    #[error("tool error")]
    Tool,
    #[error("cancelled")]
    Cancelled,
}

#[async_trait]
pub trait ModelGateway: Send + Sync {
    async fn complete(&self, req: &str) -> Result<String, KernelError>;
}

#[async_trait]
pub trait ToolInvoker: Send + Sync {
    async fn invoke(&self, name: &str, args: &str) -> Result<String, KernelError>;
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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RunStatus {
    Pending,
    Running,
    Succeeded,
    Failed,
    Cancelled,
}

pub fn transition(run_status: RunStatus, action: &str) -> Result<RunStatus, KernelError> {
    match (run_status, action) {
        (RunStatus::Pending, "start") => Ok(RunStatus::Running),
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
    use std::time::{Duration, Instant};

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
        responses: Arc<Mutex<VecDeque<String>>>,
        called: Arc<AtomicBool>,
    }
    impl FakeGateway {
        pub fn new(responses: Vec<String>) -> Self {
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
        async fn complete(&self, _req: &str) -> Result<String, KernelError> {
            self.called.store(true, Ordering::SeqCst);
            let mut q = self.responses.lock().unwrap();
            q.pop_front().ok_or(KernelError::Gateway)
        }
    }
}
