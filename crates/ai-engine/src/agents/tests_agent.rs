use super::traits::Agent;
use crate::prompts::PromptManager;
use crate::providers::{CompletionRequest, LlmProvider};
use crate::types::{AgentKind, AgentRequest, AgentResponse, ChatMessage};
use async_trait::async_trait;
use chrono::Utc;
use std::sync::Arc;
use tempoforge_common::AppResult;

pub struct TestGeneratorAgent {
    provider: Arc<dyn LlmProvider>,
}

impl TestGeneratorAgent {
    pub fn new(provider: Arc<dyn LlmProvider>) -> Self {
        Self { provider }
    }
}

#[async_trait]
impl Agent for TestGeneratorAgent {
    fn kind(&self) -> AgentKind {
        AgentKind::TestGenerator
    }

    async fn run(&self, request: AgentRequest) -> AppResult<AgentResponse> {
        let mut messages = vec![ChatMessage::system(PromptManager::system_prompt(self.kind()))];
        messages.extend(request.messages.clone());

        let completion = self
            .provider
            .complete(CompletionRequest {
                messages,
                temperature: 0.15,
                max_tokens: 6144,
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
