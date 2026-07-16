use crate::middleware::auth::AuthUser;
use crate::state::AppState;
use axum::{
    Json,
    body::Bytes,
    extract::State,
    http::HeaderMap,
};
use chrono::{TimeZone, Utc};
use serde::{Deserialize, Serialize};
use tempoforge_common::{ApiResponse, AppError, AppResult};
use uuid::Uuid;
use validator::Validate;

#[derive(Debug, Serialize, sqlx::FromRow)]
pub struct BillingStatus {
    pub plan: String,
    pub status: String,
    pub stripe_customer_id: Option<String>,
    pub stripe_subscription_id: Option<String>,
    pub current_period_end: Option<chrono::DateTime<Utc>>,
}

#[derive(Debug, Deserialize, Validate)]
pub struct CheckoutRequest {
    #[validate(length(min = 3, max = 200))]
    pub price_id: String,
    pub org_id: Option<Uuid>,
}

#[derive(Debug, Serialize)]
pub struct CheckoutResponse {
    pub session_id: String,
    pub checkout_url: String,
}

#[derive(Debug, Serialize)]
pub struct PortalResponse {
    pub portal_url: String,
}

pub async fn billing_status(
    State(state): State<AppState>,
    AuthUser(user): AuthUser,
) -> AppResult<ApiResponse<BillingStatus>> {
    let org_id = ensure_org(&state, &user).await?;

    let row = sqlx::query_as::<_, BillingStatus>(
        r#"
        SELECT
          plan::text as plan,
          status,
          stripe_customer_id,
          stripe_subscription_id,
          current_period_end
        FROM billing_customers
        WHERE org_id = $1
        "#,
    )
    .bind(org_id)
    .fetch_optional(&state.db)
    .await
    .map_err(|e| AppError::Internal(format!("billing status query failed: {e}")))?;

    Ok(ApiResponse::new(row.unwrap_or(BillingStatus {
        plan: "free".into(),
        status: "inactive".into(),
        stripe_customer_id: None,
        stripe_subscription_id: None,
        current_period_end: None,
    })))
}

pub async fn create_checkout(
    State(state): State<AppState>,
    AuthUser(user): AuthUser,
    Json(body): Json<CheckoutRequest>,
) -> AppResult<ApiResponse<CheckoutResponse>> {
    body.validate()?;
    let org_id = body.org_id.unwrap_or(ensure_org(&state, &user).await?);
    let customer_id = ensure_stripe_customer(&state, org_id, &user).await?;

    let (session_id, checkout_url) = state
        .stripe
        .create_checkout_session(&customer_id, &body.price_id, &org_id.to_string())
        .await?;

    sqlx::query(
        r#"
        INSERT INTO activity_logs (org_id, user_id, action, resource_type, resource_id)
        VALUES ($1, $2, 'billing.checkout_started', 'stripe_session', $3)
        "#,
    )
    .bind(org_id)
    .bind(user.user_id.as_uuid())
    .bind(&session_id)
    .execute(&state.db)
    .await
    .ok();

    Ok(ApiResponse::new(CheckoutResponse {
        session_id,
        checkout_url,
    }))
}

pub async fn create_portal(
    State(state): State<AppState>,
    AuthUser(user): AuthUser,
) -> AppResult<ApiResponse<PortalResponse>> {
    let org_id = ensure_org(&state, &user).await?;
    let customer_id = sqlx::query_scalar::<_, Option<String>>(
        "SELECT stripe_customer_id FROM billing_customers WHERE org_id = $1",
    )
    .bind(org_id)
    .fetch_optional(&state.db)
    .await
    .map_err(|e| AppError::Internal(format!("billing portal lookup failed: {e}")))?
    .flatten()
    .ok_or_else(|| AppError::BadRequest("no Stripe customer for this organization".into()))?;

    let portal_url = state.stripe.create_billing_portal(&customer_id).await?;
    Ok(ApiResponse::new(PortalResponse { portal_url }))
}

pub async fn stripe_webhook(
    State(state): State<AppState>,
    headers: HeaderMap,
    body: Bytes,
) -> AppResult<ApiResponse<&'static str>> {
    let signature = headers
        .get("stripe-signature")
        .and_then(|v| v.to_str().ok())
        .ok_or_else(|| AppError::Unauthorized("missing Stripe-Signature".into()))?;

    state.stripe.verify_webhook(&body, signature)?;

    let event: serde_json::Value = serde_json::from_slice(&body)
        .map_err(|e| AppError::BadRequest(format!("invalid webhook JSON: {e}")))?;

    let event_type = event
        .get("type")
        .and_then(|v| v.as_str())
        .unwrap_or_default();

    match event_type {
        "customer.subscription.created"
        | "customer.subscription.updated"
        | "customer.subscription.deleted" => {
            apply_subscription_event(&state, &event).await?;
        }
        "checkout.session.completed" => {
            tracing::info!("checkout.session.completed received");
        }
        other => {
            tracing::debug!(event_type = other, "ignored stripe event");
        }
    }

    Ok(ApiResponse::new("ok"))
}

