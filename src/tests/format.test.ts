import { describe, expect, it } from "vitest";
import {
  formatDateTime,
  formatDuration,
  formatNumber,
  formatPercent,
  formatRelative,
  formatTime,
  initials,
  parseDate,
  setDisplayTimeZone,
  statusLabel,
} from "../lib/format";

describe("formatTime", () => {
  it("formats a valid timestamp", () => {
    expect(formatTime("2026-01-01T12:30:00Z")).toMatch(/\d{1,2}:\d{2}/);
  });

  it("shows a placeholder for an invalid timestamp rather than NaN", () => {
    expect(formatTime("not-a-time")).toBe("--:--");
    expect(formatTime("")).toBe("--:--");
  });

  it("shows the time in the zone it is given", () => {
    // The same instant, two zones. Johannesburg is two hours ahead of
    // UTC, so the two renderings differ.
    const iso = "2026-01-01T12:30:00Z";
    expect(formatTime(iso, "UTC")).not.toBe(formatTime(iso, "Africa/Johannesburg"));
    expect(formatTime(iso, "UTC")).not.toBe(formatTime(iso, "Asia/Tokyo"));
  });

  it("falls back to the machine's zone when the zone is unusable", () => {
    // The backend accepts any IANA-shaped name, and Intl throws a
    // RangeError on a zone it does not know. A bad zone must not take
    // the view down.
    expect(() => formatTime("2026-01-01T12:30:00Z", "not/a/zone")).not.toThrow();
    expect(formatTime("2026-01-01T12:30:00Z", "not/a/zone")).toMatch(/\d{1,2}:\d{2}/);
  });

  it("uses the configured display zone when none is given", () => {
    const iso = "2026-01-01T12:30:00Z";
    // With no zone configured the machine's own zone is used.
    const machine = formatTime(iso);
    setDisplayTimeZone("UTC");
    const inUtc = formatTime(iso);
    setDisplayTimeZone("Asia/Tokyo");
    const inTokyo = formatTime(iso);
    setDisplayTimeZone(undefined);
    expect(inUtc).not.toBe(inTokyo);
    expect(formatTime(iso)).toBe(machine);
  });
});

describe("formatDateTime", () => {
  it("shows the date and time in the zone it is given", () => {
    const iso = "2026-01-01T12:30:00Z";
    expect(formatDateTime(iso, "UTC")).not.toBe(formatDateTime(iso, "Asia/Tokyo"));
  });

  it("shows a placeholder for an invalid timestamp", () => {
    expect(formatDateTime("not-a-time")).toBe("unknown");
  });

  it("falls back rather than throwing on an unusable zone", () => {
    expect(() => formatDateTime("2026-01-01T12:30:00Z", "not/a/zone")).not.toThrow();
  });
});

describe("parseDate", () => {
  it("returns null for an unparseable value", () => {
    expect(parseDate("nonsense")).toBeNull();
    expect(parseDate("")).toBeNull();
  });

  it("returns a date for a valid value", () => {
    expect(parseDate("2026-01-01T00:00:00Z")).toBeInstanceOf(Date);
  });
});

describe("formatRelative", () => {
  const now = new Date("2026-01-01T12:00:00Z");

  it("describes seconds", () => {
    expect(formatRelative("2026-01-01T11:59:30Z", now)).toBe("30s ago");
  });

  it("describes minutes", () => {
    expect(formatRelative("2026-01-01T11:45:00Z", now)).toBe("15m ago");
  });

  it("describes hours", () => {
    expect(formatRelative("2026-01-01T09:00:00Z", now)).toBe("3h ago");
  });

  it("describes days", () => {
    expect(formatRelative("2025-12-30T12:00:00Z", now)).toBe("2d ago");
  });

  it("does not describe a future moment as elapsed", () => {
    expect(formatRelative("2026-01-01T13:00:00Z", now)).toBe("in the future");
  });

  it("reports an unknown time rather than a wrong one", () => {
    expect(formatRelative("nonsense", now)).toBe("unknown");
  });
});

describe("formatDuration", () => {
  it("shows seconds under a minute", () => {
    expect(formatDuration(45)).toBe("45s");
  });

  it("shows minutes and seconds", () => {
    expect(formatDuration(80)).toBe("1m 20s");
  });

  it("shows hours and minutes", () => {
    expect(formatDuration(3700)).toBe("1h 1m");
  });

  it("shows a placeholder for a missing duration", () => {
    expect(formatDuration(null)).toBe("--");
  });
});

describe("formatPercent", () => {
  it("shows one decimal place", () => {
    expect(formatPercent(0.8234)).toBe("82.3%");
  });

  it("shows a placeholder for a non-finite value", () => {
    expect(formatPercent(Number.NaN)).toBe("--");
  });
});

describe("formatNumber", () => {
  it("adds thousands separators", () => {
    expect(formatNumber(1840)).toBe((1840).toLocaleString());
  });

  it("shows a placeholder for a non-finite value", () => {
    expect(formatNumber(Number.POSITIVE_INFINITY)).toBe("--");
  });
});

describe("statusLabel", () => {
  it("uses a readable label for each known status", () => {
    expect(statusLabel("succeeded")).toBe("Succeeded");
    expect(statusLabel("running")).toBe("Running");
    expect(statusLabel("waiting")).toBe("Waiting");
    expect(statusLabel("failed")).toBe("Failed");
    expect(statusLabel("queued")).toBe("Queued");
  });

  it("passes an unknown status through unchanged", () => {
    expect(statusLabel("weird")).toBe("weird");
  });
});

describe("initials", () => {
  it("takes the first letters of two words", () => {
    expect(initials("local operator")).toBe("LO");
  });

  it("takes the first two letters of one word", () => {
    expect(initials("operator")).toBe("OP");
  });

  it("handles a hyphenated name", () => {
    expect(initials("code-archaeologist")).toBe("CA");
  });

  it("falls back for an empty name", () => {
    expect(initials("")).toBe("?");
  });
});
