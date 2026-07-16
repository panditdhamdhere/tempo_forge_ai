use crate::state::AppState;
use axum::{Json, extract::State};
use serde::Serialize;
use tempoforge_common::{AppError, AppResult};

#[derive(Serialize)]
pub struct HealthResponse {
    pub status: &'static str,
    pub service: &'static str,
    pub version: &'static str,
}

pub async fn health() -> Json<HealthResponse> {
    Json(HealthResponse {
        status: "ok",
        service: "tempoforge-api",
        version: env!("CARGO_PKG_VERSION"),
    })
}

#[derive(Serialize)]
pub struct ReadyResponse {
    pub status: &'static str,
    pub database: bool,
}

pub async fn ready(State(state): State<AppState>) -> AppResult<Json<ReadyResponse>> {
    let database = sqlx::query_scalar::<_, i32>("SELECT 1")
        .fetch_one(&state.db)
        .await
        .is_ok();

    if !database {
        return Err(AppError::Internal("database not ready".into()));
    }

    Ok(Json(ReadyResponse {
        status: "ready",
        database,
    }))
}
