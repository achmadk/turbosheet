import ivm from "isolated-vm";

export interface JsRuntimeOptions {
  timeout?: number;
  memoryLimit?: number;
}

export interface PageBridge {
  goto(url: string): Promise<void>;
  url(): Promise<string>;
  click(selector: string): Promise<void>;
  fill(selector: string, value: string): Promise<void>;
  locator(selector: string): LocatorBridge;
  evaluate(js: string): Promise<string>;
  content(): Promise<string>;
  title(): Promise<string>;
}

export interface LocatorBridge {
  click(): Promise<void>;
  fill(value: string): Promise<void>;
  textContent(): Promise<string | null>;
  isVisible(): Promise<boolean>;
}

export interface EvalResult {
  value: any;
  error?: string;
  timedOut?: boolean;
}

export class TimeoutError extends Error {
  constructor(
    message: string,
    public readonly timeoutMs: number,
  ) {
    super(message);
    this.name = "TimeoutError";
  }
}

export class JsRuntime {
  private isolate: ivm.Isolate;
  private context: ivm.Context;
  private global: ivm.Reference<any>;
  private disposed: boolean = false;
  private defaultTimeout: number;
  private initPromise: Promise<void>;

  constructor(options: JsRuntimeOptions = {}) {
    const { timeout = 30000, memoryLimit = 128 } = options;

    this.defaultTimeout = timeout;
    this.isolate = new ivm.Isolate({ memoryLimit });
    this.context = this.isolate.createContextSync();
    this.global = this.context.global;
    this.initPromise = Promise.resolve();
  }

  async waitForInit(): Promise<void> {
    return this.initPromise;
  }

  async injectGlobals(globals: Record<string, any>): Promise<void> {
    for (const [key, value] of Object.entries(globals)) {
      await this.global.set(key, this.wrapValue(value));
    }
  }

  private wrapValue(value: any): any {
    if (typeof value === "undefined") {
      return undefined;
    }
    if (value === null) {
      return null;
    }
    if (typeof value === "boolean") {
      return value;
    }
    if (typeof value === "number") {
      return value;
    }
    if (typeof value === "string") {
      return value;
    }
    if (typeof value === "function") {
      return this.wrapFunction(value);
    }
    if (typeof value === "object") {
      return this.wrapObject(value);
    }
    throw new Error(`Cannot wrap value of type ${typeof value}`);
  }

  private wrapFunction(fn: Function): ivm.Callback {
    return new ivm.Callback(fn as (...args: any[]) => any, { async: true });
  }

  private wrapObject(obj: Record<string, any>): ivm.Reference<any> {
    const result = new ivm.Reference<any>({});
    for (const [key, value] of Object.entries(obj)) {
      result.setSync(key, this.wrapValue(value));
    }
    return result;
  }

  async evaluate(code: string, timeout?: number): Promise<EvalResult> {
    if (this.disposed) {
      return { value: undefined, error: "Isolate already disposed" };
    }

    const evalTimeout = timeout || this.defaultTimeout;

    try {
      const result = await this.context.eval(code, {
        timeout: evalTimeout,
      });
      return { value: result };
    } catch (error: any) {
      if (error.message && error.message.includes("Script execution timed out")) {
        return {
          value: undefined,
          error: `Script execution timed out after ${evalTimeout}ms`,
          timedOut: true,
        };
      }
      return { value: undefined, error: error.message };
    }
  }

  async evaluateAsync(code: string, timeout?: number): Promise<EvalResult> {
    if (this.disposed) {
      return { value: undefined, error: "Isolate already disposed" };
    }

    const evalTimeout = timeout || this.defaultTimeout;

    try {
      const result = await this.context.eval(code, {
        timeout: evalTimeout,
      });
      return { value: result };
    } catch (error: any) {
      if (error.message && error.message.includes("Script execution timed out")) {
        return {
          value: undefined,
          error: `Script execution timed out after ${evalTimeout}ms`,
          timedOut: true,
        };
      }
      return { value: undefined, error: error.message };
    }
  }

  async terminate(timeout: number = 1000): Promise<void> {
    return new Promise((resolve) => {
      if (this.disposed) {
        resolve();
        return;
      }

      const terminateTimeout = setTimeout(() => {
        this.isolate.dispose();
        this.disposed = true;
        resolve();
      }, timeout);

      try {
        this.isolate.dispose();
        clearTimeout(terminateTimeout);
        this.disposed = true;
      } catch {
        clearTimeout(terminateTimeout);
      }

      resolve();
    });
  }

  async dispose(): Promise<void> {
    if (!this.disposed) {
      this.isolate.dispose();
      this.disposed = true;
    }
  }

  isDisposed(): boolean {
    return this.disposed;
  }

  getDefaultTimeout(): number {
    return this.defaultTimeout;
  }

