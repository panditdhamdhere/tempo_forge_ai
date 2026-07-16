use crate::middleware::auth::AuthUser;
use crate::state::AppState;
use axum::{
    Json,
    extract::{Path, State},
};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use tempoforge_auth::{decrypt_secret, encrypt_secret};
use tempoforge_common::{ApiResponse, AppError, AppResult};
use uuid::Uuid;
use validator::Validate;

#[derive(Debug, Serialize, sqlx::FromRow)]
pub struct SecretMeta {
    pub id: Uuid,
    pub name: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Deserialize, Validate)]
pub struct UpsertSecretRequest {
    #[validate(length(min = 2, max = 80))]
    pub name: String,
    #[validate(length(min = 1, max = 10_000))]
    pub value: String,
    pub org_id: Option<Uuid>,
}

#[derive(Debug, Serialize)]
pub struct RevealedSecret {
    pub name: String,
    pub value: String,
}

pub async fn list_secrets(
    State(state): State<AppState>,
    AuthUser(user): AuthUser,
) -> AppResult<ApiResponse<Vec<SecretMeta>>> {
    let org_id = ensure_org(&state, &user).await?;
    let rows = sqlx::query_as::<_, SecretMeta>(
        r#"
        SELECT id, name, created_at, updated_at
        FROM org_secrets
        WHERE org_id = $1
        ORDER BY name ASC
        "#,
    )
    .bind(org_id)
    .fetch_all(&state.db)
    .await
    .map_err(|e| AppError::Internal(format!("list secrets failed: {e}")))?;
    Ok(ApiResponse::new(rows))
}

pub async fn upsert_secret(
    State(state): State<AppState>,
    AuthUser(user): AuthUser,
    Json(body): Json<UpsertSecretRequest>,
) -> AppResult<ApiResponse<SecretMeta>> {
    body.validate()?;
    let org_id = body.org_id.unwrap_or(ensure_org(&state, &user).await?);
    let ciphertext = encrypt_secret(&body.value, &state.config.encryption_key)?;

    let row = sqlx::query_as::<_, SecretMeta>(
        r#"
        INSERT INTO org_secrets (org_id, name, ciphertext, created_by)
        VALUES ($1, $2, $3, $4)
        ON CONFLICT (org_id, name) DO UPDATE
          SET ciphertext = EXCLUDED.ciphertext,
              updated_at = NOW()
        RETURNING id, name, created_at, updated_at
        "#,
    )
    .bind(org_id)
    .bind(&body.name)
    .bind(ciphertext)
    .bind(user.user_id.as_uuid())
    .fetch_one(&state.db)
    .await
    .map_err(|e| AppError::Internal(format!("upsert secret failed: {e}")))?;

    Ok(ApiResponse::new(row))
}

pub async fn reveal_secret(
    State(state): State<AppState>,
    AuthUser(user): AuthUser,
    Path(name): Path<String>,
) -> AppResult<ApiResponse<RevealedSecret>> {
    let org_id = ensure_org(&state, &user).await?;
    let ciphertext = sqlx::query_scalar::<_, String>(
        "SELECT ciphertext FROM org_secrets WHERE org_id = $1 AND name = $2",
    )
    .bind(org_id)
    .bind(&name)
    .fetch_optional(&state.db)
    .await
    .map_err(|e| AppError::Internal(format!("reveal secret failed: {e}")))?
    .ok_or_else(|| AppError::NotFound("secret not found".into()))?;

    let value = decrypt_secret(&ciphertext, &state.config.encryption_key)?;

    sqlx::query(
        r#"
        INSERT INTO activity_logs (org_id, user_id, action, resource_type, resource_id)
        VALUES ($1, $2, 'secret.revealed', 'org_secret', $3)
        "#,
    )
    .bind(org_id)
    .bind(user.user_id.as_uuid())
    .bind(&name)
    .execute(&state.db)
    .await
    .ok();

    Ok(ApiResponse::new(RevealedSecret { name, value }))
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

    Ok(id)
}
