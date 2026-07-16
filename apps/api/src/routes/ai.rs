use crate::middleware::auth::AuthUser;
use crate::state::AppState;
use axum::{
    Json,
    extract::{Path, State},
};
use serde::Deserialize;
use tempoforge_ai_engine::types::{AgentKind, AgentRequest, AgentResponse, ChatMessage};
use tempoforge_common::{ApiResponse, AppError, AppResult, ConversationId, ProjectId};
use uuid::Uuid;
use validator::Validate;

#[derive(Debug, Deserialize, Validate)]
pub struct AgentRunBody {
    #[validate(length(min = 1, max = 100_000))]
    pub prompt: String,
    pub project_id: Option<Uuid>,
    pub conversation_id: Option<Uuid>,
    #[serde(default)]
    pub context: serde_json::Value,
    #[serde(default)]
    pub history: Vec<ChatMessage>,
}

pub async fn run_agent(
    State(state): State<AppState>,
    AuthUser(user): AuthUser,
    Path(agent): Path<String>,
    Json(body): Json<AgentRunBody>,
) -> AppResult<ApiResponse<AgentResponse>> {
    body.validate()?;

    let mut messages = body.history;
    messages.push(ChatMessage::user(body.prompt.clone()));

    let request = AgentRequest {
        id: Uuid::new_v4(),
        agent: AgentKind::Chat,
        user_id: user.user_id,
        project_id: body.project_id.map(ProjectId),
        conversation_id: body.conversation_id.map(ConversationId),
        messages,
        context: if body.context.is_null() {
            serde_json::json!({})
        } else {
            body.context
        },
    };

    let response = state.ai.route_by_name(&agent, request).await?;
    Ok(ApiResponse::new(response))
}

#[derive(Debug, Deserialize, Validate)]
pub struct ChatBody {
    #[validate(length(min = 1, max = 100_000))]
    pub message: String,
    pub conversation_id: Option<Uuid>,
    pub project_id: Option<Uuid>,
}

pub async fn chat(
    State(state): State<AppState>,
    AuthUser(user): AuthUser,
    Json(body): Json<ChatBody>,
) -> AppResult<ApiResponse<AgentResponse>> {
    body.validate()?;
    let request = AgentRequest {
        id: Uuid::new_v4(),
        agent: AgentKind::Chat,
        user_id: user.user_id,
        project_id: body.project_id.map(ProjectId),
        conversation_id: body.conversation_id.map(ConversationId),
        messages: vec![ChatMessage::user(body.message)],
        context: serde_json::json!({}),
    };

    let response = state
        .ai
        .run(request)
        .await
        .map_err(|e| AppError::Upstream(e.to_string()))?;

    Ok(ApiResponse::new(response))
}
