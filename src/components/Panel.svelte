<script lang="ts">
  /** A titled panel. The unit of layout across every view. */
  interface Props {
    title: string;
    subtitle?: string;
    flush?: boolean;
    children?: import("svelte").Snippet;
    actions?: import("svelte").Snippet;
  }

  const { title, subtitle = "", flush = false, children, actions }: Props = $props();
</script>

<section class="panel">
  <header class="head">
    <div class="titles">
      <h3>{title}</h3>
      {#if subtitle}
        <span class="subtitle">{subtitle}</span>
      {/if}
    </div>
    {#if actions}
      <div class="actions">{@render actions()}</div>
    {/if}
  </header>
  <div class="body" class:flush>
    {@render children?.()}
  </div>
</section>

<style>
  .panel {
    display: flex;
    flex-direction: column;
    min-width: 0;
    min-height: 0;
    background: var(--bg-panel);
    border: 1px solid var(--border);
    border-radius: var(--radius-lg);
    overflow: hidden;
  }

  .head {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: var(--space-3);
    padding: var(--space-3) var(--space-4);
    border-bottom: 1px solid var(--border);
    flex: none;
  }

  .titles {
    display: flex;
    flex-direction: column;
    min-width: 0;
  }

  h3 {
    font-size: 13px;
    font-weight: 650;
    letter-spacing: 0.01em;
  }

  .subtitle {
    font-size: 11px;
    color: var(--text-muted);
  }

  .actions {
    display: flex;
    align-items: center;
    gap: var(--space-2);
    flex: none;
  }

  .body {
    padding: var(--space-4);
    overflow: auto;
    min-height: 0;
    flex: 1;
  }

  .body.flush {
    padding: 0;
  }
</style>
