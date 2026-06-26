#!/usr/bin/env node

const EventEmitter = require("events");
const { SelectorGenerator } = require("./selectors");
const { TestGenerator } = require("./assertions");

class RecordingBrowser extends EventEmitter {
  constructor(options = {}) {
    super();
    this.options = options;
    this.selectorGenerator = new SelectorGenerator();
    this.isRecording = false;
    this.page = null;
    this.actions = [];
    this.navigations = [];
    this.assertions = [];
    this.startTime = null;
  }

  async start() {
    const TURBOSHEET = require("..");
    console.log("Launching browser for recording...");

    this.browser = await TURBOSHEET.launch({ headless: false });
    this.context = await this.browser.newContext();
    this.page = await this.context.newPage();

    await this.setupRecording();
    this.isRecording = true;
    this.startTime = Date.now();

    console.log("Recording started. Navigate to a page and interact with it.");
    console.log("Press Ctrl+C to stop recording.\n");

    return this;
  }

  async setupRecording() {
    this.page.on("console", (msg) => {
      if (msg.type() === "error") {
        console.log(`  [Console Error] ${msg.text()}`);
      }
    });

    await this.page.exposeFunction("__recordClick", async (data) => {
      const selector = this.selectorGenerator.suggestSelector(data.element);
      const locator = this.selectorGenerator.generateLocator(selector, this.page);

      const action = {
        type: "click",
        selector: selector.selector,
        selectorType: selector.type,
        locator,
        timestamp: Date.now(),
        url: this.page.url(),
      };

      this.actions.push(action);
      console.log(`  [Click] ${selector.type}: ${selector.selector}`);
    });

    await this.page.exposeFunction("__recordFill", async (data) => {
      const selector = this.selectorGenerator.suggestSelector(data.element);
      const locator = this.selectorGenerator.generateLocator(selector, this.page);

      const action = {
        type: "fill",
        selector: selector.selector,
        selectorType: selector.type,
        locator,
        value: data.value,
        timestamp: Date.now(),
        url: this.page.url(),
      };

      this.actions.push(action);
      console.log(`  [Fill] ${selector.type}: ${selector.selector} = "${data.value}"`);
    });

    await this.page.exposeFunction("__recordNavigation", async (data) => {
      const nav = {
        type: "navigation",
        url: data.url,
        timestamp: Date.now(),
      };
      this.navigations.push(nav);
      console.log(`  [Navigate] ${data.url}`);
    });

    await this.page.addInitScript(() => {
      document.addEventListener(
        "click",
        async (e) => {
          e.preventDefault();
          e.stopPropagation();

          const target = e.target;
          const elementData = extractElementData(target);

          if (target.tagName === "INPUT" || target.tagName === "TEXTAREA") {
            const originalValue = target.value;
            target.addEventListener(
              "input",
              () => {
                window.__recordFill({
                  element: elementData,
                  value: target.value,
                  previousValue: originalValue,
                });
              },
              { once: true },
            );
          }

          window.__recordClick({ element: elementData });
        },
        true,
      );

      function extractElementData(el) {
        const data = {
          tagName: el.tagName,
          id: el.id,
          className: el.className,
          name: el.getAttribute("name"),
          placeholder: el.getAttribute("placeholder"),
          "data-testid": el.getAttribute("data-testid"),
          "aria-label": el.getAttribute("aria-label"),
          "aria-role": el.getAttribute("role"),
          textContent: el.textContent?.trim().substring(0, 100),
          testIdAttribute: "data-testid",
        };
        return data;
      }

      const observer = new MutationObserver((mutations) => {
        for (const mutation of mutations) {
          if (mutation.type === "childList" && mutation.addedNodes.length > 0) {
            for (const node of mutation.addedNodes) {
              if (node.nodeType === Node.ELEMENT_NODE) {
                window.__recordNavigation({ url: window.location.href });
                break;
              }
            }
          }
        }
      });

      observer.observe(document.body, { childList: true, subtree: true });
    });
  }

  async stop() {
    if (!this.isRecording) return null;

    this.isRecording = false;
    console.log("\nStopping recording...");

    const duration = Date.now() - this.startTime;
    const result = {
      actions: this.actions,
      navigations: this.navigations,
      assertions: this.assertions,
      duration,
      actionCount: this.actions.length,
      navCount: this.navigations.length,
    };

    if (this.browser) {
      await this.browser.close();
    }

    return result;
  }

  generateCode() {
    const generator = new TestGenerator();

    const recording = {
      actions: this.actions,
      navigations: this.navigations,
      assertions: this.assertions,
    };

    return generator.generateFromRecording(recording);
  }
}

async function runRecording(args) {
  let url = "https://example.com";
  let output = "recorded-test.tsheet.ts";

  for (let i = 0; i < args.length; i++) {
    if (args[i] === "--url" && i + 1 < args.length) url = args[++i];
    else if (args[i] === "--output" && i + 1 < args.length) output = args[++i];
  }

  const recorder = new RecordingBrowser({ url, output });

  const cleanup = async () => {
    console.log("\nGenerating test file...");
    const code = recorder.generateCode();

    const { writeFileSync } = require("fs");
    writeFileSync(output, code, "utf-8");

    console.log(`\nTest saved to: ${output}`);
    console.log(`Actions recorded: ${recorder.actions.length}`);
    console.log(`Navigations recorded: ${recorder.navigations.length}`);
    process.exit(0);
  };

  process.on("SIGINT", cleanup);
  process.on("SIGTERM", cleanup);

  await recorder.start();

  await recorder.page.goto(url);

  await new Promise(() => {});
}

module.exports = { RecordingBrowser, runRecording };

if (require.main === module) {
  runRecording(process.argv.slice(2)).catch((e) => {
    console.error("Error:", e.message);
    process.exit(1);
  });
}
