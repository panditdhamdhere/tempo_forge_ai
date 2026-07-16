use super::{CompletionRequest, CompletionResponse, LlmProvider, ProviderConfig, ProviderKind};
use crate::types::{ChatMessage, Role, TokenUsage};
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use tempoforge_common::{AppError, AppResult};

pub struct OpenAiCompatibleProvider {
    config: ProviderConfig,
    http: reqwest::Client,
}

impl OpenAiCompatibleProvider {
    pub fn new(config: ProviderConfig) -> Self {
        Self {
            config,
            http: reqwest::Client::new(),
        }
    }
}

#[derive(Serialize)]
struct ChatCompletionBody<'a> {
    model: &'a str,
    messages: Vec<ApiMessage<'a>>,
    temperature: f32,
    max_tokens: u32,
}

#[derive(Serialize)]
struct ApiMessage<'a> {
    role: &'a str,
    content: &'a str,
}

#[derive(Deserialize)]
struct ChatCompletionResponse {
    model: String,
    choices: Vec<Choice>,
    usage: Option<ApiUsage>,
}

#[derive(Deserialize)]
struct Choice {
    message: ChoiceMessage,
}

#[derive(Deserialize)]
struct ChoiceMessage {
    content: Option<String>,
}

#[derive(Deserialize)]
struct ApiUsage {
    prompt_tokens: u32,
    completion_tokens: u32,
    total_tokens: u32,
}

fn role_str(role: Role) -> &'static str {
    match role {
        Role::System => "system",
        Role::User => "user",
        Role::Assistant => "assistant",
        Role::Tool => "tool",
    }
}

#[async_trait]
impl LlmProvider for OpenAiCompatibleProvider {
    fn kind(&self) -> ProviderKind {
        self.config.kind
    }

    fn model(&self) -> &str {
        &self.config.model
    }

    async fn complete(&self, request: CompletionRequest) -> AppResult<CompletionResponse> {
        let messages: Vec<ApiMessage<'_>> = request
            .messages
            .iter()
            .map(|m: &ChatMessage| ApiMessage {
                role: role_str(m.role),
                content: &m.content,
            })
            .collect();

        let body = ChatCompletionBody {
            model: &self.config.model,
            messages,
            temperature: request.temperature,
            max_tokens: request.max_tokens,
        };

        let url = format!(
            "{}/chat/completions",
            self.config.base_url.trim_end_matches('/')
        );

        let mut req = self.http.post(&url).json(&body);
        if !self.config.api_key.is_empty() {
            req = req.bearer_auth(&self.config.api_key);
        }

        let response = req
            .send()
            .await
            .map_err(|e| AppError::Upstream(format!("LLM request failed: {e}")))?;

        if !response.status().is_success() {
            let status = response.status();
            let text = response.text().await.unwrap_or_default();
            return Err(AppError::Upstream(format!(
                "LLM error status={status}: {text}"
            )));
        }

        let parsed: ChatCompletionResponse = response
            .json()
            .await
            .map_err(|e| AppError::Upstream(format!("LLM response parse failed: {e}")))?;

        let content = parsed
            .choices
            .first()
            .and_then(|c| c.message.content.clone())
            .unwrap_or_default();

        let usage = parsed
            .usage
            .map(|u| TokenUsage {
                prompt_tokens: u.prompt_tokens,
                completion_tokens: u.completion_tokens,
                total_tokens: u.total_tokens,
            })
            .unwrap_or_default();

        Ok(CompletionResponse {
            content,
            model: parsed.model,
            usage,
        })
    }
}
