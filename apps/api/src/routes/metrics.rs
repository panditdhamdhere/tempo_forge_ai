use crate::services::metrics::metrics_handle;
use axum::response::IntoResponse;

pub async fn prometheus_metrics() -> impl IntoResponse {
    metrics_handle().render()
}
