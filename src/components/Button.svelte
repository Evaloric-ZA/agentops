<script lang="ts">
  /**
   * A button.
   *
   * A control that is outside v0.1 is disabled with an explanatory tooltip
   * rather than silently doing nothing.
   */
  interface Props {
    label: string;
    onClick?: () => void;
    variant?: "primary" | "secondary" | "ghost" | "danger";
    disabled?: boolean;
    disabledReason?: string;
    type?: "button" | "submit";
  }

  const {
    label,
    onClick,
    variant = "secondary",
    disabled = false,
    disabledReason = "",
    type = "button",
  }: Props = $props();
</script>

<button
  {type}
  class="btn btn-{variant}"
  {disabled}
  title={disabled && disabledReason ? disabledReason : undefined}
  aria-disabled={disabled}
  onclick={() => onClick?.()}
>
  {label}
</button>

<style>
  .btn {
    padding: 6px 12px;
    border-radius: var(--radius-md);
    border: 1px solid var(--border-strong);
    background: var(--bg-raised);
    color: var(--text-primary);
    font-size: 12px;
    font-weight: 600;
    transition: background var(--transition), border-color var(--transition);
    white-space: nowrap;
  }

  .btn:hover:not(:disabled) {
    background: var(--bg-hover);
  }

  .btn-primary {
    background: var(--accent-strong);
    border-color: var(--accent-strong);
    color: #ffffff;
  }

  .btn-primary:hover:not(:disabled) {
    background: var(--accent);
  }

  .btn-ghost {
    background: transparent;
    border-color: transparent;
    color: var(--text-secondary);
  }

  .btn-danger {
    background: transparent;
    border-color: var(--danger);
    color: var(--danger);
  }

  .btn-danger:hover:not(:disabled) {
    background: var(--danger-soft);
  }
</style>
