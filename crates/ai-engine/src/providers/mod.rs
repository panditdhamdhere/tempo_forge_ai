mod openai_compatible;

use crate::types::{ChatMessage, TokenUsage};
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tempoforge_common::{AppError, AppResult};

pub use openai_compatible::OpenAiCompatibleProvider;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ProviderKind {
    Groq,
    OpenAi,
    Local,
}

#[derive(Debug, Clone)]
pub struct ProviderConfig {
    pub kind: ProviderKind,
    pub api_key: String,
    pub base_url: String,
    pub model: String,
}

#[derive(Debug, Clone)]
pub struct CompletionRequest {
    pub messages: Vec<ChatMessage>,
    pub temperature: f32,
    pub max_tokens: u32,
}

#[derive(Debug, Clone)]
pub struct CompletionResponse {
    pub content: String,
    pub model: String,
    pub usage: TokenUsage,
}

#[async_trait]
pub trait LlmProvider: Send + Sync {
    fn kind(&self) -> ProviderKind;
    fn model(&self) -> &str;
    async fn complete(&self, request: CompletionRequest) -> AppResult<CompletionResponse>;
}

pub fn create_provider(config: ProviderConfig) -> AppResult<Arc<dyn LlmProvider>> {
    if config.base_url.is_empty() {
        return Err(AppError::Internal("LLM base URL is required".into()));
    }
    Ok(Arc::new(OpenAiCompatibleProvider::new(config)))
}

pub fn provider_from_env() -> AppResult<Arc<dyn LlmProvider>> {
    let kind = match std::env::var("AI_PROVIDER")
        .unwrap_or_else(|_| "groq".into())
        .to_lowercase()
        .as_str()
    {
        "openai" => ProviderKind::OpenAi,
        "local" => ProviderKind::Local,
        _ => ProviderKind::Groq,
    };

    let (api_key, base_url, model) = match kind {
        ProviderKind::Groq => (
            std::env::var("GROQ_API_KEY").unwrap_or_default(),
            std::env::var("GROQ_BASE_URL")
                .unwrap_or_else(|_| "https://api.groq.com/openai/v1".into()),
            std::env::var("GROQ_MODEL")
                .unwrap_or_else(|_| "llama-3.3-70b-versatile".into()),
        ),
        ProviderKind::OpenAi => (
            std::env::var("OPENAI_API_KEY").unwrap_or_default(),
            std::env::var("OPENAI_BASE_URL")
                .unwrap_or_else(|_| "https://api.openai.com/v1".into()),
            std::env::var("OPENAI_MODEL").unwrap_or_else(|_| "gpt-4o".into()),
        ),
        ProviderKind::Local => (
            std::env::var("LOCAL_LLM_API_KEY").unwrap_or_else(|_| "ollama".into()),
            std::env::var("LOCAL_LLM_BASE_URL")
                .unwrap_or_else(|_| "http://localhost:11434/v1".into()),
            std::env::var("LOCAL_LLM_MODEL").unwrap_or_else(|_| "llama3.2".into()),
        ),
    };

    create_provider(ProviderConfig {
        kind,
        api_key,
        base_url,
        model,
    })
}
