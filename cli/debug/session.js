const EventEmitter = require("events");

const STEP_STATES = {
  PENDING: "pending",
  RUNNING: "running",
  PASSED: "passed",
  FAILED: "failed",
  SKIPPED: "skipped",
};

class DebugSession extends EventEmitter {
  constructor(options = {}) {
    super();
    this.options = options;
    this.browser = null;
    this.page = null;
    this.context = null;
    this.actions = [];
    this.currentStep = 0;
    this.isPaused = true;
    this.isRunning = false;
    this.isStopped = false;
    this.onPauseCallback = null;
    this.stepStates = new Map();
  }

  async start(url = "https://example.com") {
    const TURBOSHEET = require("../..");

    console.log("Starting interactive debug session...");

    this.browser = await TURBOSHEET.launch({ headless: false });
    this.context = await this.browser.newContext();
    this.page = await this.context.newPage();

    await this.setupHandlers();

    this.isRunning = true;
    this.isPaused = true;

    console.log("Browser launched. Commands:");
    console.log("  next/n     - Run next action");
    console.log("  step/s     - Step into current action");
    console.log("  continue/c - Continue to end");
    console.log("  snapshot/p - Show DOM snapshot");
    console.log("  evaluate/e - Evaluate JavaScript");
    console.log("  goto <url> - Navigate to URL");
    console.log("  screenshot - Take screenshot");
    console.log("  quit/q     - Exit debug session\n");

    if (url) {
      await this.goto(url);
    }

    return this;
  }

  async setupHandlers() {
    this.page.on("console", (msg) => {
      const type = msg.type();
      const text = msg.text();
      if (type === "error") {
        console.log(`  [Console Error] ${text}`);
      } else if (this.isPaused) {
        console.log(`  [${type.toUpperCase()}] ${text}`);
      }
    });

    this.page.on("pageerror", (error) => {
      console.log(`  [Page Error] ${error.message}`);
      this.stepStates.set(this.currentStep, STEP_STATES.FAILED);
    });
  }

  async waitForPause() {
    return new Promise((resolve) => {
      this.onPauseCallback = resolve;
    });
  }

  pause() {
    this.isPaused = true;
    if (this.onPauseCallback) {
      this.onPauseCallback();
      this.onPauseCallback = null;
    }
    this.emit("pause", { step: this.currentStep });
  }

  resume() {
    this.isPaused = false;
    this.emit("resume");
  }

  async runAction(action, index) {
    this.currentStep = index;
    this.stepStates.set(index, STEP_STATES.RUNNING);

    console.log(
      `\n[Step ${index + 1}] ${action.type}: ${action.selector || action.value || action.url || ""}`,
    );

    try {
      const result = await this.executeAction(action);
      this.stepStates.set(index, STEP_STATES.PASSED);
      console.log(`  Result: PASSED`);
      return { success: true, result };
    } catch (error) {
      this.stepStates.set(index, STEP_STATES.FAILED);
      console.log(`  Result: FAILED - ${error.message}`);
      return { success: false, error: error.message };
    }
  }

  async executeAction(action) {
    switch (action.type) {
      case "click":
        return await this.page.locator(action.selector).click();

      case "fill":
        return await this.page.locator(action.selector).fill(action.value);

      case "goto":
        return await this.page.goto(action.url);

      case "check":
        return await this.page.locator(action.selector).check();

      case "uncheck":
        return await this.page.locator(action.selector).uncheck();

      case "select":
        return await this.page.locator(action.selector).selectOption(action.value);

      case "hover":
        return await this.page.locator(action.selector).hover();

      case "dblclick":
        return await this.page.locator(action.selector).dblclick();

      case "press":
        return await this.page.locator(action.selector).press(action.key);

      case "type":
        return await this.page.locator(action.selector).type(action.value);

      default:
        throw new Error(`Unknown action type: ${action.type}`);
    }
  }

