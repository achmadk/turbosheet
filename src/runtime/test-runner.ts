import { JsRuntime, IsolatedRuntimePool } from "./js-runtime";
import * as fs from "fs";
import * as path from "path";

export interface TestSuite {
  name: string;
  tests: TestDefinition[];
  hooks: HookDefinition[];
  beforeAllHooks: HookDefinition[];
  afterAllHooks: HookDefinition[];
  beforeEachHooks: HookDefinition[];
  afterEachHooks: HookDefinition[];
  children: TestSuite[];
}

export interface TestDefinition {
  name: string;
  fn: string;
  timeout?: number;
  retries?: number;
}

export interface HookDefinition {
  fn: string;
  timeout?: number;
  type?: "beforeEach" | "afterEach" | "beforeAll" | "afterAll";
}

export interface TestResult {
  name: string;
  status: "passed" | "failed" | "skipped" | "timeout";
  error?: string;
  duration: number;
  logs?: string[];
}

class TimedExecution {
  private startTime: number = 0;
  private timeoutId?: NodeJS.Timeout;

  start(timeoutMs: number, onTimeout: () => void): void {
    this.startTime = Date.now();
    if (timeoutMs > 0) {
      this.timeoutId = setTimeout(onTimeout, timeoutMs);
    }
  }

  stop(): number {
    if (this.timeoutId) {
      clearTimeout(this.timeoutId);
      this.timeoutId = undefined;
    }
    return Date.now() - this.startTime;
  }

  getElapsed(): number {
    return Date.now() - this.startTime;
  }
}

export class TestFileExecutor {
  private page: any;
  private browser: any;
  private defaultTimeout: number;
  private runtimePool: IsolatedRuntimePool;

  constructor(page: any, browser: any, defaultTimeout: number = 30000) {
    this.page = page;
    this.browser = browser;
    this.defaultTimeout = defaultTimeout;
    this.runtimePool = new IsolatedRuntimePool(4, defaultTimeout);
  }

  async injectTestGlobals(runtime: JsRuntime): Promise<void> {
    await runtime.injectGlobals({
      test: this.createTestAPI(),
      expect: this.createExpectAPI(),
      console: {
        log: (...args: any[]) => console.log("[TEST]", ...args),
        error: (...args: any[]) => console.error("[TEST ERROR]", ...args),
        warn: (...args: any[]) => console.warn("[TEST WARN]", ...args),
        info: (...args: any[]) => console.info("[TEST INFO]", ...args),
      },
    });

    if (this.page) {
      await runtime.injectPageBridge(this.page);
    }
  }

  private createTestAPI() {
    const tests: TestDefinition[] = [];
    const hooks: HookDefinition[] = [];
    const suites: TestSuite[] = [];
    let currentSuite: TestSuite | null = null;

    const testAPI = {
      describe: (name: string, fn: () => void) => {
        const suite: TestSuite = {
          name,
          tests: [],
          hooks: [],
          beforeAllHooks: [],
          afterAllHooks: [],
          beforeEachHooks: [],
          afterEachHooks: [],
          children: [],
        };
        const parentSuite = currentSuite;
        currentSuite = suite;
        fn();
        if (parentSuite) {
          parentSuite.children.push(suite);
        } else {
          suites.push(suite);
        }
        currentSuite = parentSuite;
      },

      it: (name: string, fn: () => void | Promise<void>, timeout?: number) => {
        tests.push({ name, fn: fn.toString(), timeout });
        if (currentSuite) {
          currentSuite.tests.push({ name, fn: fn.toString(), timeout });
        }
      },

      test: (name: string, fn: () => void | Promise<void>, timeout?: number) => {
        tests.push({ name, fn: fn.toString(), timeout });
        if (currentSuite) {
          currentSuite.tests.push({ name, fn: fn.toString(), timeout });
        }
      },

      beforeAll: (fn: () => void | Promise<void>, timeout?: number) => {
        hooks.push({ fn: fn.toString(), timeout, type: "beforeAll" });
        if (currentSuite) {
          currentSuite.beforeAllHooks.push({ fn: fn.toString(), timeout });
        }
      },

      afterAll: (fn: () => void | Promise<void>, timeout?: number) => {
        hooks.push({ fn: fn.toString(), timeout, type: "afterAll" });
        if (currentSuite) {
          currentSuite.afterAllHooks.push({ fn: fn.toString(), timeout });
        }
      },

      beforeEach: (fn: () => void | Promise<void>, timeout?: number) => {
        hooks.push({ fn: fn.toString(), timeout, type: "beforeEach" });
        if (currentSuite) {
          currentSuite.beforeEachHooks.push({ fn: fn.toString(), timeout });
        }
      },

      afterEach: (fn: () => void | Promise<void>, timeout?: number) => {
        hooks.push({ fn: fn.toString(), timeout, type: "afterEach" });
        if (currentSuite) {
          currentSuite.afterEachHooks.push({ fn: fn.toString(), timeout });
        }
      },
    };

    return testAPI;
  }

