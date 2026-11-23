<script lang="ts">
  /** The run registry with a detail timeline. */
  import { api } from "../lib/tauri";
  import { createResource } from "../stores/data";
  import { formatDateTime, formatDuration, formatRelative, statusLabel } from "../lib/format";
  import Panel from "../components/Panel.svelte";
  import StateBlock from "../components/StateBlock.svelte";
  import StatusPill from "../components/StatusPill.svelte";
  import Button from "../components/Button.svelte";
  import type { Run, RunEvent, RunStatus } from "../domain/types";

  interface Props {
    initialSearch: string;
  }

  const { initialSearch }: Props = $props();

  let statusFilter = $state<RunStatus | "all">("all");
  let search = $state("");
  let selectedId = $state<string | null>(null);
  let initialised = false;

  $effect(() => {
    // The prop is an initial value, not a live one.
    if (!initialised) {
      search = initialSearch;
      initialised = true;
    }
  });

  const runs = createResource<Run[]>(
    () => api.listRuns(statusFilter, search),
    (rows) => rows.length === 0,
  );
  const events = createResource<RunEvent[]>(() => api.runEvents(""), (rows) => rows.length === 0);

  $effect(() => {
    void statusFilter;
    void search;
    void runs.load();
  });

  const selected = $derived(($runs.data ?? []).find((run) => run.id === selectedId) ?? null);

  async function select(run: Run) {
    selectedId = run.id;
    await events.load();
  }
</script>

