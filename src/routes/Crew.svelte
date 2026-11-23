<script lang="ts">
  /** The crew: compact status rows for every agent. */
  import { api } from "../lib/tauri";
  import { createResource } from "../stores/data";
  import { formatRelative, initials } from "../lib/format";
  import Panel from "../components/Panel.svelte";
  import StateBlock from "../components/StateBlock.svelte";
  import StatusPill from "../components/StatusPill.svelte";
  import type { Agent } from "../domain/types";

  const agents = createResource<Agent[]>(() => api.listAgents(), (rows) => rows.length === 0);

  $effect(() => {
    void agents.load();
  });

  const counts = $derived.by(() => {
    const tally = new Map<string, number>();
    for (const agent of $agents.data ?? []) {
      tally.set(agent.status, (tally.get(agent.status) ?? 0) + 1);
    }
    return [...tally.entries()].sort((a, b) => b[1] - a[1]);
  });
</script>

<div class="page">
  <div class="tally">
    {#each counts as [status, count] (status)}
      <span class="tally-item">
        <StatusPill status={status} />
        <strong>{count}</strong>
      </span>
    {/each}
  </div>

  <Panel title="Crew" subtitle="Compact status for every agent" flush>
    <StateBlock
      status={$agents.status}
      error={$agents.error}
      emptyTitle="No agents are registered"
      emptyBody="The local registry is empty."
    />
    {#if $agents.status === "loaded" && ($agents.data ?? []).length > 0}
      <ul class="crew">
        {#each $agents.data ?? [] as agent (agent.id)}
          <li>
            <span class="avatar">{initials(agent.displayName)}</span>
            <span class="name">{agent.displayName}</span>
            <span class="cap">{agent.capabilities.filter((c) => c.granted).length} granted</span>
            <span class="heartbeat">{formatRelative(agent.lastHeartbeat)}</span>
            <StatusPill status={agent.status} />
          </li>
        {/each}
      </ul>
    {/if}
  </Panel>
</div>

<style>
  .page {
    padding: var(--space-4);
    height: 100%;
    overflow-y: auto;
    display: flex;
    flex-direction: column;
    gap: var(--space-4);
  }

  .tally {
    display: flex;
    gap: var(--space-4);
    flex-wrap: wrap;
  }

  .tally-item {
    display: inline-flex;
    align-items: center;
    gap: var(--space-2);
    font-size: 13px;
  }

  .crew {
    list-style: none;
    margin: 0;
    padding: 0;
  }

  .crew li {
    display: grid;
    grid-template-columns: 28px minmax(120px, 1fr) auto auto auto;
    align-items: center;
    gap: var(--space-3);
    padding: var(--space-2) var(--space-4);
    border-bottom: 1px solid var(--border);
    font-size: 12px;
  }

  .crew li:last-child {
    border-bottom: 0;
  }

  .avatar {
    width: 24px;
    height: 24px;
    border-radius: var(--radius-sm);
    background: var(--accent-soft);
    color: var(--accent);
    display: grid;
    place-items: center;
    font-size: 10px;
    font-weight: 700;
  }

  .name {
    font-weight: 600;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .cap,
  .heartbeat {
    font-size: 11px;
    color: var(--text-muted);
    white-space: nowrap;
  }
</style>
