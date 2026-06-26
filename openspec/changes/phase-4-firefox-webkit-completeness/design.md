## Context

Turbosheet currently implements `PageEngine` and `ContextEngine` trait methods across three browser backends:

- **Chromium** (CDP via `chromiumoxide`): 100% trait coverage, production-ready
- **Firefox** (W3C WebDriver via `geckodriver`): ~75% trait coverage — 7 `PageEngine` methods are no-op stubs
- **WebKit** (W3C WebDriver via `safaridriver`/`WebKitWebDriver`): ~80% trait coverage — same 7 methods as no-op stubs

The core limitation is that classic W3C WebDriver (HTTP JSON wire protocol) provides no event-driven primitives. Features like `wait_for_request`, `wait_for_response`, and `expose_function` require polling or workarounds. Mozilla's **WebDriver BiDi** protocol (standardized at W3C, supported in Firefox 134+) solves this by adding a WebSocket transport with event subscriptions and bidirectional command/event flow.

Firefox fully supports BiDi. WebKit (`safaridriver`) does **not** support BiDi as of Safari 18/macOS 15 — WebKit stubs will be implemented via classic WebDriver + evaluate-based workarounds.

## Goals / Non-Goals

**Goals:**

- Eliminate all no-op stubs in FirefoxPageEngine (7 methods → real implementations)
- Eliminate all no-op stubs in WebKitPageEngine (same 7 methods → evaluate-based workarounds)
- Fix `url()` to return actual page URL (currently placeholder `"about:blank"`)
- Implement `ContextEngine::pages()` for both engines to track live pages
- Add BiDi transport layer (WebSocket client, session management, BiDi command/event types)
- Implement `wait_for_request` / `wait_for_response` for Firefox via BiDi network events
- Implement `expose_function` / `register_binding` via BiDi `script.addPreloadScript`
- Implement `set_content` for both engines using navigation + document.write
- Implement `set_input_files` via WebDriver element upload endpoint
- `evaluate_handle` returns a serialized JS value wrapper
- Gate BiDi dependency behind optional `bidi` feature flag

**Non-Goals:**

- No changes to Chromium CDP engine (already feature-complete)
- No changes to the `PageEngine` / `ContextEngine` trait signatures
- No breaking JS API changes
- BiDi for WebKit is explicitly out of scope (not supported by safaridriver)
- Full BiDi spec implementation — only the commands needed for missing stubs
- No BiDi for Chromium (CDP is already superior for Chromium targets)
- No video recording support for Firefox/WebKit (requires CDP screencast API)

## Decisions

### D1: BiDi dependency — `tokio-tungstenite` behind `bidi` feature flag

- **Chosen**: Add `tokio-tungstenite = { version = "0.24", optional = true }` as an optional dependency
- **Why**: `tokio-tungstenite` is the standard async WebSocket crate in Rust ecosystem, works seamlessly with tokio runtime already in use, and handles TLS, ping/pong, and fragmentation out of the box
- **Feature wiring**: `bidi = ["tokio-tungstenite"]` in Cargo.toml; `firefox` feature implies `bidi` (since Firefox requires BiDi for network events)
- **Alternatives considered**: `tungstenite` (sync-only, would require blocking thread wrapper — inferior ergonomics). Native hyper upgrade (too low-level)

### D2: BiDi architecture — standalone module, not embedded in WebDriverClient

- **Chosen**: New `src/engine/bidi/` module with `BidiClient` struct managing its own WebSocket connection
- **Why**: BiDi has a fundamentally different transport model (WebSocket bidirectional, not HTTP request/response). Mixing it into `WebDriverClient`'s HTTP client would create confusing dual-transport abstractions. BiDi session is tied to a WebDriver session but the transport is independent.
- **Structure**:
  ```
  src/engine/bidi/
    mod.rs         — module def, re-exports
    transport.rs   — low-level WS connect, read/write frames, automatic reconnection
    client.rs      — BidiClient: send_command(), subscribe(), event_stream()
    types.rs       — BiDi command/event serde types (serde_json::Value wrappers)
    events.rs      — typed event enums (network, script, log, etc.)
  ```
- **Alternatives considered**: Embedding in `WebDriverClient` with a `BidiSession` variant — too coupled, breaks single-responsibility

### D3: URL caching strategy

