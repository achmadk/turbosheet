class SmartSelector {
  constructor(options = {}) {
    this.testIdAttribute = options.testIdAttribute || "data-testid";
  }

  analyze(element) {
    const attrs = element.attributes || {};
    const tag = element.tagName?.toLowerCase() || "div";
    const text = element.textContent?.trim() || "";
    const role = attrs.role || this.inferRole(tag);

    const candidates = [];

    if (attrs[this.testIdAttribute]) {
      candidates.push({
        type: "testId",
        selector: attrs[this.testIdAttribute],
        priority: 1,
      });
    }

    if (role && role !== "presentation") {
      candidates.push({
        type: "role",
        selector: role,
        priority: 2,
      });
    }

    if (text && text.length < 100) {
      if (tag === "button" || tag === "a") {
        candidates.push({
          type: "text",
          selector: text,
          priority: 3,
        });
      } else {
        candidates.push({
          type: "text",
          selector: text,
          priority: 4,
        });
      }
    }

    if (attrs.placeholder) {
      candidates.push({
        type: "placeholder",
        selector: attrs.placeholder,
        priority: 5,
      });
    }

    if (attrs["aria-label"]) {
      candidates.push({
        type: "label",
        selector: attrs["aria-label"],
        priority: 6,
      });
    }

    if (attrs.id) {
      candidates.push({
        type: "css",
        selector: `#${attrs.id}`,
        priority: 7,
      });
    }

    if (attrs.class) {
      const classes = attrs.class
        .split(" ")
        .filter((c) => c && !c.match(/^(active|hover|focus|disabled)$/i));
      if (classes.length > 0) {
        candidates.push({
          type: "css",
          selector: `${tag}.${classes[0]}`,
          priority: 8,
        });
      }
    }

    candidates.sort((a, b) => a.priority - b.priority);

    return candidates[0] || { type: "css", selector: tag, priority: 99 };
  }

  inferRole(tag) {
    const roleMap = {
      a: "link",
      button: "button",
      input: "textbox",
      textarea: "textbox",
      select: "listbox",
      checkbox: "checkbox",
      radio: "radio",
      nav: "navigation",
      header: "banner",
      footer: "contentinfo",
      main: "main",
      article: "article",
      section: "region",
      aside: "complementary",
      img: "img",
      h1: "heading",
      h2: "heading",
      h3: "heading",
      h4: "heading",
      h5: "heading",
      h6: "heading",
    };
    return roleMap[tag] || null;
  }

  generateLocator(selector) {
    switch (selector.type) {
      case "testId":
        return `getByTestId(${this.quote(selector.selector)})`;
      case "role":
        return `getByRole(${this.quote(selector.selector)})`;
      case "text":
        return `getByText(${this.quote(selector.selector)})`;
      case "label":
        return `getByLabel(${this.quote(selector.selector)})`;
      case "placeholder":
        return `getByPlaceholder(${this.quote(selector.selector)})`;
      case "title":
        return `getByTitle(${this.quote(selector.selector)})`;
      case "css":
      default:
        return `locator('${selector.selector}')`;
    }
  }

  quote(str) {
    if (str.includes("'")) {
      return `"${str}"`;
    }
    return `'${str}'`;
  }
}

class SelectorGenerator {
  constructor() {
    this.smartSelector = new SmartSelector();
    this.actions = [];
    this.selectors = new Map();
  }

  onClick(element, metadata = {}) {
    const selector = this.smartSelector.analyze(element);
    this.selectors.set("click", selector);

    const locator = this.smartSelector.generateLocator(selector);

    this.actions.push({
      type: "click",
      locator,
      selector,
      timestamp: Date.now(),
      ...metadata,
    });

    return `await page.locator(${locator}).click();`;
  }

  onFill(element, value, metadata = {}) {
    const selector = this.smartSelector.analyze(element);
    this.selectors.set("fill", selector);

    const locator = this.smartSelector.generateLocator(selector);

    this.actions.push({
      type: "fill",
      locator,
      selector,
      value,
      timestamp: Date.now(),
      ...metadata,
    });

    return `await page.locator(${locator}).fill('${value}');`;
  }

  onNavigation(url, metadata = {}) {
    this.actions.push({
      type: "navigation",
      url,
      timestamp: Date.now(),
      ...metadata,
    });

    return `await page.goto('${url}');`;
  }

  onAssert(element, assertionType, expected) {
    const selector = this.smartSelector.analyze(element);
    this.selectors.set(`assert_${this.actions.length}`, selector);

    const locator = this.smartSelector.generateLocator(selector);

    let assertion;
    switch (assertionType) {
      case "visible":
        assertion = `await expect(page.locator(${locator})).toBeVisible();`;
        break;
      case "text":
        assertion = `await expect(page.locator(${locator})).toHaveText(${this.smartSelector.quote(expected)});`;
        break;
      case "value":
        assertion = `await expect(page.locator(${locator})).toHaveValue(${this.smartSelector.quote(expected)});`;
        break;
      default:
        assertion = `await expect(page.locator(${locator})).toBeVisible();`;
    }

    return assertion;
  }

  suggestSelector(element) {
    return this.smartSelector.analyze(element);
  }

  getActions() {
    return this.actions;
  }

  reset() {
    this.actions = [];
    this.selectors.clear();
  }
}

module.exports = { SmartSelector, SelectorGenerator };

if (require.main === module) {
  const element = {
    tagName: "button",
    textContent: "Submit",
    attributes: {
      "data-testid": "submit-btn",
      class: "btn btn-primary",
    },
  };

  const generator = new SelectorGenerator();
  const selector = generator.suggestSelector(element);

  console.log("Smart selector analysis:");
  console.log(JSON.stringify(selector, null, 2));
  console.log("\nGenerated code:");
  console.log(generator.onClick(element));
}
