use crate::state::AppState;
use axum::{
    extract::{FromRequestParts, State},
    http::request::Parts,
};
use tempoforge_auth::{AuthContext, Role};
use tempoforge_common::{AppError, UserId};
use uuid::Uuid;

pub struct AuthUser(pub AuthContext);

impl FromRequestParts<AppState> for AuthUser {
    type Rejection = AppError;

    async fn from_request_parts(
        parts: &mut Parts,
        state: &AppState,
    ) -> Result<Self, Self::Rejection> {
        // Rate limit by IP / bearer fingerprint.
        let ip = parts
            .headers
            .get("x-forwarded-for")
            .and_then(|v| v.to_str().ok())
            .unwrap_or("unknown");
        state.rate_limiter.check(ip)?;

        let auth_header = parts
            .headers
            .get(axum::http::header::AUTHORIZATION)
            .and_then(|v| v.to_str().ok());

        let Some(header) = auth_header else {
            if state.config.allow_dev_auth {
                return Ok(AuthUser(ensure_dev_user(state).await?));
            }
            return Err(AppError::Unauthorized("missing Authorization header".into()));
        };

        let token = header
            .strip_prefix("Bearer ")
            .ok_or_else(|| AppError::Unauthorized("expected Bearer token".into()))?;

        if token == "dev" && state.config.allow_dev_auth {
            return Ok(AuthUser(ensure_dev_user(state).await?));
        }

        let Some(clerk) = &state.clerk else {
            if state.config.allow_dev_auth {
                return Ok(AuthUser(ensure_dev_user(state).await?));
            }
            return Err(AppError::Unauthorized("auth provider not configured".into()));
        };

        let mut ctx = clerk.authenticate(token).await?;
        // Resolve or create local user id from clerk subject.
        ctx.user_id = upsert_user(state, &ctx).await?;
        Ok(AuthUser(ctx))
    }
}

async fn ensure_dev_user(state: &AppState) -> Result<AuthContext, AppError> {
    let mut ctx = dev_context();
    let id = sqlx::query_scalar::<_, Uuid>(
        r#"
        INSERT INTO users (id, clerk_user_id, email, name)
        VALUES ($1, $2, $3, 'Dev User')
        ON CONFLICT (clerk_user_id)
        DO UPDATE SET email = EXCLUDED.email, updated_at = NOW()
        RETURNING id
        "#,
    )
    .bind(ctx.user_id.as_uuid())
    .bind(&ctx.clerk_user_id)
    .bind(&ctx.email)
    .fetch_one(&state.db)
    .await
    .map_err(|e| AppError::Internal(format!("dev user upsert failed: {e}")))?;
    ctx.user_id = UserId(id);
    Ok(ctx)
}

fn dev_context() -> AuthContext {
    AuthContext {
        user_id: UserId(Uuid::parse_str("00000000-0000-4000-8000-000000000001").unwrap()),
        clerk_user_id: "user_dev".into(),
        org_id: None,
        role: Role::Owner,
        email: Some("dev@tempoforge.ai".into()),
        session_id: Some("dev_session".into()),
    }
}

async fn upsert_user(state: &AppState, ctx: &AuthContext) -> Result<UserId, AppError> {
    let row = sqlx::query_scalar::<_, Uuid>(
        r#"
        INSERT INTO users (clerk_user_id, email)
        VALUES ($1, $2)
        ON CONFLICT (clerk_user_id)
        DO UPDATE SET email = COALESCE(EXCLUDED.email, users.email), updated_at = NOW()
        RETURNING id
        "#,
    )
    .bind(&ctx.clerk_user_id)
    .bind(&ctx.email)
    .fetch_one(&state.db)
    .await
    .map_err(|e| AppError::Internal(format!("user upsert failed: {e}")))?;

    Ok(UserId(row))
}

/// Optional auth for public + authenticated hybrid routes.
#[allow(dead_code)]
pub struct OptionalAuth(pub Option<AuthContext>);

impl FromRequestParts<AppState> for OptionalAuth {
    type Rejection = AppError;

    async fn from_request_parts(
        parts: &mut Parts,
        state: &AppState,
    ) -> Result<Self, Self::Rejection> {
        match AuthUser::from_request_parts(parts, state).await {
            Ok(AuthUser(ctx)) => Ok(OptionalAuth(Some(ctx))),
            Err(AppError::Unauthorized(_)) => Ok(OptionalAuth(None)),
            Err(other) => Err(other),
        }
    }
}

// Silence unused import warning for State in some rustc versions
#[allow(dead_code)]
fn _state_marker(_: State<AppState>) {}
