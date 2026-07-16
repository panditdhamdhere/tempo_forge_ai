use std::net::IpAddr;
use tempoforge_common::{AppError, AppResult};

const DEFAULT_ENCRYPTION_KEY: &str =
    "0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef";

#[derive(Clone, Debug)]
pub struct Config {
    pub app_env: String,
    pub host: IpAddr,
    pub port: u16,
    pub database_url: String,
    #[allow(dead_code)]
    pub redis_url: String,
    pub app_url: String,
    pub api_public_url: String,
    pub cors_origins: Vec<String>,
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
        let app_env = std::env::var("APP_ENV").unwrap_or_else(|_| "development".into());
        let is_prod_like = matches!(app_env.as_str(), "production" | "staging");

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

        let app_url = std::env::var("APP_URL").unwrap_or_else(|_| "http://localhost:3000".into());
        let api_public_url =
            std::env::var("API_PUBLIC_URL").unwrap_or_else(|_| format!("http://localhost:{port}"));

        let cors_origins = parse_cors_origins(&app_url);

        let allow_dev_auth = !is_prod_like
            && std::env::var("ALLOW_DEV_AUTH")
                .map(|v| v == "1" || v.eq_ignore_ascii_case("true"))
                .unwrap_or(true);

        let encryption_key = std::env::var("ENCRYPTION_KEY").unwrap_or_else(|_| {
            if is_prod_like {
                String::new()
            } else {
                DEFAULT_ENCRYPTION_KEY.into()
            }
        });

        let config = Self {
            app_env,
            host,
            port,
            database_url,
            redis_url,
            app_url,
            api_public_url,
            cors_origins,
            clerk_jwks_url: std::env::var("CLERK_JWKS_URL").unwrap_or_default(),
            clerk_issuer: std::env::var("CLERK_ISSUER").unwrap_or_default(),
            jwt_audience: std::env::var("JWT_AUDIENCE")
                .unwrap_or_else(|_| "tempoforge-api".into()),
            encryption_key,
            rate_limit_per_minute: std::env::var("API_RATE_LIMIT_PER_MINUTE")
                .ok()
                .and_then(|v| v.parse().ok())
                .unwrap_or(120),
            allow_dev_auth,
            stripe_secret_key: std::env::var("STRIPE_SECRET_KEY").unwrap_or_default(),
            stripe_webhook_secret: std::env::var("STRIPE_WEBHOOK_SECRET").unwrap_or_default(),
        };

        config.validate()?;
        Ok(config)
    }

    pub fn is_prod_like(&self) -> bool {
        matches!(self.app_env.as_str(), "production" | "staging")
    }

    pub fn validate(&self) -> AppResult<()> {
        if !self.is_prod_like() {
            return Ok(());
        }

        let mut missing = Vec::new();

        if self.clerk_jwks_url.is_empty() {
            missing.push("CLERK_JWKS_URL");
        }
        if self.clerk_issuer.is_empty() {
            missing.push("CLERK_ISSUER");
        }
        if self.encryption_key.is_empty() || self.encryption_key == DEFAULT_ENCRYPTION_KEY {
            missing.push("ENCRYPTION_KEY (must be a unique 64-char hex, not the example default)");
        }
        if self.database_url.contains("tempoforge:tempoforge@localhost") {
            missing.push("DATABASE_URL (must not use local default credentials)");
        }
        if self.app_url.contains("localhost") {
            missing.push("APP_URL (must be the public HTTPS frontend origin)");
        }
        if self.allow_dev_auth {
            missing.push("ALLOW_DEV_AUTH must be disabled in staging/production");
        }

        if !missing.is_empty() {
            return Err(AppError::Internal(format!(
                "refusing to boot in {}: missing/invalid config: {}",
                self.app_env,
                missing.join(", ")
            )));
        }

        Ok(())
    }
}

fn parse_cors_origins(app_url: &str) -> Vec<String> {
    let mut origins = Vec::new();
    if let Ok(raw) = std::env::var("CORS_ORIGINS") {
        for part in raw.split(',') {
            let trimmed = part.trim();
            if !trimmed.is_empty() {
                origins.push(trimmed.trim_end_matches('/').to_string());
            }
        }
    }
    if origins.is_empty() {
        origins.push(app_url.trim_end_matches('/').to_string());
    }
    origins
}

#[cfg(test)]
mod tests {
    #[test]
    fn validate_rejects_default_encryption_in_prod_like() {
        let cfg = super::Config {
            app_env: "staging".into(),
            host: "0.0.0.0".parse().unwrap(),
            port: 8080,
            database_url: "postgres://u:p@db.example:5432/tempoforge".into(),
            redis_url: "redis://redis:6379".into(),
            app_url: "https://staging.example".into(),
            api_public_url: "https://api.staging.example".into(),
            cors_origins: vec!["https://staging.example".into()],
            clerk_jwks_url: "https://example.clerk.accounts.dev/.well-known/jwks.json".into(),
            clerk_issuer: "https://example.clerk.accounts.dev".into(),
            jwt_audience: "tempoforge-api".into(),
            encryption_key: super::DEFAULT_ENCRYPTION_KEY.into(),
            rate_limit_per_minute: 120,
            allow_dev_auth: false,
            stripe_secret_key: String::new(),
            stripe_webhook_secret: String::new(),
        };
        assert!(cfg.validate().is_err());
    }
}
