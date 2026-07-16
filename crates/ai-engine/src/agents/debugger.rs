use super::traits::Agent;
use crate::prompts::PromptManager;
use crate::providers::{CompletionRequest, LlmProvider};
use crate::types::{AgentKind, AgentRequest, AgentResponse, ChatMessage};
use async_trait::async_trait;
use chrono::Utc;
use std::sync::Arc;
use tempoforge_common::AppResult;

pub struct DebuggerAgent {
    provider: Arc<dyn LlmProvider>,
}

impl DebuggerAgent {
    pub fn new(provider: Arc<dyn LlmProvider>) -> Self {
        Self { provider }
    }
}

#[async_trait]
impl Agent for DebuggerAgent {
    fn kind(&self) -> AgentKind {
        AgentKind::Debugger
    }

    async fn run(&self, request: AgentRequest) -> AppResult<AgentResponse> {
        let context = serde_json::to_string_pretty(&request.context).unwrap_or_default();
        let mut messages = vec![
            ChatMessage::system(PromptManager::system_prompt(self.kind())),
            ChatMessage::system(format!("Transaction / debug context JSON:\n{context}")),
        ];
        messages.extend(request.messages.clone());

        let completion = self
            .provider
            .complete(CompletionRequest {
                messages,
                temperature: 0.1,
                max_tokens: 4096,
            })
            .await?;

        Ok(AgentResponse {
            request_id: request.id,
            agent: self.kind(),
            content: completion.content,
            files: vec![],
            findings: vec![],
            follow_ups: vec![],
            model: completion.model,
            created_at: Utc::now(),
            usage: completion.usage,
        })
    }
}
