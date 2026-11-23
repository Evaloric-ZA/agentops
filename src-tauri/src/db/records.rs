//! Database records.
//!
//! These are the on-disk shapes. They are intentionally separate from the
//! domain types in `domain/mod.rs`: a record is what SQLite stores, a domain
//! type is what the command layer returns. Keeping them apart means a
//! schema change does not silently change the UI contract.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentRow {
    pub id: String,
    pub display_name: String,
    pub adapter: String,
    pub host_class: String,
    pub status: String,
    pub trust_level: String,
    pub current_run_id: Option<String>,
    pub last_heartbeat: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CapabilityRow {
    pub agent_id: String,
    pub name: String,
    pub description: String,
    pub granted: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RunRow {
    pub id: String,
    pub agent_id: String,
    pub task: String,
    pub status: String,
    pub started_at: String,
    pub ended_at: Option<String>,
    pub duration_seconds: Option<i64>,
    pub artifacts: String,
    pub model: String,
    pub compute_units: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StageRow {
    pub run_id: String,
    pub ordinal: i64,
    pub name: String,
    pub status: String,
    pub started_at: String,
    pub ended_at: Option<String>,
    pub detail: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EventRow {
    pub id: String,
    pub run_id: String,
    pub agent_id: String,
    pub kind: String,
    pub message: String,
    pub at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApprovalRow {
    pub id: String,
    pub run_id: String,
    pub agent_id: String,
    pub severity: String,
    pub reason: String,
    pub evidence: String,
    pub confidence: f64,
    pub state: String,
    pub created_at: String,
    pub decided_at: Option<String>,
    pub comment: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DecisionRow {
    pub id: String,
    pub approval_id: String,
    pub outcome: String,
    pub comment: Option<String>,
    pub at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolRow {
    pub id: String,
    pub name: String,
    pub kind: String,
    pub status: String,
    pub latency_ms: Option<i64>,
    pub success_rate: f64,
    pub calls_last_hour: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PolicyRow {
    pub id: String,
    pub name: String,
    pub scope: String,
    pub effect: String,
    pub enabled: bool,
    pub detail: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TeamRow {
    pub id: String,
    pub name: String,
    pub role: String,
    pub on_shift: bool,
    pub agent_ids: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditRow {
    pub id: String,
    pub at: String,
    pub actor: String,
    pub action: String,
    pub subject: String,
    pub outcome: String,
    pub detail: String,
}