  async getSnapshot() {
    console.log("\n--- DOM Snapshot ---");

    const snapshot = await this.page.evaluate(() => {
      const walker = document.createTreeWalker(document.body, NodeFilter.SHOW_ELEMENT, {
        acceptNode: (node) => {
          if (node.children.length === 0 && node.textContent.trim() === "") {
            return NodeFilter.FILTER_REJECT;
          }
          return NodeFilter.FILTER_ACCEPT;
        },
      });

      const nodes = [];
      let count = 0;
      let node;
      while ((node = walker.nextNode()) && count < 50) {
        const tag = node.tagName.toLowerCase();
        const text = node.textContent?.trim().substring(0, 50) || "";
        const attrs = [];
        if (node.id) attrs.push(`#${node.id}`);
        if (node.className && typeof node.className === "string") {
          attrs.push(
            ...node.className
              .split(" ")
              .filter((c) => c)
              .map((c) => `.${c}`)
              .slice(0, 2),
          );
        }
        const role = node.getAttribute("role");
        if (role) attrs.push(`[role=${role}]`);

        nodes.push(`${tag}${attrs.join("")}: "${text}"`);
        count++;
      }
      return nodes.join("\n");
    });

    console.log(snapshot || "(empty)");
    console.log("--- End Snapshot ---\n");

    return snapshot;
  }

  async getAccessibilityTree() {
    const tree = await this.page.evaluate(() => {
      const getAriaLabel = (el) => {
        return (
          el.getAttribute("aria-label") ||
          el.getAttribute("aria-labelledby") ||
          el.getAttribute("alt") ||
          el.textContent?.trim().substring(0, 50)
        );
      };

      const walker = document.createTreeWalker(document.body, NodeFilter.SHOW_ELEMENT, {
        acceptNode: (node) => {
          const role = node.getAttribute("role");
          if (
            role ||
            node.tagName === "BUTTON" ||
            node.tagName === "INPUT" ||
            node.tagName === "A"
          ) {
            return NodeFilter.FILTER_ACCEPT;
          }
          return NodeFilter.FILTER_SKIP;
        },
      });

      const nodes = [];
      let count = 0;
      let node;
      while ((node = walker.nextNode()) && count < 30) {
        const role = node.getAttribute("role") || node.tagName.toLowerCase();
        const label = getAriaLabel(node);
        const state = [];
        if (node.disabled) state.push("disabled");
        if (node.readOnly) state.push("readonly");
        if (node.checked) state.push("checked");

        nodes.push(
          `${role}${label ? ` "${label}"` : ""}${state.length ? ` [${state.join(", ")}]` : ""}`,
        );
        count++;
      }
      return nodes.join("\n");
    });

    console.log("\n--- Accessibility Tree ---");
    console.log(tree || "(empty)");
    console.log("--- End Tree ---\n");

    return tree;
  }

  async evaluate(js) {
    try {
      const result = await this.page.evaluate(js);
      console.log("\n--- Evaluate Result ---");
      console.log(JSON.stringify(result, null, 2));
      console.log("--- End Result ---\n");
      return result;
    } catch (error) {
      console.log(`\nEvaluate Error: ${error.message}\n`);
      throw error;
    }
  }

  async goto(url) {
    console.log(`\nNavigating to: ${url}`);
    await this.page.goto(url);
    console.log(`Current URL: ${this.page.url()}`);
    console.log(`Title: ${await this.page.title()}`);
  }

  async takeScreenshot() {
    const screenshot = await this.page.screenshot();
    const path = `debug-screenshot-${Date.now()}.png`;
    const { writeFileSync } = require("fs");
    writeFileSync(path, screenshot);
    console.log(`\nScreenshot saved: ${path}\n`);
    return path;
  }

  addAction(action) {
    this.actions.push(action);
    this.stepStates.set(this.actions.length - 1, STEP_STATES.PENDING);
  }

  getStatus() {
    return {
      currentStep: this.currentStep,
      totalActions: this.actions.length,
      isPaused: this.isPaused,
      isRunning: this.isRunning,
      states: Object.fromEntries(this.stepStates),
    };
  }

  async stop() {
    this.isStopped = true;
    this.isRunning = false;
    if (this.browser) {
      await this.browser.close();
    }
    console.log("\nDebug session ended.");
  }
}

async function createDebugSession(url) {
  const session = new DebugSession();
  await session.start(url);
  return session;
}

module.exports = { DebugSession, createDebugSession, STEP_STATES };

if (require.main === module) {
  const url = process.argv[2] || "https://example.com";
  const session = new DebugSession();
  session.start(url).catch((e) => {
    console.error("Error:", e.message);
    process.exit(1);
  });
}