  private createExpectAPI() {
    return {
      expect: (actual: any) => {
        return {
          toBe: (expected: any) => {
            if (actual !== expected) {
              throw new Error(`Expected ${expected} but got ${actual}`);
            }
          },
          toEqual: (expected: any) => {
            if (JSON.stringify(actual) !== JSON.stringify(expected)) {
              throw new Error(
                `Expected ${JSON.stringify(expected)} but got ${JSON.stringify(actual)}`,
              );
            }
          },
          toBeTruthy: () => {
            if (!actual) {
              throw new Error(`Expected truthy but got ${actual}`);
            }
          },
          toBeFalsy: () => {
            if (actual) {
              throw new Error(`Expected falsy but got ${actual}`);
            }
          },
          toContain: (item: any) => {
            if (Array.isArray(actual)) {
              if (!actual.includes(item)) {
                throw new Error(`Expected array to contain ${item}`);
              }
            } else if (typeof actual === "string") {
              if (!actual.includes(item)) {
                throw new Error(`Expected string to contain ${item}`);
              }
            }
          },
          toHaveLength: (length: number) => {
            if (actual.length !== length) {
              throw new Error(`Expected length ${length} but got ${actual.length}`);
            }
          },
          toThrow: () => {
            if (typeof actual !== "function") {
              throw new Error("Expected a function");
            }
            actual();
            throw new Error("Expected function to throw");
          },
        };
      },
    };
  }

  async extractSuiteStructure(filePath: string): Promise<TestSuite[]> {
    const code = fs.readFileSync(filePath, "utf-8");
    const runtime = await this.runtimePool.acquire();

    try {
      await this.injectTestGlobals(runtime);
      const result = await runtime.evaluate(code);

      if (result.error) {
        throw new Error(`Failed to parse test file: ${result.error}`);
      }

      return [];
    } finally {
      await this.runtimePool.release(runtime);
    }
  }

  async executeTestWithTimeout(
    runtime: JsRuntime,
    testCode: string,
    timeoutMs: number,
  ): Promise<{ result: TestResult; timedOut: boolean }> {
    const timer = new TimedExecution();
    let timedOut = false;

    timer.start(timeoutMs, async () => {
      timedOut = true;
      await runtime.terminate(1000);
    });

    try {
      const result = await runtime.evaluateAsync(testCode, timeoutMs);
      const duration = timer.stop();

      if (timedOut) {
        return {
          result: {
            name: "test",
            status: "timeout",
            error: `Test timed out after ${timeoutMs}ms`,
            duration,
          },
          timedOut: true,
        };
      }

      if (result.timedOut) {
        return {
          result: {
            name: "test",
            status: "timeout",
            error: result.error,
            duration,
          },
          timedOut: true,
        };
      }

      if (result.error) {
        return {
          result: {
            name: "test",
            status: "failed",
            error: result.error,
            duration,
          },
          timedOut: false,
        };
      }

      return {
        result: {
          name: "test",
          status: "passed",
          duration,
        },
        timedOut: false,
      };
    } catch (error: any) {
      const duration = timer.stop();
      return {
        result: {
          name: "test",
          status: "failed",
          error: error.message,
          duration,
        },
        timedOut,
      };
    }
  }

