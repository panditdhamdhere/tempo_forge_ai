use std::net::IpAddr;
use tempoforge_common::{AppError, AppResult};

#[derive(Clone, Debug)]
#[allow(dead_code)]
pub struct Config {
    pub host: IpAddr,
    pub port: u16,
    pub database_url: String,
    pub redis_url: String,
    pub app_url: String,
    pub clerk_jwks_url: String,
    pub clerk_issuer: String,
    pub jwt_audience: String,
    pub encryption_key: String,
    pub rate_limit_per_minute: u32,
    pub allow_dev_auth: bool,
    pub stripe_secret_key: String,
    pub stripe_webhook_secret: String,
}

impl Config {
    pub fn from_env() -> AppResult<Self> {
        let host = std::env::var("API_HOST")
            .unwrap_or_else(|_| "0.0.0.0".into())
            .parse()
            .map_err(|e| AppError::Internal(format!("invalid API_HOST: {e}")))?;

        let port = std::env::var("API_PORT")
            .unwrap_or_else(|_| "8080".into())
            .parse()
            .map_err(|e| AppError::Internal(format!("invalid API_PORT: {e}")))?;

        let database_url = std::env::var("DATABASE_URL").unwrap_or_else(|_| {
            "postgres://tempoforge:tempoforge@localhost:5432/tempoforge".into()
        });

        let redis_url =
            std::env::var("REDIS_URL").unwrap_or_else(|_| "redis://localhost:6379".into());

        let allow_dev_auth = std::env::var("APP_ENV")
            .unwrap_or_else(|_| "development".into())
            == "development";

        Ok(Self {
            host,
            port,
            database_url,
            redis_url,
            app_url: std::env::var("APP_URL").unwrap_or_else(|_| "http://localhost:3000".into()),
            clerk_jwks_url: std::env::var("CLERK_JWKS_URL").unwrap_or_default(),
            clerk_issuer: std::env::var("CLERK_ISSUER").unwrap_or_default(),
            jwt_audience: std::env::var("JWT_AUDIENCE")
                .unwrap_or_else(|_| "tempoforge-api".into()),
            encryption_key: std::env::var("ENCRYPTION_KEY").unwrap_or_else(|_| {
                "0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef".into()
            }),
            rate_limit_per_minute: std::env::var("API_RATE_LIMIT_PER_MINUTE")
                .ok()
                .and_then(|v| v.parse().ok())
                .unwrap_or(120),
            allow_dev_auth,
            stripe_secret_key: std::env::var("STRIPE_SECRET_KEY").unwrap_or_default(),
            stripe_webhook_secret: std::env::var("STRIPE_WEBHOOK_SECRET").unwrap_or_default(),
        })
    }
}
