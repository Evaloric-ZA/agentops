import { describe, expect, it, vi } from "vitest";

const invoke = vi.fn();

vi.mock("@tauri-apps/api/core", () => ({
  invoke: (...args: unknown[]) => invoke(...args),
}));

describe("the command bridge", () => {
  it("passes the command name and arguments through", async () => {
    invoke.mockResolvedValue([]);
    const { api } = await import("../lib/tauri");
    await api.listRuns("running", "module");
    expect(invoke).toHaveBeenCalledWith("list_runs", {
      status: "running",
      search: "module",
    });
  });

  it("sends null for an omitted filter", async () => {
    invoke.mockResolvedValue([]);
    const { api } = await import("../lib/tauri");
    await api.listRuns();
    expect(invoke).toHaveBeenCalledWith("list_runs", { status: null, search: null });
  });

  it("turns a structured error into a CommandError with its code", async () => {
    invoke.mockRejectedValue({ code: "not_found", message: "record not found" });
    const { api, CommandError } = await import("../lib/tauri");
    await expect(api.getAgent("agent-missing")).rejects.toBeInstanceOf(CommandError);
    try {
      await api.getAgent("agent-missing");
    } catch (error) {
      expect((error as { code: string }).code).toBe("not_found");
      expect((error as Error).message).toBe("record not found");
    }
  });

  it("turns a transport failure into a CommandError", async () => {
    invoke.mockRejectedValue(new Error("the window is gone"));
    const { api, CommandError } = await import("../lib/tauri");
    await expect(api.listAgents()).rejects.toBeInstanceOf(CommandError);
  });

  it("turns an unknown rejection into a CommandError", async () => {
    invoke.mockRejectedValue("something odd");
    const { api, CommandError } = await import("../lib/tauri");
    await expect(api.listAgents()).rejects.toBeInstanceOf(CommandError);
  });
});