- **Chosen**: Each `FirefoxPageEngine` / `WebKitPageEngine` holds `current_url: Arc<RwLock<String>>` initialized to `"about:blank"`
- **Updated on**: `goto()` (set to target URL), `reload()` (re-evaluate `window.location.href`), `go_back()`/`go_forward()` (re-evaluate after navigation)
- **Why**: W3C WebDriver current URL is only available via async GET `/url` command. Calling it synchronously is impossible. Caching gives synchronous `url()` access with O(1) cost. The cache is updated on every navigation event and can be manually refreshed via evaluate.
- **Edge case**: If user navigates via `evaluate("window.location='http://x'")`, the cache could be stale. Mitigation: `url()` first checks if the cached URL is `about:blank` (the initial state) and makes a real WebDriver call; otherwise returns cached value.

### D4: Firefox pages tracking

- **Chosen**: `FirefoxContextEngine` holds `pages: Arc<RwLock<Vec<Arc<dyn PageEngine>>>>`
- **Pages are added** in `new_page()` and removed in the page's `close()` via a weak callback or explicit deregistration
- **Why**: Required by `ContextEngine::pages()` trait. Without tracking, the method returns empty vec which breaks event dispatch.

### D5: `wait_for_request` / `wait_for_response` implementation

- **Firefox (BiDi)**: Subscribe to `network.beforeRequestSent`, `network.responseCompleted`, `network.fetchError` events. Each event carries request ID, URL, and metadata. `wait_for_request(url)` resolves when a `network.beforeRequestSent` event matches the URL. `wait_for_response(url)` resolves on `network.responseCompleted`. Uses `tokio::sync::oneshot` channels.
- **WebKit (fallback)**: No BiDi available. Use polling via `evaluate("performance.getEntriesByType('resource')")` to check if a URL appears in resource timing. Poll every 200ms with 30s timeout. Less precise but functional.
- **Why two paths**: BiDi is event-driven (instant, no polling). Classic WebDriver lacks push events entirely — polling is the only option.

### D6: `expose_function` / `register_binding` implementation

- **Firefox (BiDi)**: Use `script.addPreloadScript` to inject a function stub that connects to the BiDi `script.callFunctionOn` channel. The function's implementation is stored in a BiDi-managed map, called via `script.callFunctionOn` when invoked.
- **WebKit (fallback)**: Use `evaluate` to inject the function into `window`. Simpler but doesn't survive page navigations (unlike CDP's `addScriptToEvaluateOnNewDocument`). Mitigation: re-expose on `wait_for_selector` or navigation.
- **Why not WebDriver `execute/sync` for everything?**: `expose_function` needs to run in page context with callback support. Classic WebDriver `execute/async` can handle this but is clunky and has no persistence across navigations.

### D7: `set_content` implementation

- Both engines: Navigate to `"about:blank"` via WebDriver POST `/url`, then use `evaluate("document.open(); document.write(content); document.close()")` to inject the HTML. This preserves the page context and allows script execution.
- The WebDriver approach is consistent with how `WebDriverPage` from other frameworks (Playwright, Selenium) implements it.

### D8: `set_input_files` implementation

- Both engines: Use WebDriver POST `/element/{id}/upload` to send file path(s). This is the standard W3C WebDriver endpoint for file input elements.
- Unlike CDP's `DOM.setFileInputFiles`, WebDriver requires the file to exist on the machine running the WebDriver server. This is a documented limitation.

## Risks / Trade-offs

| Risk                                           | Impact                                                                             | Mitigation                                                                          |
| ---------------------------------------------- | ---------------------------------------------------------------------------------- | ----------------------------------------------------------------------------------- |
| Firefox BiDi protocol changes between versions | `network` and `script` module commands may change shape                            | Pin minimum Firefox version; integration tests; fallback to polling on BiDi failure |
| WebKit never gets BiDi support                 | WebKit will always have less precise event-driven features (polling-based waiters) | Acceptable — Safari is a secondary browser target. Document limitation in docs.     |
| URL cache desync                               | `url()` returns stale value after evaluate-based navigations                       | Mitigate with initial-state check; add `_refresh_url()` internal method             |
| BiDi WebSocket connection drops                | Mid-flight network monitoring breaks                                               | Add reconnection logic with exponential backoff; event buffer during reconnection   |
| `tokio-tungstenite` adds compile time          | New crate adds ~0.5s to cold builds                                                | Feature-gated behind `bidi`, only compiled when needed                              |
| Page tracking adds memory overhead             | Per-engine `Vec<Arc<dyn PageEngine>>` is negligible                                | Pages are `Arc`-shared — no extra allocation per read                               |
