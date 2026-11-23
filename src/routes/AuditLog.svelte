<script lang="ts">
  /** The audit log: everything that has been recorded, newest first. */
  import { api } from "../lib/tauri";
  import { createResource } from "../stores/data";
  import { formatDateTime } from "../lib/format";
  import Panel from "../components/Panel.svelte";
  import StateBlock from "../components/StateBlock.svelte";
  import StatusPill from "../components/StatusPill.svelte";
  import type { AuditEvent } from "../domain/types";

  const audit = createResource<AuditEvent[]>(
    () => api.listAudit(200),
    (rows) => rows.length === 0,
  );

  $effect(() => {
    void audit.load();
  });
</script>

<div class="page">
  <Panel title="Audit log" subtitle="Every recorded action" flush>
    <StateBlock
      status={$audit.status}
      error={$audit.error}
      emptyTitle="Nothing has been recorded"
      emptyBody="The audit log is empty."
    />
    {#if $audit.status === "loaded" && ($audit.data ?? []).length > 0}
      <table>
        <thead>
          <tr>
            <th scope="col">When</th>
            <th scope="col">Actor</th>
            <th scope="col">Action</th>
            <th scope="col">Subject</th>
            <th scope="col">Outcome</th>
            <th scope="col">Detail</th>
          </tr>
        </thead>
        <tbody>
          {#each $audit.data ?? [] as event (event.id)}
            <tr>
              <td class="when">{formatDateTime(event.at)}</td>
              <td>{event.actor}</td>
              <td class="mono">{event.action}</td>
              <td class="mono">{event.subject}</td>
              <td><StatusPill status={event.outcome === "recorded" ? "approved" : "pending"} label={event.outcome} /></td>
              <td class="detail">{event.detail}</td>
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
    position: sticky;
    top: 0;
    background: var(--bg-panel);
  }

  tbody td {
    padding: var(--space-2) var(--space-4);
    border-bottom: 1px solid var(--border);
    vertical-align: top;
  }

  .when {
    white-space: nowrap;
    color: var(--text-secondary);
  }

  .mono {
    font-family: var(--font-mono);
    font-size: 11px;
  }

  .detail {
    color: var(--text-muted);
    max-width: 320px;
  }
</style>
