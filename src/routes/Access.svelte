<script lang="ts">
  /** Access: trust levels and granted capabilities, across the whole crew. */
  import { api } from "../lib/tauri";
  import { createResource } from "../stores/data";
  import Panel from "../components/Panel.svelte";
  import StateBlock from "../components/StateBlock.svelte";
  import StatusPill from "../components/StatusPill.svelte";
  import type { Agent } from "../domain/types";

  const agents = createResource<Agent[]>(() => api.listAgents(), (rows) => rows.length === 0);

  $effect(() => {
    void agents.load();
  });

  const trustTone = (level: string) =>
    level === "trusted" ? "approved" : level === "limited" ? "pending" : "failed";

  const granted = $derived(
    ($agents.data ?? []).flatMap((agent) =>
      agent.capabilities
        .filter((capability) => capability.granted)
        .map((capability) => ({ agent, capability })),
    ),
  );

  const withheld = $derived(
    ($agents.data ?? []).flatMap((agent) =>
      agent.capabilities
        .filter((capability) => !capability.granted)
        .map((capability) => ({ agent, capability })),
    ),
  );
</script>

<div class="page">
  <div class="grid">
    <Panel title="Trust levels" subtitle="How much each agent is trusted" flush>
      <StateBlock
        status={$agents.status}
        error={$agents.error}
        emptyTitle="No agents are registered"
        emptyBody="The local registry is empty."
      />
      {#if $agents.status === "loaded" && ($agents.data ?? []).length > 0}
        <ul class="rows">
          {#each $agents.data ?? [] as agent (agent.id)}
            <li>
              <span class="name">{agent.displayName}</span>
              <span class="host">{agent.hostClass}</span>
              <StatusPill status={trustTone(agent.trustLevel)} label={agent.trustLevel} />
            </li>
          {/each}
        </ul>
      {/if}
    </Panel>

    <Panel title="Granted capabilities" subtitle="What each agent may do" flush>
      <StateBlock
        status={$agents.status}
        error={$agents.error}
        emptyTitle="Nothing is granted"
        emptyBody="No agent has a granted capability."
      />
      {#if $agents.status === "loaded" && granted.length > 0}
        <ul class="rows">
          {#each granted as row (row.agent.id + row.capability.name)}
            <li>
              <span class="name">{row.agent.displayName}</span>
              <span class="cap">{row.capability.name}</span>
              <StatusPill status="approved" label="granted" />
            </li>
          {/each}
        </ul>
      {/if}
    </Panel>

    <Panel title="Withheld capabilities" subtitle="Requested but not granted" flush>
      <StateBlock
        status={$agents.status}
        error={$agents.error}
        emptyTitle="Nothing is withheld"
        emptyBody="Every requested capability is granted."
      />
      {#if $agents.status === "loaded" && withheld.length > 0}
        <ul class="rows">
          {#each withheld as row (row.agent.id + row.capability.name)}
            <li>
              <span class="name">{row.agent.displayName}</span>
              <span class="cap">{row.capability.name}</span>
              <StatusPill status="pending" label="withheld" />
            </li>
          {/each}
        </ul>
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
    grid-template-columns: repeat(auto-fit, minmax(300px, 1fr));
    gap: var(--space-4);
    align-items: start;
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
    padding: var(--space-2) var(--space-4);
    border-bottom: 1px solid var(--border);
    font-size: 12px;
  }

  .rows li:last-child {
    border-bottom: 0;
  }

  .name {
    flex: 1;
    font-weight: 600;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .host,
  .cap {
    color: var(--text-muted);
    font-size: 11px;
    font-family: var(--font-mono);
  }
</style>