  async executeTest(filePath: string): Promise<TestResult[]> {
    const code = fs.readFileSync(filePath, "utf-8");
    const results: TestResult[] = [];
    const runtime = await this.runtimePool.acquire();

    try {
      await this.injectTestGlobals(runtime);

      const wrappedCode = `
        (function() {
          const tests = [];
          const suites = [];
          const test = {
            describe: function(name, fn) {
              const suite = { name: name, tests: [], hooks: [] };
              suites.push(suite);
              fn();
            },
            it: function(name, fn) {
              tests.push({ name: name, fn: fn.toString() });
            },
            test: function(name, fn) {
              tests.push({ name: name, fn: fn.toString() });
            },
            beforeAll: function(fn) {},
            afterAll: function(fn) {},
            beforeEach: function(fn) {},
            afterEach: function(fn) {}
          };
          ${code}
          return { tests: tests, suites: suites };
        })()
      `;

      const parseResult = await runtime.evaluate(wrappedCode);

      if (parseResult.error) {
        results.push({
          name: path.basename(filePath),
          status: "failed",
          error: `Parse error: ${parseResult.error}`,
          duration: 0,
        });
        return results;
      }

      return results;
    } finally {
      await this.runtimePool.release(runtime);
    }
  }

  async executeTestSuite(
    suite: TestSuite,
    runtime: JsRuntime,
    parentHooks: HookDefinition[] = [],
  ): Promise<TestResult[]> {
    const results: TestResult[] = [];

    const allBeforeEachHooks = [...parentHooks, ...suite.beforeEachHooks];
    const allBeforeAllHooks = [...parentHooks, ...suite.beforeAllHooks];

    for (const hook of allBeforeAllHooks) {
      const hookResult = await this.executeHookWithTimeout(runtime, hook, "beforeAll");
      if (!hookResult.success) {
        for (const test of suite.tests) {
          results.push({
            name: test.name,
            status: "skipped",
            error: `Skipped due to beforeAll hook failure: ${hookResult.error}`,
            duration: 0,
          });
        }
        return results;
      }
    }

    for (const test of suite.tests) {
      for (const hook of allBeforeEachHooks) {
        await this.executeHookWithTimeout(runtime, hook, "beforeEach");
      }

      const testTimeout = test.timeout || this.defaultTimeout;
      const { result } = await this.executeTestWithTimeout(runtime, test.fn, testTimeout);
      result.name = test.name;
      results.push(result);

      for (const hook of suite.afterEachHooks) {
        await this.executeHookWithTimeout(runtime, hook, "afterEach");
      }
    }

    for (const hook of suite.afterAllHooks) {
      await this.executeHookWithTimeout(runtime, hook, "afterAll");
    }

    for (const child of suite.children) {
      const childResults = await this.executeTestSuite(child, runtime, allBeforeEachHooks);
      results.push(...childResults);
    }

    return results;
  }

  private async executeHookWithTimeout(
    runtime: JsRuntime,
    hook: HookDefinition,
    // oxlint-disable-next-line no-unused-vars
    hookType: string,
  ): Promise<{ success: boolean; error?: string }> {
    const timeoutMs = hook.timeout || this.defaultTimeout;
    const result = await this.executeTestWithTimeout(runtime, hook.fn, timeoutMs);
    return { success: result.result.status === "passed", error: result.result.error };
  }

  async dispose(): Promise<void> {
    await this.runtimePool.terminateAll();
  }
}

export class TestRunner {
  private suites: Map<string, TestSuite> = new Map();
  private defaultTimeout: number;
  private runtimePool: IsolatedRuntimePool;

  constructor(defaultTimeout: number = 30000) {
    this.defaultTimeout = defaultTimeout;
    this.runtimePool = new IsolatedRuntimePool(4, defaultTimeout);
  }

  async loadTestFile(filePath: string): Promise<TestSuite> {
    const executor = new TestFileExecutor(null, null, this.defaultTimeout);
    const suites = await executor.extractSuiteStructure(filePath);
    this.suites.set(filePath, suites[0]);
    await executor.dispose();
    return suites[0];
  }

  // oxlint-disable-next-line no-unused-vars
  async executeTest(filePath: string, testName?: string): Promise<TestResult[]> {
    const executor = new TestFileExecutor(null, null, this.defaultTimeout);
    const results = await executor.executeTest(filePath);
    await executor.dispose();
    return results;
  }

  async dispose(): Promise<void> {
    await this.runtimePool.terminateAll();
  }
}

export default { TestRunner, TestFileExecutor };
