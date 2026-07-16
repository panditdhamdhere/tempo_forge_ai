use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use tempoforge_ai_engine::types::{ChatMessage, Role};
use tempoforge_common::{AppError, AppResult};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, sqlx::FromRow)]
pub struct ConversationRow {
    pub id: Uuid,
    pub org_id: Option<Uuid>,
    pub project_id: Option<Uuid>,
    pub user_id: Uuid,
    pub agent: String,
    pub title: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, sqlx::FromRow)]
pub struct MessageRow {
    pub id: Uuid,
    pub conversation_id: Uuid,
    pub role: String,
    pub content: String,
    pub metadata: serde_json::Value,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Deserialize)]
pub struct CreateConversation {
    pub user_id: Uuid,
    pub agent: String,
    pub title: String,
    pub project_id: Option<Uuid>,
    pub org_id: Option<Uuid>,
}

pub async fn create_conversation(
    pool: &PgPool,
    input: CreateConversation,
) -> AppResult<ConversationRow> {
    sqlx::query_as::<_, ConversationRow>(
        r#"
        INSERT INTO ai_conversations (org_id, project_id, user_id, agent, title)
        VALUES ($1, $2, $3, $4, $5)
        RETURNING id, org_id, project_id, user_id, agent, title, created_at, updated_at
        "#,
    )
    .bind(input.org_id)
    .bind(input.project_id)
    .bind(input.user_id)
    .bind(&input.agent)
    .bind(&input.title)
    .fetch_one(pool)
    .await
    .map_err(|e| AppError::Internal(format!("create conversation failed: {e}")))
}

pub async fn get_conversation(
    pool: &PgPool,
    id: Uuid,
    user_id: Uuid,
) -> AppResult<ConversationRow> {
    sqlx::query_as::<_, ConversationRow>(
        r#"
        SELECT id, org_id, project_id, user_id, agent, title, created_at, updated_at
        FROM ai_conversations
        WHERE id = $1 AND user_id = $2
        "#,
    )
    .bind(id)
    .bind(user_id)
    .fetch_optional(pool)
    .await
    .map_err(|e| AppError::Internal(format!("get conversation failed: {e}")))?
    .ok_or_else(|| AppError::NotFound("conversation not found".into()))
}

pub async fn list_conversations(
    pool: &PgPool,
    user_id: Uuid,
    limit: i64,
) -> AppResult<Vec<ConversationRow>> {
    sqlx::query_as::<_, ConversationRow>(
        r#"
        SELECT id, org_id, project_id, user_id, agent, title, created_at, updated_at
        FROM ai_conversations
        WHERE user_id = $1
        ORDER BY updated_at DESC
        LIMIT $2
        "#,
    )
    .bind(user_id)
    .bind(limit)
    .fetch_all(pool)
    .await
    .map_err(|e| AppError::Internal(format!("list conversations failed: {e}")))
}

pub async fn list_messages(pool: &PgPool, conversation_id: Uuid) -> AppResult<Vec<MessageRow>> {
    sqlx::query_as::<_, MessageRow>(
        r#"
        SELECT id, conversation_id, role, content, metadata, created_at
        FROM ai_messages
        WHERE conversation_id = $1
        ORDER BY created_at ASC
        "#,
    )
    .bind(conversation_id)
    .fetch_all(pool)
    .await
    .map_err(|e| AppError::Internal(format!("list messages failed: {e}")))
}

pub async fn append_message(
    pool: &PgPool,
    conversation_id: Uuid,
    role: &str,
    content: &str,
    metadata: serde_json::Value,
) -> AppResult<MessageRow> {
    let row = sqlx::query_as::<_, MessageRow>(
        r#"
        INSERT INTO ai_messages (conversation_id, role, content, metadata)
        VALUES ($1, $2, $3, $4)
        RETURNING id, conversation_id, role, content, metadata, created_at
        "#,
    )
    .bind(conversation_id)
    .bind(role)
    .bind(content)
    .bind(metadata)
    .fetch_one(pool)
    .await
    .map_err(|e| AppError::Internal(format!("append message failed: {e}")))?;

    sqlx::query("UPDATE ai_conversations SET updated_at = NOW() WHERE id = $1")
        .bind(conversation_id)
        .execute(pool)
        .await
        .ok();

    Ok(row)
}

pub fn rows_to_chat_messages(rows: &[MessageRow]) -> Vec<ChatMessage> {
    rows.iter()
        .filter_map(|row| {
            let role = match row.role.as_str() {
                "system" => Role::System,
                "assistant" => Role::Assistant,
                "tool" => Role::Tool,
                "user" => Role::User,
                _ => return None,
            };
            Some(ChatMessage {
                role,
                content: row.content.clone(),
                name: None,
            })
        })
        .collect()
}

pub fn title_from_prompt(prompt: &str) -> String {
    let trimmed = prompt.trim().replace('\n', " ");
    if trimmed.chars().count() <= 72 {
        trimmed
    } else {
        format!("{}…", trimmed.chars().take(71).collect::<String>())
    }
}
