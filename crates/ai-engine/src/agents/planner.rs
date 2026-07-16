use super::traits::Agent;
use crate::prompts::PromptManager;
use crate::providers::{CompletionRequest, LlmProvider};
use crate::types::{AgentKind, AgentRequest, AgentResponse, ChatMessage};
use async_trait::async_trait;
use chrono::Utc;
use std::sync::Arc;
use tempoforge_common::AppResult;

pub struct PlannerAgent {
    provider: Arc<dyn LlmProvider>,
}

impl PlannerAgent {
    pub fn new(provider: Arc<dyn LlmProvider>) -> Self {
        Self { provider }
    }
}

#[async_trait]
impl Agent for PlannerAgent {
    fn kind(&self) -> AgentKind {
        AgentKind::Planner
    }

    async fn run(&self, request: AgentRequest) -> AppResult<AgentResponse> {
        let mut messages = vec![ChatMessage::system(PromptManager::system_prompt(self.kind()))];
        messages.extend(request.messages.clone());

        let completion = self
            .provider
            .complete(CompletionRequest {
                messages,
                temperature: 0.2,
                max_tokens: 2048,
            })
            .await?;

        let follow_ups = extract_questions(&completion.content);

        Ok(AgentResponse {
            request_id: request.id,
            agent: self.kind(),
            content: completion.content,
            files: vec![],
            findings: vec![],
            follow_ups,
            model: completion.model,
            created_at: Utc::now(),
            usage: completion.usage,
        })
    }
}

fn extract_questions(content: &str) -> Vec<String> {
    if let Ok(value) = serde_json::from_str::<serde_json::Value>(content) {
        if let Some(arr) = value.get("questions").and_then(|q| q.as_array()) {
            return arr
                .iter()
                .filter_map(|v| v.as_str().map(|s| s.to_string()))
                .collect();
        }
    }
    vec![]
}
