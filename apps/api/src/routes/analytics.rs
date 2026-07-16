use crate::middleware::auth::AuthUser;
use crate::state::AppState;
use axum::extract::State;
use tempoforge_common::{ApiResponse, AppResult};

pub async fn dashboard(
    State(state): State<AppState>,
    AuthUser(_user): AuthUser,
) -> AppResult<ApiResponse<tempoforge_analytics::AnalyticsDashboard>> {
    let dash = state.analytics.dashboard().await?;
    Ok(ApiResponse::new(dash))
}
