// injected-actions.ts
// On-demand payload for heavy actionability, geometry, and interactions

(function () {
  // Find the internal Symbol store via the core script's proxy reference.
  // If the core hasn't been injected (shouldn't happen), fall back to a local store.
  const pub = (window as any).__TS_GLOBAL__;
  const _key =
    (pub && pub.__tsSym) || (typeof Symbol !== "undefined" ? Symbol("ts") : "__ts_internal");
  const ts = ((window as any)[_key] = (window as any)[_key] || {});

  // ── Chain selector parser ──────────────────────────────────

  ts.parseSelector = function (selector: string) {
    const parts = selector
      .split(">>")
      .map((p: string) => p.trim())
      .filter((p: string) => p.length > 0);
    const parsePart = (part: string) => {
      const match = part.match(/^([a-zA-Z0-9_-]+)=(.*)$/);
      return match ? { type: match[1], value: match[2] } : { type: "css", value: part };
    };
    const parsedParts = parts.map(parsePart);
    if (parsedParts.length === 0) return { type: "css", value: "", chain: [] };
    const buildChain = (index: number): any => {
      if (index >= parsedParts.length) return [];
      const current = parsedParts[index];
      return {
        type: current.type,
        value: current.value,
        chain: index + 1 < parsedParts.length ? [buildChain(index + 1)] : [],
      };
    };
    return buildChain(0);
  };

  function getImplicitRole(el: Element): string | null {
    const tag = el.tagName.toLowerCase();
    if (tag === "button") return "button";
    if (tag === "a" && el.hasAttribute("href")) return "link";
    if (tag === "input") {
      const type = el.getAttribute("type") || "text";
      if (type === "checkbox") return "checkbox";
      if (type === "radio") return "radio";
      if (type === "submit" || type === "button") return "button";
      return "textbox";
    }
    if (
      tag === "h1" ||
      tag === "h2" ||
      tag === "h3" ||
      tag === "h4" ||
      tag === "h5" ||
      tag === "h6"
    )
      return "heading";
    return null;
  }

  function getAccessibleName(el: Element): string {
    let name = el.getAttribute("aria-label");
    if (name) return name;
    const labelledBy = el.getAttribute("aria-labelledby");
    if (labelledBy) {
      const labelEl = document.getElementById(labelledBy);
      if (labelEl) return labelEl.textContent || "";
    }
    return el.textContent || "";
  }

  ts.getByRole = function (
    role: string,
    name?: string,
    root: Document | Element = document,
  ): Element[] {
    return Array.from(root.querySelectorAll("*")).filter((el: Element) => {
      const elRole = el.getAttribute("role") || getImplicitRole(el);
      if (elRole !== role) return false;
      if (name !== undefined) return getAccessibleName(el).includes(name);
      return true;
    });
  };

  ts.getByText = function (
    text: string,
    options: { exact?: boolean } = {},
    root: Document | Element = document,
  ): Element[] {
    return Array.from(root.querySelectorAll("*")).filter((el: Element) => {
      if (el.children.length > 0) {
        const hasDirectText = Array.from(el.childNodes).some(
          (node: Node) => node.nodeType === Node.TEXT_NODE && node.textContent?.trim().length,
        );
        if (!hasDirectText) return false;
      }
      const content = (el.textContent || "").trim();
      return options.exact ? content === text : content.includes(text);
    });
  };

  ts.getByLabel = function (
    text: string,
    options: { exact?: boolean } = {},
    root: Document | Element = document,
  ): Element[] {
    const labels = Array.from(root.querySelectorAll("label")).filter((label: Element) => {
      const content = (label.textContent || "").trim();
      return options.exact ? content === text : content.includes(text);
    });
    const elements: Element[] = [];
    for (const label of labels) {
      const htmlFor = label.getAttribute("for");
      if (htmlFor) {
        const el = root.querySelector(`#${htmlFor}`);
        if (el) elements.push(el);
      } else {
        const input = label.querySelector("input, select, textarea");
        if (input) elements.push(input);
      }
    }
    const ariaLabels = Array.from(root.querySelectorAll("[aria-label]")).filter((el: Element) => {
      const content = (el.getAttribute("aria-label") || "").trim();
      return options.exact ? content === text : content.includes(text);
    });
    return [...new Set([...elements, ...ariaLabels])];
  };

  ts.getByTestId = function (testId: string, root: Document | Element = document): Element[] {
    return Array.from(root.querySelectorAll(`[data-testid="${testId}"]`));
  };

  ts.getByPlaceholder = function (
    text: string,
    options: { exact?: boolean } = {},
    root: Document | Element = document,
  ): Element[] {
    return Array.from(root.querySelectorAll("[placeholder]")).filter((el: Element) => {
      const content = el.getAttribute("placeholder") || "";
      return options.exact ? content === text : content.includes(text);
    });
  };

  ts.evaluateParsedSelectorAll = function (
    parsed: any,
    root: Document | Element = document,
  ): Element[] {
    let currentElements = [root];
    let currentParsed = parsed;
    while (currentParsed) {
      if (currentParsed.type === "nth") {
        const index = parseInt(currentParsed.value, 10);
        currentElements =
          index === -1 ? currentElements.slice(-1) : currentElements.slice(index, index + 1);
      } else {
        let nextElements: Element[] = [];
        for (const el of currentElements) {
          let found: Element[] = [];
          if (currentParsed.type === "css") {
            found = Array.from(el.querySelectorAll(currentParsed.value));
          } else if (currentParsed.type === "text") {
            const isExact =
              currentParsed.value.startsWith('"') && currentParsed.value.endsWith('"');
            found = ts.getByText(
              isExact ? currentParsed.value.slice(1, -1) : currentParsed.value,
              { exact: isExact },
              el,
            );
          } else if (currentParsed.type === "role") {
            const match = currentParsed.value.match(/^([^[]+)(?:\[name="(.*)"\])?$/);
            found = match
              ? ts.getByRole(match[1], match[2], el)
              : ts.getByRole(currentParsed.value, undefined, el);
          } else if (currentParsed.type === "data-testid") {
            found = ts.getByTestId(currentParsed.value, el);
          } else if (currentParsed.type === "label") {
            const isExact =
              currentParsed.value.startsWith('"') && currentParsed.value.endsWith('"');
            found = ts.getByLabel(
              isExact ? currentParsed.value.slice(1, -1) : currentParsed.value,
              { exact: isExact },
              el,
            );
          } else if (currentParsed.type === "placeholder") {
            const isExact =
              currentParsed.value.startsWith('"') && currentParsed.value.endsWith('"');
            found = ts.getByPlaceholder(
              isExact ? currentParsed.value.slice(1, -1) : currentParsed.value,
              { exact: isExact },
              el,
            );
          }
          nextElements.push(...found);
        }
        currentElements = [...new Set(nextElements)];
      }
      if (currentParsed.chain && currentParsed.chain.length > 0) {
        currentParsed = currentParsed.chain[0];
      } else {
        break;
      }
    }
    return currentElements.filter((el) => el !== document) as Element[];
  };

  // Override core's thin querySelector with chain-aware version
  ts.querySelector = function (
    selector: string,
    root: Document | Element = document,
  ): Element | null {
    const parsed = ts.parseSelector(selector);
    const elements = ts.evaluateParsedSelectorAll(parsed, root);
    return elements.length > 0 ? elements[0] : null;
  };

  ts.querySelectorAll = function (
    selector: string,
    root: Document | Element = document,
  ): Element[] {
    const parsed = ts.parseSelector(selector);
    return ts.evaluateParsedSelectorAll(parsed, root);
  };

  // ── Actionability ──────────────────────────────────────────

  ts.checkActionability = async function (selector: string) {
    const el = ts.querySelector(selector);
    if (!el) throw new Error("Element not found");

    const style = window.getComputedStyle(el);
    if (style.display === "none" || style.visibility === "hidden" || style.opacity === "0") {
      throw new Error("Element is not visible");
    }
    const rect = el.getBoundingClientRect();
    if (rect.width === 0 || rect.height === 0) {
      throw new Error("Element has 0 bounding box");
    }

    if ((el as any).disabled) {
      throw new Error("Element is disabled");
    }

    const prevRect = rect;
    await new Promise((r) => requestAnimationFrame(() => requestAnimationFrame(r)));
    const newRect = el.getBoundingClientRect();
    if (
      prevRect.x !== newRect.x ||
      prevRect.y !== newRect.y ||
      prevRect.width !== newRect.width ||
      prevRect.height !== newRect.height
    ) {
      throw new Error("Element is animating");
    }

    const cx = newRect.x + newRect.width / 2;
    const cy = newRect.y + newRect.height / 2;
    const elementFromPoint = document.elementFromPoint(cx, cy);
    if (elementFromPoint && !el.contains(elementFromPoint) && !elementFromPoint.contains(el)) {
      throw new Error("Element is covered by " + elementFromPoint.tagName);
    }

    return "ok";
  };

  ts.waitForActionability = async function (selector: string, timeoutMs: number = 30000) {
    const start = Date.now();
    // @ts-expect-error Browser-injected script — Promise.withResolvers() is available in modern browsers
    const { resolve, reject } = Promise.withResolvers();
    let isDone = false;
    let checking = false;
    let interval: any;

    const finish = (err?: Error) => {
      if (isDone) return;
      isDone = true;
      observer.disconnect();
      window.removeEventListener("scroll", tryCheck, true);
      window.removeEventListener("resize", tryCheck, true);
      window.removeEventListener("transitionend", tryCheck, true);
      window.removeEventListener("animationend", tryCheck, true);
      clearInterval(interval);
      if (err) reject(err);
      else resolve("ok");
    };

    const tryCheck = async () => {
      if (isDone || checking) return;
      checking = true;
      try {
        await ts.checkActionability(selector);
        finish();
      } catch (e: any) {
        if (Date.now() - start > timeoutMs) {
          finish(new Error(`Timeout waiting for actionability of ${selector}: ${e.message}`));
        }
      } finally {
        checking = false;
      }
    };

    const observer = new MutationObserver(tryCheck);
    observer.observe(document, {
      childList: true,
      subtree: true,
      attributes: true,
      characterData: true,
    });
    window.addEventListener("scroll", tryCheck, true);
    window.addEventListener("resize", tryCheck, true);
    window.addEventListener("transitionend", tryCheck, true);
    window.addEventListener("animationend", tryCheck, true);

    await tryCheck(); // Initial check

    interval = setInterval(tryCheck, 500);

    setTimeout(() => {
      if (!isDone) {
        finish(new Error(`Timeout waiting for actionability of ${selector}`));
      }
    }, timeoutMs);
  };

  ts.content = async function () {
    return document.documentElement.outerHTML;
  };
  ts.title = async function () {
    return document.title;
  };

  ts.isVisible = async function (selector: string) {
    const el = ts.querySelector(selector);
    if (!el) return false;
    const style = window.getComputedStyle(el);
    return (
      style && style.display !== "none" && style.visibility !== "hidden" && style.opacity !== "0"
    );
  };

  ts.isEnabled = async function (selector: string) {
    const el = ts.querySelector(selector);
    if (!el) return false;
    return !(el as any).disabled;
  };

  ts.isDisabled = async function (selector: string) {
    const el = ts.querySelector(selector);
    if (!el) return false;
    return !!(el as any).disabled;
  };

  ts.dblclick = async function (selector: string) {
    const el = ts.querySelector(selector);
    if (!el) return false;
    const evt = new MouseEvent("dblclick", { bubbles: true, cancelable: true, view: window });
    el.dispatchEvent(evt);
    return true;
  };

  ts.rightClick = async function (selector: string) {
    const el = ts.querySelector(selector);
    if (!el) return false;
    const evt = new MouseEvent("contextmenu", {
      bubbles: true,
      cancelable: true,
      view: window,
      button: 2,
    });
    el.dispatchEvent(evt);
    return true;
  };

  ts.check = async function (selector: string) {
    const el = ts.querySelector(selector) as HTMLInputElement;
    if (!el) return false;
    if (!el.checked) {
      el.checked = true;
      el.dispatchEvent(new Event("change", { bubbles: true }));
      el.dispatchEvent(new Event("input", { bubbles: true }));
    }
    return true;
  };

  ts.uncheck = async function (selector: string) {
    const el = ts.querySelector(selector) as HTMLInputElement;
    if (!el) return false;
    if (el.checked) {
      el.checked = false;
      el.dispatchEvent(new Event("change", { bubbles: true }));
      el.dispatchEvent(new Event("input", { bubbles: true }));
    }
    return true;
  };

  ts.select = async function (selector: string, value: string) {
    const el = ts.querySelector(selector) as HTMLSelectElement;
    if (!el) return false;
    el.value = value;
    el.dispatchEvent(new Event("change", { bubbles: true }));
    return true;
  };

  ts.focus = async function (selector: string) {
    const el = ts.querySelector(selector) as HTMLElement;
    if (!el) return false;
    el.focus();
    return true;
  };

  ts.blur = async function (selector: string) {
    const el = ts.querySelector(selector) as HTMLElement;
    if (!el) return false;
    el.blur();
    return true;
  };

  ts.scrollIntoView = async function (selector: string) {
    const el = ts.querySelector(selector);
    if (!el) return false;
    el.scrollIntoView({ behavior: "smooth", block: "center" });
    return true;
  };

  // ── Engine-invoked actions (migrated from inline format!() JS) ─────

  ts.hover = async function (selector: string) {
    const el = ts.querySelector(selector);
    if (!el) return false;
    const evt = new MouseEvent("mouseover", { bubbles: true, cancelable: true, view: window });
    el.dispatchEvent(evt);
    return true;
  };

  ts.innerText = async function (selector: string): Promise<string | null> {
    const el = ts.querySelector(selector);
    if (!el) return null;
    return (el as HTMLElement).innerText;
  };

  ts.innerHTML = async function (selector: string): Promise<string | null> {
    const el = ts.querySelector(selector);
    if (!el) return null;
    return el.innerHTML;
  };

  ts.getAttribute = async function (selector: string, name: string): Promise<string | null> {
    const el = ts.querySelector(selector);
    if (!el) return null;
    return el.getAttribute(name);
  };

  ts.dragAndDrop = async function (sourceSelector: string, targetSelector: string) {
    const src = ts.querySelector(sourceSelector);
    const dst = ts.querySelector(targetSelector);
    if (!src || !dst) return false;
    const dt = new DataTransfer();
    src.dispatchEvent(new DragEvent("dragstart", { dataTransfer: dt, bubbles: true }));
    dst.dispatchEvent(new DragEvent("drop", { dataTransfer: dt, bubbles: true }));
    src.dispatchEvent(new DragEvent("dragend", { dataTransfer: dt, bubbles: true }));
    return true;
  };

  ts.addScriptTag = async function (content: string) {
    const s = document.createElement("script");
    s.textContent = content;
    document.head.appendChild(s);
    return true;
  };

  ts.addStyleTag = async function (content: string) {
    const s = document.createElement("style");
    s.textContent = content;
    document.head.appendChild(s);
    return true;
  };
})();
