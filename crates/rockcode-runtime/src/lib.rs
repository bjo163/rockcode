use rockcode_kernel::{
    transition, CancellationToken, ChatMessage, CompletionRequest, EventSink, KernelError,
    ModelGateway, RunStatus,
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

        transition(RunStatus::Pending, "start")?;
        self.sink.emit("{\"type\":\"run.started\"}").await?;

        if token.is_cancelled() {
            self.sink.emit("{\"type\":\"run.cancelled\"}").await?;
            return Ok(RunStatus::Cancelled);
        }

        let request = CompletionRequest::new(vec![ChatMessage::user("ping")]);
        self.gateway.complete(request).await?;
        self.sink.emit("{\"type\":\"run.succeeded\"}").await?;

        transition(RunStatus::Running, "succeed")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use async_trait::async_trait;
    use rockcode_kernel::CompletionResponse;
    use std::sync::atomic::{AtomicUsize, Ordering};
    use std::sync::Mutex;

    struct FakeGateway {
        calls: AtomicUsize,
    }

    impl FakeGateway {
        fn new() -> Self {
            Self {
                calls: AtomicUsize::new(0),
            }
        }
    }

    #[async_trait]
    impl ModelGateway for FakeGateway {
        async fn complete(
            &self,
            _request: CompletionRequest,
        ) -> Result<CompletionResponse, KernelError> {
            self.calls.fetch_add(1, Ordering::SeqCst);
            Ok(CompletionResponse {
                content: "pong".to_string(),
                finish_reason: Some("stop".to_string()),
            })
        }
    }

    #[derive(Default)]
    struct RecordingSink {
        events: Mutex<Vec<String>>,
    }

    #[async_trait]
    impl EventSink for RecordingSink {
        async fn emit(&self, event: &str) -> Result<(), KernelError> {
            self.events.lock().unwrap().push(event.to_string());
            Ok(())
        }
    }

    #[tokio::test]
    async fn run_once_completes_and_emits_ordered_lifecycle_events() {
        let gateway = Arc::new(FakeGateway::new());
        let sink = Arc::new(RecordingSink::default());
        let runtime = Runtime::new(gateway.clone(), sink.clone());

        let status = runtime.run_once(&CancellationToken::new()).await.unwrap();

        assert!(matches!(status, RunStatus::Succeeded));
        assert_eq!(gateway.calls.load(Ordering::SeqCst), 1);
        assert_eq!(
            *sink.events.lock().unwrap(),
            vec![
                "{\"type\":\"run.started\"}".to_string(),
                "{\"type\":\"run.succeeded\"}".to_string()
            ]
        );
    }

    #[tokio::test]
    async fn run_once_does_not_call_provider_when_already_cancelled() {
        let gateway = Arc::new(FakeGateway::new());
        let sink = Arc::new(RecordingSink::default());
        let runtime = Runtime::new(gateway.clone(), sink.clone());
        let token = CancellationToken::new();
        token.cancel();

        let status = runtime.run_once(&token).await.unwrap();

        assert!(matches!(status, RunStatus::Cancelled));
        assert_eq!(gateway.calls.load(Ordering::SeqCst), 0);
        assert_eq!(
            *sink.events.lock().unwrap(),
            vec!["{\"type\":\"run.cancelled\"}".to_string()]
        );
    }
}
