## Why

TurboSheet currently uses an ad-hoc `page.evaluate()` pattern that constructs JavaScript strings for every browser interaction. This results in duplicated logic across engine implementations, excessive CDP round-trips per action (3-10+), and fragile string construction. A pre-injected script engine architecture is needed to resolve these bottlenecks, dramatically reducing per-action latency and unifying browser-side logic, laying a scalable foundation for advanced features like swarm scaling, AI agent interactions, and robust network interception.

## What Changes

- Implement a hybrid injection engine with two modules: a lightweight `injected-core.js` for document start and an on-demand `injected-actions.js` payload.
- Add `inject_core_script` and `register_binding` methods to the `PageEngine` trait.
- Replace all inline `page.evaluate()` format strings in Chromium, Firefox, and WebKit implementations with binding calls to the pre-injected functions.
- Introduce a real transparent proxy to fix network request interception (HTTPS CONNECT tunneling, POST data capture) or migrate to CDP-based network interception.
- Establish a CDP Event Dispatcher architecture to process lifecycle and network events reactively instead of polling.
- Remove polling loops from `wait_for_actionability()`, utilizing MutationObserver logic inside the injected script instead.
- Include a stealth mode using IIFE wrapping, symbol isolation, and runtime name randomization.

## Capabilities

### New Capabilities

- `injected-script-engine`: The core architecture change that injects scripts on new document creation and establishes a communication bridge using CDP bindings.
- `cdp-event-dispatcher`: Reactive subscription to CDP events such as frame navigations, network requests, dialog events, and binding calls.
- `network-interception`: Implementation of robust request interception, utilizing a transparent proxy tunnel for HTTPS or CDP-based interception, properly handling POST payloads and continuation routes.

### Modified Capabilities

## Impact

- **Performance:** Reduces CDP round-trips for interactions from 3-10+ down to 1, lowering latency significantly.
- **Codebase:** Drastically reduces duplicated JavaScript evaluation strings across Chromium, Firefox, and WebKit modules (e.g., `chromium.rs`, `firefox.rs`, `webkit.rs`, `locator.rs`).
- **Build Process:** Incorporates a new `js/` directory that is compiled and minified (via `rolldown`) into the Rust binary directly via `include_str!()`.
- **APIs:** The internal `PageEngine` trait signature will expand, impacting any future engine implementations, but user-facing APIs should remain consistent or become significantly faster and less flaky.
