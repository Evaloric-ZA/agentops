<script lang="ts">
  /**
   * The application shell.
   *
   * It owns the navigation, the settings, and the simulator driver. The
   * simulator is driven from here by an interval whose period is derived
   * from the configured speed, so the backend only ever advances when the
   * frontend asks it to. Nothing runs faster than the UI can render.
   */
  import Sidebar from "../components/Sidebar.svelte";
  import Topbar from "../components/Topbar.svelte";
  import NightShift from "../routes/NightShift.svelte";
  import Agents from "../routes/Agents.svelte";
  import Runs from "../routes/Runs.svelte";
  import WaitingOnYou from "../routes/WaitingOnYou.svelte";
  import Policies from "../routes/Policies.svelte";
  import Access from "../routes/Access.svelte";
  import Compute from "../routes/Compute.svelte";
  import AuditLog from "../routes/AuditLog.svelte";
  import Tools from "../routes/Tools.svelte";
  import Team from "../routes/Team.svelte";
  import Settings from "../routes/Settings.svelte";
  import Crew from "../routes/Crew.svelte";
  import { api } from "../lib/tauri";
  import { currentView, loadSettings, settings, shellError } from "../stores/app";
  import type { Agent, Approval, SimulatorView } from "../domain/types";

  let agents = $state<Agent[]>([]);
  let approvals = $state<Approval[]>([]);
  let simulator = $state<SimulatorView>({
    running: false,
    step: 0,
    speed: 1,
    lastOutcome: null,
  });
  let globalSearch = $state("");
  let focusApprovalId = $state("");

  $effect(() => {
    void loadSettings();
  });

  /** Reload the data the shell shows in the chrome. */
  async function refreshShell() {
    try {
      const [agentRows, approvalRows, simulatorState] = await Promise.all([
        api.listAgents(),
        api.listApprovals("pending"),
        api.simulatorState(),
      ]);
      agents = agentRows;
      approvals = approvalRows;
      simulator = simulatorState;
    } catch (error) {
      shellError.set(error instanceof Error ? error.message : String(error));
    }
  }

  $effect(() => {
    void refreshShell();
  });

  // The simulator driver. The period is the base interval divided by the
  // configured speed, clamped so the UI is never asked to render faster
  // than it can keep up.
  $effect(() => {
    if (!simulator.running) return;
    const period = Math.max(400, Math.round(2500 / simulator.speed));
    const handle = setInterval(async () => {
      try {
        simulator = await api.simulatorTick();
        await refreshShell();
      } catch (error) {
        shellError.set(error instanceof Error ? error.message : String(error));
      }
    }, period);
    return () => clearInterval(handle);
  });

  async function toggleSimulator() {
    try {
      simulator = simulator.running
        ? await api.simulatorPause()
        : await api.simulatorStart();
      await refreshShell();
    } catch (error) {
      shellError.set(error instanceof Error ? error.message : String(error));
    }
  }

  function onSearch(term: string) {
    globalSearch = term;
    if (term.trim().length > 0) {
      currentView.set("runs");
    }
  }

  function selectApproval(id: string) {
    focusApprovalId = id;
    currentView.set("waiting");
  }
</script>

<div class="shell">
  <Sidebar {agents} />

  <div class="right">
    <Topbar
      onSearch={onSearch}
      notificationsEnabled={$settings.notificationsEnabled}
      pendingApprovals={approvals.length}
      onOpenApprovals={() => selectApproval("")}
    />

    <div class="sim-bar">
      <span class="sim-label">Simulator</span>
      <span class="sim-state" class:running={simulator.running}>
        {simulator.running ? "running" : "paused"}
      </span>
      <span class="sim-step">step {simulator.step}</span>
      {#if simulator.lastOutcome}
        <span class="sim-outcome">last: {simulator.lastOutcome}</span>
      {/if}
      <button class="sim-toggle" onclick={toggleSimulator}>
        {simulator.running ? "Pause" : "Start"}
      </button>
      <span class="sim-note">All activity is simulated and local.</span>
    </div>

    {#if $shellError}
      <div class="shell-error" role="alert">
        <span>{$shellError}</span>
        <button onclick={() => shellError.set(null)}>Dismiss</button>
      </div>
    {/if}

    <main class="view">
      {#if $currentView === "night-shift"}
        <NightShift onSelectApproval={selectApproval} />
      {:else if $currentView === "agents"}
        <Agents initialSearch={globalSearch} />
      {:else if $currentView === "runs"}
        <Runs initialSearch={globalSearch} />
      {:else if $currentView === "waiting"}
        <WaitingOnYou focusId={focusApprovalId} />
      {:else if $currentView === "policies"}
        <Policies />
      {:else if $currentView === "access"}
        <Access />
      {:else if $currentView === "compute"}
        <Compute />
      {:else if $currentView === "audit"}
        <AuditLog />
      {:else if $currentView === "tools"}
        <Tools />
      {:else if $currentView === "team"}
        <Team />
      {:else if $currentView === "settings"}
        <Settings />
      {:else if $currentView === "crew"}
        <Crew />
      {/if}
    </main>
  </div>
</div>

<style>
  .shell {
    display: flex;
    height: 100%;
    overflow: hidden;
  }

  .right {
    flex: 1;
    min-width: 0;
    display: flex;
    flex-direction: column;
    overflow: hidden;
  }

  .sim-bar {
    display: flex;
    align-items: center;
    gap: var(--space-3);
    padding: 5px var(--space-4);
    background: var(--bg-sunken);
    border-bottom: 1px solid var(--border);
    font-size: 11px;
    color: var(--text-muted);
    flex: none;
    flex-wrap: wrap;
  }

  .sim-label {
    font-weight: 700;
    text-transform: uppercase;
    letter-spacing: 0.07em;
  }

  .sim-state {
    color: var(--text-secondary);
  }

  .sim-state.running {
    color: var(--ok);
  }

  .sim-outcome {
    font-family: var(--font-mono);
  }

  .sim-toggle {
    border: 1px solid var(--border-strong);
    background: var(--bg-raised);
    border-radius: var(--radius-md);
    padding: 3px 10px;
    font-size: 11px;
    font-weight: 600;
  }

  .sim-toggle:hover {
    background: var(--bg-hover);
  }

  .sim-note {
    margin-left: auto;
    font-style: italic;
  }

  .shell-error {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: var(--space-3);
    padding: var(--space-2) var(--space-4);
    background: var(--danger-soft);
    color: var(--danger);
    font-size: 12px;
    flex: none;
  }

  .shell-error button {
    border: 1px solid currentColor;
    background: transparent;
    color: inherit;
    border-radius: var(--radius-sm);
    padding: 2px 8px;
    font-size: 11px;
  }

  .view {
    flex: 1;
    min-height: 0;
    overflow: hidden;
  }
</style>
