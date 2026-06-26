## Context

TurboSheet has a solid foundational architecture with its `injected-script-engine` and Rust core. However, it relies heavily on mock/stub implementations for critical browser automation features, a basic CSS-only selector engine, and lacks a robust framework layer (CLI, config, events). To compete with Playwright and Cypress, we must replace these stubs with real implementations using CDP, WebDriver, and advanced injected scripts.

## Goals / Non-Goals

**Goals:**

- Replace 42+ silent stubs with real CDP/WebDriver implementations.
- Introduce a Playwright-style selector engine with `text=`, `role=`, and locator chaining.
- Enable full HTTPS interception using the CDP Fetch domain instead of a TCP proxy.
- Provide a complete assertion library with custom failure messages and `.not` negation.
- Deliver a first-class developer experience with a CLI, config loader, and detailed error context.
- Bring Firefox and WebKit up to parity with Chromium, utilizing BiDi where possible.

**Non-Goals:**

- Completely rewriting the underlying browser drivers.
- Replacing the Rust core with Node.js.
- Supporting legacy browsers (IE11).

## Decisions

- **Implement vs. Throw on Stubs**: We will implement all core CDP stubs (e.g., `set_viewport_size`, `reload`). For features we cannot immediately implement (e.g., touch emulation like `swipe`, `pinch`), we will throw a clear `NotImplementedError` rather than silently succeeding.
  - _Rationale_: Silent failures destroy trust. Failing explicitly allows users to know what's supported.
- **Custom Selector Engine in JS**: The new selector engine will parse strings like `role=button >> text=Submit` inside `injected-core.ts`.
  - _Rationale_: Doing this in Rust requires constant async calls to the browser. Injecting a parser in JS allows synchronous, deep DOM traversal (including Shadow DOM piercing) with high performance.
- **CDP Fetch Domain for Network Interception**: We will deprecate the TCP tunnel proxy for Chromium and use `Fetch.enable` and `Fetch.requestPaused`.
  - _Rationale_: The proxy cannot intercept HTTPS bodies without a complex MitM certificate setup. CDP Fetch gives native, unencrypted access to all requests.
- **Event Subscriptions via Tokio Channels**: We will expose `page.on('event')` by bridging CDP events through Tokio channels to N-API `ThreadsafeFunction` callbacks.
  - _Rationale_: This is the standard way to map Rust async streams to Node.js EventEmitters.
- **Config Loader via Worker**: The CLI will use a worker process (via `tsx`) to read `turbosheet.config.ts` and pass the serialized `TestConfig` back to the Rust runner.
  - _Rationale_: Node.js ecosystem config files are usually TypeScript. We cannot easily parse arbitrary TS in Rust without a JS runtime.

## Risks / Trade-offs

- **[Risk] Custom JS selector parsing is slow on large DOMs** → Mitigation: Optimize the parser and cache ASTs for frequently used selectors. Prioritize native `querySelectorAll` when possible.
- **[Risk] CDP Fetch domain adds overhead to every network request** → Mitigation: Only enable the Fetch domain if network mocking/interception is explicitly requested by the test.
- **[Risk] Firefox BiDi support is incomplete** → Mitigation: Use a hybrid approach for Firefox—BiDi for events and logging, classic WebDriver for navigation and clicks, until BiDi matures.
- **[Risk] Replacing stubs breaks existing tests that relied on them succeeding** → Mitigation: Provide a legacy toggle in the config, and release this as a major version bump.
