use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use tempoforge_common::{ConversationId, ProjectId, UserId};
use uuid::Uuid;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AgentKind {
    Planner,
    CodeGenerator,
    Auditor,
    Debugger,
    Architect,
    DocumentationWriter,
    TestGenerator,
    DeploymentAssistant,
    Chat,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Role {
    System,
    User,
    Assistant,
    Tool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatMessage {
    pub role: Role,
    pub content: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
}

impl ChatMessage {
    pub fn system(content: impl Into<String>) -> Self {
        Self {
            role: Role::System,
            content: content.into(),
            name: None,
        }
    }

    pub fn user(content: impl Into<String>) -> Self {
        Self {
            role: Role::User,
            content: content.into(),
            name: None,
        }
    }

    pub fn assistant(content: impl Into<String>) -> Self {
        Self {
            role: Role::Assistant,
            content: content.into(),
            name: None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentRequest {
    pub id: Uuid,
    pub agent: AgentKind,
    pub user_id: UserId,
    pub project_id: Option<ProjectId>,
    pub conversation_id: Option<ConversationId>,
    pub messages: Vec<ChatMessage>,
    pub context: serde_json::Value,
}

impl AgentRequest {
    pub fn new(agent: AgentKind, user_id: UserId, prompt: impl Into<String>) -> Self {
        Self {
            id: Uuid::new_v4(),
            agent,
            user_id,
            project_id: None,
            conversation_id: None,
            messages: vec![ChatMessage::user(prompt)],
            context: serde_json::json!({}),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GeneratedFile {
    pub path: String,
    pub content: String,
    pub language: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Finding {
    pub severity: Severity,
    pub title: String,
    pub description: String,
    pub location: Option<String>,
    pub recommendation: String,
    pub diff: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Severity {
    Critical,
    High,
    Medium,
    Low,
    Info,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentResponse {
    pub request_id: Uuid,
    pub agent: AgentKind,
    pub content: String,
    pub files: Vec<GeneratedFile>,
    pub findings: Vec<Finding>,
    pub follow_ups: Vec<String>,
    pub model: String,
    pub created_at: DateTime<Utc>,
    pub usage: TokenUsage,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct TokenUsage {
    pub prompt_tokens: u32,
    pub completion_tokens: u32,
    pub total_tokens: u32,
}
