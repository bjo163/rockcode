use std::{env, time::Duration};

use async_trait::async_trait;
use reqwest::Client;
use rockcode_kernel::{
    ChatMessage, CompletionRequest, CompletionResponse, KernelError, ModelGateway, ToolCall,
};
use serde_json::{json, Value};
use thiserror::Error;
use url::Url;

const DEFAULT_TIMEOUT: Duration = Duration::from_secs(90);

#[derive(Debug, Error, PartialEq, Eq)]
pub enum ProviderConfigError {
    #[error("missing required environment variable: {0}")]
    MissingEnvironment(&'static str),
    #[error("provider base URL must be a valid HTTP or HTTPS URL without query or fragment")]
    InvalidBaseUrl,
    #[error("provider API key cannot be empty")]
    EmptyApiKey,
    #[error("provider model ID cannot be empty")]
    EmptyModel,
    #[error("could not initialize the provider HTTP client")]
    HttpClient,
}

pub struct OpenAiCompatibleConfig {
    base_url: String,
    api_key: String,
    model: String,
    timeout: Duration,
}

impl OpenAiCompatibleConfig {
    pub fn new(
        base_url: impl Into<String>,
        api_key: impl Into<String>,
        model: impl Into<String>,
    ) -> Result<Self, ProviderConfigError> {
        let base_url = base_url.into().trim_end_matches('/').to_string();
        let parsed_url = Url::parse(&base_url).map_err(|_| ProviderConfigError::InvalidBaseUrl)?;
        let host = parsed_url.host_str().unwrap_or_default();
        let loopback_host = matches!(host, "localhost" | "127.0.0.1" | "[::1]" | "::1");
        if !matches!(parsed_url.scheme(), "http" | "https")
            || host.is_empty()
            || parsed_url.query().is_some()
            || parsed_url.fragment().is_some()
            || !parsed_url.username().is_empty()
            || parsed_url.password().is_some()
            || (parsed_url.scheme() == "http" && !loopback_host)
        {
            return Err(ProviderConfigError::InvalidBaseUrl);
        }

        let api_key = api_key.into().trim().to_string();
        if api_key.is_empty() {
            return Err(ProviderConfigError::EmptyApiKey);
        }

        let model = model.into().trim().to_string();
        if model.is_empty() {
            return Err(ProviderConfigError::EmptyModel);
        }

        Ok(Self {
            base_url,
            api_key,
            model,
            timeout: DEFAULT_TIMEOUT,
        })
    }

    pub fn from_env() -> Result<Self, ProviderConfigError> {
        let base_url = env::var("ROCKCODE_PROVIDER_BASE_URL")
            .map_err(|_| ProviderConfigError::MissingEnvironment("ROCKCODE_PROVIDER_BASE_URL"))?;
        let api_key = env::var("ROCKCODE_PROVIDER_API_KEY")
            .map_err(|_| ProviderConfigError::MissingEnvironment("ROCKCODE_PROVIDER_API_KEY"))?;
        let model = env::var("ROCKCODE_MODEL_ID")
            .map_err(|_| ProviderConfigError::MissingEnvironment("ROCKCODE_MODEL_ID"))?;

        Self::new(base_url, api_key, model)
    }

    fn completions_endpoint(&self) -> String {
        format!("{}/chat/completions", self.base_url)
    }
}

pub struct OpenAiCompatibleGateway {
    config: OpenAiCompatibleConfig,
    client: Client,
}

impl OpenAiCompatibleGateway {
    pub fn new(config: OpenAiCompatibleConfig) -> Result<Self, ProviderConfigError> {
        let client = Client::builder()
            .timeout(config.timeout)
            .build()
            .map_err(|_| ProviderConfigError::HttpClient)?;

        Ok(Self { config, client })
    }

    pub fn from_env() -> Result<Self, ProviderConfigError> {
        Self::new(OpenAiCompatibleConfig::from_env()?)
    }

