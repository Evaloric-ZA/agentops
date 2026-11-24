// The frontend integration tests.
//
// These drive the real components against a deterministic mock of the
// Tauri command boundary. Nothing here reaches the Rust side: every
// command is answered by a fixture, so a test states exactly what the
// backend returned and what the operator would see.
//
// The tests are about the seam between the two halves: the command name
// and arguments that go out, and the text that comes back in.

import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import { cleanup, fireEvent, render, screen, within } from "@testing-library/svelte";
import { tick } from "svelte";

const invoke = vi.fn();

vi.mock("@tauri-apps/api/core", () => ({
  invoke: (...args: unknown[]) => invoke(...args),
}));

import App from "../app/App.svelte";
import Agents from "../routes/Agents.svelte";
import Policies from "../routes/Policies.svelte";
import Runs from "../routes/Runs.svelte";
import Settings from "../routes/Settings.svelte";
import WaitingOnYou from "../routes/WaitingOnYou.svelte";
import { currentView } from "../stores/app";
import type {
  Agent,
  AppSettings,
  Approval,
  DashboardSummary,
  Policy,
  Run,
  SimulatorView,
} from "../domain/types";

type Route = (args: Record<string, unknown>) => unknown;

const routes = new Map<string, Route>();

const SETTINGS: AppSettings = {
  theme: "dark",
  compactMode: false,
  simulatorSpeed: 1,
  timezone: "UTC",
  notificationsEnabled: true,
};

const AGENT_ALPHA: Agent = {
  id: "agent-alpha",
  displayName: "Alpha Agent",
  adapter: "local-adapter",
  hostClass: "workstation",
  status: "working",
  trustLevel: "trusted",
  capabilities: [{ name: "read", description: "Reads the local files", granted: true }],
  currentRunId: "run-alpha",
  lastHeartbeat: "2026-01-01T18:00:00Z",
};

const AGENT_BETA: Agent = {
  id: "agent-beta",
  displayName: "Beta Agent",
  adapter: "local-adapter",
  hostClass: "server",
  status: "idle",
  trustLevel: "limited",
  capabilities: [],
  currentRunId: null,
  lastHeartbeat: "2026-01-01T18:00:00Z",
};

const RUN_ONE: Run = {
  id: "run-alpha",
  agentId: "agent-alpha",
  agentName: "Alpha Agent",
  task: "migrate the sample index",
  status: "running",
  startedAt: "2026-01-01T18:00:00Z",
  endedAt: null,
  durationSeconds: null,
  artifacts: [],
  stages: [],
};

const RUN_TWO: Run = {
  ...RUN_ONE,
  id: "run-beta",
  agentId: "agent-beta",
  agentName: "Beta Agent",
  task: "write the nightly report",
  status: "succeeded",
  endedAt: "2026-01-01T19:00:00Z",
  durationSeconds: 3600,
};

const APPROVAL: Approval = {
  id: "approval-000000",
  runId: "run-alpha",
  agentId: "agent-alpha",
  agentName: "Alpha Agent",
  severity: "high",
  reason: "The sample write needs a decision",
  evidence: [],
  confidence: 0.8,
  state: "pending",
  createdAt: "2026-01-01T18:00:00Z",
  decidedAt: null,
  comment: null,
};

const POLICY: Policy = {
  id: "policy-read-only",
  name: "Read only",
  scope: "the workspace",
  effect: "deny",
  enabled: true,
  detail: "Nothing may be written without a decision",
};

const SUMMARY: DashboardSummary = {
  runsTotal: 42,
  finishedAutonomously: 30,
  waitingOnYou: 3,
  computeUsedUnits: 120,
  agentStatusCounts: [{ status: "working", count: 2 }],
  computeByModel: [],
  reviewerDisagreement: [],
  toolHealth: [],
};

const SIMULATOR: SimulatorView = {
  running: false,
  step: 0,
  speed: 1,
  lastOutcome: null,
};

/** The simulator's state, as the mocked backend holds it. */
let simulatorState: SimulatorView = { ...SIMULATOR };

/** Install the default answer for every command the frontend uses. */
function installRoutes(): void {
  routes.clear();
  routes.set("get_settings", () => SETTINGS);
  routes.set("update_settings", (args) => args.settings);
  routes.set("list_agents", () => [AGENT_ALPHA, AGENT_BETA]);
  routes.set("get_agent", () => AGENT_ALPHA);
  routes.set("agent_events", () => []);
  routes.set("list_runs", () => [RUN_ONE, RUN_TWO]);
  routes.set("get_run", () => RUN_ONE);
  routes.set("run_events", () => []);
  routes.set("list_approvals", () => [APPROVAL]);
  routes.set("decide_approval", () => ({
    ...APPROVAL,
    state: "approved",
    decidedAt: "2026-01-01T18:05:00Z",
  }));
  routes.set("list_audit", () => []);
  routes.set("list_policies", () => [POLICY]);
  routes.set("set_policy_enabled", () => undefined);
  routes.set("list_tools", () => []);
  routes.set("list_team", () => []);
  routes.set("dashboard_summary", () => SUMMARY);
  routes.set("dashboard_timeline", () => []);
  simulatorState = { ...SIMULATOR };
  routes.set("simulator_state", () => simulatorState);
  routes.set("simulator_start", () => {
    simulatorState = { ...simulatorState, running: true };
    return simulatorState;
  });
  routes.set("simulator_pause", () => {
    simulatorState = { ...simulatorState, running: false };
    return simulatorState;
  });
  routes.set("simulator_tick", () => {
    simulatorState = { ...simulatorState, step: simulatorState.step + 1 };
    return simulatorState;
  });
  routes.set("simulator_set_speed", () => simulatorState);
  routes.set("simulator_reset", () => {
    simulatorState = { ...SIMULATOR };
    return simulatorState;
  });
}

