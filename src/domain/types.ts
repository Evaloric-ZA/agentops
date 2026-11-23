// The TypeScript mirror of the Rust domain types.
//
// These are the shapes the command layer returns. Keep them in step with
// `src-tauri/src/domain/mod.rs`.

export type AgentStatus = "working" | "idle" | "approval" | "failed" | "offline";
export type TrustLevel = "untrusted" | "limited" | "trusted";
export type RunStatus = "queued" | "running" | "waiting" | "succeeded" | "failed";
export type ApprovalState = "pending" | "approved" | "rejected";
export type Severity = "low" | "medium" | "high";

export interface AgentCapability {
  name: string;
  description: string;
  granted: boolean;
}

export interface Agent {
  id: string;
  displayName: string;
  adapter: string;
  hostClass: string;
  status: AgentStatus;
  trustLevel: TrustLevel;
  capabilities: AgentCapability[];
  currentRunId: string | null;
  lastHeartbeat: string;
}

export interface RunStage {
  name: string;
  status: RunStatus;
  startedAt: string;
  endedAt: string | null;
  detail: string;
}

export interface RunEvent {
  id: string;
  runId: string;
  agentId: string;
  kind: string;
  message: string;
  at: string;
}

export interface Run {
  id: string;
  agentId: string;
  agentName: string;
  task: string;
  status: RunStatus;
  startedAt: string;
  endedAt: string | null;
  durationSeconds: number | null;
  artifacts: string[];
  stages: RunStage[];
}

export interface Approval {
  id: string;
  runId: string;
  agentId: string;
  agentName: string;
  severity: Severity;
  reason: string;
  evidence: string[];
  confidence: number;
  state: ApprovalState;
  createdAt: string;
  decidedAt: string | null;
  comment: string | null;
}

export interface Tool {
  id: string;
  name: string;
  kind: string;
  status: string;
  latencyMs: number | null;
  successRate: number;
  callsLastHour: number;
}

export interface Policy {
  id: string;
  name: string;
  scope: string;
  effect: string;
  enabled: boolean;
  detail: string;
}

export interface AuditEvent {
  id: string;
  at: string;
  actor: string;
  action: string;
  subject: string;
  outcome: string;
  detail: string;
}

export interface TeamMember {
  id: string;
  name: string;
  role: string;
  onShift: boolean;
  agentIds: string[];
}

export interface AppSettings {
  theme: string;
  compactMode: boolean;
  simulatorSpeed: number;
  timezone: string;
  notificationsEnabled: boolean;
}

export interface AgentStatusCount {
  status: AgentStatus;
  count: number;
}

export interface ComputeByModel {
  model: string;
  units: number;
}

export interface DisagreementRow {
  runId: string;
  subject: string;
  reviewers: number;
  split: string;
}

export interface ToolHealthRow {
  toolId: string;
  name: string;
  status: string;
  successRate: number;
  latencyMs: number | null;
}

export interface DashboardSummary {
  runsTotal: number;
  finishedAutonomously: number;
  waitingOnYou: number;
  computeUsedUnits: number;
  agentStatusCounts: AgentStatusCount[];
  computeByModel: ComputeByModel[];
  reviewerDisagreement: DisagreementRow[];
  toolHealth: ToolHealthRow[];
}

export interface TimelineEntry {
  runId: string;
  agentId: string;
  agentName: string;
  status: string;
  task: string;
  startMinute: number;
  endMinute: number;
  severity: Severity | null;
}

export interface SimulatorView {
  running: boolean;
  step: number;
  speed: number;
  lastOutcome: string | null;
}

/** The structured error every command returns on failure. */
export interface AppError {
  code: string;
  message: string;
}
