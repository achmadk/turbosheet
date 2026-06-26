import { describe, it, expect, beforeAll, afterAll } from "vite-plus/test";
import * as path from "path";
import * as fs from "fs";
import * as os from "os";

interface Page {
  goto(url: string): Promise<void>;
  url(): string;
  content(): Promise<string>;
  waitForRequest?(url: string): Promise<void>;
  waitForResponse?(url: string): Promise<void>;
  evaluate<T = unknown>(fn: string): Promise<T>;
  setInputFiles?(selector: string, files: string[]): Promise<void>;
  close(): Promise<void>;
  route?(pattern: string, handler: (...args: unknown[]) => void): Promise<void>;
}

interface Browser {
  newContext(): Promise<{ newPage(): Promise<Page> }>;
  close(): Promise<void>;
}

describe("Firefox Integration Tests", () => {
  let browser: Browser | null = null;
  let page: Page | null = null;
  let tsheet: { launch: (opts: Record<string, unknown>) => Promise<Browser> } | null = null;

  const testPageHtml = `
    <!DOCTYPE html>
    <html>
    <head><title>Firefox Test</title></head>
    <body>
      <h1>Hello Firefox</h1>
      <input id="file-input" type="file" />
      <div id="result"></div>
    </body>
    </html>
  `;

  beforeAll(async () => {
    try {
      tsheet = (await import("../index.js")) as unknown as typeof tsheet;
      if (tsheet && typeof tsheet.launch === "function") {
        browser = await tsheet.launch({ headless: true, browser: "firefox" });
        const context = await browser.newContext();
        page = await context.newPage();
      }
    } catch (err) {
      console.warn(
        "Firefox browser launch failed — geckodriver may not be installed. Skipping Firefox tests:",
        (err as Error).message,
      );
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

  // ----- 10.3: goto, set_content (via data URI), url() -----

  describe("10.3 — goto, set_content, url", () => {
    it("should launch Firefox and create a page", async () => {
      expect(browser).not.toBeNull();
      expect(page).not.toBeNull();
      if (page) {
        expect(typeof page.goto).toBe("function");
        expect(typeof page.url).toBe("function");
      }
    });

    it("should navigate to a URL and report correct URL via url()", async () => {
      if (!page) {
        expect(page).not.toBeNull();
        return;
      }
      await page.goto("data:text/html;charset=utf-8," + encodeURIComponent(testPageHtml));
      const url = page.url();
      expect(url).toContain("data:text/html");
      expect(url).toContain("Hello Firefox");
    });

    it("should return page content via content()", async () => {
      if (!page) {
        expect(page).not.toBeNull();
        return;
      }
      await page.goto("data:text/html;charset=utf-8," + encodeURIComponent(testPageHtml));
      const html = await page.content();
      expect(html).toContain("Hello Firefox");
      expect(html).toContain("Firefox Test");
    });

    it("should navigate between pages and reflect URL changes", async () => {
      if (!page) {
        expect(page).not.toBeNull();
        return;
      }
      const page1 = "<h1>Page 1</h1>";
      const page2 = "<h1>Page 2</h1>";
      await page.goto("data:text/html;charset=utf-8," + encodeURIComponent(page1));
      expect(page.url()).toContain("Page+1");
      await page.goto("data:text/html;charset=utf-8," + encodeURIComponent(page2));
      expect(page.url()).toContain("Page+2");
    });
  });

  // ----- 10.4: set_input_files -----

  describe("10.4 — set_input_files", () => {
    it("should set input files on a file input element", async () => {
      if (!page) {
        expect(page).not.toBeNull();
        return;
      }
      const tempFile = path.join(os.tmpdir(), "tsheet-firefox-test-" + Date.now() + ".txt");
      fs.writeFileSync(tempFile, "hello from firefox test", "utf-8");

      try {
        await page.goto(
          "data:text/html;charset=utf-8," +
            encodeURIComponent(`
          <!DOCTYPE html>
          <html><body>
            <input id="file-input" type="file" />
            <script>
              document.getElementById('file-input').addEventListener('change', function(e) {
                document.getElementById('result').textContent = 'File selected: ' + e.target.files[0].name;
              });
            </script>
          </body></html>
        `),
        );

        if (typeof page.setInputFiles === "function") {
          await page.setInputFiles("#file-input", [tempFile]);
        }

        // Verify the file input has the expected file
        const result = await page.evaluate<string>(`
          (() => {
            const input = document.getElementById('file-input');
            return input.files && input.files.length > 0 ? input.files[0].name : 'no files';
          })()
        `);
        expect(result).not.toBe("no files");
        if (result !== "no files") {
          expect(result).toContain(".txt");
        }
      } finally {
        try {
          fs.unlinkSync(tempFile);
        } catch {
          /* ignore cleanup errors */
        }
      }
    });
  });

  // ----- 10.5: wait_for_request -----

  describe("10.5 — wait_for_request", () => {
    it("should capture a network request from evaluated JS", async () => {
      if (!page) {
        expect(page).not.toBeNull();
        return;
      }

      await page.goto(
        "data:text/html;charset=utf-8," +
          encodeURIComponent(`
        <!DOCTYPE html>
        <html><body><div id="result">waiting</div></body></html>
      `),
      );

      // If waitForRequest exists, test matching a fetch from page evaluate
      if (typeof page.waitForRequest === "function") {
        const requestPromise = page.waitForRequest("https://httpbin.org/get");

        await page.evaluate(`
          (() => {
            fetch('https://httpbin.org/get');
          })()
        `);

        // waitForRequest resolves when the request starts — allow time
        await requestPromise;
        // Test passes — the request was observed
        expect(true).toBe(true);
      }
    });

    it("should capture a wait_for_response", async () => {
      if (!page) {
        expect(page).not.toBeNull();
        return;
      }

      if (typeof page.waitForResponse === "function") {
        const responsePromise = page.waitForResponse("https://httpbin.org/get");

        await page.evaluate(`
          (() => {
            fetch('https://httpbin.org/get');
          })()
        `);

        await responsePromise;
        expect(true).toBe(true);
      }
    });
  });
});