<div class="runs">
  <Panel title="Run registry" subtitle="Every run in the local database" flush>
    {#snippet actions()}
      <label class="sr-only" for="run-search">Search runs</label>
      <input
        id="run-search"
        class="filter"
        type="search"
        placeholder="Search task or agent"
        bind:value={search}
      />
      <label class="sr-only" for="run-status">Filter by status</label>
      <select id="run-status" class="filter" bind:value={statusFilter}>
        <option value="all">All statuses</option>
        <option value="running">Running</option>
        <option value="waiting">Waiting</option>
        <option value="succeeded">Succeeded</option>
        <option value="failed">Failed</option>
        <option value="queued">Queued</option>
      </select>
    {/snippet}

    <StateBlock
      status={$runs.status}
      error={$runs.error}
      emptyTitle="No runs match"
      emptyBody="Adjust the filter, or start the simulator."
    />

    {#if $runs.status === "loaded" && ($runs.data ?? []).length > 0}
      <table>
        <thead>
          <tr>
            <th scope="col">Run</th>
            <th scope="col">Agent</th>
            <th scope="col">Task</th>
            <th scope="col">Status</th>
            <th scope="col">Started</th>
            <th scope="col">Duration</th>
          </tr>
        </thead>
        <tbody>
          {#each $runs.data ?? [] as run (run.id)}
            <tr class:selected={run.id === selectedId}>
              <td>
                <button class="linkish" onclick={() => select(run)}>{run.id}</button>
              </td>
              <td>{run.agentName}</td>
              <td class="task">{run.task}</td>
              <td><StatusPill status={run.status} label={statusLabel(run.status)} /></td>
              <td title={formatDateTime(run.startedAt)}>{formatRelative(run.startedAt)}</td>
              <td>{formatDuration(run.durationSeconds)}</td>
            </tr>
          {/each}
        </tbody>
      </table>
    {/if}
  </Panel>

  {#if selected}
    <aside class="detail" aria-label="Run detail">
      <header>
        <h2>{selected.task}</h2>
        <Button label="Close" variant="ghost" onClick={() => (selectedId = null)} />
      </header>

      <dl>
        <dt>Run</dt>
        <dd class="mono">{selected.id}</dd>
        <dt>Agent</dt>
        <dd>{selected.agentName}</dd>
        <dt>Status</dt>
        <dd><StatusPill status={selected.status} label={statusLabel(selected.status)} /></dd>
        <dt>Started</dt>
        <dd>{formatDateTime(selected.startedAt)}</dd>
        <dt>Ended</dt>
        <dd>{selected.endedAt ? formatDateTime(selected.endedAt) : "still running"}</dd>
        <dt>Duration</dt>
        <dd>{formatDuration(selected.durationSeconds)}</dd>
        <dt>Artifacts</dt>
        <dd class="mono">{selected.artifacts.join(", ") || "none"}</dd>
      </dl>

      <h3>Stages</h3>
      <ol class="stages">
        {#each selected.stages as stage (stage.name)}
          <li>
            <StatusPill status={stage.status} label={statusLabel(stage.status)} />
            <span class="stage-name">{stage.name}</span>
            <span class="stage-detail">{stage.detail}</span>
          </li>
        {/each}
      </ol>

      <h3>Event timeline</h3>
      {#if $events.status === "loading"}
        <p class="muted">Loading events…</p>
      {:else if $events.status === "failed"}
        <p class="muted">Could not load the events: {$events.error}</p>
      {:else if ($events.data ?? []).length === 0}
        <p class="muted">No events recorded for this run.</p>
      {:else}
        <ul class="events">
          {#each $events.data ?? [] as event (event.id)}
            <li>
              <span class="event-kind">{event.kind}</span>
              <span class="event-at">{formatDateTime(event.at)}</span>
              <span class="event-message">{event.message}</span>
            </li>
          {/each}
        </ul>
      {/if}
    </aside>
  {/if}
</div>

<style>
  .runs {
    display: flex;
    gap: var(--space-4);
    padding: var(--space-4);
    height: 100%;
    overflow: hidden;
  }

  .runs > :global(.panel) {
    flex: 1;
    min-width: 0;
  }

  .filter {
    padding: 5px 8px;
    border-radius: var(--radius-md);
    border: 1px solid var(--border);
    background: var(--bg-base);
    font-size: 12px;
  }

  table {
    width: 100%;
    border-collapse: collapse;
    font-size: 12px;
  }

  thead th {
    text-align: left;
    font-size: 10px;
    text-transform: uppercase;
    letter-spacing: 0.07em;
    color: var(--text-muted);
    padding: var(--space-2) var(--space-4);
    border-bottom: 1px solid var(--border);
    position: sticky;
    top: 0;
    background: var(--bg-panel);
  }

  tbody td {
    padding: var(--space-2) var(--space-4);
    border-bottom: 1px solid var(--border);
    vertical-align: middle;
  }

  tbody tr:hover {
    background: var(--bg-hover);
  }

  tbody tr.selected {
    background: var(--accent-soft);
  }

  .task {
    max-width: 320px;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .linkish {
    border: 0;
    background: transparent;
    color: var(--accent);
    font-family: var(--font-mono);
    font-size: 11px;
    padding: 0;
  }

  .detail {
    width: 360px;
    flex: none;
    overflow-y: auto;
    background: var(--bg-panel);
    border: 1px solid var(--border);
    border-radius: var(--radius-lg);
    padding: var(--space-4);
  }

  .detail header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: var(--space-2);
  }

  .detail h2 {
    font-size: 14px;
  }

  .detail h3 {
    font-size: 12px;
    text-transform: uppercase;
    letter-spacing: 0.06em;
    color: var(--text-muted);
    margin: var(--space-4) 0 var(--space-2);
  }

  dl {
    display: grid;
    grid-template-columns: 84px 1fr;
    gap: 4px var(--space-2);
    margin: var(--space-3) 0 0;
    font-size: 12px;
  }

  dt {
    color: var(--text-muted);
  }

  dd {
    margin: 0;
  }

  .mono {
    font-family: var(--font-mono);
    font-size: 11px;
  }

  .stages {
    list-style: none;
    margin: 0;
    padding: 0;
    counter-reset: stage;
  }

  .stages li {
    display: flex;
    align-items: center;
    gap: var(--space-2);
    padding: var(--space-2) 0;
    border-bottom: 1px solid var(--border);
  }

  .stage-name {
    font-weight: 600;
    font-size: 12px;
  }

  .stage-detail {
    font-size: 11px;
    color: var(--text-muted);
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .events {
    list-style: none;
    margin: 0;
    padding: 0;
  }

  .events li {
    display: grid;
    grid-template-columns: 1fr auto;
    gap: 2px var(--space-2);
    padding: var(--space-2) 0;
    border-bottom: 1px solid var(--border);
    font-size: 12px;
  }

  .event-kind {
    font-family: var(--font-mono);
    font-size: 10px;
    color: var(--accent);
  }

  .event-at {
    font-size: 10px;
    color: var(--text-muted);
  }

  .event-message {
    grid-column: 1 / -1;
    color: var(--text-secondary);
  }

  .muted {
    font-size: 12px;
    color: var(--text-muted);
  }

  @media (max-width: 1024px) {
    .runs {
      flex-direction: column;
      overflow-y: auto;
    }

    .detail {
      width: 100%;
    }
  }
</style>
