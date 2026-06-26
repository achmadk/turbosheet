#!/usr/bin/env node

const { existsSync } = require("fs");
const { join } = require("path");

const TURBOSHEET = require("..");

const args = process.argv.slice(2);
const command = args[0];

const COMMANDS = {
  test: runTest,
  install: runInstall,
  devices: runDevices,
  "show-trace": require("./show-trace").showTrace,
  "export-trace": require("./export-trace").exportTrace,
  codegen: runCodegen,
  debug: runDebug,
  migrate: runMigrate,
  init: runInit,
  help: showHelp,
};

async function main() {
  if (!command) {
    showHelp();
    return;
  }

  const handler = COMMANDS[command];
  if (!handler) {
    console.error(`Unknown command: ${command}`);
    console.error(`Run 'tsheet help' for usage information`);
    process.exit(1);
    return;
  }

  try {
    await handler(args.slice(1));
  } catch (e) {
    console.error(`Error: ${e.message}`);
    process.exit(1);
  }
}

function showHelp() {
  console.log(`
TurboSheet CLI - High-performance browser automation

Usage: tsheet <command> [options]

Commands:
  test [files...]        Run tests (default: all .tsheet.ts files)
  install [browser]     Install browser (chromium, firefox, webkit)
  devices                List available device presets
  show-trace <file>     Open trace file in viewer
  codegen [--url <url>]  Record test actions to file
  debug [--url <url>]    Interactive debug session
  migrate <from>        Migrate tests from (playwright|cypress|puppeteer)
  init [--ci <type>]    Initialize project (github|gitlab|jenkins)
  help                  Show this help

Test Options:
  --reporter <type>     Reporter: dot, line, list, json, html, github, junit
  --workers <n>         Number of parallel workers (default: 1)
  --retries <n>         Retry failed tests n times
  --timeout <ms>        Test timeout in milliseconds
  --grep <pattern>      Only run tests matching pattern
  --shard <n>/<m>      Run shard n of m (for CI)
  --headed              Run tests in headed mode (visible browser window)
  --browser <type>      Browser engine: chromium, firefox, webkit (default: chromium)
  --output <dir>        Output directory for test results, reports, and screenshots
  --update-snapshots    Update expected snapshot files
  --project <name>      Run tests matching the specified project

Examples:
  tsheet test
  tsheet test --reporter html --workers 4
  tsheet install chromium
  tsheet codegen --url https://example.com
  tsheet init --ci github
`);
}

async function runTest(args) {
  const cliConfig = parseTestArgs(args);

  let config = {};
  const configPath = cliConfig.config || "turbosheet.config.ts";
  if (existsSync(configPath)) {
    try {
      config = TURBOSHEET.load_config(configPath);
    } catch (e) {
      console.error(`Failed to load config from ${configPath}:`, e.message);
    }
  }

  // Merge CLI args over config file
  const finalConfig = {
    ...config,
    ...cliConfig,
  };

  console.log("Starting TurboSheet test runner...");
  console.log(`Workers: ${finalConfig.workers || 1}`);
  console.log(`Reporter: ${finalConfig.reporter || "list"}`);

  const testFiles = args.filter(
    (a) => !a.startsWith("--") && !args[args.indexOf(a) - 1]?.startsWith("--"),
  );

  try {
    const { run_tests } = TURBOSHEET;
    const result = await run_tests({
      test_dir: finalConfig.testDir || finalConfig.test_dir || ".",
      test_match: testFiles.length > 0 ? testFiles : finalConfig.test_match,
      workers: finalConfig.workers,
      retries: finalConfig.retries,
      timeout: finalConfig.timeout,
      reporter: finalConfig.reporter,
      grep: finalConfig.grep,
      shard: finalConfig.shard,
      headless: finalConfig.headless,
      browser: finalConfig.browser,
      output_dir: finalConfig.output_dir,
      update_snapshots: finalConfig.update_snapshots,
      project: finalConfig.project,
    });

    console.log(`\nTests completed: ${result.length} total`);
    const passed = result.filter((r) => r.status === "passed").length;
    const failed = result.filter((r) => r.status === "failed").length;
    console.log(`Passed: ${passed}, Failed: ${failed}`);

    if (failed > 0) {
      process.exit(1);
    }
  } catch (e) {
    console.error("Test runner error:", e.message);
    process.exit(1);
  }
}

function parseTestArgs(args) {
  const config = {};
  for (let i = 0; i < args.length; i++) {
    const arg = args[i];
    if (arg === "--reporter" && i + 1 < args.length) config.reporter = args[++i];
    else if (arg === "--workers" && i + 1 < args.length) config.workers = parseInt(args[++i], 10);
    else if (arg === "--retries" && i + 1 < args.length) config.retries = parseInt(args[++i], 10);
    else if (arg === "--timeout" && i + 1 < args.length) config.timeout = parseInt(args[++i], 10);
    else if (arg === "--grep" && i + 1 < args.length) config.grep = args[++i];
    else if (arg === "--config" && i + 1 < args.length) config.config = args[++i];
    else if (arg === "--headed") config.headless = false;
    else if (arg === "--browser" && i + 1 < args.length) config.browser = args[++i];
    else if (arg === "--output" && i + 1 < args.length) config.output_dir = args[++i];
    else if (arg === "--update-snapshots") config.update_snapshots = true;
    else if (arg === "--project" && i + 1 < args.length) config.project = args[++i];
    else if (arg === "--shard" && i + 1 < args.length) {
      const [n, m] = args[++i].split("/").map(Number);
      config.shard = { current: n, total: m };
    }
  }
  return config;
}

async function runInstall(args) {
  const browser = args[0] || "chromium";
  console.log(`Installing ${browser}...`);

  try {
    const path = await TURBOSHEET.install(browser);
    console.log(`Successfully installed ${browser} to ${path}`);
  } catch (e) {
    console.error(`Failed to install ${browser}:`, e.message);
    throw e;
  }
}

async function runDevices() {
  const devices = TURBOSHEET.devices();

  console.log(`\nAvailable device presets (${devices.length}):\n`);
  console.log("Name                    viewport         mobile  touch");
  console.log("─".repeat(60));

  for (const d of devices.slice(0, 20)) {
    const v = d.viewport;
    console.log(
      `${d.name.padEnd(24)} ${v.width}x${v.height}     ${d.isMobile ? "✓" : ""}    ${d.hasTouch ? "✓" : ""}`,
    );
  }

  if (devices.length > 20) {
    console.log(`\n... and ${devices.length - 20} more devices`);
  }

  console.log("\nUse: test.use(devices['<name>'])");
}

async function runCodegen(args) {
  const { runRecording } = require("./codegen/browser-recorder");
  await runRecording(args);
}

async function runDebug(args) {
  const { runDebug: run } = require("./debug/cli");
  await run(args);
}

async function runMigrate(args) {
  const { runMigrate: doMigrate } = require("./migrate");
  await doMigrate(args);
}

async function runInit(args) {
  let ci = "github";
  for (let i = 0; i < args.length; i++) {
    if (args[i] === "--ci" && i + 1 < args.length) ci = args[++i];
  }

  console.log(`Initializing TurboSheet project with ${ci} CI...`);

  const templateDir = join(__dirname, "templates");
  console.log("\nNote: Full init implementation requires template files");
  console.log(`Templates would be copied from: ${templateDir}`);
}

main().catch((e) => {
  console.error("Fatal error:", e.message);
  process.exit(1);
});
