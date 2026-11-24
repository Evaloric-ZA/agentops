<script lang="ts">
  /** The approval queue: approve, reject or comment, locally. */
  import { api } from "../lib/tauri";
  import { createResource } from "../stores/data";
  import { formatDateTime, formatPercent, formatRelative } from "../lib/format";
  import Panel from "../components/Panel.svelte";
  import StateBlock from "../components/StateBlock.svelte";
  import StatusPill from "../components/StatusPill.svelte";
  import Button from "../components/Button.svelte";
  import type { Approval } from "../domain/types";

  interface Props {
    focusId: string;
  }

  const { focusId }: Props = $props();

  let filter = $state<"pending" | "all">("pending");
  let selectedId = $state<string | null>(null);
  let comment = $state("");
  let busy = $state(false);
  let message = $state<string | null>(null);
  let messageTone = $state<"ok" | "danger">("ok");

  const approvals = createResource<Approval[]>(
    () => api.listApprovals(filter),
    (rows) => rows.length === 0,
  );

  $effect(() => {
    void filter;
    void approvals.load();
  });

  $effect(() => {
    if (focusId) selectedId = focusId;
  });

  const selected = $derived(
    ($approvals.data ?? []).find((approval) => approval.id === selectedId) ?? null,
  );

  async function decide(outcome: "approved" | "rejected") {
    if (!selected) return;
    busy = true;
    message = null;
    try {
      await api.decideApproval(selected.id, outcome, comment.trim() || undefined);
      message = `The approval was ${outcome}. The decision is in the audit log.`;
      messageTone = "ok";
      comment = "";
      await approvals.load();
    } catch (error) {
      message = error instanceof Error ? error.message : String(error);
      messageTone = "danger";
    } finally {
      busy = false;
    }
  }
</script>

