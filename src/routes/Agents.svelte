<script lang="ts">
  /** The agent registry: searchable, filterable cards with a detail drawer. */
  import { api } from "../lib/tauri";
  import { createResource } from "../stores/data";
  import { formatRelative, initials } from "../lib/format";
  import Panel from "../components/Panel.svelte";
  import StateBlock from "../components/StateBlock.svelte";
  import StatusPill from "../components/StatusPill.svelte";
  import Button from "../components/Button.svelte";
  import type { Agent, AgentStatus, RunEvent } from "../domain/types";

  interface Props {
    initialSearch: string;
  }

  const { initialSearch }: Props = $props();

  const agents = createResource<Agent[]>(() => api.listAgents(), (rows) => rows.length === 0);
  const events = createResource<RunEvent[]>(() => api.agentEvents(""), (rows) => rows.length === 0);

  let term = $state("");
  let statusFilter = $state<AgentStatus | "all">("all");
  let selectedId = $state<string | null>(null);
  let initialised = false;

  $effect(() => {
    // The prop is an initial value, not a live one: it seeds the search
    // once, and later changes do not overwrite what the operator typed.
    if (!initialised) {
      term = initialSearch;
      initialised = true;
    }
  });

  $effect(() => {
    void agents.load();
  });

  const filtered = $derived.by(() => {
    const rows = $agents.data ?? [];
    const needle = term.trim().toLowerCase();
    return rows.filter((agent) => {
      const matchesStatus =
        statusFilter === "all" || agent.status === statusFilter;
      const matchesTerm =
        needle.length === 0 ||
        agent.displayName.toLowerCase().includes(needle) ||
        agent.adapter.toLowerCase().includes(needle) ||
        agent.hostClass.toLowerCase().includes(needle);
      return matchesStatus && matchesTerm;
    });
  });

  const selected = $derived(
    ($agents.data ?? []).find((agent) => agent.id === selectedId) ?? null,
  );

  async function select(agent: Agent) {
    selectedId = agent.id;
    await events.load();
  }
</script>

