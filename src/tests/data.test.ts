import { describe, expect, it } from "vitest";
import { get } from "svelte/store";
import { createResource } from "../stores/data";

describe("createResource", () => {
  it("starts idle", () => {
    const resource = createResource(async () => [1, 2, 3]);
    expect(get(resource).status).toBe("idle");
    expect(get(resource).data).toBeNull();
    expect(get(resource).error).toBeNull();
  });

  it("moves to loaded when the loader succeeds", async () => {
    const resource = createResource(async () => [1, 2, 3]);
    await resource.load();
    expect(get(resource).status).toBe("loaded");
    expect(get(resource).data).toEqual([1, 2, 3]);
  });

  it("moves to empty when the loader returns nothing", async () => {
    const resource = createResource(async () => [] as number[], (rows) => rows.length === 0);
    await resource.load();
    expect(get(resource).status).toBe("empty");
  });

  it("moves to failed and keeps the message when the loader throws", async () => {
    const resource = createResource(async () => {
      throw new Error("the command failed");
    });
    await resource.load();
    const state = get(resource);
    expect(state.status).toBe("failed");
    expect(state.error).toBe("the command failed");
    expect(state.data).toBeNull();
  });

  it("passes a non-Error rejection through as a string", async () => {
    const resource = createResource(async () => {
      throw "a bare string";
    });
    await resource.load();
    expect(get(resource).error).toBe("a bare string");
  });
});

describe("the navigation model", () => {
  it("covers every view the shell can open", async () => {
    const { NAV_ITEMS } = await import("../stores/app");
    const ids = NAV_ITEMS.map((item) => item.id);
    expect(new Set(ids).size).toBe(ids.length);
    expect(ids).toContain("night-shift");
    expect(ids).toContain("settings");
    expect(ids).toContain("crew");
  });

  it("gives every item a hint", async () => {
    const { NAV_ITEMS } = await import("../stores/app");
    for (const item of NAV_ITEMS) {
      expect(item.hint.length).toBeGreaterThan(0);
      expect(item.label.length).toBeGreaterThan(0);
    }
  });
});
