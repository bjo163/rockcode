use std::{env, time::Duration};

use async_trait::async_trait;
use reqwest::{Client, Url};
use rockcode_kernel::{
    ChatMessage, CompletionRequest, CompletionResponse, KernelError, ModelGateway,
};
use serde_json::{json, Value};
use thiserror::Error;

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
        let messages = request
            .messages
            .iter()
            .map(|message: &ChatMessage| {
                json!({
                    "role": message.role,
                    "content": message.content
                })
            })
            .collect::<Vec<Value>>();

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

        payload
    }
}

fn parse_response(body: &Value) -> Result<CompletionResponse, KernelError> {
    let content = body
        .pointer("/choices/0/message/content")
        .and_then(Value::as_str)
        .ok_or(KernelError::Gateway)?;

    let finish_reason = body
        .pointer("/choices/0/finish_reason")
        .and_then(Value::as_str)
        .map(str::to_string);

    Ok(CompletionResponse {
        content: content.to_string(),
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
    use rockcode_kernel::MessageRole;

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
            OpenAiCompatibleConfig::new("http://provider.example.test/v1", "secret", "model")
                .err(),
            Some(ProviderConfigError::InvalidBaseUrl)
        );
        assert!(OpenAiCompatibleConfig::new(
            "http://127.0.0.1:43170/v1",
            "secret",
            "model"
        )
        .is_ok());
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
    fn maps_canonical_messages_to_openai_compatible_request() {
        let gateway = OpenAiCompatibleGateway::new(config()).unwrap();
        let mut request = CompletionRequest::new(vec![
            ChatMessage::system("You are a coding assistant."),
            ChatMessage::user("Inspect this repository."),
        ]);
        request.temperature = Some(0.2);
        request.max_output_tokens = Some(1024);

        let payload = gateway.build_request(&request);

        assert_eq!(payload["model"], "test-model");
        assert_eq!(payload["messages"][0]["role"], "system");
        assert_eq!(payload["messages"][1]["role"], "user");
        assert_eq!(payload["temperature"], 0.2);
        assert_eq!(payload["max_tokens"], 1024);
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
        assert_eq!(parsed.finish_reason.as_deref(), Some("stop"));
    }

    #[test]
    fn rejects_incomplete_or_tool_only_response_shapes() {
        assert!(parse_response(&json!({"choices": []})).is_err());
        assert!(parse_response(&json!({
            "choices": [{"message": {"tool_calls": []}, "finish_reason": "tool_calls"}]
        }))
        .is_err());
    }

    #[test]
    fn serializes_message_roles_using_canonical_lowercase_names() {
        let message = ChatMessage::new(MessageRole::Assistant, "Hello");
        let serialized = serde_json::to_value(message).unwrap();

        assert_eq!(serialized["role"], "assistant");
    }
}