<div class="agents">
  <Panel title="Agent registry" subtitle="Every seeded agent" flush>
    {#snippet actions()}
      <label class="sr-only" for="agent-search">Search agents</label>
      <input
        id="agent-search"
        class="filter"
        type="search"
        placeholder="Search agents"
        bind:value={term}
      />
      <label class="sr-only" for="agent-status">Filter by status</label>
      <select id="agent-status" class="filter" bind:value={statusFilter}>
        <option value="all">All statuses</option>
        <option value="working">Working</option>
        <option value="idle">Idle</option>
        <option value="approval">Approval</option>
        <option value="failed">Failed</option>
        <option value="offline">Offline</option>
      </select>
    {/snippet}

    <StateBlock
      status={$agents.status}
      error={$agents.error}
      emptyTitle="No agents are registered"
      emptyBody="The local registry is empty."
    />

    {#if $agents.status === "loaded"}
      {#if filtered.length === 0}
        <div class="no-match">
          No agent matches that filter. Clear the search to see all ten.
        </div>
      {:else}
        <div class="grid">
          {#each filtered as agent (agent.id)}
            <button
              class="card"
              class:selected={agent.id === selectedId}
              onclick={() => select(agent)}
              aria-pressed={agent.id === selectedId}
            >
              <div class="card-head">
                <span class="avatar">{initials(agent.displayName)}</span>
                <div class="names">
                  <span class="name">{agent.displayName}</span>
                  <span class="meta">{agent.adapter} &middot; {agent.hostClass}</span>
                </div>
                <StatusPill status={agent.status} />
              </div>
              <div class="card-foot">
                <span class="trust">trust: {agent.trustLevel}</span>
                <span class="heartbeat">{formatRelative(agent.lastHeartbeat)}</span>
              </div>
            </button>
          {/each}
        </div>
      {/if}
    {/if}
  </Panel>

  {#if selected}
    <aside class="drawer" aria-label="Agent detail">
      <header>
        <h2>{selected.displayName}</h2>
        <Button label="Close" variant="ghost" onClick={() => (selectedId = null)} />
      </header>

      <dl>
        <dt>Identifier</dt>
        <dd class="mono">{selected.id}</dd>
        <dt>Adapter</dt>
        <dd>{selected.adapter}</dd>
        <dt>Host class</dt>
        <dd>{selected.hostClass}</dd>
        <dt>Trust level</dt>
        <dd>{selected.trustLevel}</dd>
        <dt>Status</dt>
        <dd><StatusPill status={selected.status} /></dd>
        <dt>Current run</dt>
        <dd class="mono">{selected.currentRunId ?? "none"}</dd>
        <dt>Last heartbeat</dt>
        <dd>{formatRelative(selected.lastHeartbeat)}</dd>
      </dl>

      <h3>Capabilities</h3>
      <ul class="caps">
        {#each selected.capabilities as capability (capability.name)}
          <li>
            <StatusPill
              status={capability.granted ? "approved" : "pending"}
              label={capability.granted ? "granted" : "not granted"}
            />
            <div>
              <span class="cap-name">{capability.name}</span>
              <span class="cap-desc">{capability.description}</span>
            </div>
          </li>
        {/each}
      </ul>

      <h3>Event history</h3>
      {#if $events.status === "loading"}
        <p class="muted">Loading events…</p>
      {:else if $events.status === "failed"}
        <p class="muted">Could not load the event history: {$events.error}</p>
      {:else if ($events.data ?? []).length === 0}
        <p class="muted">No events recorded for this agent yet.</p>
      {:else}
        <ul class="events">
          {#each $events.data ?? [] as event (event.id)}
            <li>
              <span class="event-kind">{event.kind}</span>
              <span class="event-message">{event.message}</span>
              <span class="event-at">{formatRelative(event.at)}</span>
            </li>
          {/each}
        </ul>
      {/if}
    </aside>
  {/if}
</div>

<style>
  .agents {
    display: flex;
    gap: var(--space-4);
    padding: var(--space-4);
    height: 100%;
    overflow: hidden;
  }

  .agents > :global(.panel) {
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

  .grid {
    display: grid;
    grid-template-columns: repeat(auto-fill, minmax(280px, 1fr));
    gap: var(--space-3);
    padding: var(--space-4);
  }

  .card {
    display: flex;
    flex-direction: column;
    gap: var(--space-2);
    text-align: left;
    padding: var(--space-3);
    border-radius: var(--radius-lg);
    border: 1px solid var(--border);
    background: var(--bg-raised);
    color: inherit;
  }

  .card:hover {
    border-color: var(--border-strong);
  }

  .card.selected {
    border-color: var(--accent);
    box-shadow: 0 0 0 1px var(--accent);
  }

  .card-head {
    display: flex;
    align-items: center;
    gap: var(--space-2);
  }

  .avatar {
    width: 30px;
    height: 30px;
    border-radius: var(--radius-md);
    background: var(--accent-soft);
    color: var(--accent);
    display: grid;
    place-items: center;
    font-size: 11px;
    font-weight: 700;
    flex: none;
  }

  .names {
    flex: 1;
    min-width: 0;
    display: flex;
    flex-direction: column;
  }

  .name {
    font-weight: 600;
    font-size: 13px;
  }

  .meta {
    font-size: 11px;
    color: var(--text-muted);
  }

  .card-foot {
    display: flex;
    justify-content: space-between;
    font-size: 11px;
    color: var(--text-muted);
  }

  .no-match {
    padding: var(--space-6);
    text-align: center;
    color: var(--text-muted);
    font-size: 13px;
  }

  .drawer {
    width: 340px;
    flex: none;
    overflow-y: auto;
    background: var(--bg-panel);
    border: 1px solid var(--border);
    border-radius: var(--radius-lg);
    padding: var(--space-4);
  }

  .drawer header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: var(--space-2);
  }

  .drawer h2 {
    font-size: 15px;
  }

  .drawer h3 {
    font-size: 12px;
    text-transform: uppercase;
    letter-spacing: 0.06em;
    color: var(--text-muted);
    margin: var(--space-4) 0 var(--space-2);
  }

  dl {
    display: grid;
    grid-template-columns: 92px 1fr;
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

  .caps {
    list-style: none;
    margin: 0;
    padding: 0;
  }

  .caps li {
    display: flex;
    align-items: flex-start;
    gap: var(--space-2);
    padding: var(--space-2) 0;
    border-bottom: 1px solid var(--border);
  }

  .caps li:last-child {
    border-bottom: 0;
  }

  .cap-name {
    display: block;
    font-size: 12px;
    font-weight: 600;
  }

  .cap-desc {
    display: block;
    font-size: 11px;
    color: var(--text-muted);
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

  .event-message {
    grid-column: 1 / -1;
    color: var(--text-secondary);
  }

  .event-at {
    font-size: 10px;
    color: var(--text-muted);
    grid-row: 1;
    grid-column: 2;
  }

  .muted {
    font-size: 12px;
    color: var(--text-muted);
  }

  @media (max-width: 900px) {
    .agents {
      flex-direction: column;
      overflow-y: auto;
    }

    .drawer {
      width: 100%;
    }
  }
</style>
