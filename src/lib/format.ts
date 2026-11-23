// Formatting helpers shared by the views.
//
// Times are stored in UTC and localised for display. The helpers here are
// pure functions so they can be unit tested without a browser.

import type { RunStatus } from "../domain/types";

/** Format an RFC 3339 timestamp as a local time, e.g. "14:32". */
export function formatTime(iso: string): string {
  const date = parseDate(iso);
  if (!date) return "--:--";
  return date.toLocaleTimeString(undefined, { hour: "2-digit", minute: "2-digit" });
}

/** Format an RFC 3339 timestamp as a local date and time. */
export function formatDateTime(iso: string): string {
  const date = parseDate(iso);
  if (!date) return "unknown";
  return date.toLocaleString(undefined, {
    year: "numeric",
    month: "short",
    day: "2-digit",
    hour: "2-digit",
    minute: "2-digit",
  });
}

/** A short "3m ago" style description of how long ago a moment was. */
export function formatRelative(iso: string, now: Date = new Date()): string {
  const date = parseDate(iso);
  if (!date) return "unknown";
  const seconds = Math.round((now.getTime() - date.getTime()) / 1000);
  if (seconds < 0) return "in the future";
  if (seconds < 60) return `${seconds}s ago`;
  const minutes = Math.floor(seconds / 60);
  if (minutes < 60) return `${minutes}m ago`;
  const hours = Math.floor(minutes / 60);
  if (hours < 24) return `${hours}h ago`;
  return `${Math.floor(hours / 24)}d ago`;
}

/** A human duration from seconds, e.g. "1m 20s". */
export function formatDuration(seconds: number | null): string {
  if (seconds === null || !Number.isFinite(seconds)) return "--";
  if (seconds < 60) return `${Math.round(seconds)}s`;
  const minutes = Math.floor(seconds / 60);
  const rest = Math.round(seconds % 60);
  if (minutes < 60) return `${minutes}m ${rest}s`;
  const hours = Math.floor(minutes / 60);
  return `${hours}h ${minutes % 60}m`;
}

/** A percentage with one decimal place. */
export function formatPercent(value: number): string {
  if (!Number.isFinite(value)) return "--";
  return `${(value * 100).toFixed(1)}%`;
}

/** A number with thousands separators. */
export function formatNumber(value: number): string {
  if (!Number.isFinite(value)) return "--";
  return value.toLocaleString();
}

/** The human label for a run status. */
export function statusLabel(status: RunStatus | string): string {
  switch (status) {
    case "succeeded":
      return "Succeeded";
    case "running":
      return "Running";
    case "waiting":
      return "Waiting";
    case "failed":
      return "Failed";
    case "queued":
      return "Queued";
    default:
      return status;
  }
}

/** Parse an RFC 3339 timestamp, returning null for anything invalid. */
export function parseDate(iso: string): Date | null {
  if (!iso) return null;
  const date = new Date(iso);
  return Number.isNaN(date.getTime()) ? null : date;
}

/** The initials shown in an avatar chip. */
export function initials(name: string): string {
  const parts = name.split(/[\s-]+/).filter(Boolean);
  if (parts.length === 0) return "?";
  if (parts.length === 1) return parts[0].slice(0, 2).toUpperCase();
  return (parts[0][0] + parts[1][0]).toUpperCase();
}
