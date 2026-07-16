mod ai;
mod analytics;
mod audit;
mod billing;
mod deployments;
mod explorer;
mod health;
mod keys;
mod metrics;
mod projects;
mod sdk;
mod secrets;

use crate::middleware::metrics::track_metrics;
use crate::middleware::request_id::attach_request_id;
use crate::middleware::security_headers::security_headers;
use crate::state::AppState;
use axum::{
    Json, Router,
    middleware,
    routing::{get, post},
};
use tower_http::cors::{Any, CorsLayer};
use tower_http::limit::RequestBodyLimitLayer;
use tower_http::trace::TraceLayer;

pub fn router(state: AppState) -> Router {
    let api = Router::new()
        .route("/health", get(health::health))
        .route("/ready", get(health::ready))
        .route("/metrics", get(metrics::prometheus_metrics))
        .route(
            "/projects",
            get(projects::list_projects).post(projects::create_project),
        )
        .route("/ai/agents/{agent}", post(ai::run_agent))
        .route("/ai/chat", post(ai::chat))
        .route("/ai/conversations", get(ai::list_conversations))
        .route(
            "/ai/conversations/{id}/messages",
            get(ai::get_conversation_messages),
        )
        .route("/audit", post(audit::run_audit))
        .route("/explorer/tx/{hash}", get(explorer::get_transaction))
        .route("/explorer/address/{address}", get(explorer::get_address))
        .route("/explorer/blocks/latest", get(explorer::latest_block))
        .route("/analytics/dashboard", get(analytics::dashboard))
        .route("/deployments/plan", post(deployments::plan_deployment))
        .route(
            "/deployments",
            get(deployments::list_deployments).post(deployments::create_deployment),
        )
        .route(
            "/deployments/{id}",
            get(deployments::get_deployment).patch(deployments::update_deployment),
        )
        .route("/sdk/generate", post(sdk::generate_sdk))
        .route("/billing/status", get(billing::billing_status))
        .route("/billing/checkout", post(billing::create_checkout))
        .route("/billing/portal", post(billing::create_portal))
        .route("/billing/webhook", post(billing::stripe_webhook))
        .route("/keys", get(keys::list_keys).post(keys::create_key))
        .route("/keys/{id}/revoke", post(keys::revoke_key))
        .route("/secrets", get(secrets::list_secrets).post(secrets::upsert_secret))
        .route("/secrets/{name}", get(secrets::reveal_secret))
        .route("/openapi.json", get(openapi_spec));

    Router::new()
        .nest("/api/v1", api)
        .layer(RequestBodyLimitLayer::new(2 * 1024 * 1024))
        .layer(middleware::from_fn(security_headers))
        .layer(middleware::from_fn(track_metrics))
        .layer(middleware::from_fn(attach_request_id))
        .layer(TraceLayer::new_for_http())
        .layer(
            CorsLayer::new()
                .allow_origin(Any)
                .allow_methods(Any)
                .allow_headers(Any),
        )
        .with_state(state)
}

async fn openapi_spec() -> Json<serde_json::Value> {
    Json(serde_json::json!({
        "openapi": "3.1.0",
        "info": {
            "title": "TempoForge AI API",
            "version": "1.0.0",
            "description": "AI-powered developer platform for Tempo Blockchain"
        },
        "servers": [{"url": "/api/v1"}],
        "paths": {
            "/health": {"get": {"summary": "Liveness"}},
            "/ready": {"get": {"summary": "Readiness"}},
            "/metrics": {"get": {"summary": "Prometheus metrics"}},
            "/keys": {"get": {"summary": "List API keys"}, "post": {"summary": "Create API key"}},
            "/secrets": {"get": {"summary": "List encrypted secrets"}, "post": {"summary": "Upsert secret"}},
            "/projects": {
                "get": {"summary": "List projects"},
                "post": {"summary": "Create project"}
            },
            "/ai/agents/{agent}": {"post": {"summary": "Run AI agent"}},
            "/billing/webhook": {"post": {"summary": "Stripe webhooks"}}
        }
    }))
}
