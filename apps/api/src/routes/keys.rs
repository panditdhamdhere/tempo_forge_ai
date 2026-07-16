use crate::middleware::auth::AuthUser;
use crate::state::AppState;
use axum::{
    Json,
    extract::{Path, State},
};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use tempoforge_auth::{ApiKeyHasher, generate_api_key};
use tempoforge_common::{ApiResponse, AppError, AppResult};
use uuid::Uuid;
use validator::Validate;

#[derive(Debug, Serialize, sqlx::FromRow)]
pub struct ApiKeyView {
    pub id: Uuid,
    pub name: String,
    pub key_prefix: String,
    pub scopes: Vec<String>,
    pub last_used_at: Option<DateTime<Utc>>,
    pub revoked_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Serialize)]
pub struct CreatedApiKey {
    pub key: ApiKeyView,
    /// Raw secret — shown once.
    pub raw_key: String,
}

#[derive(Debug, Deserialize, Validate)]
pub struct CreateKeyRequest {
    #[validate(length(min = 2, max = 80))]
    pub name: String,
    #[serde(default)]
    pub scopes: Vec<String>,
    pub org_id: Option<Uuid>,
}

pub async fn list_keys(
    State(state): State<AppState>,
    AuthUser(user): AuthUser,
) -> AppResult<ApiResponse<Vec<ApiKeyView>>> {
    let org_id = ensure_org(&state, &user).await?;
    let rows = sqlx::query_as::<_, ApiKeyView>(
        r#"
        SELECT id, name, key_prefix, scopes, last_used_at, revoked_at, created_at
        FROM api_keys
        WHERE org_id = $1
        ORDER BY created_at DESC
        "#,
    )
    .bind(org_id)
    .fetch_all(&state.db)
    .await
    .map_err(|e| AppError::Internal(format!("list api keys failed: {e}")))?;
    Ok(ApiResponse::new(rows))
}

pub async fn create_key(
    State(state): State<AppState>,
    AuthUser(user): AuthUser,
    Json(body): Json<CreateKeyRequest>,
) -> AppResult<ApiResponse<CreatedApiKey>> {
    body.validate()?;
    let org_id = body.org_id.unwrap_or(ensure_org(&state, &user).await?);
    let (raw, prefix, hash) = generate_api_key();
    let scopes = if body.scopes.is_empty() {
        vec!["api:read".into(), "api:write".into()]
    } else {
        body.scopes
    };

    let row = sqlx::query_as::<_, ApiKeyView>(
        r#"
        INSERT INTO api_keys (org_id, created_by, name, key_prefix, key_hash, scopes)
        VALUES ($1, $2, $3, $4, $5, $6)
        RETURNING id, name, key_prefix, scopes, last_used_at, revoked_at, created_at
        "#,
    )
    .bind(org_id)
    .bind(user.user_id.as_uuid())
    .bind(&body.name)
    .bind(&prefix)
    .bind(&hash)
    .bind(&scopes)
    .fetch_one(&state.db)
    .await
    .map_err(|e| AppError::Internal(format!("create api key failed: {e}")))?;

    // Ensure hasher path is used (hash already stored).
    debug_assert!(ApiKeyHasher::verify(&raw, &hash));

    Ok(ApiResponse::new(CreatedApiKey {
        key: row,
        raw_key: raw,
    }))
}

pub async fn revoke_key(
    State(state): State<AppState>,
    AuthUser(user): AuthUser,
    Path(id): Path<Uuid>,
) -> AppResult<ApiResponse<ApiKeyView>> {
    let org_id = ensure_org(&state, &user).await?;
    let row = sqlx::query_as::<_, ApiKeyView>(
        r#"
        UPDATE api_keys
        SET revoked_at = NOW()
        WHERE id = $1 AND org_id = $2 AND revoked_at IS NULL
        RETURNING id, name, key_prefix, scopes, last_used_at, revoked_at, created_at
        "#,
    )
    .bind(id)
    .bind(org_id)
    .fetch_optional(&state.db)
    .await
    .map_err(|e| AppError::Internal(format!("revoke api key failed: {e}")))?
    .ok_or_else(|| AppError::NotFound("api key not found".into()))?;

    Ok(ApiResponse::new(row))
}

async fn ensure_org(state: &AppState, user: &tempoforge_auth::AuthContext) -> AppResult<Uuid> {
    let slug = format!("personal-{}", &user.clerk_user_id);
    let id = sqlx::query_scalar::<_, Uuid>(
        r#"
        INSERT INTO organizations (name, slug)
        VALUES ($1, $2)
        ON CONFLICT (slug) DO UPDATE SET updated_at = NOW()
        RETURNING id
        "#,
    )
    .bind("Personal")
    .bind(&slug)
    .fetch_one(&state.db)
    .await
    .map_err(|e| AppError::Internal(format!("org ensure failed: {e}")))?;

    sqlx::query(
        r#"
        INSERT INTO organization_members (org_id, user_id, role)
        VALUES ($1, $2, 'owner')
        ON CONFLICT DO NOTHING
        "#,
    )
    .bind(id)
    .bind(user.user_id.as_uuid())
    .execute(&state.db)
    .await
    .ok();

    Ok(id)
}
