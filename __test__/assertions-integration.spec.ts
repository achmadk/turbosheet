import { describe, it, expect, beforeAll, afterAll, beforeEach, afterEach } from "vite-plus/test";

describe("Locator Visibility Integration Tests", () => {
  let browser: any = null;
  let page: any = null;

  const createTestHtml = (bodyContent: string) =>
    `data:text/html;charset=utf-8,${encodeURIComponent(`<!DOCTYPE html><html><head><title>Test</title></head><body>${bodyContent}</body></html>`)}`;

  beforeAll(async () => {
    try {
      const tsheet = await import("../index.js");
      if (tsheet && typeof tsheet.launch === "function") {
        browser = await tsheet.launch({ headless: true });
        const context = await browser.newContext();
        page = await context.newPage();
      }
    } catch (err) {
      console.warn("Browser launch failed:", err);
    }
  }, 60000);

  afterAll(async () => {
    if (page) {
      await page.close();
    }
    if (browser) {
      await browser.close();
    }
  }, 30000);

  describe("toBeVisible with actual hidden element", () => {
    it("should detect visible element as visible", async () => {
      if (!page) {
        expect(page).not.toBeNull();
        return;
      }
      await page.goto(createTestHtml('<div id="visible">I am visible</div>'));
      const locator = page.locator("#visible");
      const isVisible = await locator.isVisible();
      expect(isVisible).toBe(true);
    });

    it("should detect hidden element as not visible", async () => {
      if (!page) {
        expect(page).not.toBeNull();
        return;
      }
      await page.goto(createTestHtml('<div id="hidden" style="display: none;">I am hidden</div>'));
      const locator = page.locator("#hidden");
      const isVisible = await locator.isVisible();
      expect(isVisible).toBe(false);
    });

    it("should detect element with visibility:hidden as not visible", async () => {
      if (!page) {
        expect(page).not.toBeNull();
        return;
      }
      await page.goto(
        createTestHtml('<div id="hidden-visibility" style="visibility: hidden;">Hidden</div>'),
      );
      const locator = page.locator("#hidden-visibility");
      const isVisible = await locator.isVisible();
      expect(isVisible).toBe(false);
    });

    it("should detect element with opacity:0 as not visible", async () => {
      if (!page) {
        expect(page).not.toBeNull();
        return;
      }
      await page.goto(
        createTestHtml('<div id="transparent" style="opacity: 0;">Transparent</div>'),
      );
      const locator = page.locator("#transparent");
      const isVisible = await locator.isVisible();
      expect(isVisible).toBe(false);
    });
  });

  describe("toHaveURL with actual navigation", () => {
    it("should report correct URL after navigation", async () => {
      if (!page) {
        expect(page).not.toBeNull();
        return;
      }
      const testUrl = createTestHtml("<p>Test page content</p>");
      await page.goto(testUrl);
      const url = page.url();
      expect(url).toContain("data:text/html");
    });

    it("should update URL when navigating", async () => {
      if (!page) {
        expect(page).not.toBeNull();
        return;
      }
      await page.goto(createTestHtml("<p>Page 1</p>"));
      const url1 = page.url();
      await page.goto(createTestHtml("<p>Page 2</p>"));
      const url2 = page.url();
      expect(url1).not.toBe(url2);
    });
  });
});

describe("Fixtures Integration Tests", () => {
  let browser: any = null;
  let page: any = null;
  const createdFixtures: string[] = [];
  const disposedFixtures: string[] = [];

  beforeAll(async () => {
    try {
      const tsheet = await import("../index.js");
      if (tsheet && typeof tsheet.launch === "function") {
        browser = await tsheet.launch({ headless: true });
        const context = await browser.newContext();
        page = await context.newPage();
        createdFixtures.push("browser-context");
      }
    } catch (err) {
      console.warn("Browser launch failed:", err);
    }
  }, 60000);

  afterAll(async () => {
    if (page) {
      disposedFixtures.push("page");
      await page.close();
    }
    if (browser) {
      disposedFixtures.push("browser");
      await browser.close();
    }
  });

  beforeEach(() => {
    createdFixtures.push("beforeEach-fixture");
  });

  afterEach(() => {
    disposedFixtures.push("afterEach-fixture");
  });

  it("should setup browser context before tests", () => {
    expect(createdFixtures).toContain("browser-context");
  });

  it("should run beforeEach before each test", () => {
    expect(createdFixtures).toContain("beforeEach-fixture");
  });

  it("should run afterEach after each test", () => {
    expect(disposedFixtures).toContain("afterEach-fixture");
  });

  it("should have page.close method for cleanup", () => {
    // afterAll runs after ALL tests, so we verify the cleanup mechanism exists
    // rather than checking afterAll's (future) side effects
    if (page) {
      expect(typeof page.close).toBe("function");
    } else {
      // If page wasn't created, at minimum validate the fixture pattern
      expect(typeof (page !== null ? page.close : null)).toBe("function");
    }
  });

  it("should have browser.close method for cleanup", () => {
    if (browser) {
      expect(typeof browser.close).toBe("function");
    } else {
      expect(typeof (browser !== null ? browser.close : null)).toBe("function");
    }
  });
});

describe("Hooks Integration Tests", () => {
  let browser: any = null;
  let page: any = null;
  const executionOrder: string[] = [];

  beforeAll(async () => {
    executionOrder.push("beforeAll-1-start");
    try {
      const tsheet = await import("../index.js");
      if (tsheet && typeof tsheet.launch === "function") {
        browser = await tsheet.launch({ headless: true });
        const context = await browser.newContext();
        page = await context.newPage();
      }
    } catch (err) {
      console.warn("Browser launch failed:", err);
    }
    executionOrder.push("beforeAll-1-end");
  }, 60000);

  beforeEach(() => {
    executionOrder.push("beforeEach");
  });

  afterEach(() => {
    executionOrder.push("afterEach");
  });

  afterAll(async () => {
    executionOrder.push("afterAll-start");
    if (page) {
      await page.close();
    }
    if (browser) {
      await browser.close();
    }
    executionOrder.push("afterAll-end");
  }, 30000);

  it("should execute hooks in correct order - test 1", () => {
    executionOrder.push("test-1");
    expect(executionOrder.length).toBeGreaterThan(0);
  });

  it("should execute hooks in correct order - test 2", () => {
    executionOrder.push("test-2");
    expect(executionOrder[0]).toBe("beforeAll-1-start");
    expect(executionOrder[executionOrder.length - 1]).toBe("test-2");
  });

  it("should have beforeAll run before tests", () => {
    executionOrder.push("test-3");
    const beforeAllIndex = executionOrder.indexOf("beforeAll-1-start");
    const testIndex = executionOrder.indexOf("test-3");
    expect(beforeAllIndex).toBeLessThan(testIndex);
  });
});
