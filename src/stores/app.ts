// Application-wide UI state: the current view, the theme and the settings.

import { writable, derived, get } from "svelte/store";
import type { AppSettings } from "../domain/types";
import { api } from "../lib/tauri";

/** The navigation model. Every entry opens a real view. */
export interface NavItem {
  id: ViewId;
  label: string;
  group: "Operate" | "Govern" | "Workspace" | "Crew";
  hint: string;
}

export type ViewId =
  | "night-shift"
  | "agents"
  | "runs"
  | "waiting"
  | "policies"
  | "access"
  | "compute"
  | "audit"
  | "tools"
  | "team"
  | "settings"
  | "crew";

export const NAV_ITEMS: NavItem[] = [
  { id: "night-shift", label: "Night Shift", group: "Operate", hint: "The live operations overview" },
  { id: "agents", label: "Agents", group: "Operate", hint: "The agent registry" },
  { id: "runs", label: "Runs", group: "Operate", hint: "Every run and its timeline" },
  { id: "waiting", label: "Waiting on You", group: "Operate", hint: "Approvals that need a human" },
  { id: "policies", label: "Policies", group: "Govern", hint: "The local policy set" },
  { id: "access", label: "Access", group: "Govern", hint: "Trust levels and capabilities" },
  { id: "compute", label: "Compute", group: "Govern", hint: "Where the compute went" },
  { id: "audit", label: "Audit Log", group: "Govern", hint: "Everything that has been recorded" },
  { id: "tools", label: "Tools", group: "Workspace", hint: "The tool registry and its health" },
  { id: "team", label: "Team", group: "Workspace", hint: "Who is on shift" },
  { id: "settings", label: "Settings", group: "Workspace", hint: "Theme, density and simulator" },
  { id: "crew", label: "Crew", group: "Crew", hint: "Compact status for every agent" },
];

/** The view currently open. */
export const currentView = writable<ViewId>("night-shift");

/** Whether the left navigation is collapsed. */
export const sidebarCollapsed = writable(false);

/** The persisted settings. */
export const settings = writable<AppSettings>({
  theme: "dark",
  compactMode: false,
  simulatorSpeed: 1,
  timezone: "UTC",
  notificationsEnabled: true,
});

/** Whether the settings have loaded from the database yet. */
export const settingsLoaded = writable(false);

/** The last error the shell should surface. */
export const shellError = writable<string | null>(null);

/** The current theme, derived from the settings. */
export const theme = derived(settings, ($settings) => $settings.theme);

/**
 * Apply the theme to the document element.
 *
 * The attribute is set on <html> so the CSS variable sets in app.css
 * switch wholesale.
 */
export function applyTheme(value: string): void {
  const root = document.documentElement;
  root.setAttribute("data-theme", value === "light" ? "light" : "dark");
  root.setAttribute("data-density", get(settings).compactMode ? "compact" : "comfortable");
}

/** Load the settings and apply the theme. */
export async function loadSettings(): Promise<void> {
  try {
    const loaded = await api.getSettings();
    settings.set(loaded);
    applyTheme(loaded.theme);
  } catch (error) {
    shellError.set(error instanceof Error ? error.message : String(error));
  } finally {
    settingsLoaded.set(true);
  }
}

/**
 * Update the settings and persist them.
 *
 * The store is updated from the server's response, not from the local
 * edit, so what the UI shows is what was actually stored.
 */
export async function saveSettings(next: AppSettings): Promise<boolean> {
  try {
    const saved = await api.updateSettings(next);
    settings.set(saved);
    applyTheme(saved.theme);
    shellError.set(null);
    return true;
  } catch (error) {
    shellError.set(error instanceof Error ? error.message : String(error));
    return false;
  }
}

/** Navigate to a view. */
export function navigate(view: ViewId): void {
  currentView.set(view);
}

/** The label of the current view, used in the breadcrumb. */
export const currentViewLabel = derived(currentView, ($view) => {
  return NAV_ITEMS.find((item) => item.id === $view)?.label ?? "Night Shift";
});
