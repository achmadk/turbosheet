class AssertionSuggester {
  constructor() {
    this.rules = this.buildRules();
  }

  buildRules() {
    return [
      {
        action: "click",
        assertions: [
          {
            type: "toBeVisible",
            target: "same",
            reason: "Element should be visible before clicking",
          },
          { type: "toBeEnabled", target: "same", reason: "Button should be enabled" },
        ],
      },
      {
        action: "fill",
        assertions: [
          {
            type: "toHaveValue",
            target: "same",
            expected: "dynamic",
            reason: "Input should have the filled value",
          },
          { type: "toBeVisible", target: "same", reason: "Input should be visible after filling" },
        ],
      },
      {
        action: "goto",
        assertions: [
          {
            type: "toHaveURL",
            target: "same",
            expected: "url",
            reason: "Page should navigate to the expected URL",
          },
          {
            type: "toHaveTitle",
            target: "page",
            expected: "dynamic",
            reason: "Page should have expected title",
          },
        ],
      },
      {
        action: "check",
        assertions: [{ type: "toBeChecked", target: "same", reason: "Checkbox should be checked" }],
      },
      {
        action: "uncheck",
        assertions: [
          { type: "not.toBeChecked", target: "same", reason: "Checkbox should be unchecked" },
        ],
      },
      {
        action: "select",
        assertions: [
          {
            type: "toHaveValue",
            target: "same",
            expected: "dynamic",
            reason: "Option should be selected",
          },
        ],
      },
      {
        action: "hover",
        assertions: [
          {
            type: "toBeVisible",
            target: "same",
            reason: "Element should remain visible after hover",
          },
        ],
      },
    ];
  }

  suggestAssertions(action, context = {}) {
    const matchingRules = this.rules.filter((r) => r.action === action.type);
    const suggestions = [];

    for (const rule of matchingRules) {
      for (const assertion of rule.assertions) {
        const suggested = this.createSuggestion(action, assertion, context);
        if (suggested) {
          suggestions.push(suggested);
        }
      }
    }

    return suggestions;
  }

  createSuggestion(action, assertionRule, context) {
    let target, expected, selector;

    if (assertionRule.target === "same") {
      selector = action.selector;
      target = action.locator || `locator('${action.selector}')`;
    } else if (assertionRule.target === "page") {
      selector = "page";
      target = "page";
    }

    if (assertionRule.expected === "dynamic") {
      expected = this.inferExpected(action, assertionRule.type);
    } else if (assertionRule.expected === "url") {
      expected = action.url || context.currentUrl;
    } else {
      expected = assertionRule.expected;
    }

    const code = this.generateAssertionCode(assertionRule.type, target, expected);

    return {
      actionType: action.type,
      assertionType: assertionRule.type,
      target: selector,
      expected,
      code,
      reason: assertionRule.reason,
      confidence: this.calculateConfidence(action, assertionRule),
    };
  }

  inferExpected(action, assertionType) {
    switch (assertionType) {
      case "toHaveValue":
        return action.value || "";
      case "toHaveText":
        return action.value || action.text || "";
      default:
        return null;
    }
  }

  generateAssertionCode(assertionType, target, expected) {
    const quote = (s) => (typeof s === "string" ? `'${s}'` : s);

    switch (assertionType) {
      case "toBeVisible":
        return `await expect(page.locator('${target}')).toBeVisible();`;
      case "toBeEnabled":
        return `await expect(page.locator('${target}')).toBeEnabled();`;
      case "toBeChecked":
        return `await expect(page.locator('${target}')).toBeChecked();`;
      case "not.toBeChecked":
        return `await expect(page.locator('${target}')).not.toBeChecked();`;
      case "toHaveValue":
        return `await expect(page.locator('${target}')).toHaveValue(${quote(expected)});`;
      case "toHaveText":
        return `await expect(page.locator('${target}')).toHaveText(${quote(expected)});`;
      case "toHaveURL":
        return `await expect(page).toHaveURL(${quote(expected)});`;
      case "toHaveTitle":
        return `await expect(page).toHaveTitle(${quote(expected)});`;
      default:
        return `await expect(page.locator('${target}')).toBeVisible();`;
    }
  }

  calculateConfidence(action, assertionRule) {
    let confidence = 0.8;

    if (action.selector && action.selector.includes("testid")) {
      confidence += 0.1;
    }

    if (assertionRule.target === "same") {
      confidence += 0.05;
    }

    return Math.min(confidence, 1.0);
  }

  selectBestSuggestion(suggestions) {
    if (suggestions.length === 0) return null;
    return suggestions.sort((a, b) => b.confidence - a.confidence)[0];
  }
}

class TestGenerator {
  constructor() {
    this.suggester = new AssertionSuggester();
  }

  generateFromRecording(recording) {
    const lines = [];

    lines.push("import { test, expect } from 'tsheet';");
    lines.push("");
    lines.push("test.describe('Recorded Test', () => {");

    if (recording.navigations && recording.navigations.length > 0) {
      lines.push("");
      for (const nav of recording.navigations) {
        lines.push(`  test('should navigate to ${nav.url}', async ({ page }) => {`);
        lines.push(`    await page.goto('${nav.url}');`);

        const suggestions = this.suggester.suggestAssertions(
          { type: "goto", url: nav.url },
          { currentUrl: nav.url },
        );
        const best = this.suggester.selectBestSuggestion(suggestions);
        if (best) {
          lines.push(`    ${best.code}`);
        }
        lines.push("  });");
      }
    }

    if (recording.actions && recording.actions.length > 0) {
      lines.push("");
      lines.push("  test('main user flow', async ({ page }) => {");

      for (const action of recording.actions) {
        const code = this.generateActionCode(action);
        if (code) {
          lines.push(`    ${code}`);
        }

        const suggestions = this.suggester.suggestAssertions(action, {
          currentUrl: action.url,
        });
        const best = this.suggester.selectBestSuggestion(suggestions);
        if (best) {
          lines.push(`    ${best.code}`);
        }
      }

      lines.push("  });");
    }

    lines.push("});");

    return lines.join("\n");
  }

  generateActionCode(action) {
    switch (action.type) {
      case "click":
        return `await page.locator('${action.locator || action.selector}').click();`;
      case "fill":
        return `await page.locator('${action.locator || action.selector}').fill('${action.value}');`;
      case "goto":
        return `await page.goto('${action.url}');`;
      case "check":
        return `await page.locator('${action.locator || action.selector}').check();`;
      case "uncheck":
        return `await page.locator('${action.locator || action.selector}').uncheck();`;
      case "hover":
        return `await page.locator('${action.locator || action.selector}').hover();`;
      case "dblclick":
        return `await page.locator('${action.locator || action.selector}').dblclick();`;
      case "press":
        return `await page.locator('${action.locator || action.selector}').press('${action.key}');`;
      case "select":
        return `await page.locator('${action.locator || action.selector}').selectOption('${action.value}');`;
      default:
        return null;
    }
  }
}

module.exports = { AssertionSuggester, TestGenerator };

if (require.main === module) {
  const suggester = new AssertionSuggester();

  const action = {
    type: "fill",
    selector: 'input[name="email"]',
    locator: "getByRole('textbox')",
    value: "test@example.com",
  };
  const suggestions = suggester.suggestAssertions(action);

  console.log("Action:", action.type);
  console.log("\nSuggested assertions:");
  for (const s of suggestions) {
    console.log(`  [${(s.confidence * 100).toFixed(0)}%] ${s.code}`);
    console.log(`    Reason: ${s.reason}`);
  }
}
