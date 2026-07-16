use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use tempoforge_ai_engine::types::{Finding, Severity};
use tempoforge_common::AuditReportId;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditSummary {
    pub critical: u32,
    pub high: u32,
    pub medium: u32,
    pub low: u32,
    pub info: u32,
}

impl AuditSummary {
    pub fn from_findings(findings: &[Finding]) -> Self {
        let mut summary = Self {
            critical: 0,
            high: 0,
            medium: 0,
            low: 0,
            info: 0,
        };
        for f in findings {
            match f.severity {
                Severity::Critical => summary.critical += 1,
                Severity::High => summary.high += 1,
                Severity::Medium => summary.medium += 1,
                Severity::Low => summary.low += 1,
                Severity::Info => summary.info += 1,
            }
        }
        summary
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditReport {
    pub id: AuditReportId,
    pub title: String,
    pub source_hash: String,
    pub findings: Vec<Finding>,
    pub summary: AuditSummary,
    pub ai_narrative: Option<String>,
    pub created_at: DateTime<Utc>,
}

impl AuditReport {
    pub fn new(title: impl Into<String>, source: &str, findings: Vec<Finding>, ai_narrative: Option<String>) -> Self {
        let source_hash = blake3_hex(source);
        let summary = AuditSummary::from_findings(&findings);
        Self {
            id: AuditReportId(Uuid::new_v4()),
            title: title.into(),
            source_hash,
            findings,
            summary,
            ai_narrative,
            created_at: Utc::now(),
        }
    }
}

fn blake3_hex(input: &str) -> String {
    // Lightweight fingerprint without pulling blake3 into this crate's public graph twice.
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};
    let mut hasher = DefaultHasher::new();
    input.hash(&mut hasher);
    format!("{:016x}", hasher.finish())
}
