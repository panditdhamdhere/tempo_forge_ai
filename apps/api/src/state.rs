use crate::config::Config;
use crate::services::rate_limit::RateLimiter;
use sqlx::postgres::PgPoolOptions;
use sqlx::PgPool;
use std::sync::Arc;
use tempoforge_ai_engine::providers::provider_from_env;
use tempoforge_ai_engine::AgentOrchestrator;
use tempoforge_analytics::AnalyticsService;
use tempoforge_auth::{ClerkAuth, ClerkConfig};
use tempoforge_blockchain::TempoClient;
use tempoforge_common::{AppError, AppResult};
use tempoforge_explorer::ExplorerService;

#[derive(Clone)]
pub struct AppState {
    pub config: Config,
    pub db: PgPool,
    pub clerk: Option<ClerkAuth>,
    pub ai: Arc<AgentOrchestrator>,
    #[allow(dead_code)]
    pub tempo: TempoClient,
    pub explorer: ExplorerService,
    pub analytics: AnalyticsService,
    pub rate_limiter: RateLimiter,
}

impl AppState {
    pub async fn new(config: Config) -> AppResult<Self> {
        let db = PgPoolOptions::new()
            .max_connections(20)
            .connect(&config.database_url)
            .await
            .map_err(|e| AppError::Internal(format!("database connect failed: {e}")))?;

        // Migrations are optional at boot so the API can start before Postgres is ready in some CI paths.
        if let Err(err) = sqlx::migrate!("../../migrations").run(&db).await {
            tracing::warn!(error = %err, "migration run skipped/failed — ensure DB is up");
        }

        let clerk = if config.clerk_jwks_url.is_empty() || config.clerk_issuer.is_empty() {
            tracing::warn!("Clerk JWKS/issuer not configured — using development auth bypass");
            None
        } else {
            Some(ClerkAuth::new(ClerkConfig {
                jwks_url: config.clerk_jwks_url.clone(),
                issuer: config.clerk_issuer.clone(),
                audience: config.jwt_audience.clone(),
            }))
        };

        let provider = provider_from_env()?;
        let ai = Arc::new(AgentOrchestrator::with_defaults(provider));
        let tempo = TempoClient::from_env();
        let explorer = ExplorerService::new(tempo.clone());
        let analytics = AnalyticsService::new(tempo.clone());
        let rate_limiter = RateLimiter::new(config.rate_limit_per_minute);

        Ok(Self {
            config,
            db,
            clerk,
            ai,
            tempo,
            explorer,
            analytics,
            rate_limiter,
        })
    }
}