  async createPageBridge(page: any): Promise<ivm.Reference<any>> {
    const pageObj = new ivm.Reference<any>({});

    const gotoCallback = new ivm.Callback(
      async (url: string) => {
        return await page.goto(url);
      },
      { async: true },
    );

    const urlCallback = new ivm.Callback(
      async () => {
        return page.url();
      },
      { async: true },
    );

    const clickCallback = new ivm.Callback(
      async (selector: string) => {
        const locator = page.locator(selector);
        return await locator.click();
      },
      { async: true },
    );

    const fillCallback = new ivm.Callback(
      async (selector: string, value: string) => {
        const locator = page.locator(selector);
        return await locator.fill(value);
      },
      { async: true },
    );

    const locatorCallback = new ivm.Callback(
      async (selector: string) => {
        return await this.createLocatorBridge(page, selector);
      },
      { async: true },
    );

    const evaluateCallback = new ivm.Callback(
      async (js: string) => {
        return await page.evaluate(js);
      },
      { async: true },
    );

    const contentCallback = new ivm.Callback(
      async () => {
        return await page.content();
      },
      { async: true },
    );

    const titleCallback = new ivm.Callback(
      async () => {
        return await page.title();
      },
      { async: true },
    );

    await pageObj.set("goto", gotoCallback);
    await pageObj.set("url", urlCallback);
    await pageObj.set("click", clickCallback);
    await pageObj.set("fill", fillCallback);
    await pageObj.set("locator", locatorCallback);
    await pageObj.set("evaluate", evaluateCallback);
    await pageObj.set("content", contentCallback);
    await pageObj.set("title", titleCallback);

    return pageObj;
  }

  async createLocatorBridge(page: any, selector: string): Promise<ivm.Reference<any>> {
    const locatorObj = new ivm.Reference<any>({});

    const locator = page.locator(selector);

    const clickCallback = new ivm.Callback(
      async () => {
        return await locator.click();
      },
      { async: true },
    );

    const fillCallback = new ivm.Callback(
      async (value: string) => {
        return await locator.fill(value);
      },
      { async: true },
    );

    const textContentCallback = new ivm.Callback(
      async () => {
        return await locator.textContent();
      },
      { async: true },
    );

    const isVisibleCallback = new ivm.Callback(
      async () => {
        return await locator.isVisible();
      },
      { async: true },
    );

    await locatorObj.set("click", clickCallback);
    await locatorObj.set("fill", fillCallback);
    await locatorObj.set("textContent", textContentCallback);
    await locatorObj.set("isVisible", isVisibleCallback);

    return locatorObj;
  }

  async injectPageBridge(page: any): Promise<void> {
    const pageBridge = await this.createPageBridge(page);
    await this.global.set("page", pageBridge);
  }
}

export class IsolatedRuntimePool {
  private pool: JsRuntime[] = [];
  private maxSize: number;
  private defaultTimeout: number;

  constructor(maxSize: number = 4, defaultTimeout: number = 30000) {
    this.maxSize = maxSize;
    this.defaultTimeout = defaultTimeout;
  }

  async acquire(): Promise<JsRuntime> {
    if (this.pool.length > 0) {
      return this.pool.pop()!;
    }
    return new JsRuntime({ timeout: this.defaultTimeout });
  }

  async release(runtime: JsRuntime): Promise<void> {
    if (this.pool.length < this.maxSize && !runtime.isDisposed()) {
      await runtime.dispose();
      this.pool.push(runtime);
    } else {
      await runtime.dispose();
    }
  }

  async terminateAll(): Promise<void> {
    for (const runtime of this.pool) {
      await runtime.dispose();
    }
    this.pool = [];
  }
}

export class TestSuiteExtractor {
  async extractSuiteStructure(code: string): Promise<{
    describes: DescribeBlock[];
    tests: TestCase[];
    hooks: Hook[];
  }> {
    const describes: DescribeBlock[] = [];
    const tests: TestCase[] = [];
    const hooks: Hook[] = [];

    const extractScript = `
      (function() {
        const describes = [];
        const tests = [];
        const hooks = [];
        const test = { describe: function(name, fn) { describes.push({ name: name, children: [] }); fn(); } };
        const testCase = { it: function(name, fn) { tests.push({ name: name }); } };
        const hook = { beforeAll: function(fn) { hooks.push({ type: 'beforeAll', fn: fn.toString() }); } };
        eval(${JSON.stringify(code)});
        return { describes: describes, tests: tests, hooks: hooks };
      })()
    `;

    try {
      const isolate = new ivm.Isolate();
      const context = await isolate.createContext();
      const global = context.global;
      await global.set("_describes", new ivm.Reference([]));
      await global.set("_tests", new ivm.Reference([]));
      await global.set("_hooks", new ivm.Reference([]));

      const result = await context.eval(extractScript, { timeout: 5000 });
      isolate.dispose();
      return result;
    } catch {
      return { describes, tests, hooks };
    }
  }
}

export interface DescribeBlock {
  name: string;
  children: DescribeBlock[];
}

export interface TestCase {
  name: string;
  fn?: Function;
}

export interface Hook {
  type: "beforeAll" | "afterAll" | "beforeEach" | "afterEach";
  fn?: Function;
}

export default { JsRuntime, TestSuiteExtractor };