async fn apply_subscription_event(state: &AppState, event: &serde_json::Value) -> AppResult<()> {
    let object = event
        .pointer("/data/object")
        .ok_or_else(|| AppError::BadRequest("subscription object missing".into()))?;

    let customer_id = object
        .get("customer")
        .and_then(|v| v.as_str())
        .ok_or_else(|| AppError::BadRequest("subscription missing customer".into()))?;

    let subscription_id = object.get("id").and_then(|v| v.as_str());
    let status = object
        .get("status")
        .and_then(|v| v.as_str())
        .unwrap_or("inactive");

    let org_id = object
        .pointer("/metadata/org_id")
        .and_then(|v| v.as_str())
        .and_then(|s| Uuid::parse_str(s).ok());

    let period_end = object
        .get("current_period_end")
        .and_then(|v| v.as_i64())
        .and_then(|ts| Utc.timestamp_opt(ts, 0).single());

    let plan = match event.get("type").and_then(|v| v.as_str()) {
        Some("customer.subscription.deleted") => "free",
        _ if status == "active" || status == "trialing" => "pro",
        _ => "free",
    };

    if let Some(org_id) = org_id {
        sqlx::query(
            r#"
            INSERT INTO billing_customers (org_id, stripe_customer_id, stripe_subscription_id, plan, status, current_period_end, updated_at)
            VALUES ($1, $2, $3, $4::plan_tier, $5, $6, NOW())
            ON CONFLICT (org_id) DO UPDATE SET
              stripe_customer_id = EXCLUDED.stripe_customer_id,
              stripe_subscription_id = EXCLUDED.stripe_subscription_id,
              plan = EXCLUDED.plan,
              status = EXCLUDED.status,
              current_period_end = EXCLUDED.current_period_end,
              updated_at = NOW()
            "#,
        )
        .bind(org_id)
        .bind(customer_id)
        .bind(subscription_id)
        .bind(plan)
        .bind(status)
        .bind(period_end)
        .execute(&state.db)
        .await
        .map_err(|e| AppError::Internal(format!("billing upsert failed: {e}")))?;

        sqlx::query("UPDATE organizations SET plan = $2::plan_tier, updated_at = NOW() WHERE id = $1")
            .bind(org_id)
            .bind(plan)
            .execute(&state.db)
            .await
            .ok();
    } else {
        sqlx::query(
            r#"
            UPDATE billing_customers
            SET stripe_subscription_id = $2,
                plan = $3::plan_tier,
                status = $4,
                current_period_end = $5,
                updated_at = NOW()
            WHERE stripe_customer_id = $1
            "#,
        )
        .bind(customer_id)
        .bind(subscription_id)
        .bind(plan)
        .bind(status)
        .bind(period_end)
        .execute(&state.db)
        .await
        .map_err(|e| AppError::Internal(format!("billing update by customer failed: {e}")))?;
    }

    Ok(())
}

async fn ensure_org(state: &AppState, user: &tempoforge_auth::AuthContext) -> AppResult<Uuid> {
    let slug = format!("personal-{}", &user.clerk_user_id);
    let id = sqlx::query_scalar::<_, Uuid>(
        r#"
        INSERT INTO organizations (name, slug)
        VALUES ($1, $2)
        ON CONFLICT (slug) DO UPDATE SET updated_at = NOW()
        RETURNING id
        "#,
    )
    .bind("Personal")
    .bind(&slug)
    .fetch_one(&state.db)
    .await
    .map_err(|e| AppError::Internal(format!("org ensure failed: {e}")))?;

    sqlx::query(
        r#"
        INSERT INTO organization_members (org_id, user_id, role)
        VALUES ($1, $2, 'owner')
        ON CONFLICT DO NOTHING
        "#,
    )
    .bind(id)
    .bind(user.user_id.as_uuid())
    .execute(&state.db)
    .await
    .ok();

    Ok(id)
}

async fn ensure_stripe_customer(
    state: &AppState,
    org_id: Uuid,
    user: &tempoforge_auth::AuthContext,
) -> AppResult<String> {
    if let Some(existing) = sqlx::query_scalar::<_, Option<String>>(
        "SELECT stripe_customer_id FROM billing_customers WHERE org_id = $1",
    )
    .bind(org_id)
    .fetch_optional(&state.db)
    .await
    .map_err(|e| AppError::Internal(format!("customer lookup failed: {e}")))?
    .flatten()
    {
        return Ok(existing);
    }

    let email = user
        .email
        .clone()
        .unwrap_or_else(|| format!("{}@users.tempoforge.local", user.clerk_user_id));

    let customer_id = state
        .stripe
        .create_customer(&email, &org_id.to_string())
        .await?;

    sqlx::query(
        r#"
        INSERT INTO billing_customers (org_id, stripe_customer_id, plan, status, updated_at)
        VALUES ($1, $2, 'free', 'inactive', NOW())
        ON CONFLICT (org_id) DO UPDATE SET
          stripe_customer_id = EXCLUDED.stripe_customer_id,
          updated_at = NOW()
        "#,
    )
    .bind(org_id)
    .bind(&customer_id)
    .execute(&state.db)
    .await
    .map_err(|e| AppError::Internal(format!("customer persist failed: {e}")))?;

    Ok(customer_id)
}
