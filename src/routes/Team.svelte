<script lang="ts">
  /** Who is on shift, and which agents they cover. */
  import { api } from "../lib/tauri";
  import { createResource } from "../stores/data";
  import Panel from "../components/Panel.svelte";
  import StateBlock from "../components/StateBlock.svelte";
  import StatusPill from "../components/StatusPill.svelte";
  import type { TeamMember } from "../domain/types";

  const team = createResource<TeamMember[]>(
    () => api.listTeam(),
    (rows) => rows.length === 0,
  );

  $effect(() => {
    void team.load();
  });
</script>

<div class="page">
  <Panel title="Team" subtitle="The seeded operators and reviewers" flush>
    <StateBlock
      status={$team.status}
      error={$team.error}
      emptyTitle="No team members"
      emptyBody="Nobody is on shift."
    />
    {#if $team.status === "loaded" && ($team.data ?? []).length > 0}
      <ul class="members">
        {#each $team.data ?? [] as member (member.id)}
          <li>
            <span class="avatar" aria-hidden="true">{member.name.charAt(0)}</span>
            <div class="who">
              <span class="name">{member.name}</span>
              <span class="role">{member.role}</span>
            </div>
            <div class="agents">
              {#each member.agentIds as agentId (agentId)}
                <span class="chip">{agentId}</span>
              {/each}
            </div>
            <StatusPill
              status={member.onShift ? "approved" : "idle"}
              label={member.onShift ? "on shift" : "off shift"}
            />
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

  .members {
    list-style: none;
    margin: 0;
    padding: 0;
  }

  .members li {
    display: flex;
    align-items: center;
    gap: var(--space-3);
    padding: var(--space-3) var(--space-4);
    border-bottom: 1px solid var(--border);
  }

  .members li:last-child {
    border-bottom: 0;
  }

  .avatar {
    width: 28px;
    height: 28px;
    border-radius: 50%;
    background: var(--accent-soft);
    color: var(--accent);
    display: grid;
    place-items: center;
    font-weight: 700;
    font-size: 12px;
    flex: none;
  }

  .who {
    display: flex;
    flex-direction: column;
    min-width: 150px;
  }

  .name {
    font-weight: 600;
    font-size: 13px;
  }

  .role {
    font-size: 11px;
    color: var(--text-muted);
  }

  .agents {
    flex: 1;
    display: flex;
    flex-wrap: wrap;
    gap: 4px;
  }

  .chip {
    font-family: var(--font-mono);
    font-size: 10px;
    padding: 2px 6px;
    border-radius: var(--radius-sm);
    background: var(--bg-hover);
    color: var(--text-secondary);
  }
</style>
