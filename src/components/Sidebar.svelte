<script lang="ts">
  /** The left navigation. Collapsible, and grouped the way the spec groups it. */
  import { NAV_ITEMS, currentView, sidebarCollapsed, navigate, type ViewId } from "../stores/app";

  interface Props {
    agents: { id: string; displayName: string; status: string }[];
  }

  const { agents }: Props = $props();

  const groups = ["Operate", "Govern", "Workspace", "Crew"] as const;
  const collapsed = $derived($sidebarCollapsed);

  function itemsIn(group: string) {
    return NAV_ITEMS.filter((item) => item.group === group);
  }
</script>

<nav class="side" class:collapsed aria-label="Primary">
  <div class="brand">
    <span class="mark" aria-hidden="true"></span>
    {#if !collapsed}
      <span class="brand-name">AgentOps</span>
      <span class="brand-version">v0.1</span>
    {/if}
  </div>

  <div class="groups">
    {#each groups as group (group)}
      <div class="group">
        {#if !collapsed}
          <span class="group-label">{group}</span>
        {/if}
        <ul>
          {#each itemsIn(group) as item (item.id)}
            <li>
              <button
                class="nav-item"
                class:active={$currentView === item.id}
                title={collapsed ? item.label : item.hint}
                aria-current={$currentView === item.id ? "page" : undefined}
                onclick={() => navigate(item.id as ViewId)}
              >
                <span class="nav-glyph" aria-hidden="true">{item.label.charAt(0)}</span>
                {#if !collapsed}
                  <span class="nav-label">{item.label}</span>
                {/if}
              </button>
            </li>
          {/each}
        </ul>
      </div>
    {/each}
  </div>

  <div class="crew" class:collapsed>
    {#if !collapsed}
      <span class="group-label">Crew</span>
    {/if}
    <ul class="crew-list">
      {#each agents.slice(0, collapsed ? 6 : agents.length) as agent (agent.id)}
        <li>
          <span class="crew-dot crew-{agent.status}" title="{agent.displayName}: {agent.status}"></span>
          {#if !collapsed}
            <span class="crew-name">{agent.displayName}</span>
            <span class="crew-status">{agent.status}</span>
          {/if}
        </li>
      {/each}
    </ul>
  </div>

  <button
    class="collapse"
    onclick={() => ($sidebarCollapsed = !$sidebarCollapsed)}
    aria-expanded={!collapsed}
    title={collapsed ? "Expand the navigation" : "Collapse the navigation"}
  >
    {collapsed ? "»" : "« Collapse"}
  </button>
</nav>

<style>
  .side {
    display: flex;
    flex-direction: column;
    width: 236px;
    flex: none;
    background: var(--bg-sunken);
    border-right: 1px solid var(--border);
    overflow: hidden;
    transition: width var(--transition);
  }

  .side.collapsed {
    width: 58px;
  }

  .brand {
    display: flex;
    align-items: center;
    gap: var(--space-2);
    padding: var(--space-4);
    border-bottom: 1px solid var(--border);
    flex: none;
  }

  .mark {
    width: 22px;
    height: 22px;
    border-radius: 7px;
    background: var(--accent-strong);
    flex: none;
    position: relative;
  }

  .mark::after {
    content: "";
    position: absolute;
    inset: 5px;
    border-radius: 50%;
    border: 2px solid #ffffff;
  }

  .brand-name {
    font-weight: 700;
    letter-spacing: -0.02em;
  }

  .brand-version {
    font-size: 10px;
    color: var(--text-muted);
    border: 1px solid var(--border);
    border-radius: 4px;
    padding: 0 4px;
  }

  .groups {
    flex: 1;
    overflow-y: auto;
    padding: var(--space-3) var(--space-2);
  }

  .group + .group {
    margin-top: var(--space-4);
  }

  .group-label {
    display: block;
    font-size: 10px;
    font-weight: 700;
    text-transform: uppercase;
    letter-spacing: 0.09em;
    color: var(--text-muted);
    padding: 0 var(--space-2) var(--space-1);
  }

  ul {
    list-style: none;
    margin: 0;
    padding: 0;
  }

  .nav-item {
    display: flex;
    align-items: center;
    gap: var(--space-2);
    width: 100%;
    padding: 6px var(--space-2);
    border: 0;
    border-radius: var(--radius-md);
    background: transparent;
    color: var(--text-secondary);
    font-size: 13px;
    text-align: left;
  }

  .nav-item:hover {
    background: var(--bg-hover);
    color: var(--text-primary);
  }

  .nav-item.active {
    background: var(--accent-soft);
    color: var(--accent);
    font-weight: 600;
  }

  .nav-glyph {
    width: 20px;
    height: 20px;
    border-radius: var(--radius-sm);
    background: var(--bg-hover);
    display: grid;
    place-items: center;
    font-size: 10px;
    font-weight: 700;
    flex: none;
  }

  .nav-item.active .nav-glyph {
    background: var(--accent-strong);
    color: #ffffff;
  }

  .nav-label {
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .crew {
    border-top: 1px solid var(--border);
    padding: var(--space-3) var(--space-2);
    flex: none;
    max-height: 32%;
    overflow-y: auto;
  }

  .crew-list li {
    display: flex;
    align-items: center;
    gap: var(--space-2);
    padding: 3px var(--space-2);
    font-size: 12px;
  }

  .crew-dot {
    width: 7px;
    height: 7px;
    border-radius: 50%;
    flex: none;
    background: var(--text-muted);
  }

  .crew-working {
    background: var(--ok);
  }
  .crew-approval {
    background: var(--warn);
  }
  .crew-failed {
    background: var(--danger);
  }
  .crew-offline {
    background: var(--text-muted);
  }

  .crew-name {
    flex: 1;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
    color: var(--text-secondary);
  }

  .crew-status {
    font-size: 10px;
    color: var(--text-muted);
  }

  .collapse {
    border: 0;
    border-top: 1px solid var(--border);
    background: transparent;
    color: var(--text-muted);
    padding: var(--space-2);
    font-size: 12px;
    flex: none;
  }

  .collapse:hover {
    color: var(--text-primary);
    background: var(--bg-hover);
  }
</style>
