<script lang="ts">
  /** One of the four summary cards. */
  interface Props {
    label: string;
    value: string | number;
    detail?: string;
    tone?: "accent" | "ok" | "warn" | "info";
  }

  const { label, value, detail = "", tone = "accent" }: Props = $props();
</script>

<div class="card tone-{tone}">
  <span class="label">{label}</span>
  <strong class="value">{value}</strong>
  {#if detail}
    <span class="detail">{detail}</span>
  {/if}
</div>

<style>
  .card {
    display: flex;
    flex-direction: column;
    gap: 2px;
    padding: var(--space-4);
    background: var(--bg-panel);
    border: 1px solid var(--border);
    border-radius: var(--radius-lg);
    border-left: 3px solid var(--accent);
    min-width: 0;
  }

  .tone-ok {
    border-left-color: var(--ok);
  }
  .tone-warn {
    border-left-color: var(--warn);
  }
  .tone-info {
    border-left-color: var(--info);
  }

  .label {
    font-size: 11px;
    font-weight: 600;
    text-transform: uppercase;
    letter-spacing: 0.06em;
    color: var(--text-muted);
  }

  .value {
    font-size: 26px;
    line-height: 1.15;
    font-weight: 650;
    color: var(--text-primary);
    font-variant-numeric: tabular-nums;
  }

  .detail {
    font-size: 12px;
    color: var(--text-secondary);
  }
</style>
