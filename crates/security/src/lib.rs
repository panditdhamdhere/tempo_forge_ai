//! Deterministic Solidity vulnerability detectors + AI auditor bridge.

pub mod detectors;
pub mod report;

pub use detectors::analyze_source;
pub use report::{AuditReport, AuditSummary};
