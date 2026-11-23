<script lang="ts">
  /** The top bar: breadcrumb, search, notifications and the local user menu. */
  import { currentViewLabel, navigate, type ViewId } from "../stores/app";

  interface Props {
    onSearch: (term: string) => void;
    notificationsEnabled: boolean;
    pendingApprovals: number;
    onOpenApprovals: () => void;
  }

  const {
    onSearch,
    notificationsEnabled,
    pendingApprovals,
    onOpenApprovals,
  }: Props = $props();

  let term = $state("");
  let menuOpen = $state(false);

  const today = new Date();
  const dateLabel = today.toLocaleDateString(undefined, {
    weekday: "long",
    year: "numeric",
    month: "long",
    day: "numeric",
  });

  function submit(event: SubmitEvent) {
    event.preventDefault();
    onSearch(term);
  }

  function openApprovals() {
    onOpenApprovals();
    navigate("waiting" as ViewId);
  }
</script>

<header class="top">
  <nav class="crumbs" aria-label="Breadcrumb">
    <button class="crumb" onclick={() => navigate("night-shift" as ViewId)}>AgentOps</button>
    <span class="sep" aria-hidden="true">/</span>
    <span class="crumb current" aria-current="page">{$currentViewLabel}</span>
  </nav>

  <form class="search" onsubmit={submit} role="search">
    <label class="sr-only" for="global-search">Search the local records</label>
    <input
      id="global-search"
      type="search"
      placeholder="Search agents, runs and tasks"
      bind:value={term}
    />
  </form>

  <div class="right">
    <span class="date" title="The local date">{dateLabel}</span>

    <button
      class="icon-button"
      onclick={openApprovals}
      title={pendingApprovals > 0
        ? `${pendingApprovals} approval(s) waiting`
        : "No approvals are waiting"}
    >
      <span aria-hidden="true">◆</span>
      <span class="sr-only">Approvals waiting on you</span>
      {#if notificationsEnabled && pendingApprovals > 0}
        <span class="badge">{pendingApprovals}</span>
      {/if}
    </button>

    <div class="user">
      <button
        class="user-button"
        onclick={() => (menuOpen = !menuOpen)}
        aria-expanded={menuOpen}
        aria-haspopup="menu"
      >
        <span class="avatar" aria-hidden="true">OP</span>
        <span class="user-name">Local Operator</span>
      </button>
      {#if menuOpen}
        <div class="menu" role="menu">
          <p class="menu-note">The operator is a local identity. There is no account service.</p>
          <button role="menuitem" onclick={() => (menuOpen = false)}>Close menu</button>
        </div>
      {/if}
    </div>
  </div>
</header>

<style>
  .top {
    display: flex;
    align-items: center;
    gap: var(--space-4);
    padding: 0 var(--space-4);
    height: 52px;
    flex: none;
    background: var(--bg-panel);
    border-bottom: 1px solid var(--border);
  }

  .crumbs {
    display: flex;
    align-items: center;
    gap: var(--space-2);
    flex: none;
  }

  .crumb {
    border: 0;
    background: transparent;
    color: var(--text-muted);
    font-size: 13px;
    padding: 0;
  }

  .crumb.current {
    color: var(--text-primary);
    font-weight: 600;
  }

  .sep {
    color: var(--text-muted);
  }

  .search {
    flex: 1;
    max-width: 420px;
  }

  .search input {
    width: 100%;
    padding: 6px 10px;
    border-radius: var(--radius-md);
    border: 1px solid var(--border);
    background: var(--bg-base);
    font-size: 13px;
  }

  .right {
    display: flex;
    align-items: center;
    gap: var(--space-3);
    margin-left: auto;
    flex: none;
  }

  .date {
    font-size: 12px;
    color: var(--text-muted);
    white-space: nowrap;
  }

  .icon-button {
    position: relative;
    border: 1px solid var(--border);
    background: var(--bg-raised);
    border-radius: var(--radius-md);
    width: 30px;
    height: 30px;
    display: grid;
    place-items: center;
    color: var(--text-secondary);
  }

  .badge {
    position: absolute;
    top: -6px;
    right: -6px;
    min-width: 16px;
    height: 16px;
    padding: 0 4px;
    border-radius: 8px;
    background: var(--danger);
    color: #fff;
    font-size: 10px;
    font-weight: 700;
    display: grid;
    place-items: center;
  }

  .user {
    position: relative;
  }

  .user-button {
    display: flex;
    align-items: center;
    gap: var(--space-2);
    border: 0;
    background: transparent;
    padding: 0;
  }

  .avatar {
    width: 26px;
    height: 26px;
    border-radius: 50%;
    background: var(--accent-soft);
    color: var(--accent);
    display: grid;
    place-items: center;
    font-size: 10px;
    font-weight: 700;
  }

  .user-name {
    font-size: 12px;
    color: var(--text-secondary);
    white-space: nowrap;
  }

  .menu {
    position: absolute;
    right: 0;
    top: calc(100% + 8px);
    width: 250px;
    padding: var(--space-3);
    background: var(--bg-raised);
    border: 1px solid var(--border-strong);
    border-radius: var(--radius-md);
    box-shadow: var(--shadow);
    z-index: 20;
  }

  .menu-note {
    margin: 0 0 var(--space-2);
    font-size: 12px;
    color: var(--text-secondary);
  }

  .menu button {
    width: 100%;
    padding: 5px 8px;
    border: 1px solid var(--border);
    border-radius: var(--radius-sm);
    background: var(--bg-panel);
    font-size: 12px;
  }

  @media (max-width: 900px) {
    .date,
    .user-name {
      display: none;
    }
  }
</style>
