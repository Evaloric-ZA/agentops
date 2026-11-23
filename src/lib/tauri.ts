// The typed bridge to the Rust command layer.
//
// Every call goes through `invoke`, so the frontend never touches SQLite,
// the filesystem or the simulator internals. Errors arrive as the structured
// `AppError` the Rust side defines.

import { invoke } from "@tauri-apps/api/core";
import type {
  Agent,
  AppError,
  AppSettings,
  Approval,
  AuditEvent,
  DashboardSummary,
  Policy,
  Run,
  RunEvent,
  SimulatorView,
  TeamMember,
  TimelineEntry,
  Tool,
} from "../domain/types";

/** An error raised by a command, carrying its stable code. */
export class CommandError extends Error {
  readonly code: string;

  constructor(code: string, message: string) {
    super(message);
    this.name = "CommandError";
    this.code = code;
  }
}

/**
 * Invoke a command and normalise its failure mode.
 *
 * A rejected invocation is either a structured `AppError` (the shape the
 * Rust side serialises) or a transport failure. Both become a
 * `CommandError` so callers have exactly one error type to handle.
 */
async function call<T>(command: string, args?: Record<string, unknown>): Promise<T> {
  try {
    return await invoke<T>(command, args);
  } catch (raw) {
    if (raw && typeof raw === "object" && "code" in raw && "message" in raw) {
      const structured = raw as AppError;
      throw new CommandError(structured.code, structured.message);
    }
    if (raw instanceof Error) {
      throw new CommandError("transport_error", raw.message);
    }
    throw new CommandError("unknown_error", String(raw));
  }
}

export const api = {
  dashboardSummary: () => call<DashboardSummary>("dashboard_summary"),
  dashboardTimeline: () => call<TimelineEntry[]>("dashboard_timeline"),

  listAgents: () => call<Agent[]>("list_agents"),
  getAgent: (agentId: string) => call<Agent>("get_agent", { agentId }),
  agentEvents: (agentId: string) => call<RunEvent[]>("agent_events", { agentId }),

  listRuns: (status?: string, search?: string) =>
    call<Run[]>("list_runs", { status: status ?? null, search: search ?? null }),
  getRun: (runId: string) => call<Run>("get_run", { runId }),
  runEvents: (runId: string) => call<RunEvent[]>("run_events", { runId }),

  listApprovals: (stateFilter?: string) =>
    call<Approval[]>("list_approvals", { stateFilter: stateFilter ?? null }),
  decideApproval: (approvalId: string, outcome: string, comment?: string) =>
    call<Approval>("decide_approval", { approvalId, outcome, comment: comment ?? null }),

  listAudit: (limit?: number) => call<AuditEvent[]>("list_audit", { limit: limit ?? null }),
  listPolicies: () => call<Policy[]>("list_policies"),
  setPolicyEnabled: (policyId: string, enabled: boolean) =>
    call<void>("set_policy_enabled", { policyId, enabled }),
  listTools: () => call<Tool[]>("list_tools"),
  listTeam: () => call<TeamMember[]>("list_team"),

  getSettings: () => call<AppSettings>("get_settings"),
  updateSettings: (settings: AppSettings) => call<AppSettings>("update_settings", { settings }),

  simulatorState: () => call<SimulatorView>("simulator_state"),
  simulatorStart: () => call<SimulatorView>("simulator_start"),
  simulatorPause: () => call<SimulatorView>("simulator_pause"),
  simulatorTick: () => call<SimulatorView>("simulator_tick"),
  simulatorSetSpeed: (speed: number) => call<SimulatorView>("simulator_set_speed", { speed }),
  simulatorReset: () => call<SimulatorView>("simulator_reset"),
};
