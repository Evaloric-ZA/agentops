<script lang="ts">
  /**
   * The loading, empty and failed states.
   *
   * Every data-backed view renders one of these, so a page never shows a
   * blank area with no explanation.
   */
  interface Props {
    status: "idle" | "loading" | "loaded" | "empty" | "failed";
    error?: string | null;
    emptyTitle?: string;
    emptyBody?: string;
  }

  const {
    status,
    error = null,
    emptyTitle = "Nothing here yet",
    emptyBody = "When the data arrives it will appear here.",
  }: Props = $props();
</script>

{#if status === "loading"}
  <div class="state" role="status" aria-live="polite">
    <span class="spinner" aria-hidden="true"></span>
    <span class="title">Loading</span>
    <span class="body">Fetching the local records.</span>
  </div>
{:else if status === "failed"}
  <div class="state state-error" role="alert">
    <span class="title">Could not load this view</span>
    <span class="body">{error ?? "The command failed without a message."}</span>
    <span class="body muted">Nothing was changed. The local database is untouched.</span>
  </div>
{:else if status === "empty"}
  <div class="state" role="status">
    <span class="title">{emptyTitle}</span>
    <span class="body">{emptyBody}</span>
  </div>
{:else if status === "idle"}
  <div class="state" role="status">
    <span class="body">Not loaded yet.</span>
  </div>
{/if}

<style>
  .state {
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    gap: var(--space-2);
    padding: var(--space-6) var(--space-4);
    text-align: center;
    color: var(--text-secondary);
    min-height: 120px;
  }

  .state-error {
    color: var(--danger);
  }

  .title {
    font-weight: 600;
    color: var(--text-primary);
  }

  .state-error .title {
    color: var(--danger);
  }

  .body {
    font-size: 13px;
    max-width: 46ch;
  }

  .muted {
    color: var(--text-muted);
    font-size: 12px;
  }

  .spinner {
    width: 20px;
    height: 20px;
    border-radius: 50%;
    border: 2px solid var(--border-strong);
    border-top-color: var(--accent);
    animation: spin 700ms linear infinite;
  }

  @keyframes spin {
    to {
      transform: rotate(360deg);
    }
  }
</style>
