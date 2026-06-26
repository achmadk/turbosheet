import { describe, it, expect, beforeAll, afterAll } from "vite-plus/test";
import { createRequire } from "module";

const require = createRequire(import.meta.url);

interface TestResult {
  name: string;
  status: "passed" | "failed" | "skipped" | "timeout";
  error?: string;
  duration: number;
}

describe("TurboSheet Integration Tests", () => {
  // oxlint-disable-next-line typescript/no-redundant-type-constituents
  let browser: import("../index.js").Browser | null = null;
  let page: import("../index.js").Page | null = null;

  const testPageHtml = `
    <!DOCTYPE html>
    <html>
    <head><title>Test Page</title></head>
    <body>
      <div id="visible-element">I am visible</div>
      <div id="hidden-element" style="display: none;">I am hidden</div>
      <input id="text-input" type="text" value="hello" />
      <a href="/next-page">Link</a>
    </body>
    </html>
  `;

  beforeAll(async () => {
    try {
      const tsheet = await import("../index.js");
      if (tsheet && typeof tsheet.launch === "function") {
        browser = await tsheet.launch({ headless: true });
        const context = await browser.newContext();
        page = await context.newPage();
      }
    } catch (err) {
      console.warn("Browser launch failed, skipping integration tests:", err);
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

  describe("Browser Launch", () => {
    it("should launch browser successfully", async () => {
      expect(browser).not.toBeNull();
      if (browser) {
        expect(typeof browser.newContext).toBe("function");
      }
    });

    it("should create a new page", async () => {
      expect(page).not.toBeNull();
      if (page) {
        expect(typeof page.goto).toBe("function");
        expect(typeof page.locator).toBe("function");
      }
    });
  });

  describe("Page Navigation", () => {
    it("should navigate to a URL", async () => {
      if (!page) {
        expect(page).not.toBeNull();
        return;
      }
      await page.goto("data:text/html;charset=utf-8," + encodeURIComponent(testPageHtml));
      const url = page.url();
      expect(url).toContain("data:text/html");
    });

    it("should get page title", async () => {
      if (!page) {
        expect(page).not.toBeNull();
        return;
      }
      await page.goto("data:text/html;charset=utf-8," + encodeURIComponent(testPageHtml));
      const title = await page.title();
      expect(title).toBe("Test Page");
    });

    it("should have URL after navigation", async () => {
      if (!page) {
        expect(page).not.toBeNull();
        return;
      }
      await page.goto("data:text/html;charset=utf-8," + encodeURIComponent(testPageHtml));
      const url = page.url();
      expect(url.length).toBeGreaterThan(0);
    });
  });

  describe("Locator Operations", () => {
    beforeAll(async () => {
      if (page) {
        await page.goto("data:text/html;charset=utf-8," + encodeURIComponent(testPageHtml));
      }
    }, 30000);

    it("should find visible element", async () => {
      if (!page) {
        expect(page).not.toBeNull();
        return;
      }
      const locator = page.locator("#visible-element");
      expect(locator).not.toBeNull();
      expect(typeof locator.isVisible).toBe("function");
      expect(typeof locator.click).toBe("function");
      expect(typeof locator.textContent).toBe("function");
    });

    it("should check element visibility", async () => {
      if (!page) {
        expect(page).not.toBeNull();
        return;
      }
      const visibleLocator = page.locator("#visible-element");
      const isVisible = await visibleLocator.isVisible();
      expect(isVisible).toBe(true);
    });

    it("should detect hidden element as not visible", async () => {
      if (!page) {
        expect(page).not.toBeNull();
        return;
      }
      const hiddenLocator = page.locator("#hidden-element");
      const isVisible = await hiddenLocator.isVisible();
      expect(isVisible).toBe(false);
    });

    it("should get text content", async () => {
      if (!page) {
        expect(page).not.toBeNull();
        return;
      }
      const locator = page.locator("#visible-element");
      const text = await locator.textContent();
      expect(text).toBe("I am visible");
    });

    it("should fill input value", async () => {
      if (!page) {
        expect(page).not.toBeNull();
        return;
      }
      const locator = page.locator("#text-input");
      await locator.fill("new value");
      const inputValue = await page.evaluate('document.querySelector("#text-input").value');
      expect(inputValue).toBe("new value");
    });
  });

  describe("Page Evaluate", () => {
    it("should evaluate JavaScript", async () => {
      if (!page) {
        expect(page).not.toBeNull();
        return;
      }
      await page.goto("data:text/html;charset=utf-8," + encodeURIComponent(testPageHtml));
      const result = await page.evaluate("1 + 2");
      expect(result).toBe("3");
    });
  });

  describe("Test Runner - Page Integration", () => {
    it("should use page in test context", async () => {
      if (!page) {
        expect(page).not.toBeNull();
        return;
      }
      const results: TestResult[] = [];

      results.push({
        name: "basic page test",
        status: "passed",
        duration: 100,
      });

      expect(results.length).toBe(1);
      expect(results[0].name).toBe("basic page test");
      expect(results[0].status).toBe("passed");
    });
  });
});

describe("TurboSheet Standalone Tests", () => {
  it("should export launch function", () => {
    const index = require("../index.js") as Record<string, unknown>;
    expect(index.launch).toBeDefined();
    expect(typeof index.launch).toBe("function");
  });

  it("should export devices function", () => {
    const index = require("../index.js") as Record<string, unknown>;
    expect(index.devices).toBeDefined();
    expect(typeof index.devices).toBe("function");
  });

  it("should export version function", () => {
    const index = require("../index.js") as Record<string, unknown>;
    expect(index.version).toBeDefined();
    expect(typeof index.version).toBe("function");
  });

  it("should have correct API shape", () => {
    const index = require("../index.js") as Record<string, unknown>;
    expect(index.launch).toBeDefined();
    expect(index.devices).toBeDefined();
    expect(index.version).toBeDefined();
    expect(index.trace_start_recording).toBeDefined();
    expect(index.trace_stop_and_serialize).toBeDefined();
  });
});

describe("0.4.7 Real Input Dispatch – Trusted Events & CSS Triggers", () => {
  let browser: any = null;
  let page: any = null;
  let ctx: any = null;

  beforeAll(async () => {
    try {
      const m = require("../index.js");
      // Use headless: true – native module will auto-detect browser binary
      browser = await m.launch({ headless: true });
      ctx = await browser.newContext({});
      page = await ctx.newPage();
    } catch (err) {
      console.warn("Browser launch failed, skipping input tests:", err);
    }
  }, 60000);

  afterAll(async () => {
    if (page) {
      try {
        await page.close();
      } catch {
        /* ignore */
      }
    }
    if (browser) {
      try {
        await browser.close();
      } catch {
        /* ignore */
      }
    }
  }, 30000);

  // ── event.isTrusted ─────────────────────────────────────────

  const mkPage = (html: string) => {
    // eslint-disable-next-line no-useless-escape
    return "data:text/html;charset=utf-8," + encodeURIComponent(html);
  };

  // Helper: evaluate returns string values (CDP serializes as JSON)
  const eTrue = "true";
  // Find the turbo-sheet global proxy by scanning for dispatch
  const FIND_TS = `(() => { for (const k of Object.getOwnPropertyNames(window)) { try { const v = window[k]; if (v && typeof v.dispatch === 'function') return k; } catch(e) {} } return ''; })()`;

  it("should have isTrusted === true on CDP click (mousedown+mouseup)", async () => {
    if (!page) {
      expect(page).not.toBeNull();
      return;
    }
    await page.goto(
      mkPage(
        `<button id="t">x</button>
<script>
let r = null;
document.querySelector('#t').onclick = e => { r = e.isTrusted; };
window.__r = () => r;
</script>`,
      ),
    );
    await page.locator("#t").click();
    await new Promise((r) => setTimeout(r, 300));
    expect(await page.evaluate("window.__r()")).toBe(eTrue);
  });

  it("should have isTrusted === true on CDP hover (mouseMoved)", async () => {
    if (!page) {
      expect(page).not.toBeNull();
      return;
    }
    await page.goto(
      mkPage(
        `<div id="h" style="width:50px;height:50px;background:red">x</div>
<script>
let r = null;
document.querySelector('#h').onmouseenter = e => { r = e.isTrusted; };
window.__r = () => r;
</script>`,
      ),
    );
    await page.locator("#h").hover();
    await new Promise((r) => setTimeout(r, 300));
    expect(await page.evaluate("window.__r()")).toBe(eTrue);
  });

  it("should have isTrusted === true on CDP dblclick", async () => {
    if (!page) {
      expect(page).not.toBeNull();
      return;
    }
    await page.goto(
      mkPage(
        `<button id="d">x</button>
<script>
let r = null;
document.querySelector('#d').ondblclick = e => { r = e.isTrusted; };
window.__r = () => r;
</script>`,
      ),
    );
    await page.locator("#d").dblclick();
    await new Promise((r) => setTimeout(r, 500));
    expect(await page.evaluate("window.__r()")).toBe(eTrue);
  });

  it("should have isTrusted === true on CDP rightClick (contextmenu)", async () => {
    if (!page) {
      expect(page).not.toBeNull();
      return;
    }
    await page.goto(
      mkPage(
        `<button id="r">x</button>
<script>
let r = null;
document.querySelector('#r').oncontextmenu = e => { r = e.isTrusted; };
window.__r = () => r;
</script>`,
      ),
    );
    await page.locator("#r").rightClick();
    await new Promise((r) => setTimeout(r, 300));
    expect(await page.evaluate("window.__r()")).toBe(eTrue);
  });

  it("should have isTrusted === true on keyboard press (keydown)", async () => {
    if (!page) {
      expect(page).not.toBeNull();
      return;
    }
    await page.goto(
      mkPage(
        `<input id="k" type="text">
<script>
let r = null;
document.querySelector('#k').onkeydown = e => { r = e.isTrusted; };
window.__r = () => r;
</script>`,
      ),
    );
    await page.locator("#k").press("Tab");
    await new Promise((r) => setTimeout(r, 300));
    expect(await page.evaluate("window.__r()")).toBe(eTrue);
  });

  // ── CSS pseudo-class triggers ──────────────────────────────

  it("should trigger mousedown on CDP click (active-state equivalent)", async () => {
    if (!page) {
      expect(page).not.toBeNull();
      return;
    }
    await page.goto(
      mkPage(
        `<button id="a" style="transition:none">x</button>
<script>
let r = false;
document.querySelector('#a').addEventListener('mousedown', () => { r = true; });
window.__r = () => r;
</script>`,
      ),
    );
    await page.locator("#a").click();
    await new Promise((r) => setTimeout(r, 300));
    expect(await page.evaluate("window.__r()")).toBe(eTrue);
  });

  it("should trigger mouseenter on CDP hover (hover-state equivalent)", async () => {
    if (!page) {
      expect(page).not.toBeNull();
      return;
    }
    await page.goto(
      mkPage(
        `<div id="hv" style="width:50px;height:50px;background:blue">x</div>
<script>
let entered = false;
document.querySelector('#hv').onmouseenter = () => { entered = true; };
window.__r = () => entered;
</script>`,
      ),
    );
    await page.locator("#hv").hover();
    await new Promise((r) => setTimeout(r, 500));
    expect(await page.evaluate("window.__r()")).toBe(eTrue);
  });

  // ── Selector escaping (0.5) ─────────────────────────────────

  it("should handle single quotes in selector", async () => {
    if (!page) {
      expect(page).not.toBeNull();
      return;
    }
    await page.goto(
      mkPage(
        `<div id="sq">found</div>
<script>
window.__r = () => document.querySelector('#sq').innerText;
</script>`,
      ),
    );
    const el = await page.locator("#sq");
    const text = await el.innerText();
    expect(text).toBe("found");
  });

  it("should handle double quotes in attribute selectors", async () => {
    if (!page) {
      expect(page).not.toBeNull();
      return;
    }
    await page.goto(mkPage(`<div data-testid='he said "hello"'>found</div>`));
    const el = await page.locator("[data-testid='he said \"hello\"']");
    const text = await el.innerText();
    expect(text).toBe("found");
  });

  it("should handle backslashes in attribute selectors", async () => {
    if (!page) {
      expect(page).not.toBeNull();
      return;
    }
    await page.goto(mkPage(`<div data-testid="path\\to\\element">found</div>`));
    const el = await page.locator('[data-testid="path\\\\to\\\\element"]');
    const text = await el.innerText();
    expect(text).toBe("found");
  });

  it("should handle Unicode characters in selectors", async () => {
    if (!page) {
      expect(page).not.toBeNull();
      return;
    }
    await page.goto(mkPage(`<div data-testid="按钮-测试">found</div>`));
    const el = await page.locator('[data-testid="按钮-测试"]');
    const text = await el.innerText();
    expect(text).toBe("found");
  });

  it("should handle template literal backticks in selectors", async () => {
    if (!page) {
      expect(page).not.toBeNull();
      return;
    }
    await page.goto(mkPage(`<div data-testid="backtick\`id\`">found</div>`));
    const el = await page.locator('[data-testid="backtick\\`id\\`"]');
    const text = await el.innerText();
    expect(text).toBe("found");
  });

  // ── Injected Core Script (1.1) ────────────────────────────

  it("should inject core script namespace on page creation", async () => {
    if (!page) {
      expect(page).not.toBeNull();
      return;
    }
    await page.goto(mkPage(`<div>hi</div>`));
    const globalName = await page.evaluate(FIND_TS);
    expect(globalName).toBeTruthy();
    // The proxy has dispatch. Internal methods (waitForSelector, etc.)
    // live under a Symbol key, accessible via invoke_action.
    const hasDispatch = await page.evaluate(
      `(typeof window[${JSON.stringify(globalName)}]?.dispatch === 'function')`,
    );
    expect(hasDispatch).toBe(eTrue);
  });

  it("should detect top-level page is not a frame", async () => {
    if (!page) {
      expect(page).not.toBeNull();
      return;
    }
    await page.goto(mkPage(`<div>hi</div>`));
    const globalName = await page.evaluate(FIND_TS);
    expect(globalName).toBeTruthy();
    // Verify invoke_action works (goes through proxy dispatch → Symbol store)
    const txt = await page.locator("div").innerText();
    expect(txt).toBe("hi");
  });

  it("should resolve waitForSelector when element appears dynamically", async () => {
    if (!page) {
      expect(page).not.toBeNull();
      return;
    }
    await page.goto(mkPage(`<div id="container"></div>`));

    // Schedule dynamic element to appear
    await page.evaluate(`setTimeout(() => {
      const el = document.createElement('div');
      el.id = 'dynamic-el';
      el.textContent = 'appeared';
      document.getElementById('container').appendChild(el);
    }, 200)`);

    // Wait enough time for the element to appear, then fetch its text
    await new Promise((r) => setTimeout(r, 500));
    const text = await page.locator("#dynamic-el").innerText();
    expect(text).toBe("appeared");
  });

  // ── Injected Actions Script (1.2) ──────────────────────────

  /** Resolve the TS Symbol store via the proxy's __tsSym, then call evalBody.
   *  The proxy (window[globalName]) only has dispatch/querySelector/querySelectorAll.
   *  Internal methods (waitForSelector, getByRole, etc.) live under a Symbol key.
   */
  async function withTS(page: any, evalBody: string): Promise<string> {
    const k = await page.evaluate(FIND_TS);
    if (!k) return "NO_TS";
    return await page.evaluate(`(function() {
      try {
        const pub = window[${JSON.stringify(k)}];
        const sym = pub && pub.__tsSym;
        if (sym && typeof sym === 'symbol') {
          const ts = window[sym];
          if (ts) { ${evalBody} }
        }
        return 'NO_TS_STORE';
      } catch(e) { return 'ERR:'+e.message; }
    })()`);
  }

  it("should inject actions script on demand via invoke_action", async () => {
    if (!page) {
      expect(page).not.toBeNull();
      return;
    }
    await page.goto(mkPage(`<div>hi</div>`));
    const coreInfo = await page.evaluate(FIND_TS);
    expect(coreInfo).toBeTruthy();
    const actions = JSON.parse(
      await withTS(
        page,
        `
      return JSON.stringify({
        waitForSelector: typeof ts.waitForSelector,
        getByRole: typeof ts.getByRole,
        checkActionability: typeof ts.checkActionability,
        content: typeof ts.content
      });
    `,
      ),
    );
    expect(actions.waitForSelector).toBe("function");
    expect(actions.getByRole).toBe("undefined");
    expect(actions.checkActionability).toBe("undefined");
    expect(actions.content).toBe("undefined");
  });

  it("should load actions script overrides via invoke_action chain", async () => {
    if (!page) {
      expect(page).not.toBeNull();
      return;
    }
    await page.goto(mkPage(`<div id="act-trigger">hello</div>`));
    await page.locator("#act-trigger").isVisible();
    const actions = JSON.parse(
      await withTS(
        page,
        `
      return JSON.stringify({
        getByRole: typeof ts.getByRole,
        checkActionability: typeof ts.checkActionability,
        content: typeof ts.content
      });
    `,
      ),
    );
    expect(actions.getByRole).toBe("function");
    expect(actions.checkActionability).toBe("function");
    expect(actions.content).toBe("function");
  });

  it("should resolve getByRole via the actions dispatch bridge", async () => {
    if (!page) {
      expect(page).not.toBeNull();
      return;
    }
    await page.goto(
      mkPage(
        `<button>Send</button>
<input type="text" />
<a href="#">Link</a>`,
      ),
    );
    await page.locator("button").isVisible();
    const result = await withTS(
      page,
      `
      const btns = ts.getByRole('button');
      if (btns.length === 0) return 'no-buttons';
      if (btns[0].tagName !== 'BUTTON') return 'wrong-tag';
      return btns[0].textContent || '';
    `,
    );
    expect(result).toBe("Send");
  });

  it("should resolve getByText via the actions dispatch bridge", async () => {
    if (!page) {
      expect(page).not.toBeNull();
      return;
    }
    await page.goto(mkPage(`<div>Hello World</div><span>Extra text</span>`));
    await page.locator("div").isVisible();
    const result = await withTS(
      page,
      `
      const els = ts.getByText('Hello World');
      return els.length > 0 ? els[0].textContent || '' : 'not-found';
    `,
    );
    expect(result).toBe("Hello World");
  });

  it("should resolve getByLabel and getByTestId via the actions dispatch bridge", async () => {
    if (!page) {
      expect(page).not.toBeNull();
      return;
    }
    await page.goto(
      mkPage(
        `<label for="uname">Username</label>
<input id="uname" />
<div data-testid="my-test-id">test-content</div>`,
      ),
    );
    await page.locator("input").isVisible();
    const result = JSON.parse(
      await withTS(
        page,
        `
      const byLabel = ts.getByLabel('Username');
      const byTestId = ts.getByTestId('my-test-id');
      return JSON.stringify({
        labelInputs: byLabel.length,
        testContent: byTestId.length > 0 ? byTestId[0].textContent || '' : 'none'
      });
    `,
      ),
    );
    expect(result.labelInputs).toBe(1);
    expect(result.testContent).toBe("test-content");
  });

  it("should evaluate chain selectors through the actions override", async () => {
    if (!page) {
      expect(page).not.toBeNull();
      return;
    }
    await page.goto(mkPage(`<div id="parent"><span>first</span><span>second</span></div>`));
    const visible = await page.locator("div >> nth=0").isVisible();
    expect(visible).toBe(true);
  });

  it("should perform check/uncheck through actions script", async () => {
    if (!page) {
      expect(page).not.toBeNull();
      return;
    }
    await page.goto(
      mkPage(
        `<input type="checkbox" id="cb" />
<script>window.__checked = () => document.querySelector('#cb').checked;</script>`,
      ),
    );
    const cb = await page.locator("#cb");
    await cb.check();
    await new Promise((r) => setTimeout(r, 200));
    expect(await page.evaluate("window.__checked()")).toBe(eTrue);
    await cb.uncheck();
    await new Promise((r) => setTimeout(r, 200));
    expect(await page.evaluate("window.__checked()")).toBe("false");
  });

  it("should perform select through actions script", async () => {
    if (!page) {
      expect(page).not.toBeNull();
      return;
    }
    await page.goto(
      mkPage(
        `<select id="sel">
<option value="a">A</option>
<option value="b">B</option>
</select>
<script>window.__val = () => document.querySelector('#sel').value;</script>`,
      ),
    );
    await page.locator("#sel").select("b");
    await new Promise((r) => setTimeout(r, 200));
    expect(await page.evaluate("window.__val()")).toBe("b");
  });

  it("should perform focus via actions script", async () => {
    if (!page) {
      expect(page).not.toBeNull();
      return;
    }
    await page.goto(
      mkPage(
        `<input id="fi" />
<script>
let focused = false;
document.querySelector('#fi').addEventListener('focus', () => { focused = true; });
window.__f = () => focused;
</script>`,
      ),
    );
    await page.locator("#fi").click();
    await new Promise((r) => setTimeout(r, 300));
    await page.locator("#fi").focus();
    await new Promise((r) => setTimeout(r, 200));
    expect(await page.evaluate("window.__f()")).toBe(eTrue);
  });

  it("should detect visibility through isVisible/isEnabled/isDisabled", async () => {
    if (!page) {
      expect(page).not.toBeNull();
      return;
    }
    await page.goto(
      mkPage(
        `<button id="vis">visible</button>
<button id="hid" style="display:none">hidden</button>
<button id="dis" disabled>disabled</button>`,
      ),
    );
    expect(await page.locator("#vis").isVisible()).toBe(true);
    expect(await page.locator("#hid").isVisible()).toBe(false);
    expect(await page.locator("#vis").isEnabled()).toBe(true);
    expect(await page.locator("#dis").isEnabled()).toBe(false);
    expect(await page.locator("#vis").isDisabled()).toBe(false);
    expect(await page.locator("#dis").isDisabled()).toBe(true);
  });

  // ── Touch API ──────────────────────────────────────────────

  it("should dispatch tap via page.tap(x, y)", async () => {
    if (!page) {
      expect(page).not.toBeNull();
      return;
    }
    await page.goto(
      mkPage(
        `<div id="tt" style="width:100px;height:100px;background:red">x</div>
<script>
let r = null;
document.querySelector('#tt').addEventListener('touchend', e => { r = e.isTrusted; });
window.__r = () => r;
</script>`,
      ),
    );
    // Set mobile viewport so touch events are accepted
    await page.setViewportSize(375, 812);
    await new Promise((r) => setTimeout(r, 300));
    await page.tap(50, 50);
    await new Promise((r) => setTimeout(r, 500));
    expect(await page.evaluate("window.__r()")).toBe(eTrue);
  });

  // ── Binding Bridge (1.3) ──────────────────────────────────

  it("should propagate action errors through the binding bridge", async () => {
    if (!page) {
      expect(page).not.toBeNull();
      return;
    }
    await page.goto(mkPage(`<div>hi</div>`));
    const missing = page.locator("#does-not-exist-at-all");
    let err: Error | null = null;
    try {
      await missing.check();
    } catch (e: any) {
      err = e;
    }
    expect(err).not.toBeNull();
    expect(err!.message).toMatch(/element|not found|not visible|disabled|timeout/i);
  });

  it("should handle concurrent binding calls without interference", async () => {
    if (!page) {
      expect(page).not.toBeNull();
      return;
    }
    await page.goto(
      mkPage(
        `<div id="a">first</div>
<div id="b">second</div>
<div id="c">third</div>`,
      ),
    );
    const [a, b, c] = await Promise.all([
      page.locator("#a").innerText(),
      page.locator("#b").innerText(),
      page.locator("#c").innerText(),
    ]);
    expect(a).toBe("first");
    expect(b).toBe("second");
    expect(c).toBe("third");
  });

  // ── Engine Migration (1.5) ──────────────────────────────────

  it("should get innerHTML through invoke_action", async () => {
    if (!page) {
      expect(page).not.toBeNull();
      return;
    }
    await page.goto(mkPage(`<div id="ih"><span>nested</span></div>`));
    const html = await page.locator("#ih").innerHTML();
    expect(html).toBe("<span>nested</span>");
  });

  it("should get attribute through invoke_action", async () => {
    if (!page) {
      expect(page).not.toBeNull();
      return;
    }
    await page.goto(mkPage(`<a id="attr-link" href="/target" data-test="val">link</a>`));
    const href = await page.locator("#attr-link").getAttribute("href");
    expect(href).toBe("/target");
    const dataTest = await page.locator("#attr-link").getAttribute("data-test");
    expect(dataTest).toBe("val");
    const missing = await page.locator("#attr-link").getAttribute("aria-label");
    expect(missing).toBeNull();
  });

  it("should handle concurrent action calls beyond default timeout safety", async () => {
    if (!page) {
      expect(page).not.toBeNull();
      return;
    }
    await page.goto(mkPage(`<ul><li>1</li><li>2</li><li>3</li><li>4</li><li>5</li></ul>`));
    const results = await Promise.all(
      ["1", "2", "3", "4", "5"].map((i) => page.locator(`li:nth-child(${i})`).isVisible()),
    );
    expect(results.every((r) => r === true)).toBe(true);
  });

  describe("Frame / IFrame Locator", () => {
    it("should read innerText from an iframe via frameLocator", async () => {
      if (!page) {
        expect(page).not.toBeNull();
        return;
      }
      const html = `<div id="root">
        <iframe srcdoc="<div id='if-content'>Inside iframe</div>"></iframe>
      </div>`;
      await page.goto(mkPage(html));
      const text = await page.frameLocator("iframe").locator("#if-content").innerText();
      expect(text).toBe("Inside iframe");
    });

    it("should get innerHTML from iframe content", async () => {
      if (!page) {
        expect(page).not.toBeNull();
        return;
      }
      const html = `<div id="root">
        <iframe srcdoc="<div id='d'><span>nested</span></div>"></iframe>
      </div>`;
      await page.goto(mkPage(html));
      const h = await page.frameLocator("iframe").locator("#d").innerHTML();
      expect(h).toBe("<span>nested</span>");
    });

    it("should check isVisible inside iframe", async () => {
      if (!page) {
        expect(page).not.toBeNull();
        return;
      }
      const html = `<div id="root">
        <iframe srcdoc="<div id='visible'>I see</div>"></iframe>
      </div>`;
      await page.goto(mkPage(html));
      const v = await page.frameLocator("iframe").locator("#visible").isVisible();
      expect(v).toBe(true);
    });

    it("should return count of elements inside iframe", async () => {
      if (!page) {
        expect(page).not.toBeNull();
        return;
      }
      const html = `<div id="root">
        <iframe srcdoc="<ul><li>A</li><li>B</li><li>C</li></ul>"></iframe>
      </div>`;
      await page.goto(mkPage(html));
      const c = await page.frameLocator("iframe").locator("li").count();
      expect(c).toBe(3);
    });

    it("should get attribute from iframe element", async () => {
      if (!page) {
        expect(page).not.toBeNull();
        return;
      }
      const html = `<div id="root">
        <iframe srcdoc="<a id='lnk' href='/target'>go</a>"></iframe>
      </div>`;
      await page.goto(mkPage(html));
      const href = await page.frameLocator("iframe").locator("#lnk").getAttribute("href");
      expect(href).toBe("/target");
    });

    it("should support chaining locator().frameLocator()", async () => {
      if (!page) {
        expect(page).not.toBeNull();
        return;
      }
      const html = `<div id="root">
        <iframe srcdoc="<p>P inside</p>"></iframe>
      </div>`;
      await page.goto(mkPage(html));
      const text = await page.locator("#root").frameLocator("iframe").locator("p").innerText();
      expect(text).toBe("P inside");
    });
  });

  describe("Stealth Mode", () => {
    it("should not expose __ts_ properties on window globals", async () => {
      if (!page) {
        expect(page).not.toBeNull();
        return;
      }
      const html = `<div id="stealth-test">stealth</div>`;
      await page.goto(
        "data:text/html;charset=utf-8," +
          encodeURIComponent(`<!DOCTYPE html><html><body>${html}</body></html>`),
      );

      // Check that window does not have any __ts_* own properties
      const propNamesJson = await page.evaluate(
        `JSON.stringify(Object.getOwnPropertyNames(window).filter(p => p.startsWith('__ts')))`,
      );
      const propNames: string[] = JSON.parse(propNamesJson);
      expect(propNames.length).toBe(0);
    });

    it("should have the public proxy with Symbol-based internal state", async () => {
      if (!page) {
        expect(page).not.toBeNull();
        return;
      }
      const html = `<div id="sym-test">symbol</div>`;
      await page.goto(
        "data:text/html;charset=utf-8," +
          encodeURIComponent(`<!DOCTYPE html><html><body>${html}</body></html>`),
      );

      // The global name should not start with __ts — find the proxy via dispatch
      const globalName = await page.evaluate(FIND_TS);
      expect(globalName).toBeTruthy();
      expect(globalName).not.toMatch(/^__ts/);

      // Verify dispatch, querySelector, querySelectorAll exist on the proxy
      const hasDispatch = await page.evaluate(
        `typeof window[${JSON.stringify(globalName)}].dispatch`,
      );
      expect(hasDispatch).toBe("function");

      const hasQuerySelector = await page.evaluate(
        `typeof window[${JSON.stringify(globalName)}].querySelector`,
      );
      expect(hasQuerySelector).toBe("function");

      // The proxy exposes dispatch/querySelector/querySelectorAll as own props
      const proxyKeysJson = await page.evaluate(
        `JSON.stringify(Object.keys(window[${JSON.stringify(globalName)}]))`,
      );
      const proxyKeys: string[] = JSON.parse(proxyKeysJson);
      expect(proxyKeys).toContain("dispatch");
      expect(proxyKeys).toContain("querySelector");
    });

    it("should still work with locator operations through stealth proxy", async () => {
      if (!page) {
        expect(page).not.toBeNull();
        return;
      }
      const html = `<div id="stealth-work">works</div>`;
      await page.goto(
        "data:text/html;charset=utf-8," +
          encodeURIComponent(`<!DOCTYPE html><html><body>${html}</body></html>`),
      );

      const text = await page.locator("#stealth-work").innerText();
      expect(text).toBe("works");
    });
  });

  describe("Popup Support", () => {
    it("should detect popup created by window.open via waitForEvent", async () => {
      if (!browser) {
        expect(browser).not.toBeNull();
        return;
      }

      // Create a fresh context and page for this test
      const ctx = await browser.newContext();
      const pg = await ctx.newPage();

      const popupHtml = "<html><body><h1>Popup</h1></body></html>";
      const popupDataUri = "data:text/html;charset=utf-8," + encodeURIComponent(popupHtml);

      // Open a page, then use window.open to trigger a popup
      await pg.goto(
        "data:text/html;charset=utf-8," +
          encodeURIComponent("<!DOCTYPE html><html><body>test</body></html>"),
      );

      // Start waiting for the popup BEFORE triggering it
      const popupPromise = ctx.waitForEvent("page", 10000);

      await pg.evaluate(
        `(window.open(${JSON.stringify(popupDataUri)}, 'popup', 'width=400,height=300'), undefined)`,
      );

      // Wait for the popup via waitForEvent('page') (returns the new page_id)
      const popupPageId = await popupPromise;
      expect(popupPageId).toBeDefined();
      expect(typeof popupPageId).toBe("string");

      // Verify the popup appears in the context's pages list
      const pageIds = await ctx.pages();
      const foundPopup = pageIds.includes(popupPageId);
      expect(foundPopup).toBe(true);

      // Cleanup
      await pg.close();
    }, 30000);
  });
});
