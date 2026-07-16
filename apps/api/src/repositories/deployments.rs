use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use tempoforge_common::{AppError, AppResult};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, sqlx::FromRow)]
pub struct DeploymentRow {
    pub id: Uuid,
    pub project_id: Uuid,
    pub contract_id: Option<Uuid>,
    pub network: String,
    pub status: String,
    pub address: Option<String>,
    pub tx_hash: Option<String>,
    pub artifact: serde_json::Value,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Deserialize)]
pub struct CreateDeployment {
    pub project_id: Uuid,
    pub network: String,
    pub contract_name: String,
    pub contract_id: Option<Uuid>,
    pub artifact: serde_json::Value,
}

pub async fn create_deployment(pool: &PgPool, input: CreateDeployment) -> AppResult<DeploymentRow> {
    let mut artifact = input.artifact;
    if artifact.get("contract_name").is_none() {
        artifact["contract_name"] = serde_json::json!(input.contract_name);
    }

    sqlx::query_as::<_, DeploymentRow>(
        r#"
        INSERT INTO deployments (project_id, contract_id, network, status, artifact)
        VALUES ($1, $2, $3::network_env, 'pending', $4)
        RETURNING
          id, project_id, contract_id,
          network::text as network,
          status::text as status,
          address, tx_hash, artifact, created_at, updated_at
        "#,
    )
    .bind(input.project_id)
    .bind(input.contract_id)
    .bind(&input.network)
    .bind(artifact)
    .fetch_one(pool)
    .await
    .map_err(|e| AppError::Internal(format!("create deployment failed: {e}")))
}

pub async fn list_deployments(
    pool: &PgPool,
    project_id: Option<Uuid>,
    limit: i64,
) -> AppResult<Vec<DeploymentRow>> {
    if let Some(project_id) = project_id {
        sqlx::query_as::<_, DeploymentRow>(
            r#"
            SELECT
              id, project_id, contract_id,
              network::text as network,
              status::text as status,
              address, tx_hash, artifact, created_at, updated_at
            FROM deployments
            WHERE project_id = $1
            ORDER BY created_at DESC
            LIMIT $2
            "#,
        )
        .bind(project_id)
        .bind(limit)
        .fetch_all(pool)
        .await
        .map_err(|e| AppError::Internal(format!("list deployments failed: {e}")))
    } else {
        sqlx::query_as::<_, DeploymentRow>(
            r#"
            SELECT
              id, project_id, contract_id,
              network::text as network,
              status::text as status,
              address, tx_hash, artifact, created_at, updated_at
            FROM deployments
            ORDER BY created_at DESC
            LIMIT $1
            "#,
        )
        .bind(limit)
        .fetch_all(pool)
        .await
        .map_err(|e| AppError::Internal(format!("list deployments failed: {e}")))
    }
}

pub async fn update_deployment(
    pool: &PgPool,
    id: Uuid,
    status: &str,
    address: Option<String>,
    tx_hash: Option<String>,
) -> AppResult<DeploymentRow> {
    sqlx::query_as::<_, DeploymentRow>(
        r#"
        UPDATE deployments
        SET
          status = $2::deployment_status,
          address = COALESCE($3, address),
          tx_hash = COALESCE($4, tx_hash),
          updated_at = NOW()
        WHERE id = $1
        RETURNING
          id, project_id, contract_id,
          network::text as network,
          status::text as status,
          address, tx_hash, artifact, created_at, updated_at
        "#,
    )
    .bind(id)
    .bind(status)
    .bind(address)
    .bind(tx_hash)
    .fetch_optional(pool)
    .await
    .map_err(|e| AppError::Internal(format!("update deployment failed: {e}")))?
    .ok_or_else(|| AppError::NotFound("deployment not found".into()))
}

pub async fn get_deployment(pool: &PgPool, id: Uuid) -> AppResult<DeploymentRow> {
    sqlx::query_as::<_, DeploymentRow>(
        r#"
        SELECT
          id, project_id, contract_id,
          network::text as network,
          status::text as status,
          address, tx_hash, artifact, created_at, updated_at
        FROM deployments
        WHERE id = $1
        "#,
    )
    .bind(id)
    .fetch_optional(pool)
    .await
    .map_err(|e| AppError::Internal(format!("get deployment failed: {e}")))?
    .ok_or_else(|| AppError::NotFound("deployment not found".into()))
}
