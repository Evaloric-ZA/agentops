<script lang="ts">
  /** The Night Shift overview: four cards, the timeline and the lower panels. */
  import { api } from "../lib/tauri";
  import { createResource } from "../stores/data";
  import { formatNumber, formatPercent, formatTime } from "../lib/format";
  import Panel from "../components/Panel.svelte";
  import SummaryCard from "../components/SummaryCard.svelte";
  import StatusPill from "../components/StatusPill.svelte";
  import StateBlock from "../components/StateBlock.svelte";
  import Button from "../components/Button.svelte";
  import type { DashboardSummary, TimelineEntry } from "../domain/types";

  interface Props {
    onSelectApproval: (approvalId: string) => void;
  }

  const { onSelectApproval }: Props = $props();

  const summary = createResource<DashboardSummary>(() => api.dashboardSummary());
  const timeline = createResource<TimelineEntry[]>(
    () => api.dashboardTimeline(),
    (rows) => rows.length === 0,
  );

  $effect(() => {
    void summary.load();
    void timeline.load();
  });

  const cards = $derived.by(() => {
    const data = $summary.data;
    if (!data) return [];
    return [
      {
        label: "Runs",
        value: formatNumber(data.runsTotal),
        detail: "in the current shift",
        tone: "accent" as const,
      },
      {
        label: "Finished Autonomously",
        value: formatNumber(data.finishedAutonomously),
        detail: "no human decision needed",
        tone: "ok" as const,
      },
      {
        label: "Waiting on You",
        value: formatNumber(data.waitingOnYou),
        detail: "approvals need a reviewer",
        tone: "warn" as const,
      },
      {
        label: "Compute Used",
        value: data.computeUsedUnits.toFixed(1),
        detail: "sample compute units",
        tone: "info" as const,
      },
    ];
  });

  const windowMinutes = $derived.by(() => {
    const rows = $timeline.data ?? [];
    if (rows.length === 0) return 1;
    return Math.max(1, ...rows.map((row) => row.endMinute)) + 1;
  });

  /** The shift window starts four hours before the first run. */
  function shiftStart(tick: number): string {
    const base = new Date();
    base.setHours(base.getHours() - 4);
    base.setMinutes(base.getMinutes() + tick * 60);
    return base.toISOString();
  }
</script>

