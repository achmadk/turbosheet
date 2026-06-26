## Why

The turbosheet browser automation library supports three browser engines—Chromium, Firefox, and WebKit—but only Chromium has full feature parity. Firefox and WebKit (via W3C WebDriver) have significant no-op stubs (~75% and ~80% of `PageEngine` trait methods implemented respectively), limiting their usefulness for cross-browser testing. Additionally, the codebase lacks a WebDriver BiDi transport layer, which is the modern path for event-driven features like network interception, console monitoring, and request/response waiting on non-Chromium browsers.

## What Changes

- Complete all `PageEngine` no-op stubs for **FirefoxPageEngine** (7 methods: `set_input_files`, `wait_for_request`, `wait_for_response`, `evaluate_handle`, `expose_function`, `set_content`, `register_binding`)
- Complete all `PageEngine` no-op stubs for **WebKitPageEngine** (same 7 methods)
- Fix `url()` to return the actual current URL (currently returns `"about:blank"` placeholder) by caching it on navigation
- Implement `ContextEngine::pages()` for both engines to track live pages properly
- Add `tokio-tungstenite` dependency and implement **WebDriver BiDi** transport (WebSocket connection, BiDi session management, event stream)
- Rewrite `BidiConnection` stub into a functional BiDi client that enables event-driven features (console events, network events) for Firefox
- Implement `wait_for_request` / `wait_for_response` for Firefox/WebKit via BiDi network events
- Implement `expose_function` using BiDi `script.addPreloadScript` + Runtime.callFunctionOn
- Implement `set_content` using WebDriver navigation + document.write pattern
- Implement `set_input_files` using WebDriver element upload endpoint
- Implement `register_binding` using BiDi `script.addPreloadScript`
- `evaluate_handle` returns a serialized handle identifier via evaluate result
- No breaking API changes — all existing interfaces remain unchanged

## Capabilities

### New Capabilities

- **bidi-transport**: WebSocket-based BiDi client for Firefox, handling session creation, command/event framing, and automatic reconnection
- **firefox-engine-completeness**: Full `PageEngine` and `ContextEngine` implementation for Firefox, removing all no-op stubs
- **webkit-engine-completeness**: Full `PageEngine` and `ContextEngine` implementation for WebKit (Safari), removing all no-op stubs
- **bidi-network-event-stream**: Event-driven network monitoring via BiDi `network.beforeRequestSent`, `network.responseCompleted`, and `network.fetchError` events, enabling `wait_for_request`/`wait_for_response`
- **bidi-script-execution**: BiDi `script.addPreloadScript` and `script.callFunctionOn` for function exposure and binding registration

### Modified Capabilities

- `network-interception`: Extend network interception to work with WebDriver BiDi (currently only works with Chromium's CDP Fetch domain)

## Impact

- **New dependency**: `tokio-tungstenite` for WebSocket connections to BiDi-enabled WebDriver instances
- **New module**: `src/engine/bidi/` — BiDi transport layer (client, session, event types, command types)
- **Modified modules**:
  - `src/engine/firefox.rs` — implement stubs, add BiDi integration
  - `src/engine/webkit.rs` — implement stubs (WebKit does not yet support BiDi, so classic WebDriver fallback)
  - `src/engine/webdriver.rs` — expand WebDriverClient with session management and page tracking utilities
  - `src/engine/mod.rs` — no changes (trait already defines the methods)
- **Feature flag**: `bidi` (optional, default-on for `firefox`) — gates BiDi transport dependency
- **Config**: `Cargo.toml` — add `tokio-tungstenite` behind optional `bidi` feature, add `bidi` feature to default or firefox feature set
