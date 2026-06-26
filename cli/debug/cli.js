const { DebugSession } = require("./session");
const { SelectorGenerator } = require("../codegen/selectors");

const COMMANDS = {
  next: "n",
  n: "next",
  step: "s",
  s: "step",
  continue: "c",
  c: "continue",
  snapshot: "p",
  p: "snapshot",
  "snapshot:full": "snapshot:full",
  acc: "acc",
  accessibility: "acc",
  eval: "eval",
  e: "eval",
  goto: "goto",
  screenshot: "screenshot",
  ss: "screenshot",
  quit: "quit",
  q: "quit",
  exit: "quit",
  help: "help",
  h: "help",
  status: "status",
};

class DebugCLI {
  constructor(session) {
    this.session = session;
    this.selectorGenerator = new SelectorGenerator();
    this.rl = null;
  }

  async start() {
    const readline = require("readline");
    this.rl = readline.createInterface({
      input: process.stdin,
      output: process.stdout,
      prompt: "tsheet-debug> ",
    });

    this.rl.prompt();

    this.rl.on("line", async (line) => {
      const cmd = line.trim();
      if (!cmd) {
        this.rl.prompt();
        return;
      }

      try {
        await this.handleCommand(cmd);
      } catch (e) {
        console.log(`Error: ${e.message}`);
      }

      this.rl.prompt();
    });

    this.rl.on("close", () => {
      this.session.stop();
      process.exit(0);
    });
  }

  async handleCommand(input) {
    const [cmd, ...args] = input.split(/\s+/);

    switch (COMMANDS[cmd]) {
      case "next":
        return await this.handleNext();

      case "step":
        return await this.handleStep();

      case "continue":
        return await this.handleContinue();

      case "snapshot":
        return await this.handleSnapshot(args);

      case "acc":
        return await this.handleAccessibility();

      case "eval":
        return await this.handleEval(args.join(" "));

      case "goto":
        return await this.handleGoto(args.join(" "));

      case "screenshot":
        return await this.handleScreenshot();

      case "quit":
        return await this.handleQuit();

      case "help":
        return this.printHelp();

      case "status":
        return this.printStatus();

      default:
        console.log(`Unknown command: ${cmd}`);
        this.printHelp();
    }
  }

  async handleNext() {
    const status = this.session.getStatus();

    if (status.currentStep >= status.totalActions) {
      console.log("No more actions to run.");
      return;
    }

    const action = this.session.actions[status.currentStep];
    await this.session.runAction(action, status.currentStep);
  }

  async handleStep() {
    await this.handleNext();
  }

  async handleContinue() {
    const status = this.session.getStatus();

    while (status.currentStep < status.totalActions && !this.session.isStopped) {
      const action = this.session.actions[status.currentStep];
      await this.session.runAction(action, status.currentStep);
    }

    if (!this.session.isStopped) {
      console.log("\nAll actions completed.");
    }
  }

  async handleSnapshot(args) {
    const full = args.includes("--full") || args.includes("-f");
    if (full) {
      console.log("\nFull DOM snapshot (use DOM evaluate):");
      const content = await this.session.page.content();
      console.log(content.substring(0, 2000) + "...\n");
    } else {
      await this.session.getSnapshot();
    }
  }

  async handleAccessibility() {
    await this.session.getAccessibilityTree();
  }

  async handleEval(code) {
    if (!code) {
      console.log("Usage: eval <javascript-expression>");
      return;
    }
    await this.session.evaluate(code);
  }

  async handleGoto(url) {
    if (!url) {
      console.log("Usage: goto <url>");
      return;
    }
    await this.session.goto(url);
  }

  async handleScreenshot() {
    const path = await this.session.takeScreenshot();
    console.log(`Screenshot saved to: ${path}`);
  }

  async handleQuit() {
    await this.session.stop();
    this.rl.close();
    process.exit(0);
  }

  printHelp() {
    console.log(`
Debug Commands:
  next, n       - Run next action and pause
  step, s       - Same as next
  continue, c    - Run all remaining actions
  snapshot, p    - Show DOM snapshot at current step
  acc            - Show accessibility tree
  eval, e <js>   - Evaluate JavaScript
  goto <url>     - Navigate to URL
  screenshot, ss - Take screenshot
  status         - Show current status
  quit, q        - Exit debug session
  help, h        - Show this help
`);
  }

  printStatus() {
    const status = this.session.getStatus();
    console.log(`
Status:
  Current Step: ${status.currentStep + 1}/${status.totalActions}
  Paused: ${status.isPaused}
  Running: ${status.isRunning}
  Stopped: ${this.session.isStopped}

Action States:`);
    for (let i = 0; i < this.session.actions.length; i++) {
      const action = this.session.actions[i];
      const state = status.states[i] || "pending";
      const marker = i === status.currentStep ? ">" : " ";
      console.log(
        `  ${marker} [${state}] ${action.type}: ${action.selector || action.value || action.url || ""}`,
      );
    }
    console.log();
  }
}

async function runDebug(args) {
  let url = "https://example.com";

  for (let i = 0; i < args.length; i++) {
    if (args[i] === "--url" && i + 1 < args.length) {
      url = args[++i];
    }
  }

  console.log(`Starting debug session at: ${url}\n`);

  const session = new DebugSession();
  await session.start(url);

  const cli = new DebugCLI(session);
  await cli.start();
}

module.exports = { DebugCLI, runDebug };

if (require.main === module) {
  runDebug(process.argv.slice(2)).catch((e) => {
    console.error("Error:", e.message);
    process.exit(1);
  });
}
