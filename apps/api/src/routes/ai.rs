use crate::middleware::auth::AuthUser;
use crate::repositories::conversations::{
    self, CreateConversation, rows_to_chat_messages, title_from_prompt,
};
use crate::state::AppState;
use axum::{
    Json,
    extract::{Path, State},
};
use serde::{Deserialize, Serialize};
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

#[derive(Debug, Serialize)]
pub struct AgentRunResult {
    pub conversation_id: Uuid,
    pub response: AgentResponse,
}

pub async fn run_agent(
    State(state): State<AppState>,
    AuthUser(user): AuthUser,
    Path(agent): Path<String>,
    Json(body): Json<AgentRunBody>,
) -> AppResult<ApiResponse<AgentRunResult>> {
    body.validate()?;
    persist_and_run(
        &state,
        user.user_id.as_uuid(),
        &agent,
        body.prompt,
        body.project_id,
        body.conversation_id,
        body.context,
        body.history,
    )
    .await
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
) -> AppResult<ApiResponse<AgentRunResult>> {
    body.validate()?;
    persist_and_run(
        &state,
        user.user_id.as_uuid(),
        "chat",
        body.message,
        body.project_id,
        body.conversation_id,
        serde_json::json!({}),
        vec![],
    )
    .await
}

pub async fn list_conversations(
    State(state): State<AppState>,
    AuthUser(user): AuthUser,
) -> AppResult<ApiResponse<Vec<conversations::ConversationRow>>> {
    let rows = conversations::list_conversations(&state.db, user.user_id.as_uuid(), 50).await?;
    Ok(ApiResponse::new(rows))
}

pub async fn get_conversation_messages(
    State(state): State<AppState>,
    AuthUser(user): AuthUser,
    Path(id): Path<Uuid>,
) -> AppResult<ApiResponse<Vec<conversations::MessageRow>>> {
    conversations::get_conversation(&state.db, id, user.user_id.as_uuid()).await?;
    let rows = conversations::list_messages(&state.db, id).await?;
    Ok(ApiResponse::new(rows))
}

#[allow(clippy::too_many_arguments)]
async fn persist_and_run(
    state: &AppState,
    user_id: Uuid,
    agent: &str,
    prompt: String,
    project_id: Option<Uuid>,
    conversation_id: Option<Uuid>,
    context: serde_json::Value,
    inline_history: Vec<ChatMessage>,
) -> AppResult<ApiResponse<AgentRunResult>> {
    let conversation = if let Some(id) = conversation_id {
        conversations::get_conversation(&state.db, id, user_id).await?
    } else {
        conversations::create_conversation(
            &state.db,
            CreateConversation {
                user_id,
                agent: agent.to_string(),
                title: title_from_prompt(&prompt),
                project_id,
                org_id: None,
            },
        )
        .await?
    };

    let mut messages = if conversation_id.is_some() {
        let rows = conversations::list_messages(&state.db, conversation.id).await?;
        rows_to_chat_messages(&rows)
    } else {
        inline_history
    };
    messages.push(ChatMessage::user(prompt.clone()));

    conversations::append_message(
        &state.db,
        conversation.id,
        "user",
        &prompt,
        serde_json::json!({}),
    )
    .await?;

    let request = AgentRequest {
        id: Uuid::new_v4(),
        agent: AgentKind::Chat,
        user_id: tempoforge_common::UserId(user_id),
        project_id: project_id.map(ProjectId),
        conversation_id: Some(ConversationId(conversation.id)),
        messages,
        context: if context.is_null() {
            serde_json::json!({})
        } else {
            context
        },
    };

    let response = state
        .ai
        .route_by_name(agent, request)
        .await
        .map_err(|e| AppError::Upstream(e.to_string()))?;

    conversations::append_message(
        &state.db,
        conversation.id,
        "assistant",
        &response.content,
        serde_json::json!({
            "model": response.model,
            "files": response.files,
            "findings": response.findings,
            "usage": response.usage,
        }),
    )
    .await?;

    Ok(ApiResponse::new(AgentRunResult {
        conversation_id: conversation.id,
        response,
    }))
}
