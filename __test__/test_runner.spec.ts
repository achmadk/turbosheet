import { describe, it, expect } from "vite-plus/test";
import { createRequire } from "module";
const require = createRequire(import.meta.url);
const nativeBinding = require("../turbosheet.linux-x64-gnu.node");

describe("TurboSheet Test Runner Core", () => {
  it("should execute tests and return results", async () => {
    const config = {
      testDir: ".",
      testMatch: ["**/*.tsheet.ts"],
      workers: 2,
      retries: 0,
      timeout: 30000,
      reporter: "list",
    };

    const results = await nativeBinding.runTests(config);

    expect(Array.isArray(results)).toBe(true);
  });
});
