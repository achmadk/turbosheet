## Why

TurboSheet's core browser automation has 12 Chromium stubs returning `Ok(())` silently, zero CDP event processing (the handler task discards all events), a broken network proxy that returns HTTP 501 on `route.continue()`, and synthetic JavaScript events that fail `event.isTrusted` checks. These are not gaps — they are broken public APIs that silently fail users. Additionally, all three engines (Chromium, Firefox, WebKit) duplicate ~1500 lines of inline JavaScript with ad-hoc `format!("(function() {{ ... }})()")` patterns. This foundational work fixes the broken subsystems and introduces a shared injected script engine that eliminates JS duplication and reduces CDP round-trips from 3-10+ to 1 per action.

## What Changes

- **CDP Event Dispatcher**: Replace the current no-op handler task with a proper event dispatcher that subscribes to `Page.frameNavigated`, `Runtime.bindingCalled`, `Runtime.consoleAPICalled`, `Page.javascriptDialogOpening`, `Network.requestWillBeSent`, and 8+ other CDP events. This is the prerequisite for injected.js bindings, dialog handling, console capture, network interception, URL reactive tracking, and multi-page support.

- **Injected Script Engine (Hybrid Injection)**: Replace ad-hoc `page.evaluate(format!(...))` patterns with a two-module injected script architecture:
  - `injected-core.js` (≤2KB): Pre-injected via `addScriptToEvaluateOnNewDocument`, establishes `__tsReport` binding bridge, MutationObserver, querySelector wrappers
  - `injected-actions.js` (≤15KB): On-demand actionability engine, geometry checks, auto-wait, AI snapshots. Loaded once, cached in page context. Embedded in Rust binary via `include_str!()`.

- **Network Interception Fix**: Fix three critical proxy failures: `route.continue()` returning 501 (implement passthrough forwarding), HTTPS CONNECT no-op (implement TCP tunneling), POST body never captured (read `Incoming` stream). Alternatively, switch to CDP `Fetch.enable` + `Fetch.requestPaused` for Chromium (eliminates proxy entirely).

- **12 Chromium Stub Implementations**: Wire `set_viewport_size`, `reload`, `go_back`, `go_forward`, `add_script_tag`, `add_style_tag`, `set_input_files`, `viewport_size`, `evaluate_handle`, `expose_function`, `wait_for_request`, `wait_for_response` with proper CDP commands. Each is 3-10 lines of CDP protocol calls.

- **Real Input Dispatch**: Replace `dispatchEvent(new MouseEvent(...))` synthetic events with CDP `Input.dispatchMouseEvent` / `Input.dispatchKeyEvent` / `Input.dispatchTouchEvent` for trusted `isTrusted=true` events. Affects `dblclick`, `right_click`, `hover`, `check`, `uncheck`, `press`, `press_sequentially`, `drag_and_drop`.

- **API Exports & page.on() Handlers**: Export `run_tests`, `expect`, `test` functions from `index.js`. Wire `page.on('dialog')`, `page.on('request')`, `page.on('console')`, `page.on('response')` to the CDP Event Dispatcher (currently no-ops).

## Capabilities

### New Capabilities

- `cdp-event-dispatcher`: CDP event subscription system with reactive URL caching, dialog/console/network event routing, and binding callback dispatching for injected.js bridge. (spec exists)
- `injected-script-engine`: Two-module hybrid injection (core + actions) with Rust-JS binding bridge, MutationObserver-based auto-wait, and cross-engine JS unification. (spec exists)
- `network-interception`: Fix the broken network proxy with HTTPS tunneling, `route.continue()` passthrough, POST body capture, plus CDP Fetch-based interception for Chromium. (spec exists)
- `chromium-stub-implementation`: Wire 12 stub methods with proper CDP commands — viewport control, navigation, script/style injection, file upload, history navigation.
- `real-input-dispatch`: Replace synthetic `dispatchEvent` with CDP `Input.dispatchMouseEvent` / `Input.dispatchKeyEvent` for trusted browser events.
- `api-exports-and-page-events`: Export all NAPI functions from `index.js`, wire `page.on()` event handlers to CDP dispatcher.

### Modified Capabilities

_(No existing capabilities are changing at the spec level — all work is new foundational infrastructure.)_

## Impact

- **`src/engine/chromium.rs`**: ~700 lines of inline JS replaced with injected.js binding calls. 12 stub methods wired with CDP commands. Synthetic event functions rewritten to use CDP `Input.*`. New `EventDispatcher` struct.
- **`src/engine/firefox.rs`**, **`src/engine/webkit.rs`**: Duplicated inline JS removed; shared injected.js handles all actionability/interaction logic.
- **`src/engine/mod.rs`**: `PageEngine` trait gains `inject_core_script()` and `register_binding()` methods.
- **`src/injection/`**: New module — `mod.rs` (InjectionManager), `scripts.rs` (embedded JS constants), `bindings.rs` (BindingRegistry with oneshot correlation), `stealth.rs` (runtime name randomization).
- **`js/src/`**: New TypeScript source files — `injected-core.ts`, `injected-actions.ts` with `selector.ts`, `actionability.ts`, `geometry.ts` utilities.
- **`js/dist/`**: Build output — `core.mjs` (≤2KB), `actions.mjs` (≤15KB).
- **`build.rs`**: Add JS dist file verification to ensure `include_str!()` paths resolve at compile time.
- **`src/network/proxy.rs`**: HTTPS CONNECT tunnel implementation, `Continue` passthrough forwarding, POST body capture. Or new `src/network/cdp_intercept.rs` for Chromium.
- **`src/page.rs`**: Wire `on('dialog')`, `on('request')`, `on('console')`, `on('response')` to event dispatcher.
- **`index.js`**, **`index.d.ts`**: Export `run_tests`, `expect`, `test`, `describe`, `beforeAll`, `afterAll`, `beforeEach`, `afterEach`.
- **`src/locator.rs`**: Replace polling-based `wait_for_actionability()` with injected.js MutationObserver call.
- **`src/assertions/engine.rs`**: Integrate assertion polling with injected.js stability detection.
- **`pnpm-workspace.yaml`**, **`package.json`**: Bundler dependency (rolldown) for JS minification.
- **New binary size**: +20KB from embedded injected scripts.
