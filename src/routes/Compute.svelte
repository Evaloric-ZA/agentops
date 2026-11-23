<script lang="ts">
  /** Compute allocation, by model and by agent. */
  import { api } from "../lib/tauri";
  import { createResource } from "../stores/data";
  import { formatNumber } from "../lib/format";
  import Panel from "../components/Panel.svelte";
  import StateBlock from "../components/StateBlock.svelte";
  import type { DashboardSummary, Run } from "../domain/types";

  const summary = createResource<DashboardSummary>(() => api.dashboardSummary());
  const runs = createResource<Run[]>(() => api.listRuns());

  $effect(() => {
    void summary.load();
    void runs.load();
  });

  const byAgent = $derived.by(() => {
    const totals = new Map<string, number>();
    for (const run of $runs.data ?? []) {
      totals.set(run.agentName, (totals.get(run.agentName) ?? 0) + 1);
    }
    return [...totals.entries()]
      .map(([agent, runsCount]) => ({ agent, runsCount }))
      .sort((a, b) => b.runsCount - a.runsCount);
  });

  const maxUnits = $derived(
    Math.max(1, ...($summary.data?.computeByModel ?? []).map((row) => row.units)),
  );
</script>

<div class="page">
  <div class="grid">
    <Panel title="Compute by model" subtitle="Sample compute units">
      <StateBlock
        status={$summary.status}
        error={$summary.error}
        emptyTitle="No compute recorded"
        emptyBody="Start the simulator to record some."
      />
      {#if $summary.status === "loaded" && ($summary.data?.computeByModel.length ?? 0) > 0}
        <ul class="bars">
          {#each $summary.data?.computeByModel ?? [] as row (row.model)}
            <li>
              <div class="bar-head">
                <span class="model">{row.model}</span>
                <span class="units">{row.units.toFixed(2)}</span>
              </div>
              <div class="track">
                <div class="fill" style="width: {(row.units / maxUnits) * 100}%"></div>
              </div>
            </li>
          {/each}
        </ul>
      {/if}
    </Panel>

    <Panel title="Runs by agent" subtitle="Who did the work">
      <StateBlock
        status={$runs.status}
        error={$runs.error}
        emptyTitle="No runs recorded"
        emptyBody="There is nothing to attribute yet."
      />
      {#if $runs.status === "loaded" && byAgent.length > 0}
        <ul class="rows">
          {#each byAgent as row (row.agent)}
            <li>
              <span class="name">{row.agent}</span>
              <span class="count">{formatNumber(row.runsCount)} run(s)</span>
            </li>
          {/each}
        </ul>
      {/if}
    </Panel>

    <Panel title="Total" subtitle="Across the whole shift">
      <StateBlock
        status={$summary.status}
        error={$summary.error}
        emptyTitle="Nothing to add up"
        emptyBody="The total appears once the simulator runs."
      />
      {#if $summary.status === "loaded" && $summary.data}
        <p class="total">{$summary.data.computeUsedUnits.toFixed(2)} units</p>
        <p class="muted">
          This is a synthetic figure. v0.1 measures nothing real and bills
          nothing.
        </p>
      {/if}
    </Panel>
  </div>
</div>

<style>
  .page {
    padding: var(--space-4);
    height: 100%;
    overflow-y: auto;
  }

  .grid {
    display: grid;
    grid-template-columns: repeat(auto-fit, minmax(280px, 1fr));
    gap: var(--space-4);
    align-items: start;
  }

  .bars {
    list-style: none;
    margin: 0;
    padding: 0;
  }

  .bars li {
    margin-bottom: var(--space-3);
  }

  .bar-head {
    display: flex;
    justify-content: space-between;
    font-size: 12px;
    margin-bottom: 4px;
  }

  .model {
    font-family: var(--font-mono);
    font-size: 11px;
  }

  .units {
    color: var(--text-muted);
    font-variant-numeric: tabular-nums;
  }

  .track {
    height: 8px;
    border-radius: 4px;
    background: var(--bg-base);
    overflow: hidden;
  }

  .fill {
    height: 100%;
    background: var(--accent);
    border-radius: 4px;
  }

  .rows {
    list-style: none;
    margin: 0;
    padding: 0;
  }

  .rows li {
    display: flex;
    justify-content: space-between;
    gap: var(--space-2);
    padding: var(--space-2) 0;
    border-bottom: 1px solid var(--border);
    font-size: 12px;
  }

  .rows li:last-child {
    border-bottom: 0;
  }

  .name {
    font-weight: 600;
  }

  .count {
    color: var(--text-muted);
  }

  .total {
    font-size: 28px;
    font-weight: 650;
    margin: 0 0 var(--space-2);
    font-variant-numeric: tabular-nums;
  }

  .muted {
    font-size: 12px;
    color: var(--text-muted);
    margin: 0;
  }
</style>