/** Let the effects run and the DOM settle. */
async function flush(): Promise<void> {
  await Promise.resolve();
  await tick();
  await Promise.resolve();
  await tick();
}

/** The arguments of the last call to a command. */
function lastCall(command: string): Record<string, unknown> | undefined {
  const calls = invoke.mock.calls.filter((call) => call[0] === command);
  const last = calls[calls.length - 1];
  return last?.[1] as Record<string, unknown> | undefined;
}

beforeEach(() => {
  invoke.mockReset();
  installRoutes();
  // The view store is module state, so a test that navigates would
  // otherwise decide what the next test sees.
  currentView.set("night-shift");
  invoke.mockImplementation(async (command: string, args: Record<string, unknown> = {}) => {
    const route = routes.get(command);
    if (!route) {
      throw { code: "unknown_command", message: `no answer was prepared for ${command}` };
    }
    return route(args);
  });
});

afterEach(() => {
  cleanup();
});

describe("the application shell", () => {
  it("renders the navigation, the simulator bar and the overview", async () => {
    render(App);
    await flush();

    expect(screen.getByRole("navigation", { name: "Primary" })).toBeTruthy();
    expect(screen.getByText("Simulator")).toBeTruthy();
    expect(screen.getByRole("button", { name: "Night Shift" })).toBeTruthy();
    // The overview's own summary is on screen, not an empty frame.
    expect(screen.getByText("in the current shift")).toBeTruthy();
  });

  it("navigates to another view from the sidebar", async () => {
    render(App);
    await flush();

    await fireEvent.click(screen.getByRole("button", { name: /Agents/ }));
    await flush();

    const view = within(screen.getByRole("main"));
    expect(view.getByText("Agent registry")).toBeTruthy();
    expect(view.getByText("Alpha Agent")).toBeTruthy();
  });

  it("shows the Night Shift summary numbers", async () => {
    render(App);
    await flush();

    const view = within(screen.getByRole("main"));
    expect(view.getByText("42")).toBeTruthy();
    expect(view.getByText("no human decision needed")).toBeTruthy();
  });
});

describe("the agent registry", () => {
  it("filters the agents by the search term", async () => {
    render(Agents, { props: { initialSearch: "" } });
    await flush();

    expect(screen.getByText("Alpha Agent")).toBeTruthy();
    expect(screen.getByText("Beta Agent")).toBeTruthy();

    await fireEvent.input(screen.getByLabelText("Search agents"), {
      target: { value: "beta" },
    });
    await flush();

    expect(screen.queryByText("Alpha Agent")).toBeNull();
    expect(screen.getByText("Beta Agent")).toBeTruthy();
  });

  it("filters the agents by status", async () => {
    render(Agents, { props: { initialSearch: "" } });
    await flush();

    await fireEvent.change(screen.getByLabelText("Filter by status"), {
      target: { value: "idle" },
    });
    await flush();

    expect(screen.queryByText("Alpha Agent")).toBeNull();
    expect(screen.getByText("Beta Agent")).toBeTruthy();
  });

  it("says so when nothing matches the filter", async () => {
    render(Agents, { props: { initialSearch: "" } });
    await flush();

    await fireEvent.input(screen.getByLabelText("Search agents"), {
      target: { value: "nothing matches this" },
    });
    await flush();

    expect(screen.getByText(/No agent matches that filter/)).toBeTruthy();
  });
});

describe("the run registry", () => {
  it("sends the search term and the status filter to the backend", async () => {
    render(Runs, { props: { initialSearch: "" } });
    await flush();

    await fireEvent.input(screen.getByLabelText("Search runs"), {
      target: { value: "migration" },
    });
    await flush();

    expect(lastCall("list_runs")).toEqual({ status: "all", search: "migration" });
  });
});

