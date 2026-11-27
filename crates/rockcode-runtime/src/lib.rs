use rockcode_kernel::{
    transition, AgentRun, CancellationToken, ChatMessage, CompletionRequest, EventSink,
    KernelError, ModelGateway, RunStatus, ToolCall, ToolDecision, ToolDefinition, ToolInvoker,
    ToolPolicy,
};
use serde_json::{json, Value};
use std::sync::Arc;

const MAX_AGENT_TURNS: u8 = 16;

#[derive(Clone)]
pub struct Runtime {
    gateway: Arc<dyn ModelGateway>,
    sink: Arc<dyn EventSink>,
    tools: Option<Arc<dyn ToolInvoker>>,
    policy: Option<Arc<dyn ToolPolicy>>,
}

impl Runtime {
    pub fn new(gateway: Arc<dyn ModelGateway>, sink: Arc<dyn EventSink>) -> Self {
        Self {
            gateway,
            sink,
            tools: None,
            policy: None,
        }
    }

    pub fn with_tools(
        gateway: Arc<dyn ModelGateway>,
        sink: Arc<dyn EventSink>,
        tools: Arc<dyn ToolInvoker>,
        policy: Arc<dyn ToolPolicy>,
    ) -> Self {
        Self {
            gateway,
            sink,
            tools: Some(tools),
            policy: Some(policy),
        }
    }

    pub async fn run_once(&self, token: &CancellationToken) -> Result<RunStatus, KernelError> {
        let result = self
            .run_agent(vec![ChatMessage::user("ping")], Vec::new(), token, 1)
            .await?;
        Ok(result.status)
    }

    pub async fn run_agent(
        &self,
        mut messages: Vec<ChatMessage>,
        tool_definitions: Vec<ToolDefinition>,
        token: &CancellationToken,
        max_turns: u8,
    ) -> Result<AgentRun, KernelError> {
        if max_turns == 0 || max_turns > MAX_AGENT_TURNS {
            return Err(KernelError::InvalidTurnLimit);
        }

        transition(RunStatus::Pending, "start")?;
        self.emit_event(json!({"type": "run.started"})).await?;
        let mut executed_tools = 0;

        for _turn in 0..max_turns {
            if token.is_cancelled() {
                self.emit_event(json!({"type": "run.cancelled"})).await?;
                return Ok(AgentRun {
                    status: RunStatus::Cancelled,
                    final_text: String::new(),
                    executed_tools,
                    pending_approvals: Vec::new(),
                });
            }

            let mut request = CompletionRequest::new(messages.clone());
            request.tools = tool_definitions.clone();
            let response = self.gateway.complete(request).await?;

            if response.tool_calls.is_empty() {
                self.emit_event(json!({"type": "run.succeeded"})).await?;
                return Ok(AgentRun {
                    status: RunStatus::Succeeded,
                    final_text: response.content,
                    executed_tools,
                    pending_approvals: Vec::new(),
                });
            }

            messages.push(ChatMessage::assistant_tool_calls(
                response.content,
                response.tool_calls.clone(),
            ));

            let decisions = self
                .authorize_tool_batch(&response.tool_calls, &tool_definitions)
                .await?;
            let mut pending_approvals = Vec::new();
            for (call, decision) in response.tool_calls.iter().zip(&decisions) {
                if *decision == ToolDecision::ApprovalRequired {
                    pending_approvals.push(call.id.clone());
                    self.emit_approval_required(call).await?;
                }
            }

            if !pending_approvals.is_empty() {
                transition(RunStatus::Running, "wait_approval")?;
                return Ok(AgentRun {
                    status: RunStatus::WaitingApproval,
                    final_text: String::new(),
                    executed_tools,
                    pending_approvals,
                });
            }

            for (call, decision) in response.tool_calls.iter().zip(decisions) {
                if token.is_cancelled() {
                    self.emit_event(json!({"type": "run.cancelled"})).await?;
                    return Ok(AgentRun {
                        status: RunStatus::Cancelled,
                        final_text: String::new(),
                        executed_tools,
                        pending_approvals: Vec::new(),
                    });
                }

                match decision {
                    ToolDecision::Allow => {
                        let Some(invoker) = &self.tools else {
                            self.emit_approval_required(call).await?;
                            return Ok(AgentRun {
                                status: RunStatus::WaitingApproval,
                                final_text: String::new(),
                                executed_tools,
                                pending_approvals: vec![call.id.clone()],
                            });
                        };

                        self.emit_event(json!({
                            "type": "tool.started",
                            "tool_call_id": call.id,
                            "tool_name": call.name
                        }))
                        .await?;

                        let output = invoker.invoke(&call.name, &call.arguments).await?;
                        executed_tools += 1;
                        self.emit_event(json!({
                            "type": "tool.succeeded",
                            "tool_call_id": call.id,
                            "tool_name": call.name
                        }))
                        .await?;
                        messages.push(ChatMessage::tool_result(
                            call.id.clone(),
                            call.name.clone(),
                            output,
                        ));
                    }
                    ToolDecision::Deny => {
                        self.emit_event(json!({
                            "type": "tool.denied",
                            "tool_call_id": call.id,
                            "tool_name": call.name
                        }))
                        .await?;
                        messages.push(ChatMessage::tool_result(
                            call.id.clone(),
                            call.name.clone(),
                            "Tool call denied by policy.",
                        ));
                    }
                    ToolDecision::ApprovalRequired => {
                        unreachable!("approvals are handled before invocation")
                    }
                }
            }
        }

        self.emit_event(json!({"type": "run.turn_limit_reached"}))
            .await?;
        Ok(AgentRun {
            status: RunStatus::Failed,
            final_text: "The agent reached its configured turn limit.".to_string(),
            executed_tools,
            pending_approvals: Vec::new(),
        })
    }

