use hmac::{Hmac, Mac};
use sha2::Sha256;
use tempoforge_common::{AppError, AppResult};

type HmacSha256 = Hmac<Sha256>;

#[derive(Clone)]
pub struct StripeClient {
    http: reqwest::Client,
    secret_key: String,
    webhook_secret: String,
    app_url: String,
}

impl StripeClient {
    pub fn new(secret_key: String, webhook_secret: String, app_url: String) -> Self {
        Self {
            http: reqwest::Client::new(),
            secret_key,
            webhook_secret,
            app_url,
        }
    }

    pub fn configured(&self) -> bool {
        !self.secret_key.is_empty()
    }

    pub async fn create_customer(&self, email: &str, org_id: &str) -> AppResult<String> {
        self.require_configured()?;
        let body = [
            ("email", email),
            ("metadata[org_id]", org_id),
            ("metadata[product]", "tempoforge"),
        ];
        let value = self.post_form("/v1/customers", &body).await?;
        value
            .get("id")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string())
            .ok_or_else(|| AppError::Upstream("stripe customer missing id".into()))
    }

    pub async fn create_checkout_session(
        &self,
        customer_id: &str,
        price_id: &str,
        org_id: &str,
    ) -> AppResult<(String, String)> {
        self.require_configured()?;
        let success = format!(
            "{}/dashboard/billing?checkout=success",
            self.app_url.trim_end_matches('/')
        );
        let cancel = format!(
            "{}/dashboard/billing?checkout=cancel",
            self.app_url.trim_end_matches('/')
        );
        let body = [
            ("mode", "subscription"),
            ("customer", customer_id),
            ("success_url", success.as_str()),
            ("cancel_url", cancel.as_str()),
            ("line_items[0][price]", price_id),
            ("line_items[0][quantity]", "1"),
            ("metadata[org_id]", org_id),
            ("subscription_data[metadata][org_id]", org_id),
        ];
        let value = self.post_form("/v1/checkout/sessions", &body).await?;
        let id = value
            .get("id")
            .and_then(|v| v.as_str())
            .ok_or_else(|| AppError::Upstream("checkout session missing id".into()))?
            .to_string();
        let url = value
            .get("url")
            .and_then(|v| v.as_str())
            .ok_or_else(|| AppError::Upstream("checkout session missing url".into()))?
            .to_string();
        Ok((id, url))
    }

    pub async fn create_billing_portal(&self, customer_id: &str) -> AppResult<String> {
        self.require_configured()?;
        let return_url = format!(
            "{}/dashboard/billing",
            self.app_url.trim_end_matches('/')
        );
        let body = [
            ("customer", customer_id),
            ("return_url", return_url.as_str()),
        ];
        let value = self.post_form("/v1/billing_portal/sessions", &body).await?;
        value
            .get("url")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string())
            .ok_or_else(|| AppError::Upstream("portal session missing url".into()))
    }

    pub fn verify_webhook(&self, payload: &[u8], signature_header: &str) -> AppResult<()> {
        if self.webhook_secret.is_empty() {
            return Err(AppError::Internal("STRIPE_WEBHOOK_SECRET not set".into()));
        }

        let mut timestamp = None;
        let mut signatures = Vec::new();
        for part in signature_header.split(',') {
            let mut kv = part.trim().splitn(2, '=');
            match (kv.next(), kv.next()) {
                (Some("t"), Some(v)) => timestamp = Some(v),
                (Some("v1"), Some(v)) => signatures.push(v),
                _ => {}
            }
        }

        let timestamp = timestamp
            .ok_or_else(|| AppError::Unauthorized("stripe signature missing timestamp".into()))?;
        if signatures.is_empty() {
            return Err(AppError::Unauthorized(
                "stripe signature missing v1".into(),
            ));
        }

        let signed = format!("{timestamp}.{}", String::from_utf8_lossy(payload));
        let mut mac = HmacSha256::new_from_slice(self.webhook_secret.as_bytes())
            .map_err(|e| AppError::Internal(format!("hmac init failed: {e}")))?;
        mac.update(signed.as_bytes());
        let digest = hex::encode(mac.finalize().into_bytes());

        if signatures.iter().any(|sig| timing_eq(sig, &digest)) {
            Ok(())
        } else {
            Err(AppError::Unauthorized(
                "stripe webhook signature mismatch".into(),
            ))
        }
    }

    async fn post_form(&self, path: &str, form: &[(&str, &str)]) -> AppResult<serde_json::Value> {
        let url = format!("https://api.stripe.com{path}");
        let response = self
            .http
            .post(url)
            .basic_auth(&self.secret_key, None::<&str>)
            .form(form)
            .send()
            .await
            .map_err(|e| AppError::Upstream(format!("stripe request failed: {e}")))?;

        let status = response.status();
        let body: serde_json::Value = response
            .json()
            .await
            .map_err(|e| AppError::Upstream(format!("stripe decode failed: {e}")))?;

        if !status.is_success() {
            let message = body
                .pointer("/error/message")
                .and_then(|v| v.as_str())
                .unwrap_or("stripe API error");
            return Err(AppError::Upstream(format!("stripe {status}: {message}")));
        }
        Ok(body)
    }

    fn require_configured(&self) -> AppResult<()> {
        if self.configured() {
            Ok(())
        } else {
            Err(AppError::BadRequest(
                "Stripe is not configured. Set STRIPE_SECRET_KEY.".into(),
            ))
        }
    }
}

fn timing_eq(a: &str, b: &str) -> bool {
    if a.len() != b.len() {
        return false;
    }
    a.bytes()
        .zip(b.bytes())
        .fold(0u8, |acc, (x, y)| acc | (x ^ y))
        == 0
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn verifies_valid_signature() {
        let secret = "whsec_test";
        let payload = br#"{"id":"evt_1"}"#;
        let timestamp = "1710000000";
        let signed = format!("{timestamp}.{}", String::from_utf8_lossy(payload));
        let mut mac = HmacSha256::new_from_slice(secret.as_bytes()).unwrap();
        mac.update(signed.as_bytes());
        let digest = hex::encode(mac.finalize().into_bytes());
        let header = format!("t={timestamp},v1={digest}");

        let client = StripeClient::new("sk_test".into(), secret.into(), "http://localhost:3000".into());
        assert!(client.verify_webhook(payload, &header).is_ok());
        assert!(client.verify_webhook(payload, "t=1,v1=deadbeef").is_err());
    }
}
