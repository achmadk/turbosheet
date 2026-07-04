/**
 * Worker entry point for test execution via isolated-vm.
 *
 * The Rust TestExecutor spawns this script as a child process.
 * Communication uses newline-delimited JSON-RPC over stdin/stdout.
 *
 * Supported methods:
 *   - executeTest(filePath, timeout): Execute a single test file and return results
 *   - shutdown: Gracefully shut down the worker
 */

import * as fs from "fs";
import * as path from "path";
import * as readline from "readline";
import * as swc from "@swc/core";
import { IsolatedRuntimePool } from "./js-runtime";

interface JsonRpcRequest {
  jsonrpc: string;
  id: string;
  method: string;
  params: Record<string, any>;
}

interface JsonRpcResponse {
  jsonrpc: string;
  id: string;
  result?: any;
  error?: { code: number; message: string; data?: any };
}

interface TestResultMsg {
  name: string;
  status: "passed" | "failed" | "skipped" | "timeout";
  error?: string;
  duration_ms: number;
  video_paths?: string[];
  trace_data?: string;
}

const turbosheet = require("../../index.js");

const runtimePool = new IsolatedRuntimePool(4, 30000);
let browserInstance: any = null;

async function getBrowser() {
  if (!browserInstance) {
    browserInstance = await turbosheet.launch({ headless: true });
  }
  return browserInstance;
}
function sendResponse(response: JsonRpcResponse): void {
  process.stdout.write(JSON.stringify(response) + "\n");
}

function sendError(id: string, code: number, message: string, data?: any): void {
  sendResponse({
    jsonrpc: "2.0",
    id,
    error: { code, message, data },
  });
}

function sendResult(id: string, result: any): void {
  sendResponse({
    jsonrpc: "2.0",
    id,
    result,
  });
}

/**
 * Execute a test file inside an isolated-vm runtime.
 * Parses the file for test/describe/hook definitions, then runs each test
 * capturing pass/fail status, errors, and timing.
 */
