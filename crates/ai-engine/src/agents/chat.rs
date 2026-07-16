use super::traits::Agent;
use crate::prompts::PromptManager;
use crate::providers::{CompletionRequest, LlmProvider};
use crate::rag::RagStore;
use crate::types::{AgentKind, AgentRequest, AgentResponse, ChatMessage};
use async_trait::async_trait;
use chrono::Utc;
use std::sync::Arc;
use tempoforge_common::AppResult;

pub struct ChatAgent {
    provider: Arc<dyn LlmProvider>,
    rag: Arc<dyn RagStore>,
}

impl ChatAgent {
    pub fn new(provider: Arc<dyn LlmProvider>, rag: Arc<dyn RagStore>) -> Self {
        Self { provider, rag }
    }
}

#[async_trait]
impl Agent for ChatAgent {
    fn kind(&self) -> AgentKind {
        AgentKind::Chat
    }

    async fn run(&self, request: AgentRequest) -> AppResult<AgentResponse> {
        let query = request
            .messages
            .last()
            .map(|m| m.content.as_str())
            .unwrap_or_default();
        let hits = self.rag.search("tempo", query, 4).await?;
        let context = hits
            .iter()
            .map(|h| format!("[{}] {}", h.source, h.text))
            .collect::<Vec<_>>()
            .join("\n");

        let system = if request.agent == AgentKind::Architect {
            PromptManager::system_prompt(AgentKind::Architect)
        } else if request.agent == AgentKind::DeploymentAssistant {
            PromptManager::system_prompt(AgentKind::DeploymentAssistant)
        } else {
            PromptManager::system_prompt(AgentKind::Chat)
        };

        let mut messages = vec![
            ChatMessage::system(system),
            ChatMessage::system(format!("RAG context:\n{context}")),
        ];
        messages.extend(request.messages.clone());

        let completion = self
            .provider
            .complete(CompletionRequest {
                messages,
                temperature: 0.3,
                max_tokens: 4096,
            })
            .await?;

        Ok(AgentResponse {
            request_id: request.id,
            agent: request.agent,
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
