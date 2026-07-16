use crate::middleware::auth::AuthUser;
use crate::state::AppState;
use axum::{Json, extract::State};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use tempoforge_common::{ApiResponse, AppError, AppResult, ProjectId};
use uuid::Uuid;
use validator::Validate;

#[derive(Debug, Serialize, sqlx::FromRow)]
pub struct ProjectRow {
    pub id: Uuid,
    pub org_id: Uuid,
    pub name: String,
    pub slug: String,
    pub description: String,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Deserialize, Validate)]
pub struct CreateProjectRequest {
    #[validate(length(min = 2, max = 80))]
    pub name: String,
    #[validate(length(min = 2, max = 80))]
    pub slug: String,
    #[serde(default)]
    pub description: String,
    pub org_id: Option<Uuid>,
}

pub async fn list_projects(
    State(state): State<AppState>,
    AuthUser(_user): AuthUser,
) -> AppResult<ApiResponse<Vec<ProjectRow>>> {
    let rows = sqlx::query_as::<_, ProjectRow>(
        r#"
        SELECT id, org_id, name, slug, description, created_at
        FROM projects
        ORDER BY created_at DESC
        LIMIT 100
        "#,
    )
    .fetch_all(&state.db)
    .await
    .map_err(|e| AppError::Internal(format!("list projects failed: {e}")))?;

    Ok(ApiResponse::new(rows))
}

pub async fn create_project(
    State(state): State<AppState>,
    AuthUser(user): AuthUser,
    Json(body): Json<CreateProjectRequest>,
) -> AppResult<ApiResponse<ProjectRow>> {
    body.validate()?;

    let org_id = match body.org_id {
        Some(id) => id,
        None => ensure_personal_org(&state, &user).await?,
    };

    let id = ProjectId::new().as_uuid();
    let row = sqlx::query_as::<_, ProjectRow>(
        r#"
        INSERT INTO projects (id, org_id, name, slug, description, created_by)
        VALUES ($1, $2, $3, $4, $5, $6)
        RETURNING id, org_id, name, slug, description, created_at
        "#,
    )
    .bind(id)
    .bind(org_id)
    .bind(&body.name)
    .bind(&body.slug)
    .bind(&body.description)
    .bind(user.user_id.as_uuid())
    .fetch_one(&state.db)
    .await
    .map_err(|e| {
        if e.to_string().contains("duplicate") || e.to_string().contains("unique") {
            AppError::Conflict("project slug already exists in organization".into())
        } else {
            AppError::Internal(format!("create project failed: {e}"))
        }
    })?;

    sqlx::query(
        r#"
        INSERT INTO activity_logs (org_id, user_id, action, resource_type, resource_id)
        VALUES ($1, $2, 'project.created', 'project', $3)
        "#,
    )
    .bind(org_id)
    .bind(user.user_id.as_uuid())
    .bind(row.id.to_string())
    .execute(&state.db)
    .await
    .ok();

    Ok(ApiResponse::new(row))
}

async fn ensure_personal_org(
    state: &AppState,
    user: &tempoforge_auth::AuthContext,
) -> AppResult<Uuid> {
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