    fn build_request(&self, request: &CompletionRequest) -> Value {
        let messages = request.messages.iter().map(serialize_message).collect::<Vec<Value>>();
        let mut payload = json!({
            "model": self.config.model,
            "messages": messages
        });

        if let Some(temperature) = request.temperature {
            payload["temperature"] = json!(temperature);
        }
        if let Some(max_output_tokens) = request.max_output_tokens {
            payload["max_tokens"] = json!(max_output_tokens);
        }
        if !request.tools.is_empty() {
            payload["tools"] = Value::Array(
                request.tools
                    .iter()
                    .map(|tool| {
                        json!({
                            "type": "function",
                            "function": {
                                "name": tool.name,
                                "description": tool.description,
                                "parameters": tool.parameters
                            }
                        })
                    })
                    .collect(),
            );
            payload["tool_choice"] = json!("auto");
        }

        payload
    }
}

fn serialize_message(message: &ChatMessage) -> Value {
    let mut output = json!({
        "role": message.role,
        "content": if message.content.is_empty() && !message.tool_calls.is_empty() {
            Value::Null
        } else {
            Value::String(message.content.clone())
        }
    });

    if let Some(name) = &message.name {
        output["name"] = json!(name);
    }
    if let Some(tool_call_id) = &message.tool_call_id {
        output["tool_call_id"] = json!(tool_call_id);
    }
    if !message.tool_calls.is_empty() {
        output["tool_calls"] = Value::Array(
            message.tool_calls
                .iter()
                .map(|call| {
                    json!({
                        "id": call.id,
                        "type": "function",
                        "function": {
                            "name": call.name,
                            "arguments": call.arguments
                        }
                    })
                })
                .collect(),
        );
    }

    output
}

fn parse_tool_calls(value: Option<&Value>) -> Result<Vec<ToolCall>, KernelError> {
    let Some(value) = value else {
        return Ok(Vec::new());
    };
    let calls = value.as_array().ok_or(KernelError::Gateway)?;
    let mut output = Vec::with_capacity(calls.len());
    let mut seen_ids = std::collections::HashSet::new();

    for call in calls {
        let id = call.get("id").and_then(Value::as_str).unwrap_or_default();
        let function = call.get("function").ok_or(KernelError::Gateway)?;
        let name = function.get("name").and_then(Value::as_str).unwrap_or_default();
        let arguments = function
            .get("arguments")
            .and_then(Value::as_str)
            .unwrap_or_default();

        if id.is_empty()
            || name.is_empty()
            || arguments.is_empty()
            || !seen_ids.insert(id.to_string())
        {
            return Err(KernelError::Gateway);
        }

        serde_json::from_str::<Value>(arguments).map_err(|_| KernelError::Gateway)?;
        output.push(ToolCall {
            id: id.to_string(),
            name: name.to_string(),
            arguments: arguments.to_string(),
        });
    }

    Ok(output)
}

fn parse_response(body: &Value) -> Result<CompletionResponse, KernelError> {
    let message = body
        .pointer("/choices/0/message")
        .ok_or(KernelError::Gateway)?;
    let content = message
        .get("content")
        .and_then(Value::as_str)
        .unwrap_or_default()
        .to_string();
    let tool_calls = parse_tool_calls(message.get("tool_calls"))?;

    if content.is_empty() && tool_calls.is_empty() {
        return Err(KernelError::Gateway);
    }

    let finish_reason = body
        .pointer("/choices/0/finish_reason")
        .and_then(Value::as_str)
        .map(str::to_string);

    Ok(CompletionResponse {
        content,
        tool_calls,
        finish_reason,
    })
}

#[async_trait]
impl ModelGateway for OpenAiCompatibleGateway {
    async fn complete(
        &self,
        request: CompletionRequest,
    ) -> Result<CompletionResponse, KernelError> {
        if request.messages.is_empty() {
            return Err(KernelError::Gateway);
        }

        let response = self
            .client
            .post(self.config.completions_endpoint())
            .bearer_auth(&self.config.api_key)
            .json(&self.build_request(&request))
            .send()
            .await
            .map_err(|_| KernelError::Gateway)?;

        if !response.status().is_success() {
            return Err(KernelError::Gateway);
        }

        let body = response
            .json::<Value>()
            .await
            .map_err(|_| KernelError::Gateway)?;

        parse_response(&body)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rockcode_kernel::{MessageRole, ToolDefinition};

    fn config() -> OpenAiCompatibleConfig {
        OpenAiCompatibleConfig::new("https://api.example.test/v1/", "test-secret", "test-model")
            .unwrap()
    }

    #[test]
    fn validates_provider_configuration_without_exposing_secrets() {
        assert_eq!(
            OpenAiCompatibleConfig::new("file:///tmp/model", "secret", "model").err(),
            Some(ProviderConfigError::InvalidBaseUrl)
        );
        assert_eq!(
            OpenAiCompatibleConfig::new("https://api.example.test/v1", "", "model").err(),
            Some(ProviderConfigError::EmptyApiKey)
        );
        assert_eq!(
            OpenAiCompatibleConfig::new("http://provider.example.test/v1", "secret", "model").err(),
            Some(ProviderConfigError::InvalidBaseUrl)
        );
        assert!(
            OpenAiCompatibleConfig::new("http://127.0.0.1:43170/v1", "secret", "model").is_ok()
        );
        assert_eq!(
            OpenAiCompatibleConfig::new("https://api.example.test/v1", "secret", "").err(),
            Some(ProviderConfigError::EmptyModel)
        );
    }

    #[test]
    fn constructs_chat_completions_endpoint() {
        assert_eq!(
            config().completions_endpoint(),
            "https://api.example.test/v1/chat/completions"
        );
    }

    #[test]
    fn maps_canonical_messages_and_tools_to_openai_compatible_request() {
        let gateway = OpenAiCompatibleGateway::new(config()).unwrap();
        let mut request = CompletionRequest::new(vec![
            ChatMessage::system("You are a coding assistant."),
            ChatMessage::user("Inspect this repository."),
        ]);
        request.temperature = Some(0.25);
        request.max_output_tokens = Some(1024);
        request.tools = vec![ToolDefinition {
            name: "workspace.read_file".to_string(),
            description: "Read a UTF-8 text file from the selected workspace.".to_string(),
            parameters: json!({
                "type": "object",
                "properties": {"path": {"type": "string"}},
                "required": ["path"],
                "additionalProperties": false
            }),
        }];

        let payload = gateway.build_request(&request);

        assert_eq!(payload["model"], "test-model");
        assert_eq!(payload["messages"][0]["role"], "system");
        assert_eq!(payload["messages"][1]["role"], "user");
        assert_eq!(payload["temperature"], 0.25);
        assert_eq!(payload["max_tokens"], 1024);
        assert_eq!(payload["tools"][0]["function"]["name"], "workspace.read_file");
        assert_eq!(payload["tool_choice"], "auto");
        assert!(payload.get("api_key").is_none());
    }

    #[test]
    fn parses_text_completion_and_finish_reason() {
        let response = json!({
            "choices": [{
                "message": {"role": "assistant", "content": "Ready to inspect code."},
                "finish_reason": "stop"
            }]
        });

        let parsed = parse_response(&response).unwrap();

        assert_eq!(parsed.content, "Ready to inspect code.");
        assert!(parsed.tool_calls.is_empty());
        assert_eq!(parsed.finish_reason.as_deref(), Some("stop"));
    }

    #[test]
    fn parses_structured_tool_calls_and_validates_json_arguments() {
        let response = json!({
            "choices": [{
                "message": {
                    "role": "assistant",
                    "content": null,
                    "tool_calls": [{
                        "id": "call_1",
                        "type": "function",
                        "function": {
                            "name": "workspace.read_file",
                            "arguments": "{\"path\":\"README.md\"}"
                        }
                    }]
                },
                "finish_reason": "tool_calls"
            }]
        });

        let parsed = parse_response(&response).unwrap();

        assert_eq!(parsed.content, "");
        assert_eq!(parsed.tool_calls.len(), 1);
        assert_eq!(parsed.tool_calls[0].id, "call_1");
        assert_eq!(parsed.tool_calls[0].name, "workspace.read_file");
        assert_eq!(parsed.tool_calls[0].arguments, "{\"path\":\"README.md\"}");
    }

    #[test]
    fn rejects_incomplete_or_malformed_tool_call_shapes() {
        assert!(parse_response(&json!({"choices": []})).is_err());
        assert!(parse_response(&json!({
            "choices": [{
                "message": {
                    "content": null,
                    "tool_calls": [{
                        "id": "call_1",
                        "function": {"name": "read_file", "arguments": "{bad json"}
                    }]
                },
                "finish_reason": "tool_calls"
            }]
        })).is_err());
        assert!(parse_response(&json!({
            "choices": [{"message": {"content": null, "tool_calls": []}, "finish_reason": "tool_calls"}]
        })).is_err());
    }

    #[test]
    fn serializes_assistant_tool_calls_and_tool_results() {
        let call = ToolCall {
            id: "call_1".to_string(),
            name: "workspace.read_file".to_string(),
            arguments: "{\"path\":\"README.md\"}".to_string(),
        };
        let assistant = ChatMessage::assistant_tool_calls("", vec![call]);
        let result = ChatMessage::tool_result("call_1", "workspace.read_file", "file contents");

        let assistant_json = serialize_message(&assistant);
        let result_json = serialize_message(&result);

        assert!(assistant_json["content"].is_null());
        assert_eq!(assistant_json["tool_calls"][0]["id"], "call_1");
        assert_eq!(result_json["role"], "tool");
        assert_eq!(result_json["tool_call_id"], "call_1");
        assert_eq!(result_json["name"], "workspace.read_file");
    }

    #[test]
    fn serializes_message_roles_using_canonical_lowercase_names() {
        let message = ChatMessage::new(MessageRole::Assistant, "Hello");
        let serialized = serde_json::to_value(message).unwrap();

        assert_eq!(serialized["role"], "assistant");
    }
}