    async fn authorize_tool_batch(
        &self,
        calls: &[ToolCall],
        definitions: &[ToolDefinition],
    ) -> Result<Vec<ToolDecision>, KernelError> {
        let mut decisions = Vec::with_capacity(calls.len());
        for call in calls {
            let arguments =
                serde_json::from_str::<Value>(&call.arguments).map_err(|_| KernelError::Gateway)?;
            if !arguments.is_object()
                || !definitions
                    .iter()
                    .any(|definition| definition.name == call.name)
            {
                decisions.push(ToolDecision::Deny);
                continue;
            }

            let decision = match (&self.policy, &self.tools) {
                (Some(policy), Some(_)) => policy.authorize(call).await?,
                _ => ToolDecision::ApprovalRequired,
            };
            decisions.push(decision);
        }
        Ok(decisions)
    }

    async fn emit_approval_required(&self, call: &ToolCall) -> Result<(), KernelError> {
        self.emit_event(json!({
            "type": "tool.approval_required",
            "tool_call_id": call.id,
            "tool_name": call.name,
            "arguments": call.arguments
        }))
        .await
    }

    async fn emit_event(&self, event: Value) -> Result<(), KernelError> {
        let serialized = serde_json::to_string(&event).map_err(|_| KernelError::Tool)?;
        self.sink.emit(&serialized).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use async_trait::async_trait;
    use std::collections::VecDeque;
    use std::sync::atomic::{AtomicUsize, Ordering};
    use std::sync::Mutex;

    struct FakeGateway {
        responses: Mutex<VecDeque<rockcode_kernel::CompletionResponse>>,
    }

    impl FakeGateway {
        fn new(responses: Vec<rockcode_kernel::CompletionResponse>) -> Self {
            Self {
                responses: Mutex::new(responses.into()),
            }
        }
    }

    #[async_trait]
    impl ModelGateway for FakeGateway {
        async fn complete(
            &self,
            _request: CompletionRequest,
        ) -> Result<rockcode_kernel::CompletionResponse, KernelError> {
            self.responses
                .lock()
                .unwrap()
                .pop_front()
                .ok_or(KernelError::Gateway)
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

    struct FakeTools {
        calls: AtomicUsize,
    }

    impl FakeTools {
        fn new() -> Self {
            Self {
                calls: AtomicUsize::new(0),
            }
        }
    }

    #[async_trait]
    impl ToolInvoker for FakeTools {
        async fn invoke(&self, _name: &str, _args: &str) -> Result<String, KernelError> {
            self.calls.fetch_add(1, Ordering::SeqCst);
            Ok("workspace file contents".to_string())
        }
    }

    struct FixedPolicy(ToolDecision);

    #[async_trait]
    impl ToolPolicy for FixedPolicy {
        async fn authorize(&self, _call: &ToolCall) -> Result<ToolDecision, KernelError> {
            Ok(self.0)
        }
    }

    struct SequencePolicy(Mutex<VecDeque<ToolDecision>>);

    #[async_trait]
    impl ToolPolicy for SequencePolicy {
        async fn authorize(&self, _call: &ToolCall) -> Result<ToolDecision, KernelError> {
            self.0.lock().unwrap().pop_front().ok_or(KernelError::Tool)
        }
    }

    fn read_file_call() -> rockcode_kernel::CompletionResponse {
        rockcode_kernel::CompletionResponse {
            content: String::new(),
            tool_calls: vec![ToolCall {
                id: "call_read_1".to_string(),
                name: "workspace.read_file".to_string(),
                arguments: "{\"path\":\"README.md\"}".to_string(),
            }],
            finish_reason: Some("tool_calls".to_string()),
        }
    }

    fn final_response() -> rockcode_kernel::CompletionResponse {
        rockcode_kernel::CompletionResponse {
            content: "The project readme describes RockCode.".to_string(),
            tool_calls: Vec::new(),
            finish_reason: Some("stop".to_string()),
        }
    }

    fn read_file_definition() -> ToolDefinition {
        ToolDefinition {
            name: "workspace.read_file".to_string(),
            description: "Read one file".to_string(),
            parameters: json!({"type":"object", "properties":{"path":{"type":"string"}}}),
        }
    }

    #[tokio::test]
    async fn run_once_completes_and_emits_ordered_lifecycle_events() {
        let gateway = Arc::new(FakeGateway::new(vec![final_response()]));
        let sink = Arc::new(RecordingSink::default());
        let runtime = Runtime::new(gateway, sink.clone());

        let status = runtime.run_once(&CancellationToken::new()).await.unwrap();

        assert_eq!(status, RunStatus::Succeeded);
        assert_eq!(
            *sink.events.lock().unwrap(),
            vec![
                "{\"type\":\"run.started\"}".to_string(),
                "{\"type\":\"run.succeeded\"}".to_string()
            ]
        );
    }

    #[tokio::test]
    async fn cancelled_run_does_not_call_provider() {
        let gateway = Arc::new(FakeGateway::new(vec![final_response()]));
        let sink = Arc::new(RecordingSink::default());
        let runtime = Runtime::new(gateway, sink.clone());
        let token = CancellationToken::new();
        token.cancel();

        let result = runtime
            .run_agent(vec![ChatMessage::user("ping")], Vec::new(), &token, 1)
            .await
            .unwrap();

        assert_eq!(result.status, RunStatus::Cancelled);
        assert_eq!(sink.events.lock().unwrap().len(), 2);
    }

    #[tokio::test]
    async fn policy_approved_tool_executes_and_returns_result_to_model() {
        let gateway = Arc::new(FakeGateway::new(vec![read_file_call(), final_response()]));
        let sink = Arc::new(RecordingSink::default());
        let tools = Arc::new(FakeTools::new());
        let policy = Arc::new(FixedPolicy(ToolDecision::Allow));
        let runtime = Runtime::with_tools(gateway, sink.clone(), tools.clone(), policy);

        let result = runtime
            .run_agent(
                vec![ChatMessage::user("Summarize README.md")],
                vec![read_file_definition()],
                &CancellationToken::new(),
                4,
            )
            .await
            .unwrap();

        assert_eq!(result.status, RunStatus::Succeeded);
        assert_eq!(result.final_text, "The project readme describes RockCode.");
        assert_eq!(result.executed_tools, 1);
        assert_eq!(tools.calls.load(Ordering::SeqCst), 1);
        let events = sink.events.lock().unwrap();
        assert!(events
            .iter()
            .any(|event| event.contains("\"type\":\"tool.started\"")));
        assert!(events
            .iter()
            .any(|event| event.contains("\"type\":\"tool.succeeded\"")));
    }

    #[tokio::test]
    async fn one_pending_approval_holds_the_entire_tool_batch() {
        let mut batch = read_file_call();
        batch.tool_calls.push(ToolCall {
            id: "call_write_2".to_string(),
            name: "workspace.write_file".to_string(),
            arguments: "{\"path\":\"README.md\",\"content\":\"updated\"}".to_string(),
        });
        let gateway = Arc::new(FakeGateway::new(vec![batch]));
        let sink = Arc::new(RecordingSink::default());
        let tools = Arc::new(FakeTools::new());
        let policy = Arc::new(SequencePolicy(Mutex::new(VecDeque::from([
            ToolDecision::Allow,
            ToolDecision::ApprovalRequired,
        ]))));
        let runtime = Runtime::with_tools(gateway, sink.clone(), tools.clone(), policy);

        let result = runtime
            .run_agent(
                vec![ChatMessage::user("Read and update README")],
                vec![
                    read_file_definition(),
                    ToolDefinition {
                        name: "workspace.write_file".to_string(),
                        description: "Write one file".to_string(),
                        parameters: json!({"type":"object"}),
                    },
                ],
                &CancellationToken::new(),
                2,
            )
            .await
            .unwrap();

        assert_eq!(result.status, RunStatus::WaitingApproval);
        assert_eq!(result.pending_approvals, vec!["call_write_2"]);
        assert_eq!(tools.calls.load(Ordering::SeqCst), 0);
        assert_eq!(
            sink.events
                .lock()
                .unwrap()
                .iter()
                .filter(|event| event.contains("tool.approval_required"))
                .count(),
            1
        );
    }

    #[tokio::test]
    async fn missing_policy_defaults_to_approval_and_never_executes_tool() {
        let gateway = Arc::new(FakeGateway::new(vec![read_file_call()]));
        let sink = Arc::new(RecordingSink::default());
        let runtime = Runtime::new(gateway, sink.clone());

        let result = runtime
            .run_agent(
                vec![ChatMessage::user("Read README")],
                vec![read_file_definition()],
                &CancellationToken::new(),
                2,
            )
            .await
            .unwrap();

        assert_eq!(result.status, RunStatus::WaitingApproval);
        assert_eq!(result.pending_approvals.len(), 1);
    }

    #[tokio::test]
    async fn unknown_tools_are_denied_without_execution() {
        let gateway = Arc::new(FakeGateway::new(vec![read_file_call(), final_response()]));
        let sink = Arc::new(RecordingSink::default());
        let tools = Arc::new(FakeTools::new());
        let policy = Arc::new(FixedPolicy(ToolDecision::Allow));
        let runtime = Runtime::with_tools(gateway, sink.clone(), tools.clone(), policy);

        let result = runtime
            .run_agent(
                vec![ChatMessage::user("Read README")],
                Vec::new(),
                &CancellationToken::new(),
                3,
            )
            .await
            .unwrap();

        assert_eq!(result.status, RunStatus::Succeeded);
        assert_eq!(tools.calls.load(Ordering::SeqCst), 0);
        assert!(sink
            .events
            .lock()
            .unwrap()
            .iter()
            .any(|event| event.contains("tool.denied")));
    }

    #[tokio::test]
    async fn invalid_turn_limit_is_rejected_before_model_call() {
        let gateway = Arc::new(FakeGateway::new(vec![final_response()]));
        let sink = Arc::new(RecordingSink::default());
        let runtime = Runtime::new(gateway, sink);

        assert!(matches!(
            runtime
                .run_agent(Vec::new(), Vec::new(), &CancellationToken::new(), 0)
                .await,
            Err(KernelError::InvalidTurnLimit)
        ));
    }
}
