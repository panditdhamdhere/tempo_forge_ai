use super::traits::Agent;
use crate::prompts::PromptManager;
use crate::providers::{CompletionRequest, LlmProvider};
use crate::types::{AgentKind, AgentRequest, AgentResponse, ChatMessage, GeneratedFile};
use async_trait::async_trait;
use chrono::Utc;
use std::sync::Arc;
use tempoforge_common::AppResult;

pub struct DocumentationAgent {
    provider: Arc<dyn LlmProvider>,
}

impl DocumentationAgent {
    pub fn new(provider: Arc<dyn LlmProvider>) -> Self {
        Self { provider }
    }
}

#[async_trait]
impl Agent for DocumentationAgent {
    fn kind(&self) -> AgentKind {
        AgentKind::DocumentationWriter
    }

    async fn run(&self, request: AgentRequest) -> AppResult<AgentResponse> {
        let mut messages = vec![ChatMessage::system(PromptManager::system_prompt(self.kind()))];
        messages.extend(request.messages.clone());

        let completion = self
            .provider
            .complete(CompletionRequest {
                messages,
                temperature: 0.2,
                max_tokens: 6144,
            })
            .await?;

        let files = vec![GeneratedFile {
            path: "docs/GENERATED.md".into(),
            content: completion.content.clone(),
            language: "markdown".into(),
        }];

        Ok(AgentResponse {
            request_id: request.id,
            agent: self.kind(),
            content: completion.content,
            files,
            findings: vec![],
            follow_ups: vec![],
            model: completion.model,
            created_at: Utc::now(),
            usage: completion.usage,
        })
    }
}
