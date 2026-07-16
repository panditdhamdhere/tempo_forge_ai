use super::traits::Agent;
use crate::prompts::PromptManager;
use crate::providers::{CompletionRequest, LlmProvider};
use crate::rag::RagStore;
use crate::types::{AgentKind, AgentRequest, AgentResponse, ChatMessage, Finding, Severity};
use async_trait::async_trait;
use chrono::Utc;
use std::sync::Arc;
use tempoforge_common::AppResult;

pub struct AuditorAgent {
    provider: Arc<dyn LlmProvider>,
    rag: Arc<dyn RagStore>,
}

impl AuditorAgent {
    pub fn new(provider: Arc<dyn LlmProvider>, rag: Arc<dyn RagStore>) -> Self {
        Self { provider, rag }
    }
}

#[async_trait]
impl Agent for AuditorAgent {
    fn kind(&self) -> AgentKind {
        AgentKind::Auditor
    }

    async fn run(&self, request: AgentRequest) -> AppResult<AgentResponse> {
        let hints = self
            .rag
            .search("security", "reentrancy access control delegatecall", 5)
            .await?;
        let hint_text = hints
            .iter()
            .map(|h| h.text.clone())
            .collect::<Vec<_>>()
            .join("\n");

        let mut messages = vec![
            ChatMessage::system(PromptManager::system_prompt(self.kind())),
            ChatMessage::system(format!("Security knowledge:\n{hint_text}")),
        ];
        messages.extend(request.messages.clone());

        let completion = self
            .provider
            .complete(CompletionRequest {
                messages,
                temperature: 0.1,
                max_tokens: 6144,
            })
            .await?;

        let findings = parse_findings(&completion.content);

        Ok(AgentResponse {
            request_id: request.id,
            agent: self.kind(),
            content: completion.content,
            files: vec![],
            findings,
            follow_ups: vec![],
            model: completion.model,
            created_at: Utc::now(),
            usage: completion.usage,
        })
    }
}

fn parse_findings(content: &str) -> Vec<Finding> {
    let json_slice = content.find('{').and_then(|start| {
        content.rfind('}').map(|end| &content[start..=end])
    });

    if let Some(slice) = json_slice {
        if let Ok(value) = serde_json::from_str::<serde_json::Value>(slice) {
            if let Some(arr) = value.get("findings").and_then(|f| f.as_array()) {
                return arr.iter().filter_map(finding_from_value).collect();
            }
        }
    }

    // Heuristic fallback when the model returns markdown only.
    if content.to_lowercase().contains("reentrancy") {
        return vec![Finding {
            severity: Severity::High,
            title: "Potential reentrancy".into(),
            description: "Model mentioned reentrancy; review external calls before state updates."
                .into(),
            location: None,
            recommendation: "Apply checks-effects-interactions and ReentrancyGuard.".into(),
            diff: None,
        }];
    }
    vec![]
}

fn finding_from_value(value: &serde_json::Value) -> Option<Finding> {
    Some(Finding {
        severity: parse_severity(value.get("severity").and_then(|s| s.as_str()).unwrap_or("medium")),
        title: value.get("title")?.as_str()?.to_string(),
        description: value
            .get("description")
            .and_then(|s| s.as_str())
            .unwrap_or_default()
            .to_string(),
        location: value
            .get("location")
            .and_then(|s| s.as_str())
            .map(|s| s.to_string()),
        recommendation: value
            .get("recommendation")
            .and_then(|s| s.as_str())
            .unwrap_or_default()
            .to_string(),
        diff: value
            .get("diff")
            .and_then(|s| s.as_str())
            .map(|s| s.to_string()),
    })
}

fn parse_severity(raw: &str) -> Severity {
    match raw.to_lowercase().as_str() {
        "critical" => Severity::Critical,
        "high" => Severity::High,
        "low" => Severity::Low,
        "info" => Severity::Info,
        _ => Severity::Medium,
    }
}
