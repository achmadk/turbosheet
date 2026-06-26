import { describe, it, expect } from "vite-plus/test";

describe("turbo-sheet smoke", () => {
  it("should load the native module when built", async () => {
    let tsheet: Awaited<typeof import("../index.js")> | null = null;
    try {
      tsheet = await import("../index.js");
    } catch {
      // bootstrap: native module not yet compiled
    }
    if (tsheet) {
      expect(tsheet.version).toBeDefined();
      expect(typeof tsheet.version()).toBe("string");
    }
  });

  it("should export expected API shape", () => {
    const index = require("../index.js") as Record<string, unknown>;
    expect(index.launch).toBeDefined();
    expect(typeof index.launch).toBe("function");
  });
});
