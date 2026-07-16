use crate::state::AppState;
use axum::extract::{Path, Query, State};
use serde::Deserialize;
use tempoforge_common::{ApiResponse, AppResult};

pub async fn get_transaction(
    State(state): State<AppState>,
    Path(hash): Path<String>,
) -> AppResult<ApiResponse<tempoforge_explorer::TransactionView>> {
    let view = state.explorer.get_transaction(&hash).await?;
    Ok(ApiResponse::new(view))
}

#[derive(Debug, Deserialize)]
pub struct AddressQuery {
    #[serde(default)]
    pub tokens: Option<String>,
}

pub async fn get_address(
    State(state): State<AppState>,
    Path(address): Path<String>,
    Query(query): Query<AddressQuery>,
) -> AppResult<ApiResponse<tempoforge_explorer::AddressView>> {
    let tokens = query
        .tokens
        .map(|s| {
            s.split(',')
                .map(|t| t.trim().to_string())
                .filter(|t| !t.is_empty())
                .collect::<Vec<_>>()
        })
        .unwrap_or_default();

    let view = state.explorer.get_address(&address, &tokens).await?;
    Ok(ApiResponse::new(view))
}

pub async fn latest_block(
    State(state): State<AppState>,
) -> AppResult<ApiResponse<serde_json::Value>> {
    let number = state.explorer.latest_block_number().await?;
    let block = state.explorer.get_block(number).await?;
    Ok(ApiResponse::new(serde_json::json!({
        "number": number,
        "block": block,
    })))
}