<div class="waiting">
  <Panel title="Waiting on You" subtitle="Approvals that need a human decision" flush>
    {#snippet actions()}
      <label class="sr-only" for="approval-filter">Filter approvals</label>
      <select id="approval-filter" class="filter" bind:value={filter}>
        <option value="pending">Pending</option>
        <option value="all">All</option>
      </select>
    {/snippet}

    <StateBlock
      status={$approvals.status}
      error={$approvals.error}
      emptyTitle="Nothing is waiting"
      emptyBody="No approval needs a decision right now."
    />

    {#if $approvals.status === "loaded" && ($approvals.data ?? []).length > 0}
      <ul class="queue">
        {#each $approvals.data ?? [] as approval (approval.id)}
          <li>
            <button
              class="item"
              class:selected={approval.id === selectedId}
              onclick={() => (selectedId = approval.id)}
              aria-pressed={approval.id === selectedId}
            >
              <div class="item-head">
                <StatusPill status={approval.severity} label={approval.severity} />
                <span class="item-agent">{approval.agentName}</span>
                <span class="item-age">{formatRelative(approval.createdAt)}</span>
              </div>
              <p class="item-reason">{approval.reason}</p>
              <div class="item-foot">
                <span class="mono">{approval.id}</span>
                <span>confidence {formatPercent(approval.confidence)}</span>
                <StatusPill status={approval.state} />
              </div>
            </button>
          </li>
        {/each}
      </ul>
    {/if}
  </Panel>

  {#if selected}
    <aside class="detail" aria-label="Approval detail">
      <header>
        <h2>Decision</h2>
        <Button label="Close" variant="ghost" onClick={() => (selectedId = null)} />
      </header>

      <p class="reason">{selected.reason}</p>

      <dl>
        <dt>Approval</dt>
        <dd class="mono">{selected.id}</dd>
        <dt>Run</dt>
        <dd class="mono">{selected.runId}</dd>
        <dt>Agent</dt>
        <dd>{selected.agentName}</dd>
        <dt>Severity</dt>
        <dd>{selected.severity}</dd>
        <dt>Confidence</dt>
        <dd>{formatPercent(selected.confidence)}</dd>
        <dt>Created</dt>
        <dd>{formatDateTime(selected.createdAt)}</dd>
        <dt>State</dt>
        <dd><StatusPill status={selected.state} /></dd>
      </dl>

      <h3>Evidence</h3>
      {#if selected.evidence.length === 0}
        <p class="muted">No evidence was attached to this request.</p>
      {:else}
        <ul class="evidence">
          {#each selected.evidence as item (item)}
            <li class="mono">{item}</li>
          {/each}
        </ul>
      {/if}

      {#if selected.state === "pending"}
        <h3>Your decision</h3>
        <label class="sr-only" for="approval-comment">Comment</label>
        <textarea
          id="approval-comment"
          rows="3"
          placeholder="An optional comment, recorded in the audit log"
          bind:value={comment}
        ></textarea>
        <div class="actions">
          <Button
            label="Approve"
            variant="primary"
            disabled={busy}
            onClick={() => decide("approved")}
          />
          <Button
            label="Reject"
            variant="danger"
            disabled={busy}
            onClick={() => decide("rejected")}
          />
        </div>
      {:else}
        <p class="decided">
          This approval was {selected.state}
          {selected.decidedAt ? ` on ${formatDateTime(selected.decidedAt)}` : ""}.
          {selected.comment ? ` “${selected.comment}”` : ""}
        </p>
      {/if}
    </aside>
  {/if}

  <!-- The confirmation is outside the drawer on purpose. A decided
       approval leaves the pending list, so the drawer closes and a
       message inside it would never be read. -->
  {#if message}
    <p class="message" class:danger={messageTone === "danger"} role="status">{message}</p>
  {/if}
</div>

<style>
  .waiting {
    display: flex;
    gap: var(--space-4);
    padding: var(--space-4);
    height: 100%;
    overflow: hidden;
  }

  .waiting > :global(.panel) {
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

  .queue {
    list-style: none;
    margin: 0;
    padding: 0;
  }

  .item {
    display: block;
    width: 100%;
    text-align: left;
    border: 0;
    border-bottom: 1px solid var(--border);
    background: transparent;
    padding: var(--space-3) var(--space-4);
    color: inherit;
  }

  .item:hover {
    background: var(--bg-hover);
  }

  .item.selected {
    background: var(--accent-soft);
  }

  .item-head {
    display: flex;
    align-items: center;
    gap: var(--space-2);
  }

  .item-agent {
    font-weight: 600;
    font-size: 13px;
    flex: 1;
  }

  .item-age {
    font-size: 11px;
    color: var(--text-muted);
  }

  .item-reason {
    margin: var(--space-2) 0;
    font-size: 12px;
    color: var(--text-secondary);
  }

  .item-foot {
    display: flex;
    align-items: center;
    gap: var(--space-3);
    font-size: 11px;
    color: var(--text-muted);
  }

  .mono {
    font-family: var(--font-mono);
    font-size: 11px;
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
  }

  .detail h2 {
    font-size: 15px;
  }

  .detail h3 {
    font-size: 12px;
    text-transform: uppercase;
    letter-spacing: 0.06em;
    color: var(--text-muted);
    margin: var(--space-4) 0 var(--space-2);
  }

  .reason {
    font-size: 13px;
    color: var(--text-secondary);
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

  .evidence {
    list-style: none;
    margin: 0;
    padding: 0;
  }

  .evidence li {
    padding: 3px 0;
    color: var(--text-secondary);
  }

  textarea {
    width: 100%;
    border-radius: var(--radius-md);
    border: 1px solid var(--border);
    background: var(--bg-base);
    padding: var(--space-2);
    font: inherit;
    font-size: 12px;
    resize: vertical;
  }

  .actions {
    display: flex;
    gap: var(--space-2);
    margin-top: var(--space-3);
  }

  .decided {
    font-size: 12px;
    color: var(--text-secondary);
  }

  .message {
    margin-top: var(--space-3);
    font-size: 12px;
    color: var(--ok);
  }

  .message.danger {
    color: var(--danger);
  }

  .muted {
    font-size: 12px;
    color: var(--text-muted);
  }

  @media (max-width: 1024px) {
    .waiting {
      flex-direction: column;
      overflow-y: auto;
    }

    .detail {
      width: 100%;
    }
  }
</style>
