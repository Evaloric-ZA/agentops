<script lang="ts">
  /** The tool registry and its health. */
  import { api } from "../lib/tauri";
  import { createResource } from "../stores/data";
  import { formatNumber, formatPercent } from "../lib/format";
  import Panel from "../components/Panel.svelte";
  import StateBlock from "../components/StateBlock.svelte";
  import StatusPill from "../components/StatusPill.svelte";
  import Button from "../components/Button.svelte";
  import type { Tool } from "../domain/types";

  const tools = createResource<Tool[]>(() => api.listTools(), (rows) => rows.length === 0);

  $effect(() => {
    void tools.load();
  });
</script>

<div class="page">
  <Panel title="Tools" subtitle="The registered tools and their health" flush>
    {#snippet actions()}
      <Button
        label="Register a tool"
        disabled
        disabledReason="Tool registration is a v0.2 interface. v0.1 defines the registry but does not let the frontend add to it."
        onClick={() => undefined}
      />
    {/snippet}
    <StateBlock
      status={$tools.status}
      error={$tools.error}
      emptyTitle="No tools are registered"
      emptyBody="The tool registry is empty."
    />
    {#if $tools.status === "loaded" && ($tools.data ?? []).length > 0}
      <table>
        <thead>
          <tr>
            <th scope="col">Tool</th>
            <th scope="col">Kind</th>
            <th scope="col">Status</th>
            <th scope="col">Success</th>
            <th scope="col">Latency</th>
            <th scope="col">Calls / hour</th>
          </tr>
        </thead>
        <tbody>
          {#each $tools.data ?? [] as tool (tool.id)}
            <tr>
              <td class="name">{tool.name}</td>
              <td class="mono">{tool.kind}</td>
              <td><StatusPill status={tool.status} /></td>
              <td>{formatPercent(tool.successRate)}</td>
              <td>{tool.latencyMs === null ? "—" : `${tool.latencyMs}ms`}</td>
              <td>{formatNumber(tool.callsLastHour)}</td>
            </tr>
          {/each}
        </tbody>
      </table>
    {/if}
  </Panel>
</div>

<style>
  .page {
    padding: var(--space-4);
    height: 100%;
    overflow: hidden;
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
  }

  tbody td {
    padding: var(--space-2) var(--space-4);
    border-bottom: 1px solid var(--border);
  }

  .name {
    font-weight: 600;
  }

  .mono {
    font-family: var(--font-mono);
    font-size: 11px;
    color: var(--text-secondary);
  }
</style>
