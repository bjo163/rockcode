use rockcode_kernel::{
    transition, CancellationToken, EventSink, KernelError, ModelGateway, RunStatus,
};
use std::sync::Arc;

#[derive(Clone)]
pub struct Runtime {
    gateway: Arc<dyn ModelGateway>,
    sink: Arc<dyn EventSink>,
}

impl Runtime {
    pub fn new(gateway: Arc<dyn ModelGateway>, sink: Arc<dyn EventSink>) -> Self {
        Self { gateway, sink }
    }

    pub async fn run_once(&self, token: &CancellationToken) -> Result<RunStatus, KernelError> {
        if token.is_cancelled() {
            let _ = self.sink.emit("{\"type\":\"run.cancelled\"}").await;
            return Ok(RunStatus::Cancelled);
        }
        let status = transition(RunStatus::Pending, "start")?;
        let _ = self.sink.emit("{\"type\":\"run.started\"}").await;
        if token.is_cancelled() {
            let _ = self.sink.emit("{\"type\":\"run.cancelled\"}").await;
            return Ok(RunStatus::Cancelled);
        }
        let _resp = self.gateway.complete("ping").await?;
        let _ = self.sink.emit("{\"type\":\"run.succeeded\"}").await;
        let final_status = transition(RunStatus::Running, "succeed")?;
        let _ = status;
        Ok(final_status)
    }
}
