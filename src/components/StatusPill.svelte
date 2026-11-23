<script lang="ts">
  /** A small coloured pill showing a status. */
  interface Props {
    status: string;
    label?: string;
  }

  const { status, label }: Props = $props();

  const TONE: Record<string, string> = {
    working: "ok",
    running: "ok",
    succeeded: "ok",
    approved: "ok",
    healthy: "ok",
    idle: "muted",
    queued: "muted",
    pending: "warn",
    waiting: "warn",
    approval: "warn",
    degraded: "warn",
    medium: "warn",
    failed: "danger",
    rejected: "danger",
    offline: "danger",
    high: "danger",
    low: "muted",
  };

  const tone = $derived(TONE[status] ?? "info");
</script>

<span class="pill pill-{tone}">
  <span class="dot" aria-hidden="true"></span>
  {label ?? status}
</span>

<style>
  .pill {
    display: inline-flex;
    align-items: center;
    gap: 6px;
    padding: 2px 9px;
    border-radius: 999px;
    font-size: 11px;
    font-weight: 600;
    letter-spacing: 0.02em;
    text-transform: capitalize;
    border: 1px solid transparent;
    white-space: nowrap;
  }

  .dot {
    width: 6px;
    height: 6px;
    border-radius: 50%;
    background: currentColor;
    flex: none;
  }

  .pill-ok {
    color: var(--ok);
    background: var(--ok-soft);
    border-color: var(--ok-soft);
  }
  .pill-warn {
    color: var(--warn);
    background: var(--warn-soft);
    border-color: var(--warn-soft);
  }
  .pill-danger {
    color: var(--danger);
    background: var(--danger-soft);
    border-color: var(--danger-soft);
  }
  .pill-info {
    color: var(--info);
    background: var(--info-soft);
    border-color: var(--info-soft);
  }
  .pill-muted {
    color: var(--text-muted);
    background: var(--bg-hover);
    border-color: var(--border);
  }
</style>
