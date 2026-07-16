use crate::claims::AuthContext;
use jsonwebtoken::{Algorithm, DecodingKey, Validation, decode, decode_header};
use parking_lot::RwLock;
use serde::Deserialize;
use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tempoforge_common::{AppError, AppResult};
use tracing::{debug, warn};

#[derive(Debug, Clone)]
pub struct ClerkConfig {
    pub jwks_url: String,
    pub issuer: String,
    pub audience: String,
}

#[derive(Debug, Deserialize)]
struct Jwks {
    keys: Vec<Jwk>,
}

#[derive(Debug, Clone, Deserialize)]
struct Jwk {
    kid: String,
    kty: String,
    n: Option<String>,
    e: Option<String>,
    #[allow(dead_code)]
    alg: Option<String>,
}

#[derive(Debug, Deserialize)]
struct ClerkClaims {
    sub: String,
    iss: String,
    #[allow(dead_code)]
    exp: i64,
    #[allow(dead_code)]
    iat: i64,
    #[serde(default)]
    #[allow(dead_code)]
    azp: Option<String>,
    #[serde(default)]
    sid: Option<String>,
    #[serde(default)]
    email: Option<String>,
    #[serde(default)]
    #[allow(dead_code)]
    org_id: Option<String>,
    #[serde(default)]
    org_role: Option<String>,
}

struct CachedJwks {
    keys: HashMap<String, DecodingKey>,
    fetched_at: Instant,
}

#[derive(Clone)]
pub struct ClerkAuth {
    config: ClerkConfig,
    http: reqwest::Client,
    cache: Arc<RwLock<Option<CachedJwks>>>,
}

impl ClerkAuth {
    pub fn new(config: ClerkConfig) -> Self {
        Self {
            config,
            http: reqwest::Client::builder()
                .timeout(Duration::from_secs(10))
                .build()
                .expect("reqwest client"),
            cache: Arc::new(RwLock::new(None)),
        }
    }

    pub async fn authenticate(&self, token: &str) -> AppResult<AuthContext> {
        let header = decode_header(token)
            .map_err(|e| AppError::Unauthorized(format!("invalid token header: {e}")))?;

        let kid = header
            .kid
            .ok_or_else(|| AppError::Unauthorized("token missing kid".into()))?;

        let key = self.decoding_key_for(&kid).await?;

        let mut validation = Validation::new(Algorithm::RS256);
        validation.set_issuer(&[&self.config.issuer]);
        if !self.config.audience.is_empty() {
            validation.set_audience(&[&self.config.audience]);
        } else {
            validation.validate_aud = false;
        }

        let data = decode::<ClerkClaims>(token, &key, &validation)
            .map_err(|e| AppError::Unauthorized(format!("token validation failed: {e}")))?;

        let claims = data.claims;
        if claims.iss != self.config.issuer {
            return Err(AppError::Unauthorized("issuer mismatch".into()));
        }

        let mut ctx = AuthContext::from_clerk(claims.sub, claims.email, claims.sid);
        if let Some(role) = claims.org_role.as_deref() {
            ctx.role = map_clerk_role(role);
        }
        debug!(user = %ctx.clerk_user_id, "clerk token accepted");
        Ok(ctx)
    }

    async fn decoding_key_for(&self, kid: &str) -> AppResult<DecodingKey> {
        {
            let guard = self.cache.read();
            if let Some(cache) = guard.as_ref() {
                if cache.fetched_at.elapsed() < Duration::from_secs(30 * 60) {
                    if let Some(key) = cache.keys.get(kid) {
                        return Ok(key.clone());
                    }
                }
            }
        }

        self.refresh_jwks().await?;

        let guard = self.cache.read();
        guard
            .as_ref()
            .and_then(|c| c.keys.get(kid).cloned())
            .ok_or_else(|| AppError::Unauthorized(format!("unknown signing key kid={kid}")))
    }

    async fn refresh_jwks(&self) -> AppResult<()> {
        let response = self
            .http
            .get(&self.config.jwks_url)
            .send()
            .await
            .map_err(|e| AppError::Upstream(format!("jwks fetch failed: {e}")))?;

        if !response.status().is_success() {
            warn!(status = %response.status(), "jwks fetch non-success");
            return Err(AppError::Upstream("jwks endpoint error".into()));
        }

        let jwks: Jwks = response
            .json()
            .await
            .map_err(|e| AppError::Upstream(format!("jwks parse failed: {e}")))?;

        let mut keys = HashMap::new();
        for jwk in jwks.keys {
            if jwk.kty != "RSA" {
                continue;
            }
            let (Some(n), Some(e)) = (jwk.n, jwk.e) else {
                continue;
            };
            match DecodingKey::from_rsa_components(&n, &e) {
                Ok(key) => {
                    keys.insert(jwk.kid, key);
                }
                Err(err) => warn!(error = %err, "skipping invalid jwk"),
            }
        }

        *self.cache.write() = Some(CachedJwks {
            keys,
            fetched_at: Instant::now(),
        });
        Ok(())
    }
}

fn map_clerk_role(role: &str) -> crate::claims::Role {
    match role {
        "org:admin" | "admin" => crate::claims::Role::Admin,
        "org:owner" | "owner" => crate::claims::Role::Owner,
        "org:member" | "member" => crate::claims::Role::Member,
        _ => crate::claims::Role::Viewer,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn maps_clerk_roles() {
        assert_eq!(map_clerk_role("org:admin"), crate::claims::Role::Admin);
        assert_eq!(map_clerk_role("viewer"), crate::claims::Role::Viewer);
    }
}
