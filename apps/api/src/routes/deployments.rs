use crate::middleware::auth::AuthUser;
use crate::repositories::deployments::{self, CreateDeployment};
use crate::state::AppState;
use axum::{
    Json,
    extract::{Path, Query, State},
};
use serde::Deserialize;
use tempoforge_blockchain::Network;
use tempoforge_common::{ApiResponse, AppError, AppResult};
use tempoforge_deployment::{DeploymentPlan, DeploymentService};
use uuid::Uuid;
use validator::Validate;

#[derive(Debug, Deserialize, Validate)]
pub struct PlanRequest {
    #[validate(length(min = 1, max = 80))]
    pub contract_name: String,
    #[serde(default = "default_network")]
    pub network: String,
}

fn default_network() -> String {
    "testnet".into()
}

pub async fn plan_deployment(
    AuthUser(_user): AuthUser,
    Json(body): Json<PlanRequest>,
) -> AppResult<ApiResponse<DeploymentPlan>> {
    body.validate()?;
    let network = parse_network(&body.network);
    Ok(ApiResponse::new(DeploymentService::plan(
        network,
        &body.contract_name,
    )))
}

#[derive(Debug, Deserialize, Validate)]
pub struct CreateDeploymentRequest {
    pub project_id: Uuid,
    #[validate(length(min = 1, max = 80))]
    pub contract_name: String,
    #[serde(default = "default_network")]
    pub network: String,
    pub contract_id: Option<Uuid>,
    #[serde(default)]
    pub artifact: serde_json::Value,
}

pub async fn create_deployment(
    State(state): State<AppState>,
    AuthUser(user): AuthUser,
    Json(body): Json<CreateDeploymentRequest>,
) -> AppResult<ApiResponse<deployments::DeploymentRow>> {
    body.validate()?;
    let network = parse_network(&body.network);
    let plan = DeploymentService::plan(network, &body.contract_name);

    let mut artifact = if body.artifact.is_null() {
        serde_json::json!({})
    } else {
        body.artifact
    };
    artifact["plan"] = serde_json::to_value(&plan).unwrap_or_default();

    let row = deployments::create_deployment(
        &state.db,
        CreateDeployment {
            project_id: body.project_id,
            network: body.network,
            contract_name: body.contract_name,
            contract_id: body.contract_id,
            artifact,
        },
    )
    .await?;

    sqlx::query(
        r#"
        INSERT INTO activity_logs (user_id, action, resource_type, resource_id, metadata)
        VALUES ($1, 'deployment.created', 'deployment', $2, $3)
        "#,
    )
    .bind(user.user_id.as_uuid())
    .bind(row.id.to_string())
    .bind(serde_json::json!({ "project_id": row.project_id }))
    .execute(&state.db)
    .await
    .ok();

    Ok(ApiResponse::new(row))
}

#[derive(Debug, Deserialize)]
pub struct ListDeploymentsQuery {
    pub project_id: Option<Uuid>,
}

pub async fn list_deployments(
    State(state): State<AppState>,
    AuthUser(_user): AuthUser,
    Query(query): Query<ListDeploymentsQuery>,
) -> AppResult<ApiResponse<Vec<deployments::DeploymentRow>>> {
    let rows = deployments::list_deployments(&state.db, query.project_id, 100).await?;
    Ok(ApiResponse::new(rows))
}

#[derive(Debug, Deserialize, Validate)]
pub struct UpdateDeploymentRequest {
    #[validate(length(min = 1, max = 32))]
    pub status: String,
    pub address: Option<String>,
    pub tx_hash: Option<String>,
}

pub async fn update_deployment(
    State(state): State<AppState>,
    AuthUser(user): AuthUser,
    Path(id): Path<Uuid>,
    Json(body): Json<UpdateDeploymentRequest>,
) -> AppResult<ApiResponse<deployments::DeploymentRow>> {
    body.validate()?;
    validate_status(&body.status)?;

    let row = deployments::update_deployment(
        &state.db,
        id,
        &body.status,
        body.address,
        body.tx_hash,
    )
    .await?;

    sqlx::query(
        r#"
        INSERT INTO activity_logs (user_id, action, resource_type, resource_id, metadata)
        VALUES ($1, 'deployment.updated', 'deployment', $2, $3)
        "#,
    )
    .bind(user.user_id.as_uuid())
    .bind(row.id.to_string())
    .bind(serde_json::json!({ "status": row.status }))
    .execute(&state.db)
    .await
    .ok();

    Ok(ApiResponse::new(row))
}

pub async fn get_deployment(
    State(state): State<AppState>,
    AuthUser(_user): AuthUser,
    Path(id): Path<Uuid>,
) -> AppResult<ApiResponse<deployments::DeploymentRow>> {
    let row = deployments::get_deployment(&state.db, id).await?;
    Ok(ApiResponse::new(row))
}

fn parse_network(raw: &str) -> Network {
    match raw {
        "mainnet" => Network::Mainnet,
        "local" => Network::Local,
        _ => Network::Testnet,
    }
}

fn validate_status(status: &str) -> AppResult<()> {
    match status {
        "pending" | "submitted" | "confirmed" | "verified" | "failed" => Ok(()),
        other => Err(AppError::Validation(format!("invalid status: {other}"))),
    }
}
