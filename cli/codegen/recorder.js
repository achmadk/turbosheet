#!/usr/bin/env node

const { writeFileSync } = require("fs");

class CodegenRecorder {
  constructor(options = {}) {
    this.options = {
      output: options.output || "recorded-test.tsheet.ts",
      url: options.url || "https://example.com",
      testIdAttribute: options.testIdAttribute || "data-testid",
    };

    this.actions = [];
    this.assertions = [];
    this.navigations = [];
    this.currentUrl = null;
    this.actionIndex = 0;
  }

  async start() {
    console.log("Starting TurboSheet Codegen Recorder");
    console.log(`Target URL: ${this.options.url}`);
    console.log(`Output file: ${this.options.output}`);
    console.log("\nOpening browser...");

    console.log("\nRecording actions...");
    console.log("Press Ctrl+C to stop recording\n");
  }

  recordAction(type, selector, value, options = {}) {
    const action = {
      type,
      selector,
      value,
      timestamp: Date.now(),
      ...options,
    };

    this.actions.push(action);
    this.actionIndex++;

    console.log(`  [${this.actionIndex}] ${type}: ${selector}` + (value ? ` = "${value}"` : ""));
  }

  recordAssertion(type, target, expected, options = {}) {
    const assertion = {
      type,
      target,
      expected,
      timestamp: Date.now(),
      ...options,
    };

    this.assertions.push(assertion);
  }

  recordNavigation(url) {
    this.navigations.push({
      url,
      timestamp: Date.now(),
    });
    this.currentUrl = url;
  }

  generateCode() {
    const lines = [];
    lines.push("import { test, expect } from 'tsheet';");
    lines.push("");
    lines.push("test.describe('Recorded Test', () => {");

    for (const nav of this.navigations) {
      lines.push(`  test('navigate to ${nav.url}', async ({ page }) => {`);
      lines.push(`    await page.goto('${nav.url}');`);
      lines.push("  });");
      lines.push("");
    }

    const testBody = [];
    for (const action of this.actions) {
      testBody.push(...this.formatAction(action));
    }

    if (testBody.length > 0) {
      lines.push("  test('main flow', async ({ page }) => {");
      for (const line of testBody) {
        lines.push(`    ${line}`);
      }
      lines.push("  });");
    }

    for (const assertion of this.assertions) {
      lines.push(...this.formatAssertion(assertion));
    }

    lines.push("});");

    return lines.join("\n");
  }

  formatAction(action) {
    const lines = [];
    const selector = this.formatSelector(action.selector, action.selectorType);

    switch (action.type) {
      case "click":
        lines.push(`await page.locator('${selector}').click();`);
        break;
      case "fill":
        lines.push(`await page.locator('${selector}').fill('${action.value}');`);
        break;
      case "check":
        lines.push(`await page.locator('${selector}').check();`);
        break;
      case "uncheck":
        lines.push(`await page.locator('${selector}').uncheck();`);
        break;
      case "select":
        lines.push(`await page.locator('${selector}').selectOption('${action.value}');`);
        break;
      case "hover":
        lines.push(`await page.locator('${selector}').hover();`);
        break;
      case "dblclick":
        lines.push(`await page.locator('${selector}').dblclick();`);
        break;
      case "type":
        lines.push(`await page.locator('${selector}').type('${action.value}');`);
        break;
      case "press":
        lines.push(`await page.locator('${selector}').press('${action.value}');`);
        break;
      case "goto":
        lines.push(`await page.goto('${action.url || action.value}');`);
        break;
    }

    return lines;
  }

  formatSelector(selector, type) {
    if (!type || type === "css") {
      return selector;
    }

    switch (type) {
      case "role":
        return `getByRole(${this.quote(selector)})`;
      case "text":
        return `getByText(${this.quote(selector)})`;
      case "testId":
        return `getByTestId(${this.quote(selector)})`;
      case "label":
        return `getByLabel(${this.quote(selector)})`;
      case "placeholder":
        return `getByPlaceholder(${this.quote(selector)})`;
      case "title":
        return `getByTitle(${this.quote(selector)})`;
      default:
        return selector;
    }
  }

  quote(str) {
    if (str.includes("'")) {
      return `"${str}"`;
    }
    return `'${str}'`;
  }

  formatAssertion(assertion) {
    const lines = [];
    const selector = this.formatSelector(assertion.target, assertion.selectorType);

    switch (assertion.type) {
      case "toBeVisible":
        lines.push(`await expect(page.locator('${selector}')).toBeVisible();`);
        break;
      case "toBeHidden":
        lines.push(`await expect(page.locator('${selector}')).toBeHidden();`);
        break;
      case "toHaveText":
        lines.push(
          `await expect(page.locator('${selector}')).toHaveText(${this.quote(assertion.expected)});`,
        );
        break;
      case "toHaveValue":
        lines.push(
          `await expect(page.locator('${selector}')).toHaveValue(${this.quote(assertion.expected)});`,
        );
        break;
      case "toHaveURL":
        lines.push(`await expect(page).toHaveURL(${this.quote(assertion.expected)});`);
        break;
      case "toHaveTitle":
        lines.push(`await expect(page).toHaveTitle(${this.quote(assertion.expected)});`);
        break;
    }

    return lines;
  }

  async save() {
    const code = this.generateCode();
    writeFileSync(this.options.output, code, "utf-8");
    console.log(`\nTest saved to: ${this.options.output}`);
    console.log(`Actions recorded: ${this.actions.length}`);
    console.log(`Assertions recorded: ${this.assertions.length}`);
    console.log(`Navigations recorded: ${this.navigations.length}`);
    return this.options.output;
  }
}

module.exports = { CodegenRecorder };

if (require.main === module) {
  const args = process.argv.slice(2);
  let url = "https://example.com";
  let output = "recorded-test.tsheet.ts";

  for (let i = 0; i < args.length; i++) {
    if (args[i] === "--url" && i + 1 < args.length) url = args[++i];
    else if (args[i] === "--output" && i + 1 < args.length) output = args[++i];
  }

  const recorder = new CodegenRecorder({ url, output });
  recorder.start().catch((e) => {
    console.error("Error:", e.message);
    process.exit(1);
  });
}
