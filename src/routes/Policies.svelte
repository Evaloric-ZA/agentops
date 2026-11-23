<script lang="ts">
  /** The local policy set. */
  import { api } from "../lib/tauri";
  import { createResource } from "../stores/data";
  import Panel from "../components/Panel.svelte";
  import StateBlock from "../components/StateBlock.svelte";
  import StatusPill from "../components/StatusPill.svelte";
  import type { Policy } from "../domain/types";

  const policies = createResource<Policy[]>(
    () => api.listPolicies(),
    (rows) => rows.length === 0,
  );

  $effect(() => {
    void policies.load();
  });

  async function toggle(policy: Policy) {
    await api.setPolicyEnabled(policy.id, !policy.enabled);
    await policies.load();
  }
</script>

<div class="page">
  <Panel title="Policies" subtitle="The local rules the simulator obeys" flush>
    <StateBlock
      status={$policies.status}
      error={$policies.error}
      emptyTitle="No policies are configured"
      emptyBody="The policy set is empty."
    />
    {#if $policies.status === "loaded" && ($policies.data ?? []).length > 0}
      <ul class="policies">
        {#each $policies.data ?? [] as policy (policy.id)}
          <li>
            <div class="policy-main">
              <span class="policy-name">{policy.name}</span>
              <span class="policy-detail">{policy.detail}</span>
            </div>
            <div class="policy-meta">
              <span class="scope">{policy.scope}</span>
              <StatusPill
                status={policy.effect === "deny" ? "failed" : policy.effect === "require_approval" ? "pending" : "approved"}
                label={policy.effect.replace("_", " ")}
              />
            </div>
            <label class="switch">
              <input
                type="checkbox"
                checked={policy.enabled}
                onchange={() => toggle(policy)}
              />
              <span class="sr-only">{policy.enabled ? "Disable" : "Enable"} {policy.name}</span>
              <span class="track" aria-hidden="true"></span>
            </label>
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
  }

  .policies {
    list-style: none;
    margin: 0;
    padding: 0;
  }

  .policies li {
    display: flex;
    align-items: center;
    gap: var(--space-3);
    padding: var(--space-3) var(--space-4);
    border-bottom: 1px solid var(--border);
  }

  .policy-main {
    flex: 1;
    min-width: 0;
    display: flex;
    flex-direction: column;
  }

  .policy-name {
    font-weight: 600;
    font-size: 13px;
  }

  .policy-detail {
    font-size: 11px;
    color: var(--text-muted);
  }

  .policy-meta {
    display: flex;
    align-items: center;
    gap: var(--space-2);
    flex: none;
  }

  .scope {
    font-size: 10px;
    text-transform: uppercase;
    letter-spacing: 0.07em;
    color: var(--text-muted);
  }

  .switch {
    position: relative;
    display: inline-flex;
    flex: none;
    cursor: pointer;
  }

  .switch input {
    position: absolute;
    opacity: 0;
    width: 100%;
    height: 100%;
    margin: 0;
    cursor: pointer;
  }

  .track {
    width: 34px;
    height: 19px;
    border-radius: 999px;
    background: var(--bg-hover);
    border: 1px solid var(--border-strong);
    position: relative;
    transition: background var(--transition);
  }

  .track::after {
    content: "";
    position: absolute;
    top: 2px;
    left: 2px;
    width: 13px;
    height: 13px;
    border-radius: 50%;
    background: var(--text-muted);
    transition: transform var(--transition), background var(--transition);
  }

  /* The track is a general sibling of the input, not the adjacent one:
     the screen-reader label sits between them. */
  .switch input:checked ~ .track {
    background: var(--accent-soft);
    border-color: var(--accent);
  }

  .switch input:checked ~ .track::after {
    transform: translateX(15px);
    background: var(--accent);
  }

  .switch input:focus-visible ~ .track {
    outline: 2px solid var(--accent);
    outline-offset: 2px;
  }
</style>