describe("the approval queue", () => {
  it("records a decision and says what happened", async () => {
    render(WaitingOnYou, { props: { focusId: "" } });
    await flush();

    await fireEvent.click(screen.getByText(APPROVAL.reason));
    await flush();
    await fireEvent.click(screen.getByRole("button", { name: "Approve" }));
    await flush();

    expect(lastCall("decide_approval")).toEqual({
      approvalId: APPROVAL.id,
      outcome: "approved",
      comment: null,
    });
    expect(screen.getByText(/The approval was approved/)).toBeTruthy();
  });

  it("keeps the confirmation after the approval leaves the pending list", async () => {
    // A decided approval is no longer pending, so it leaves the list the
    // drawer is drawn from. The confirmation has to survive that.
    let decided = false;
    routes.set("list_approvals", () => (decided ? [] : [APPROVAL]));
    routes.set("decide_approval", () => {
      decided = true;
      return { ...APPROVAL, state: "approved", decidedAt: "2026-01-01T18:05:00Z" };
    });

    render(WaitingOnYou, { props: { focusId: "" } });
    await flush();

    await fireEvent.click(screen.getByText(APPROVAL.reason));
    await flush();
    await fireEvent.click(screen.getByRole("button", { name: "Approve" }));
    await flush();

    expect(screen.getByText(/The approval was approved/)).toBeTruthy();
  });

  it("shows the error when the decision fails", async () => {
    routes.set("decide_approval", () => {
      throw { code: "db_error", message: "database unavailable" };
    });

    render(WaitingOnYou, { props: { focusId: "" } });
    await flush();

    await fireEvent.click(screen.getByText(APPROVAL.reason));
    await flush();
    await fireEvent.click(screen.getByRole("button", { name: "Reject" }));
    await flush();

    expect(screen.getByText("database unavailable")).toBeTruthy();
    expect(screen.queryByText(/The approval was rejected/)).toBeNull();
  });
});

describe("the policy set", () => {
  it("turns a policy off and reloads the list", async () => {
    render(Policies);
    await flush();

    await fireEvent.click(screen.getByRole("checkbox"));
    await flush();

    expect(lastCall("set_policy_enabled")).toEqual({
      policyId: POLICY.id,
      enabled: false,
    });
    expect(screen.getByText(/was disabled/)).toBeTruthy();
  });

  it("shows the error when the policy does not exist", async () => {
    routes.set("set_policy_enabled", () => {
      throw { code: "not_found", message: "record not found" };
    });

    render(Policies);
    await flush();

    await fireEvent.click(screen.getByRole("checkbox"));
    await flush();

    expect(screen.getByText(/was not changed/)).toBeTruthy();
    expect(screen.getByText(/record not found/)).toBeTruthy();
  });
});

describe("the simulator", () => {
  it("starts and pauses from the shell", async () => {
    render(App);
    await flush();

    await fireEvent.click(screen.getByRole("button", { name: "Start" }));
    await flush();
    expect(invoke).toHaveBeenCalledWith("simulator_start", undefined);

    await fireEvent.click(screen.getByRole("button", { name: "Pause" }));
    await flush();
    expect(invoke).toHaveBeenCalledWith("simulator_pause", undefined);
  });

  it("advances the simulation on its own interval", async () => {
    vi.useFakeTimers();
    try {
      routes.set("simulator_state", () => ({ ...SIMULATOR, running: true }));
      render(App);
      await flush();
      invoke.mockClear();

      await vi.advanceTimersByTimeAsync(2600);
      await flush();

      expect(invoke).toHaveBeenCalledWith("simulator_tick", undefined);
    } finally {
      vi.useRealTimers();
    }
  });

  it("resets the demo data from the settings", async () => {
    render(Settings);
    await flush();

    await fireEvent.click(screen.getByRole("button", { name: "Reset demo data" }));
    await flush();

    expect(invoke).toHaveBeenCalledWith("simulator_reset", undefined);
    expect(screen.getByText(/restored to its original seeded state/)).toBeTruthy();
  });
});

describe("a structured backend error", () => {
  it("is shown on the view that failed to load", async () => {
    routes.set("list_policies", () => {
      throw { code: "db_error", message: "database unavailable" };
    });

    render(Policies);
    await flush();

    expect(screen.getByText("Could not load this view")).toBeTruthy();
    expect(screen.getByText("database unavailable")).toBeTruthy();
    // The failure says that nothing was changed.
    expect(screen.getByText(/Nothing was changed/)).toBeTruthy();
  });
});

describe("the settings", () => {
  it("sends the edited settings and uses the server's answer", async () => {
    const saved: AppSettings = { ...SETTINGS, timezone: "Europe/London" };
    routes.set("update_settings", () => saved);

    render(Settings);
    await flush();

    await fireEvent.change(screen.getByLabelText("Timezone"), {
      target: { value: "Europe/London" },
    });
    await flush();
    await fireEvent.click(screen.getByRole("button", { name: "Save settings" }));
    await flush();

    const sent = lastCall("update_settings") as { settings: AppSettings };
    expect(sent.settings.timezone).toBe("Europe/London");
    // The server's answer is what the store holds, not the local edit.
    expect(screen.getByText("Saved.")).toBeTruthy();
  });

  it("shows the error when the settings cannot be saved", async () => {
    routes.set("update_settings", () => {
      throw { code: "invalid_input", message: "invalid input: the timezone is unknown" };
    });

    render(Settings);
    await flush();

    await fireEvent.click(screen.getByRole("button", { name: "Save settings" }));
    await flush();

    expect(screen.getByText(/the timezone is unknown/)).toBeTruthy();
    expect(screen.queryByText("Saved.")).toBeNull();
  });
});
