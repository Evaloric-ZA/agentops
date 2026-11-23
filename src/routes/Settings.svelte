<script lang="ts">
  /** Settings: theme, density, simulator speed and timezone. All persisted. */
  import { api } from "../lib/tauri";
  import { settings, saveSettings, shellError } from "../stores/app";
  import { formatDateTime } from "../lib/format";
  import Panel from "../components/Panel.svelte";
  import Button from "../components/Button.svelte";
  import type { AppSettings } from "../domain/types";

  let draft = $state<AppSettings>({ ...$settings });
  let saved = $state(false);
  let saving = $state(false);
  let resetBusy = $state(false);
  let resetMessage = $state<string | null>(null);

  $effect(() => {
    draft = { ...$settings };
  });

  const TIMEZONES = [
    "UTC",
    "Africa/Johannesburg",
    "Europe/London",
    "Europe/Berlin",
    "America/New_York",
    "America/Los_Angeles",
    "Asia/Tokyo",
    "Australia/Sydney",
  ];

  async function save() {
    saving = true;
    saved = false;
    const ok = await saveSettings({ ...draft });
    saving = false;
    saved = ok;
    if (ok) {
      await api.simulatorSetSpeed(draft.simulatorSpeed);
    }
  }

  async function resetDemo() {
    resetBusy = true;
    resetMessage = null;
    try {
      await api.simulatorReset();
      resetMessage = "The demo data was restored to its original seeded state.";
    } catch (error) {
      resetMessage = error instanceof Error ? error.message : String(error);
    } finally {
      resetBusy = false;
    }
  }
</script>

<div class="page">
  <Panel title="Settings" subtitle="Persisted in the local database">
    <div class="form">
      <div class="row">
        <label for="theme">Theme</label>
        <select id="theme" bind:value={draft.theme}>
          <option value="dark">Dark</option>
          <option value="light">Light</option>
        </select>
      </div>

      <div class="row">
        <label for="compact">Density</label>
        <select id="compact" bind:value={draft.compactMode}>
          <option value={false}>Comfortable</option>
          <option value={true}>Compact</option>
        </select>
      </div>

      <div class="row">
        <label for="speed">Simulator speed</label>
        <input
          id="speed"
          type="range"
          min="0.25"
          max="4"
          step="0.25"
          bind:value={draft.simulatorSpeed}
        />
        <span class="value">{Number(draft.simulatorSpeed).toFixed(2)}x</span>
      </div>

      <div class="row">
        <label for="timezone">Timezone</label>
        <select id="timezone" bind:value={draft.timezone}>
          {#each TIMEZONES as zone (zone)}
            <option value={zone}>{zone}</option>
          {/each}
        </select>
      </div>

      <div class="row">
        <label for="notifications">Notifications</label>
        <input
          id="notifications"
          type="checkbox"
          bind:checked={draft.notificationsEnabled}
        />
        <span class="hint">Show a badge when an approval is waiting.</span>
      </div>

      <div class="actions">
        <Button label="Save settings" variant="primary" disabled={saving} onClick={save} />
        {#if saved}
          <span class="saved" role="status">Saved.</span>
        {/if}
        {#if $shellError}
          <span class="error" role="alert">{$shellError}</span>
        {/if}
      </div>
    </div>
  </Panel>

  <Panel title="Demo data" subtitle="Restore the original seeded state">
    <p class="muted">
      The reset re-seeds every table and rewinds the simulator. The result is
      identical to the first launch, which is what makes the demo
      reproducible.
    </p>
    <div class="actions">
      <Button
        label="Reset demo data"
        variant="danger"
        disabled={resetBusy}
        onClick={resetDemo}
      />
      {#if resetMessage}
        <span class="saved" role="status">{resetMessage}</span>
      {/if}
    </div>
    <p class="muted small">Rendered at {formatDateTime(new Date().toISOString())}.</p>
  </Panel>
</div>

<style>
  .page {
    padding: var(--space-4);
    height: 100%;
    overflow-y: auto;
    display: flex;
    flex-direction: column;
    gap: var(--space-4);
    max-width: 720px;
  }

  .form {
    display: flex;
    flex-direction: column;
    gap: var(--space-3);
  }

  .row {
    display: grid;
    grid-template-columns: 160px 1fr;
    align-items: center;
    gap: var(--space-3);
  }

  label {
    font-size: 13px;
    font-weight: 600;
  }

  select,
  input[type="range"] {
    padding: 5px 8px;
    border-radius: var(--radius-md);
    border: 1px solid var(--border);
    background: var(--bg-base);
    font-size: 12px;
  }

  input[type="range"] {
    padding: 0;
  }

  .value {
    font-size: 12px;
    color: var(--text-muted);
    font-variant-numeric: tabular-nums;
  }

  .hint {
    grid-column: 2;
    font-size: 11px;
    color: var(--text-muted);
  }

  .actions {
    display: flex;
    align-items: center;
    gap: var(--space-3);
    margin-top: var(--space-2);
  }

  .saved {
    font-size: 12px;
    color: var(--ok);
  }

  .error {
    font-size: 12px;
    color: var(--danger);
  }

  .muted {
    font-size: 12px;
    color: var(--text-muted);
    margin: 0 0 var(--space-3);
  }

  .small {
    margin-top: var(--space-3);
    font-size: 11px;
  }
</style>
