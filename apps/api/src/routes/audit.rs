use crate::middleware::auth::AuthUser;
use crate::state::AppState;
use axum::{Json, extract::State};
use serde::{Deserialize, Serialize};
use tempoforge_ai_engine::types::{AgentKind, AgentRequest, ChatMessage};
use tempoforge_common::{ApiResponse, AppError, AppResult};
use tempoforge_security::{AuditReport, analyze_source};
use uuid::Uuid;
use validator::Validate;

#[derive(Debug, Deserialize, Validate)]
pub struct AuditRequest {
    #[validate(length(min = 1, max = 120))]
    pub title: String,
    #[validate(length(min = 20, max = 500_000))]
    pub source: String,
    pub project_id: Option<Uuid>,
    #[serde(default = "default_true")]
    pub use_ai: bool,
}

fn default_true() -> bool {
    true
}

#[derive(Debug, Serialize)]
pub struct AuditResponse {
    pub report: AuditReport,
}

pub async fn run_audit(
    State(state): State<AppState>,
    AuthUser(user): AuthUser,
    Json(body): Json<AuditRequest>,
) -> AppResult<ApiResponse<AuditResponse>> {
    body.validate()?;

    let mut findings = analyze_source(&body.source);
    let mut ai_narrative = None;

    if body.use_ai {
        let request = AgentRequest {
            id: Uuid::new_v4(),
            agent: AgentKind::Auditor,
            user_id: user.user_id,
            project_id: body.project_id.map(tempoforge_common::ProjectId),
            conversation_id: None,
            messages: vec![ChatMessage::user(format!(
                "Audit this Solidity source:\n\n{}",
                body.source
            ))],
            context: serde_json::json!({}),
        };

        match state.ai.run(request).await {
            Ok(resp) => {
                if !resp.findings.is_empty() {
                    findings.extend(resp.findings);
                }
                ai_narrative = Some(resp.content);
            }
            Err(err) => {
                tracing::warn!(error = %err, "AI auditor unavailable; returning static findings");
            }
        }
    }

    let report = AuditReport::new(body.title, &body.source, findings, ai_narrative);

    if let Some(project_id) = body.project_id {
        sqlx::query(
            r#"
            INSERT INTO audit_reports (id, project_id, title, source_hash, summary, findings, ai_narrative, created_by)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
            "#,
        )
        .bind(report.id.as_uuid())
        .bind(project_id)
        .bind(&report.title)
        .bind(&report.source_hash)
        .bind(serde_json::to_value(&report.summary).unwrap_or_default())
        .bind(serde_json::to_value(&report.findings).unwrap_or_default())
        .bind(&report.ai_narrative)
        .bind(user.user_id.as_uuid())
        .execute(&state.db)
        .await
        .map_err(|e| AppError::Internal(format!("persist audit failed: {e}")))?;
    }

    Ok(ApiResponse::new(AuditResponse { report }))
}
