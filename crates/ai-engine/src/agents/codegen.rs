use super::traits::Agent;
use crate::prompts::PromptManager;
use crate::providers::{CompletionRequest, LlmProvider};
use crate::rag::RagStore;
use crate::types::{AgentKind, AgentRequest, AgentResponse, ChatMessage, GeneratedFile};
use async_trait::async_trait;
use chrono::Utc;
use regex::Regex;
use std::sync::Arc;
use tempoforge_common::AppResult;

pub struct CodeGeneratorAgent {
    provider: Arc<dyn LlmProvider>,
    rag: Arc<dyn RagStore>,
}

impl CodeGeneratorAgent {
    pub fn new(provider: Arc<dyn LlmProvider>, rag: Arc<dyn RagStore>) -> Self {
        Self { provider, rag }
    }
}

#[async_trait]
impl Agent for CodeGeneratorAgent {
    fn kind(&self) -> AgentKind {
        AgentKind::CodeGenerator
    }

    async fn run(&self, request: AgentRequest) -> AppResult<AgentResponse> {
        let rag_hits = self.rag.search("tempo", "solidity foundry tip-20", 4).await?;
        let rag_block = rag_hits
            .iter()
            .map(|c| format!("- ({}) {}", c.source, c.text))
            .collect::<Vec<_>>()
            .join("\n");

        let mut messages = vec![
            ChatMessage::system(PromptManager::system_prompt(self.kind())),
            ChatMessage::system(format!("Retrieved context:\n{rag_block}")),
        ];
        messages.extend(request.messages.clone());

        let completion = self
            .provider
            .complete(CompletionRequest {
                messages,
                temperature: 0.15,
                max_tokens: 8192,
            })
            .await?;

        let files = extract_files(&completion.content);

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

fn extract_files(content: &str) -> Vec<GeneratedFile> {
    let re = Regex::new(r"(?s)```([a-zA-Z0-9_+-]+)(?:\s+path=([^\n]+))?\n(.*?)```").ok();
    let Some(re) = re else {
        return vec![];
    };

    re.captures_iter(content)
        .filter_map(|cap| {
            let language = cap.get(1)?.as_str().to_string();
            let path = cap
                .get(2)
                .map(|m| m.as_str().trim().to_string())
                .unwrap_or_else(|| default_path(&language));
            let body = cap.get(3)?.as_str().to_string();
            Some(GeneratedFile {
                path,
                content: body,
                language,
            })
        })
        .collect()
}

fn default_path(language: &str) -> String {
    match language {
        "solidity" => "contracts/Generated.sol".into(),
        "typescript" | "ts" => "frontend/src/generated.ts".into(),
        "markdown" | "md" => "README.md".into(),
        "bash" | "shell" => "scripts/deploy.sh".into(),
        _ => format!("generated.{language}"),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn extracts_pathed_files() {
        let sample = r#"
```solidity path=contracts/Token.sol
pragma solidity ^0.8.24;
```
```markdown path=README.md
# Token
```
"#;
        let files = extract_files(sample);
        assert_eq!(files.len(), 2);
        assert_eq!(files[0].path, "contracts/Token.sol");
        assert_eq!(files[1].path, "README.md");
    }
}