async function executeTestFile(
  filePath: string,
  timeoutMs: number = 30000,
  videoOnFailure: boolean = false,
  videoDir?: string,
): Promise<TestResultMsg[]> {
  const results: TestResultMsg[] = [];
  let code: string;

  try {
    const rawCode = fs.readFileSync(filePath, "utf-8");
    if (filePath.endsWith(".ts")) {
      const compiled = await swc.transform(rawCode, {
        filename: filePath,
        jsc: {
          parser: {
            syntax: "typescript",
          },
          target: "es2022",
        },
        module: {
          type: "commonjs",
        },
      });
      code = compiled.code;
    } else {
      code = rawCode;
    }
  } catch (err: any) {
    results.push({
      name: path.basename(filePath),
      status: "failed",
      error: `Failed to read or transpile test file: ${err.message}`,
      duration_ms: 0,
    });
    return results;
  }

  const runtime = await runtimePool.acquire();
  let context: any;
  let page: any;

  // Track video recording state so we can stop it in all paths
  let videoRecordingStarted = false;

  try {
    const browser = await getBrowser();
    context = await browser.newContext();

    // If video is enabled, set up context-level recording defaults
    if (videoOnFailure && videoDir) {
      const videoConfig = {
        fps: 10,
        quality: 80,
        maxWidth: 800,
        maxHeight: 600,
        outputDir: videoDir,
        retention: "on-failure",
      };
      context.video().setDefaults(JSON.stringify(videoConfig));
    }

    page = await context.newPage();
    await runtime.injectPageBridge(page);

    // Clear any previous trace data before starting this test file
    try {
      turbosheet.trace_clear();
    } catch (e) {
      // Trace recording not available (traces feature disabled)
    }

    // Start video recording if enabled
    if (videoOnFailure && page.video) {
      try {
        await page.startVideoRecording();
        videoRecordingStarted = true;
      } catch (err) {
        console.warn("[VIDEO] Failed to start recording:", err);
      }
    }

    // Wrap the test code to collect test definitions first, then execute them
    const collectorCode = `
      (function() {
        const __collected_tests = [];
        const __collected_hooks = { beforeAll: [], afterAll: [], beforeEach: [], afterEach: [] };

        const test = function(name, fn) {
          __collected_tests.push({ name: name, fn: fn });
        };
        test.describe = function(name, fn) {
          // For now, flatten describes - just collect tests
          fn();
        };
        test.beforeAll = function(fn) { __collected_hooks.beforeAll.push(fn); };
        test.afterAll = function(fn) { __collected_hooks.afterAll.push(fn); };
        test.beforeEach = function(fn) { __collected_hooks.beforeEach.push(fn); };
        test.afterEach = function(fn) { __collected_hooks.afterEach.push(fn); };
        test.skip = function(name, fn) {
          __collected_tests.push({ name: name, fn: null, skip: true });
        };
        test.only = function() {};
        test.fixme = function() {};
        test.slow = function() {};
        test.fail = function() {};

        // Simple expect shim for the collection phase
        function expect(actual) {
          return {
            toBe: function(expected) {
              if (actual !== expected) throw new Error('Expected ' + JSON.stringify(expected) + ' but got ' + JSON.stringify(actual));
            },
            toEqual: function(expected) {
              if (JSON.stringify(actual) !== JSON.stringify(expected)) {
                throw new Error('Expected ' + JSON.stringify(expected) + ' but got ' + JSON.stringify(actual));
              }
            },
            toBeTruthy: function() { if (!actual) throw new Error('Expected truthy but got ' + actual); },
            toBeFalsy: function() { if (actual) throw new Error('Expected falsy but got ' + actual); },
            toContain: function(item) {
              if (typeof actual === 'string') {
                if (!actual.includes(item)) throw new Error('Expected string to contain ' + item);
              } else if (Array.isArray(actual)) {
                if (!actual.includes(item)) throw new Error('Expected array to contain ' + item);
              }
            },
            toHaveLength: function(len) {
              if (actual.length !== len) throw new Error('Expected length ' + len + ' but got ' + actual.length);
            },
            toBeVisible: function() { /* stub for locator assertions */ },
            toBeHidden: function() { /* stub */ },
            toHaveText: function() { /* stub */ },
            not: {
              toBe: function(expected) { if (actual === expected) throw new Error('Expected not ' + expected); },
              toEqual: function(expected) {
                if (JSON.stringify(actual) === JSON.stringify(expected)) throw new Error('Expected not equal');
              },
              toBeTruthy: function() { if (actual) throw new Error('Expected falsy'); },
              toBeFalsy: function() { if (!actual) throw new Error('Expected truthy'); },
              toBeVisible: function() { /* stub */ },
            },
          };
        }

        // Real page bridge is injected as a global object before execution
        
        try {
          ${code}
        } catch(e) {
          __collected_tests.push({ name: '__parse_error__', fn: null, error: e.message });
        }

        return { tests: __collected_tests, hooks: __collected_hooks };
      })()
    `;

    const parseResult = await runtime.evaluate(collectorCode, timeoutMs);

    if (parseResult.error) {
      results.push({
        name: path.basename(filePath),
        status: "failed",
        error: `Parse error: ${parseResult.error}`,
        duration_ms: 0,
      });
      return results;
    }

    const collected = parseResult.value;
    if (!collected || !collected.tests || collected.tests.length === 0) {
      results.push({
        name: path.basename(filePath),
        status: "passed",
        duration_ms: 0,
      });
      return results;
    }

    // Check for parse errors
    for (const t of collected.tests) {
      if (t.name === "__parse_error__") {
        results.push({
          name: path.basename(filePath),
          status: "failed",
          error: t.error,
          duration_ms: 0,
        });
        return results;
      }
    }

    // Execute beforeAll hooks
    for (const hookFn of collected.hooks.beforeAll) {
      try {
        await hookFn();
      } catch (err: any) {
        // If beforeAll fails, skip all tests
        for (const t of collected.tests) {
          results.push({
            name: t.name,
            status: "skipped",
            error: `beforeAll hook failed: ${err.message}`,
            duration_ms: 0,
          });
        }
        return results;
      }
    }

    // Execute each test
    for (const testDef of collected.tests) {
      if (testDef.skip || !testDef.fn) {
        results.push({
          name: testDef.name,
          status: "skipped",
          duration_ms: 0,
        });
        continue;
      }

      // Execute beforeEach hooks
      for (const hookFn of collected.hooks.beforeEach) {
        try {
          await hookFn();
        } catch (err: any) {
          results.push({
            name: testDef.name,
            status: "failed",
            error: `beforeEach hook failed: ${err.message}`,
            duration_ms: 0,
          });
          continue;
        }
      }

      const startTime = Date.now();
      try {
        // Execute with timeout
        const fixtures = { page: typeof page !== "undefined" ? page : {} };
        const testPromise = Promise.resolve(testDef.fn(fixtures));
        const timeoutPromise = new Promise((_, reject) => {
          setTimeout(
            () => reject(new Error("Test timed out after " + timeoutMs + "ms")),
            timeoutMs,
          );
        });

        await Promise.race([testPromise, timeoutPromise]);
        const duration = Date.now() - startTime;

        results.push({
          name: testDef.name,
          status: "passed",
          duration_ms: duration,
        });
      } catch (err: any) {
        const duration = Date.now() - startTime;
        const isTimeout = err.message && err.message.includes("timed out");

        results.push({
          name: testDef.name,
          status: isTimeout ? "timeout" : "failed",
          error: err.message || String(err),
          duration_ms: duration,
        });
      }

      // Execute afterEach hooks (always, even on failure)
      for (const hookFn of collected.hooks.afterEach) {
        try {
          await hookFn();
        } catch {
          // afterEach errors are logged but don't change test status
        }
      }
    }

    // Execute afterAll hooks
    for (const hookFn of collected.hooks.afterAll) {
      try {
        await hookFn();
      } catch {
        // afterAll errors are logged but don't change test status
      }
    }
  } finally {
    // Stop video recording if it was started
    if (videoRecordingStarted && page && typeof page.stopVideoRecording === "function") {
      try {
        // Pass the overall test outcome so the Rust side can enforce retention policy
        const allPassed = results.every((r) => r.status === "passed");
        const videoPath = await page.stopVideoRecording(allPassed ? true : false);
        if (videoPath) {
          for (const r of results) {
            r.video_paths = [videoPath];
          }
          // If Rust-side retention deleted the file, clear paths
          try {
            if (!fs.existsSync(videoPath)) {
              for (const r of results) {
                r.video_paths = undefined;
              }
            }
          } catch {}
        }
      } catch (e) {
        console.warn("[VIDEO] Failed to stop recording:", e);
      }
    }

    // Serialize trace events and attach to results
    try {
      const traceData = turbosheet.trace_events_to_json();
      for (const r of results) {
        r.trace_data = traceData;
      }
    } catch (e) {
      // Trace data not available (traces feature disabled)
    }

    if (page && typeof page.close === "function") {
      await page.close();
    }
    if (context && typeof context.close === "function") {
      await context.close();
    }
    await runtimePool.release(runtime);
  }

  return results;
}

