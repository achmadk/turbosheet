(function () {
  // Internal state stored under Symbol — invisible to Object.keys(window)
  // The public proxy at __TS_GLOBAL__ only exposes dispatch, querySelector,
  // querySelectorAll so the Rust side can invoke actions.
  const _key = typeof Symbol !== "undefined" ? Symbol("ts") : "__ts_internal";
  const ts = ((window as any)[_key] = (window as any)[_key] || {});

  ts.isFrame = window !== window.top;

  ts.dispatch = async function (uuid: string, action: string, ...args: any[]) {
    try {
      if (!ts[action]) throw new Error(`Action ${action} not found`);
      const result = await ts[action](...args);
      if ((window as any).__TS_BINDING__) {
        (window as any).__TS_BINDING__(JSON.stringify({ id: uuid, ok: true, data: result }));
      }
    } catch (e: any) {
      if ((window as any).__TS_BINDING__) {
        (window as any).__TS_BINDING__(
          JSON.stringify({ id: uuid, ok: false, error: e.message || String(e) }),
        );
      }
    }
  };

  ts.querySelector = function (
    selector: string,
    root: Document | Element = document,
  ): Element | null {
    return root.querySelector(selector);
  };

  ts.querySelectorAll = function (
    selector: string,
    root: Document | Element = document,
  ): Element[] {
    return Array.from(root.querySelectorAll(selector));
  };

  ts.waitForSelector = function (selector: string, timeoutMs: number = 30000): Promise<Element> {
    const hit = document.querySelector(selector);
    if (hit) return Promise.resolve(hit);
    return new Promise((resolve, reject) => {
      const deadline = Date.now() + timeoutMs;
      const obs = new MutationObserver(() => {
        const el = document.querySelector(selector);
        if (!el) return;
        obs.disconnect();
        clearInterval(timer);
        resolve(el);
      });
      obs.observe(document, { childList: true, subtree: true, attributes: false });
      const timer = setInterval(() => {
        const el = document.querySelector(selector);
        if (el) {
          clearInterval(timer);
          obs.disconnect();
          resolve(el);
        } else if (Date.now() > deadline) {
          clearInterval(timer);
          obs.disconnect();
          reject(new Error(`waitForSelector timeout: ${selector}`));
        }
      }, 100);
      setTimeout(() => {
        clearInterval(timer);
        obs.disconnect();
        reject(new Error(`waitForSelector timeout (${timeoutMs}ms): ${selector}`));
      }, timeoutMs);
    });
  };

  // Expose a thin public proxy on the randomized global name.
  // The Rust engine only accesses dispatch / querySelector / querySelectorAll.
  // Everything else (getBy*, checkActionability, etc.) lives only under Symbol.
  const pub = ((window as any).__TS_GLOBAL__ = (window as any).__TS_GLOBAL__ || {});
  pub.dispatch = function (this: any, ...args: any[]) {
    return ts.dispatch.apply(ts, args);
  };
  pub.querySelector = function (this: any, ...args: any[]) {
    return ts.querySelector.apply(ts, args);
  };
  pub.querySelectorAll = function (this: any, ...args: any[]) {
    return ts.querySelectorAll.apply(ts, args);
  };
  // Store the Symbol key as a non-enumerable reference so injected-actions.ts
  // can find the internal store.
  try {
    Object.defineProperty(pub, "__tsSym", { value: _key, enumerable: false, configurable: true });
  } catch {}
})();
