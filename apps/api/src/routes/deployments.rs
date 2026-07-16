use crate::middleware::auth::AuthUser;
use axum::Json;
use serde::Deserialize;
use tempoforge_blockchain::Network;
use tempoforge_common::{ApiResponse, AppResult};
use tempoforge_deployment::{DeploymentPlan, DeploymentService};
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
    let network = match body.network.as_str() {
        "mainnet" => Network::Mainnet,
        "local" => Network::Local,
        _ => Network::Testnet,
    };
    Ok(ApiResponse::new(DeploymentService::plan(
        network,
        &body.contract_name,
    )))
}