async function handleRequest(request: JsonRpcRequest): Promise<void> {
  switch (request.method) {
    case "executeTest": {
      const { filePath, timeout, videoOnFailure, videoDir } = request.params;
      if (!filePath) {
        sendError(request.id, -32602, "Missing filePath parameter");
        return;
      }
      try {
        const results = await executeTestFile(
          filePath,
          timeout || 30000,
          videoOnFailure === true,
          videoDir || undefined,
        );
        sendResult(request.id, { results });
      } catch (err: any) {
        sendError(request.id, -32603, `Execution failed: ${err.message}`);
      }
      break;
    }
    case "shutdown": {
      sendResult(request.id, { status: "ok" });
      await runtimePool.terminateAll();
      process.exit(0);
      break;
    }
    default:
      sendError(request.id, -32601, `Unknown method: ${request.method}`);
  }
}

// Signal readiness
process.stdout.write(JSON.stringify({ jsonrpc: "2.0", method: "ready", params: {} }) + "\n");

// Read JSON-RPC requests from stdin, one per line
const rl = readline.createInterface({ input: process.stdin, terminal: false });

rl.on("line", async (line: string) => {
  if (!line.trim()) return;

  let request: JsonRpcRequest;
  try {
    request = JSON.parse(line);
  } catch {
    sendError("unknown", -32700, "Parse error");
    return;
  }

  await handleRequest(request);
});

rl.on("close", async () => {
  await runtimePool.terminateAll();
  process.exit(0);
});
