import { describe, it, expect, beforeAll, afterAll } from "vite-plus/test";

describe("JsRuntime Page Bridge Integration", () => {
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

  describe("Page Bridge Methods", () => {
    it("should have goto method", () => {
      expect(page).not.toBeNull();
      if (page) {
        expect(typeof page.goto).toBe("function");
      }
    });

    it("should have url method", () => {
      expect(page).not.toBeNull();
      if (page) {
        expect(typeof page.url).toBe("function");
      }
    });

    it("should have locator method", () => {
      expect(page).not.toBeNull();
      if (page) {
        expect(typeof page.locator).toBe("function");
      }
    });

    it("should have evaluate method", () => {
      expect(page).not.toBeNull();
      if (page) {
        expect(typeof page.evaluate).toBe("function");
      }
    });

    it("should have content method", () => {
      expect(page).not.toBeNull();
      if (page) {
        expect(typeof page.content).toBe("function");
      }
    });

    it("should have title method", () => {
      expect(page).not.toBeNull();
      if (page) {
        expect(typeof page.title).toBe("function");
      }
    });
  });

  describe("Locator Bridge Methods", () => {
    beforeAll(async () => {
      if (page) {
        await page.goto(createTestHtml('<div id="test">Hello</div>'));
      }
    }, 30000);

    it("should have click method on locator", () => {
      if (!page) return;
      const locator = page.locator("#test");
      expect(locator).not.toBeNull();
      expect(typeof locator.click).toBe("function");
    });

    it("should have fill method on locator", () => {
      if (!page) return;
      const locator = page.locator("#test");
      expect(typeof locator.fill).toBe("function");
    });

    it("should have textContent method on locator", () => {
      if (!page) return;
      const locator = page.locator("#test");
      expect(typeof locator.textContent).toBe("function");
    });

    it("should have isVisible method on locator", () => {
      if (!page) return;
      const locator = page.locator("#test");
      expect(typeof locator.isVisible).toBe("function");
    });
  });

  describe("End-to-End Flow", () => {
    it("should navigate and check visibility", async () => {
      if (!page) {
        expect(page).not.toBeNull();
        return;
      }
      await page.goto(createTestHtml('<div id="visible">Visible Text</div>'));
      const locator = page.locator("#visible");
      const isVisible = await locator.isVisible();
      expect(isVisible).toBe(true);
    }, 30000);

    it("should detect hidden element", async () => {
      if (!page) {
        expect(page).not.toBeNull();
        return;
      }
      await page.goto(createTestHtml('<div id="hidden" style="display:none">Hidden</div>'));
      const locator = page.locator("#hidden");
      const isVisible = await locator.isVisible();
      expect(isVisible).toBe(false);
    }, 30000);

    it("should get text content", async () => {
      if (!page) {
        expect(page).not.toBeNull();
        return;
      }
      await page.goto(createTestHtml('<span id="text">Expected Text</span>'));
      const locator = page.locator("#text");
      const text = await locator.textContent();
      expect(text).toBe("Expected Text");
    }, 30000);

    it("should navigate and verify URL", async () => {
      if (!page) {
        expect(page).not.toBeNull();
        return;
      }
      const testUrl = createTestHtml("<p>Content</p>");
      await page.goto(testUrl);
      const url = page.url();
      expect(url).toContain("data:text/html");
    }, 30000);

    it("should verify page title", async () => {
      if (!page) {
        expect(page).not.toBeNull();
        return;
      }
      const titleUrl = `data:text/html;charset=utf-8,${encodeURIComponent("<!DOCTYPE html><html><head><title>My Title</title></head><body><p>Content</p></body></html>")}`;
      await page.goto(titleUrl);
      const title = await page.title();
      expect(title).toBe("My Title");
    }, 30000);

    it("should execute JavaScript on page", async () => {
      if (!page) {
        expect(page).not.toBeNull();
        return;
      }
      await page.goto(createTestHtml("<p>Hello</p>"));
      const result = await page.evaluate('document.querySelector("p").textContent');
      expect(result).toBe("Hello");
    }, 30000);

    it("should get page content", async () => {
      if (!page) {
        expect(page).not.toBeNull();
        return;
      }
      await page.goto(createTestHtml('<div class="test">Content</div>'));
      const content = await page.content();
      expect(content).toContain("test");
    }, 30000);
  });

  describe("Error Handling", () => {
    it("should handle non-existent element gracefully", async () => {
      if (!page) {
        expect(page).not.toBeNull();
        return;
      }
      await page.goto(createTestHtml("<div>Existing</div>"));
      const locator = page.locator("#non-existent");
      const text = await locator.textContent();
      expect(text).toBeNull();
    }, 30000);

    it("should handle invalid selector gracefully", async () => {
      if (!page) {
        expect(page).not.toBeNull();
        return;
      }
      await page.goto(createTestHtml("<div>Content</div>"));
      const locator = page.locator("invalid[selector");
      const isVisible = await locator.isVisible();
      expect(isVisible).toBe(false);
    }, 30000);
  });
});

describe("Test Runner Integration", () => {
  let browser: any = null;

  beforeAll(async () => {
    try {
      const tsheet = await import("../index.js");
      if (tsheet && typeof tsheet.launch === "function") {
        browser = await tsheet.launch({ headless: true });
      }
    } catch (err) {
      console.warn("Browser launch failed:", err);
    }
  }, 60000);

  afterAll(async () => {
    if (browser) {
      await browser.close();
    }
  }, 30000);

  describe("Multiple Pages", () => {
    it("should create multiple contexts", async () => {
      if (!browser) {
        expect(browser).not.toBeNull();
        return;
      }
      const context1 = await browser.newContext();
      const context2 = await browser.newContext();
      expect(context1).not.toBe(context2);
      await context1.close();
      await context2.close();
    }, 30000);

    it("should create multiple pages per context", async () => {
      if (!browser) {
        expect(browser).not.toBeNull();
        return;
      }
      const context = await browser.newContext();
      const page1 = await context.newPage();
      const page2 = await context.newPage();
      expect(page1).not.toBe(page2);
      await page1.close();
      await page2.close();
      await context.close();
    }, 30000);
  });
});
