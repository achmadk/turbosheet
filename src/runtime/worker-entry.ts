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

    // Wrap the test code to collect definitions with modifiers, suite nesting, and hooks
    // Tests and hooks store function references (fn) for direct execution,
    // plus modifier/suite_path metadata. Serialization to fn_body happens
    // in the extractTests handler for the two-phase protocol.
    const collectorCode = `
      (function() {
        const __collected_tests = [];
        const __collected_hooks = [];
        const __collected_suites = [];
        const __suite_stack = [];

        function __current_suite_path() {
          return __suite_stack.slice();
        }

        function __describe(name, suite_type, fn) {
          __suite_stack.push(name);
          const full_path = __suite_stack.slice();
          __collected_suites.push({
            name: name,
            suite_type: suite_type,
            suite_path: full_path,
          });
          fn();
          __suite_stack.pop();
        }

        function __describe_impl(name, fn) { __describe(name, "default", fn); }
        __describe_impl.serial  = function(name, fn) { __describe(name, "serial", fn); };
        __describe_impl.parallel = function(name, fn) { __describe(name, "parallel", fn); };
        __describe_impl.skip   = function(name, fn) { __describe(name, "skip", fn); };
        __describe_impl.only   = function(name, fn) { __describe(name, "only", fn); };

        function __push_test(name, modifier, fn) {
          __collected_tests.push({
            name: name,
            modifier: modifier,
            suite_path: __current_suite_path(),
            fn: fn,
          });
        }

        const test = function(name, fn) { __push_test(name, "normal", fn); };
        test.only   = function(name, fn) { __push_test(name, "only", fn); };
        test.skip   = function(name, fn) { __push_test(name, "skip", fn); };
        test.fixme  = function(name, fn) { __push_test(name, "fixme", fn); };
        test.slow   = function(name, fn) { __push_test(name, "slow", fn); };
        test.fail   = function(name, fn) { __push_test(name, "fail", fn); };
        test.describe = __describe_impl;

        function __push_hook(hook_type, fn) {
          __collected_hooks.push({
            hook_type: hook_type,
            suite_path: __current_suite_path(),
            fn: fn,
          });
        }

        test.beforeAll   = function(fn) { __push_hook("beforeAll", fn); };
        test.afterAll    = function(fn) { __push_hook("afterAll", fn); };
        test.beforeEach  = function(fn) { __push_hook("beforeEach", fn); };
        test.afterEach   = function(fn) { __push_hook("afterEach", fn); };

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

        try {
          ${code}
        } catch(e) {
          __collected_tests.push({ name: '__parse_error__', modifier: "normal", suite_path: [], fn: null, error: e.message });
        }

        return { tests: __collected_tests, hooks: __collected_hooks, suites: __collected_suites };
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

    // Filter hooks by type from flat array
    const hooksByType = (type: string) =>
      collected.hooks.filter((h: any) => h.hook_type === type).map((h: any) => h.fn);

    const beforeAllHooks = hooksByType("beforeAll");
    const afterAllHooks = hooksByType("afterAll");
    const beforeEachHooks = hooksByType("beforeEach");
    const afterEachHooks = hooksByType("afterEach");

    // Execute beforeAll hooks
    for (const hookFn of beforeAllHooks) {
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
      if (testDef.modifier === "skip" || !testDef.fn) {
        results.push({
          name: testDef.name,
          status: "skipped",
          duration_ms: 0,
        });
        continue;
      }

      // Execute beforeEach hooks
      for (const hookFn of beforeEachHooks) {
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
      for (const hookFn of afterEachHooks) {
        try {
          await hookFn();
        } catch {
          // afterEach errors are logged but don't change test status
        }
      }
    }

    // Execute afterAll hooks
    for (const hookFn of afterAllHooks) {
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

/**
 * Phase 1: Extract test definitions from a file (no browser).
 * Reads file, transpiles, runs collector wrapper in isolated-vm,
 * returns structured test/hook/suite definitions.
 */
async function handleExtractTests(
  filePath: string,
  timeoutMs: number,
): Promise<{ tests: any[]; hooks: any[]; suites: any[] }> {
  const rawCode = fs.readFileSync(filePath, "utf-8");
  let code: string;

  if (filePath.endsWith(".ts")) {
    const compiled = await swc.transform(rawCode, {
      filename: filePath,
      jsc: {
        parser: { syntax: "typescript" },
        target: "es2022",
      },
      module: { type: "commonjs" },
    });
    code = compiled.code;
  } else {
    code = rawCode;
  }

  const runtime = await runtimePool.acquire();

  try {
    // Use same collector wrapper (no browser needed for extraction)
    const collectorCode = `
      (function() {
        const __collected_tests = [];
        const __collected_hooks = [];
        const __collected_suites = [];
        const __suite_stack = [];

        function __current_suite_path() {
          return __suite_stack.slice();
        }

        function __describe(name, suite_type, fn) {
          __suite_stack.push(name);
          const full_path = __suite_stack.slice();
          __collected_suites.push({
            name: name,
            suite_type: suite_type,
            suite_path: full_path,
          });
          fn();
          __suite_stack.pop();
        }

        function __describe_impl(name, fn) { __describe(name, "default", fn); }
        __describe_impl.serial  = function(name, fn) { __describe(name, "serial", fn); };
        __describe_impl.parallel = function(name, fn) { __describe(name, "parallel", fn); };
        __describe_impl.skip   = function(name, fn) { __describe(name, "skip", fn); };
        __describe_impl.only   = function(name, fn) { __describe(name, "only", fn); };

        function __push_test(name, modifier, fn) {
          __collected_tests.push({ name, modifier, suite_path: __current_suite_path(), fn });
        }

        const test = function(name, fn) { __push_test(name, "normal", fn); };
        test.only   = function(name, fn) { __push_test(name, "only", fn); };
        test.skip   = function(name, fn) { __push_test(name, "skip", fn); };
        test.fixme  = function(name, fn) { __push_test(name, "fixme", fn); };
        test.slow   = function(name, fn) { __push_test(name, "slow", fn); };
        test.fail   = function(name, fn) { __push_test(name, "fail", fn); };
        test.describe = __describe_impl;

        function __push_hook(hook_type, fn) {
          __collected_hooks.push({ hook_type, suite_path: __current_suite_path(), fn });
        }

        test.beforeAll   = function(fn) { __push_hook("beforeAll", fn); };
        test.afterAll    = function(fn) { __push_hook("afterAll", fn); };
        test.beforeEach  = function(fn) { __push_hook("beforeEach", fn); };
        test.afterEach   = function(fn) { __push_hook("afterEach", fn); };

        function expect(actual) {
          return {
            toBe: function(expected) {
              if (actual !== expected) throw new Error('Expected ' + JSON.stringify(expected) + ' but got ' + JSON.stringify(actual));
            },
            toEqual: function(expected) {
              if (JSON.stringify(actual) !== JSON.stringify(expected)) throw new Error('Expected ' + JSON.stringify(expected) + ' but got ' + JSON.stringify(actual));
            },
            toBeTruthy: function() { if (!actual) throw new Error('Expected truthy but got ' + actual); },
            toBeFalsy: function() { if (actual) throw new Error('Expected falsy but got ' + actual); },
            toContain: function(item) {
              if (typeof actual === 'string') { if (!actual.includes(item)) throw new Error('Expected string to contain ' + item); }
              else if (Array.isArray(actual)) { if (!actual.includes(item)) throw new Error('Expected array to contain ' + item); }
            },
            toHaveLength: function(len) {
              if (actual.length !== len) throw new Error('Expected length ' + len + ' but got ' + actual.length);
            },
            toBeVisible: function() {},
            toBeHidden: function() {},
            toHaveText: function() {},
            not: {
              toBe: function(expected) { if (actual === expected) throw new Error('Expected not ' + expected); },
              toEqual: function(expected) {
                if (JSON.stringify(actual) === JSON.stringify(expected)) throw new Error('Expected not equal');
              },
              toBeTruthy: function() { if (actual) throw new Error('Expected falsy'); },
              toBeFalsy: function() { if (!actual) throw new Error('Expected truthy'); },
              toBeVisible: function() {},
            },
          };
        }

        try {
          ${code}
        } catch(e) {
          __collected_tests.push({ name: '__parse_error__', modifier: "normal", suite_path: [], fn: null, error: e.message });
        }

        return { tests: __collected_tests, hooks: __collected_hooks, suites: __collected_suites };
      })()
    `;

    const parseResult = await runtime.evaluate(collectorCode, timeoutMs);

    if (parseResult.error) {
      return { tests: [], hooks: [], suites: [] };
    }

    const collected = parseResult.value;

    // Strip function references, serialize fn_body for IPC
    return {
      tests: (collected.tests || []).map((t: any) => ({
        name: t.name,
        modifier: t.modifier || "normal",
        suite_path: t.suite_path || [],
        fn_body: typeof t.fn === "function" ? t.fn.toString() : "",
      })),
      hooks: (collected.hooks || []).map((h: any) => ({
        hook_type: h.hook_type,
        suite_path: h.suite_path || [],
        fn_body: typeof h.fn === "function" ? h.fn.toString() : "",
      })),
      suites: (collected.suites || []).map((s: any) => ({
        name: s.name,
        suite_type: s.suite_type || "default",
        suite_path: s.suite_path || [],
      })),
    };
  } finally {
    await runtimePool.release(runtime);
  }
}

/**
 * Phase 2: Execute a single test plan (with browser).
 * Receives an ExecutionPlan, launches browser + page,
 * runs hooks and test in isolated-vm sandbox, returns PlanResponse.
 */
async function handleRunPlan(plan: any): Promise<{
  name: string;
  status: string;
  error?: string;
  duration_ms: number;
}> {
  const timeoutMs = plan.timeout_ms || 30000;
  const runtime = await runtimePool.acquire();

  try {
    const browser = await getBrowser();
    const context = await browser.newContext();
    const page = await context.newPage();
    await runtime.injectPageBridge(page);

    // Build an async script that runs the full lifecycle in one isolate context
    const execScript = buildRunPlanScript(plan);
    const result = await runtime.evaluateAsync(execScript, timeoutMs);

    await page.close();
    await context.close();

    if (result.timedOut) {
      return {
        name: plan.test_name || "unknown",
        status: "timeout",
        error: `Test timed out after ${timeoutMs}ms`,
        duration_ms: timeoutMs,
      };
    }

    if (result.error) {
      return {
        name: plan.test_name || "unknown",
        status: "failed",
        error: `Execution error: ${result.error}`,
        duration_ms: 0,
      };
    }

    return result.value;
  } finally {
    await runtimePool.release(runtime);
  }
}

/**
 * Build a self-contained script string that executes a complete test lifecycle
 * (beforeAll → beforeEach → test → afterEach → afterAll) in a single isolate eval.
 */
function buildRunPlanScript(plan: any): string {
  // Sanitize hook bodies - wrap each in an async function
  const beforeAllFns = (plan.before_all_hooks || []).map(
    (body: string) => `async () => { ${body} }`,
  );
  const beforeEachFns = (plan.before_each_hooks || []).map(
    (body: string) => `async () => { ${body} }`,
  );
  const afterEachFns = (plan.after_each_hooks || []).map(
    (body: string) => `async () => { ${body} }`,
  );
  const afterAllFns = (plan.after_all_hooks || []).map((body: string) => `async () => { ${body} }`);

  const isFail = !!plan.is_fail;
  const isFixme = !!plan.is_fixme;
  const runBeforeAll = !!plan.run_before_all;
  const runAfterAll = !!plan.run_after_all;
  const testFnBody = plan.test_fn_body || "";

  return `
    (async function() {
      const __fixtures = { page: typeof page !== "undefined" ? page : {} };

      const __beforeAll = [${beforeAllFns.join(",\n")}];
      const __beforeEach = [${beforeEachFns.join(",\n")}];
      const __afterEach = [${afterEachFns.join(",\n")}];
      const __afterAll = [${afterAllFns.join(",\n")}];

      const __testFn = async ({ page }) => { ${testFnBody} };

      const __start = Date.now();

      try {
        // beforeAll
        if (${runBeforeAll}) {
          for (const fn of __beforeAll) { try { await fn(); } catch(e) { return { status: "skipped", duration_ms: Date.now() - __start, error: "beforeAll hook failed: " + (e.message || String(e)) }; } }
        }

        // beforeEach
        for (const fn of __beforeEach) { await fn(); }

        // Test
        let __testError = null;
        try {
          await __testFn(__fixtures);
        } catch(e) {
          __testError = e.message || String(e);
        }

        const duration = Date.now() - __start;

        // afterEach (always, even on failure)
        for (const fn of __afterEach) { try { await fn(); } catch {} }

        // afterAll
        if (${runAfterAll}) {
          for (const fn of __afterAll) { try { await fn(); } catch {} }
        }

        // Modifier result mapping
        if (__testError) {
          if (${isFail}) {
            // test.fail: expected to fail, it did
            return { status: "passed", duration_ms: duration, error: null };
          }
          if (${isFixme}) {
            // test.fixme: known failure, report as fixme
            return { status: "fixme", duration_ms: duration, error: __testError };
          }
          return { status: "failed", duration_ms: duration, error: __testError };
        } else {
          if (${isFail}) {
            // test.fail: expected to fail but passed
            return { status: "failed", duration_ms: duration, error: "Expected to fail, but passed" };
          }
          return { status: "passed", duration_ms: duration, error: null };
        }
      } finally {
        // Ensure afterAll runs even if beforeAll fails
        if (${runAfterAll}) {
          for (const fn of __afterAll) { try { await fn(); } catch {} }
        }
      }
    })()
  `;
}

async function handleRequest(request: JsonRpcRequest): Promise<void> {
  switch (request.method) {
    case "extractTests": {
      const { filePath, timeout } = request.params;
      if (!filePath) {
        sendError(request.id, -32602, "Missing filePath parameter");
        return;
      }
      try {
        const result = await handleExtractTests(filePath, timeout || 30000);
        sendResult(request.id, result);
      } catch (err: any) {
        sendError(request.id, -32603, `Extraction failed: ${err.message}`);
      }
      break;
    }
    case "runPlan": {
      const { plan } = request.params;
      if (!plan) {
        sendError(request.id, -32602, "Missing plan parameter");
        return;
      }
      try {
        const result = await handleRunPlan(plan);
        sendResult(request.id, result);
      } catch (err: any) {
        sendError(request.id, -32603, `Plan execution failed: ${err.message}`);
      }
      break;
    }
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
