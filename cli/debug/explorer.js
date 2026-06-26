class DOMExplorer {
  constructor(page) {
    this.page = page;
  }

  async getElementAtPoint(x, y) {
    return await this.page.evaluate(
      ({ x, y }) => {
        const el = document.elementFromPoint(x, y);
        if (!el) return null;

        return {
          tag: el.tagName.toLowerCase(),
          id: el.id,
          classes: el.className ? el.className.split(" ").filter((c) => c) : [],
          attributes: extractAttributes(el),
          text: el.textContent?.trim().substring(0, 100),
          rect: getBoundingRect(el),
          computedStyles: getComputedStyles(el),
          children: el.children.length,
        };

        function extractAttributes(el) {
          const attrs = {};
          for (const attr of el.attributes) {
            attrs[attr.name] = attr.value;
          }
          return attrs;
        }

        function getBoundingRect(el) {
          const rect = el.getBoundingClientRect();
          return {
            x: rect.x,
            y: rect.y,
            width: rect.width,
            height: rect.height,
            top: rect.top,
            right: rect.right,
            bottom: rect.bottom,
            left: rect.left,
          };
        }

        function getComputedStyles(el) {
          const cs = window.getComputedStyle(el);
          return {
            display: cs.display,
            visibility: cs.visibility,
            opacity: cs.opacity,
            pointerEvents: cs.pointerEvents,
            cursor: cs.cursor,
            color: cs.color,
            backgroundColor: cs.backgroundColor,
            fontSize: cs.fontSize,
            fontWeight: cs.fontWeight,
          };
        }
      },
      { x, y },
    );
  }

  async getElementBySelector(selector) {
    return await this.page.evaluate((sel) => {
      const el = document.querySelector(sel);
      if (!el) return null;

      return {
        tag: el.tagName.toLowerCase(),
        id: el.id,
        classes: el.className ? el.className.split(" ").filter((c) => c) : [],
        attributes: extractAttributes(el),
        text: el.textContent?.trim().substring(0, 200),
        rect: getBoundingRect(el),
        computedStyles: getComputedStyles(el),
        children: el.children.length,
        innerHTML: el.innerHTML?.substring(0, 500),
      };

      function extractAttributes(el) {
        const attrs = {};
        for (const attr of el.attributes) {
          attrs[attr.name] = attr.value;
        }
        return attrs;
      }

      function getBoundingRect(el) {
        const rect = el.getBoundingClientRect();
        return {
          x: rect.x,
          y: rect.y,
          width: rect.width,
          height: rect.height,
        };
      }

      function getComputedStyles(el) {
        const cs = window.getComputedStyle(el);
        return {
          display: cs.display,
          visibility: cs.visibility,
          opacity: cs.opacity,
        };
      }
    }, selector);
  }

  async querySelectorAll(selector) {
    return await this.page.evaluate((sel) => {
      const els = Array.from(document.querySelectorAll(sel)).slice(0, 20);
      return els.map((el, i) => ({
        index: i,
        tag: el.tagName.toLowerCase(),
        id: el.id,
        text: el.textContent?.trim().substring(0, 50),
        rect: getBoundingRect(el),
      }));

      function getBoundingRect(el) {
        const rect = el.getBoundingClientRect();
        return {
          x: Math.round(rect.x),
          y: Math.round(rect.y),
          width: Math.round(rect.width),
          height: Math.round(rect.height),
        };
      }
    }, selector);
  }

  async getInteractiveElements() {
    return await this.page.evaluate(() => {
      const selectors = [
        "a[href]",
        "button:not([disabled])",
        "input:not([disabled])",
        "select:not([disabled])",
        "textarea:not([disabled])",
        "[onclick]",
        '[role="button"]',
        '[tabindex]:not([tabindex="-1"])',
      ];

      const elements = [];
      const seen = new Set();

      for (const sel of selectors) {
        const els = document.querySelectorAll(sel);
        for (const el of els) {
          if (seen.has(el)) continue;
          seen.add(el);

          const rect = el.getBoundingClientRect();
          if (rect.width === 0 || rect.height === 0) continue;
          if (rect.x < 0 || rect.y < 0) continue;

          elements.push({
            selector: getSelector(el),
            tag: el.tagName.toLowerCase(),
            text: el.textContent?.trim().substring(0, 50),
            rect: {
              x: Math.round(rect.x),
              y: Math.round(rect.y),
              width: Math.round(rect.width),
              height: Math.round(rect.height),
            },
            type: el.type || "",
            role: el.getAttribute("role") || "",
          });
        }
      }

      return elements.sort((a, b) => a.rect.y - b.rect.y);

      function getSelector(el) {
        if (el.id) return `#${el.id}`;
        if (el.className && typeof el.className === "string" && el.className.split(" ")[0]) {
          return `${el.tagName.toLowerCase()}.${el.className.split(" ")[0]}`;
        }
        return el.tagName.toLowerCase();
      }
    });
  }

  async getCSSSelector(elementData) {
    return await this.page.evaluate((data) => {
      const el =
        document.querySelector(`[data-testid="${data.testId}"]`) ||
        document.querySelector(`#${data.id}`) ||
        document.querySelector(data.selector);

      if (!el) return null;

      return getBestSelector(el);

      function getBestSelector(el) {
        if (el.id) return `#${el.id}`;

        const parts = [];
        let current = el;

        while (current && current !== document.body && parts.length < 3) {
          let selector = current.tagName.toLowerCase();

          if (current.id) {
            selector = `#${current.id}`;
            parts.unshift(selector);
            break;
          }

          if (current.className && typeof current.className === "string") {
            const cls = current.className.split(" ").filter((c) => c)[0];
            if (cls) selector += `.${cls}`;
          }

          parts.unshift(selector);
          current = current.parentElement;
        }

        return parts.join(" > ");
      }
    }, elementData);
  }
}

module.exports = { DOMExplorer };

if (require.main === module) {
  console.log("DOM Explorer module");
  console.log("Usage: const explorer = new DOMExplorer(page);");
  console.log("Methods:");
  console.log("  getElementAtPoint(x, y)");
  console.log("  getElementBySelector(selector)");
  console.log("  querySelectorAll(selector)");
  console.log("  getInteractiveElements()");
  console.log("  getCSSSelector(elementData)");
}