<div class="night">
  <div class="cards">
    {#each cards as card (card.label)}
      <SummaryCard label={card.label} value={card.value} detail={card.detail} tone={card.tone} />
    {/each}
  </div>

  <div class="main">
    <Panel title="Shift timeline" subtitle="Every run in the shift window" flush>
      {#snippet actions()}
        <span class="legend">
          {#each ["working", "idle", "approval", "failed"] as state (state)}
            <StatusPill status={state} />
          {/each}
        </span>
      {/snippet}
      <StateBlock
        status={$timeline.status}
        error={$timeline.error}
        emptyTitle="No runs in this shift"
        emptyBody="Start the simulator to create one."
      />
      {#if $timeline.status === "loaded"}
        <div class="timeline">
          <div class="axis">
            {#each Array.from({ length: 7 }, (_, i) => i) as tick (tick)}
              <span>{formatTime(shiftStart(tick))}</span>
            {/each}
          </div>
          {#each $timeline.data ?? [] as entry (entry.runId)}
            <div class="row">
              <span class="who" title={entry.task}>{entry.agentName}</span>
              <div class="track">
                <div
                  class="bar bar-{entry.status}"
                  style="left: {(entry.startMinute / windowMinutes) * 100}%; width: {Math.max(
                    2,
                    ((entry.endMinute - entry.startMinute) / windowMinutes) * 100,
                  )}%"
                  title="{entry.task} — {entry.status}"
                ></div>
              </div>
            </div>
          {/each}
        </div>
      {/if}
    </Panel>

    <Panel title="Waiting on You" subtitle="Approvals that need a human" flush>
      {#snippet actions()}
        <Button label="Open queue" onClick={() => onSelectApproval("")} />
      {/snippet}
      <StateBlock
        status={$summary.status}
        error={$summary.error}
        emptyTitle="Nothing is waiting"
        emptyBody="No approval needs a decision right now."
      />
      {#if $summary.status === "loaded" && $summary.data}
        {#if $summary.data.waitingOnYou === 0}
          <p class="quiet">Nothing is waiting on you.</p>
        {:else}
          <ul class="waiting">
            {#each $summary.data.reviewerDisagreement as row (row.runId)}
              <li>
                <button class="wait-row" onclick={() => onSelectApproval(row.runId)}>
                  <span class="wait-subject">{row.subject}</span>
                  <span class="wait-meta">{row.reviewers} reviewer(s) — {row.split}</span>
                </button>
              </li>
            {/each}
          </ul>
        {/if}
      {/if}
    </Panel>
  </div>

  <div class="lower">
    <Panel title="Reviewer disagreement" subtitle="Where reviewers did not align">
      <StateBlock
        status={$summary.status}
        error={$summary.error}
        emptyTitle="No disagreements"
        emptyBody="Every reviewed run had aligned reviewers."
      />
      {#if $summary.status === "loaded" && $summary.data && $summary.data.reviewerDisagreement.length > 0}
        <ul class="rows">
          {#each $summary.data.reviewerDisagreement as row (row.runId)}
            <li>
              <span class="row-main">{row.subject}</span>
              <span class="row-meta">{row.reviewers} reviewer(s)</span>
              <StatusPill status={row.split === "aligned" ? "approved" : "pending"} label={row.split} />
            </li>
          {/each}
        </ul>
      {/if}
    </Panel>

    <Panel title="Compute allocation" subtitle="By model">
      <StateBlock
        status={$summary.status}
        error={$summary.error}
        emptyTitle="No compute recorded"
        emptyBody="Start the simulator to record some."
      />
      {#if $summary.status === "loaded" && $summary.data && $summary.data.computeByModel.length > 0}
        <ul class="rows">
          {#each $summary.data.computeByModel as row (row.model)}
            <li>
              <span class="row-main">{row.model}</span>
              <span class="row-meta">{row.units.toFixed(2)} units</span>
            </li>
          {/each}
        </ul>
      {/if}
    </Panel>

    <Panel title="Tool health" subtitle="The registered tools">
      <StateBlock
        status={$summary.status}
        error={$summary.error}
        emptyTitle="No tools registered"
        emptyBody="The tool registry is empty."
      />
      {#if $summary.status === "loaded" && $summary.data && $summary.data.toolHealth.length > 0}
        <ul class="rows">
          {#each $summary.data.toolHealth as row (row.toolId)}
            <li>
              <span class="row-main">{row.name}</span>
              <span class="row-meta">
                {formatPercent(row.successRate)}
                {#if row.latencyMs !== null}&middot; {row.latencyMs}ms{/if}
              </span>
              <StatusPill status={row.status} />
            </li>
          {/each}
        </ul>
      {/if}
    </Panel>
  </div>
</div>

<style>
  .night {
    display: flex;
    flex-direction: column;
    gap: var(--space-4);
    padding: var(--space-4);
    height: 100%;
    overflow-y: auto;
  }

  .cards {
    display: grid;
    grid-template-columns: repeat(auto-fit, minmax(190px, 1fr));
    gap: var(--space-3);
    flex: none;
  }

  .main {
    display: grid;
    grid-template-columns: minmax(0, 2fr) minmax(260px, 1fr);
    gap: var(--space-4);
    min-height: 320px;
  }

  .lower {
    display: grid;
    grid-template-columns: repeat(auto-fit, minmax(260px, 1fr));
    gap: var(--space-4);
  }

  .legend {
    display: flex;
    gap: var(--space-1);
    flex-wrap: wrap;
  }

  .timeline {
    padding: var(--space-3) var(--space-4);
  }

  .axis {
    display: flex;
    justify-content: space-between;
    font-size: 10px;
    color: var(--text-muted);
    padding-left: 132px;
    margin-bottom: var(--space-2);
  }

  .row {
    display: flex;
    align-items: center;
    gap: var(--space-2);
    margin-bottom: 6px;
  }

  .who {
    width: 120px;
    flex: none;
    font-size: 12px;
    color: var(--text-secondary);
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .track {
    position: relative;
    flex: 1;
    height: 18px;
    background: var(--bg-base);
    border-radius: var(--radius-sm);
    overflow: hidden;
  }

  .bar {
    position: absolute;
    top: 3px;
    height: 12px;
    border-radius: 3px;
    background: var(--accent);
    min-width: 2px;
  }

  .bar-working,
  .bar-running {
    background: var(--ok);
  }
  .bar-idle {
    background: var(--text-muted);
  }
  .bar-approval,
  .bar-waiting {
    background: var(--warn);
  }
  .bar-failed {
    background: var(--danger);
  }

  .waiting {
    list-style: none;
    margin: 0;
    padding: 0;
  }

  .wait-row {
    display: flex;
    flex-direction: column;
    gap: 2px;
    width: 100%;
    text-align: left;
    border: 0;
    border-bottom: 1px solid var(--border);
    background: transparent;
    padding: var(--space-3) var(--space-4);
  }

  .wait-row:hover {
    background: var(--bg-hover);
  }

  .wait-subject {
    font-weight: 600;
    font-size: 13px;
  }

  .wait-meta {
    font-size: 11px;
    color: var(--text-muted);
  }

  .quiet {
    padding: var(--space-4);
    margin: 0;
    color: var(--text-muted);
    font-size: 13px;
  }

  .rows {
    list-style: none;
    margin: 0;
    padding: 0;
  }

  .rows li {
    display: flex;
    align-items: center;
    gap: var(--space-2);
    padding: var(--space-2) 0;
    border-bottom: 1px solid var(--border);
  }

  .rows li:last-child {
    border-bottom: 0;
  }

  .row-main {
    flex: 1;
    font-size: 13px;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .row-meta {
    font-size: 11px;
    color: var(--text-muted);
    white-space: nowrap;
  }

  @media (max-width: 1024px) {
    .main {
      grid-template-columns: minmax(0, 1fr);
    }
  }
</style>
