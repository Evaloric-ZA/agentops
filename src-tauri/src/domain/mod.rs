//! Domain types.
//!
//! These are the canonical shapes the Rust side owns. The frontend mirrors
//! them in `src/domain/types.ts`; database records are deliberately kept
//! separate from these (see `db/records.rs`).

pub mod ids;

use serde::{Deserialize, Serialize};

/// The lifecycle state of an agent, as shown in the registry.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AgentStatus {
    Working,
    Idle,
    Approval,
    Failed,
    Offline,
}

impl AgentStatus {
    pub fn as_str(self) -> &'static str {
        match self {
            AgentStatus::Working => "working",
            AgentStatus::Idle => "idle",
            AgentStatus::Approval => "approval",
            AgentStatus::Failed => "failed",
            AgentStatus::Offline => "offline",
        }
    }
}

/// How much an agent is trusted by the local policy set.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TrustLevel {
    Untrusted,
    Limited,
    Trusted,
}

impl TrustLevel {
    pub fn as_str(self) -> &'static str {
        match self {
            TrustLevel::Untrusted => "untrusted",
            TrustLevel::Limited => "limited",
            TrustLevel::Trusted => "trusted",
        }
    }
}

/// The state of a run.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RunStatus {
    Queued,
    Running,
    Waiting,
    Succeeded,
    Failed,
}

impl RunStatus {
    pub fn as_str(self) -> &'static str {
        match self {
            RunStatus::Queued => "queued",
            RunStatus::Running => "running",
            RunStatus::Waiting => "waiting",
            RunStatus::Succeeded => "succeeded",
            RunStatus::Failed => "failed",
        }
    }
}

/// The state of an approval request.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ApprovalState {
    Pending,
    Approved,
    Rejected,
}

impl ApprovalState {
    pub fn as_str(self) -> &'static str {
        match self {
            ApprovalState::Pending => "pending",
            ApprovalState::Approved => "approved",
            ApprovalState::Rejected => "rejected",
        }
    }
}

/// How urgent an approval is.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Severity {
    Low,
    Medium,
    High,
}

impl Severity {
    pub fn as_str(self) -> &'static str {
        match self {
            Severity::Low => "low",
            Severity::Medium => "medium",
            Severity::High => "high",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AgentCapability {
    pub name: String,
    pub description: String,
    pub granted: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Agent {
    pub id: String,
    pub display_name: String,
    pub adapter: String,
    pub host_class: String,
    pub status: AgentStatus,
    pub trust_level: TrustLevel,
    pub capabilities: Vec<AgentCapability>,
    pub current_run_id: Option<String>,
    pub last_heartbeat: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RunStage {
    pub name: String,
    pub status: RunStatus,
    pub started_at: String,
    pub ended_at: Option<String>,
    pub detail: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RunEvent {
    pub id: String,
    pub run_id: String,
    pub agent_id: String,
    pub kind: String,
    pub message: String,
    pub at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Run {
    pub id: String,
    pub agent_id: String,
    pub agent_name: String,
    pub task: String,
    pub status: RunStatus,
    pub started_at: String,
    pub ended_at: Option<String>,
    pub duration_seconds: Option<i64>,
    pub artifacts: Vec<String>,
    pub stages: Vec<RunStage>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Approval {
    pub id: String,
    pub run_id: String,
    pub agent_id: String,
    pub agent_name: String,
    pub severity: Severity,
    pub reason: String,
    pub evidence: Vec<String>,
    pub confidence: f64,
    pub state: ApprovalState,
    pub created_at: String,
    pub decided_at: Option<String>,
    pub comment: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ApprovalDecision {
    pub id: String,
    pub approval_id: String,
    pub outcome: String,
    pub comment: Option<String>,
    pub at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Tool {
    pub id: String,
    pub name: String,
    pub kind: String,
    pub status: String,
    pub latency_ms: Option<i64>,
    pub success_rate: f64,
    pub calls_last_hour: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Policy {
    pub id: String,
    pub name: String,
    pub scope: String,
    pub effect: String,
    pub enabled: bool,
    pub detail: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AuditEvent {
    pub id: String,
    pub at: String,
    pub actor: String,
    pub action: String,
    pub subject: String,
    pub outcome: String,
    pub detail: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TeamMember {
    pub id: String,
    pub name: String,
    pub role: String,
    pub on_shift: bool,
    pub agent_ids: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AppSettings {
    pub theme: String,
    pub compact_mode: bool,
    pub simulator_speed: f64,
    pub timezone: String,
    pub notifications_enabled: bool,
}

impl Default for AppSettings {
    fn default() -> Self {
        AppSettings {
            theme: "dark".to_string(),
            compact_mode: false,
            simulator_speed: 1.0,
            timezone: "UTC".to_string(),
            notifications_enabled: true,
        }
    }
}

/// The four summary cards on the Night Shift page.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DashboardSummary {
    pub runs_total: i64,
    pub finished_autonomously: i64,
    pub waiting_on_you: i64,
    pub compute_used_units: f64,
    pub agent_status_counts: Vec<AgentStatusCount>,
    pub compute_by_model: Vec<ComputeByModel>,
    pub reviewer_disagreement: Vec<DisagreementRow>,
    pub tool_health: Vec<ToolHealthRow>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AgentStatusCount {
    pub status: String,
    pub count: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ComputeByModel {
    pub model: String,
    pub units: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DisagreementRow {
    pub run_id: String,
    pub subject: String,
    pub reviewers: i64,
    pub split: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ToolHealthRow {
    pub tool_id: String,
    pub name: String,
    pub status: String,
    pub success_rate: f64,
    pub latency_ms: Option<i64>,
}

/// The simulated shift window, in minutes before "now" through "now".
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TimelineEntry {
    pub run_id: String,
    pub agent_id: String,
    pub agent_name: String,
    pub status: String,
    pub task: String,
    pub start_minute: i64,
    pub end_minute: i64,
    pub severity: Option<String>,
}
