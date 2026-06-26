## 1. Cargo.toml & Feature Flag Setup

- [x] 1.1 Add `tokio-tungstenite = { version = "0.24", optional = true }` to `[dependencies]`
- [x] 1.2 Add `bidi = ["tokio-tungstenite"]` to `[features]`
- [x] 1.3 Make `firefox` feature imply `bidi`: `firefox = ["bidi"]`
- [x] 1.4 Run `cargo check` to verify feature resolution with all combinations

## 2. BiDi Transport Module

- [x] 2.1 Create `src/engine/bidi/mod.rs` with module structure and re-exports
- [x] 2.2 Implement `transport.rs` — `BidiTransport`: WebSocket connect, read/write JSON frames, ping/pong keepalive, automatic reconnection with exponential backoff
- [x] 2.3 Implement `types.rs` — BiDi command/event wire format types: `BidiCommand { id, method, params }`, `BidiResponse { id, result, error }`, `BidiEvent { method, params }`
- [x] 2.4 Implement `client.rs` — `BidiClient`: `connect(url)`, `send_command(method, params) -> Response`, `subscribe(events)`, unbuffered event stream, pending command tracking with oneshot channels
- [x] 2.5 Implement `events.rs` — typed BiDi event enums for `network.beforeRequestSent`, `network.responseCompleted`, `network.fetchError`, `script.message`
- [x] 2.6 Add `BidiClient::from_webdriver_session()` that extracts BiDi WebSocket URL from WebDriver capabilities and establishes connection
- [x] 2.7 Export `bidi` module conditionally: `#[cfg(feature = "bidi")] pub mod bidi;` in `src/engine/mod.rs`
- [x] 2.8 Run `cargo check --features "firefox,bidi"` to validate compilation

## 3. URL Caching for Firefox & WebKit

- [x] 3.1 Add `current_url: Arc<RwLock<String>>` field to `FirefoxPageEngine` (init `"about:blank"`)
- [x] 3.2 Update URL cache in `FirefoxPageEngine::goto()` after successful navigation
- [x] 3.3 Update URL cache in `FirefoxPageEngine::reload()`, `go_back()`, `go_forward()` by evaluating `window.location.href`
- [x] 3.4 Replace `fn url()` stub with cache read in `FirefoxPageEngine`
- [x] 3.5 Repeat steps 3.1–3.4 for `WebKitPageEngine`
- [x] 3.6 Run `cargo check` to validate

## 4. Page Tracking for Firefox & WebKit Contexts

- [x] 4.1 Add `pages: Arc<RwLock<Vec<Arc<dyn PageEngine>>>>` to `FirefoxContextEngine`
- [x] 4.2 Push new page into `pages` in `FirefoxContextEngine::new_page()`
- [x] 4.3 Implement `FirefoxContextEngine::pages()` to return the tracked pages
- [x] 4.4 Repeat steps 4.1–4.3 for `WebKitContextEngine` / `WebKitPageEngine`
- [x] 4.5 Run `cargo check` and verify no regressions

## 5. Simple Stubs — set_content & set_input_files & evaluate_handle

- [x] 5.1 Implement `FirefoxPageEngine::set_content(html)`: navigate to `about:blank`, set `document.documentElement.innerHTML`
- [x] 5.2 Implement `WebKitPageEngine::set_content(html)` — same approach as 5.1
- [x] 5.3 Implement `FirefoxPageEngine::set_input_files(selector, files)`: JS-based DataTransfer approach
- [x] 5.4 Implement `WebKitPageEngine::set_input_files(selector, files)` — same approach as 5.3
- [x] 5.5 Implement `FirefoxPageEngine::evaluate_handle(js)`: evaluate via WebDriver
- [x] 5.6 Implement `WebKitPageEngine::evaluate_handle(js)` — same approach as 5.5
- [x] 5.7 Run `cargo check` to validate

## 6. BiDi Network Event Stream + UrlWaiter

- [x] 6.1 Implement `UrlWaiter` struct: stores URL pattern (string match), oneshot sender, request type filter (request/response/error), and creation timestamp
- [x] 6.2 Implement `UrlWaiterRegistry` on `BidiClient`: `register(waiter) -> Receiver`, `resolve_matching(event)`, `cleanup_expired(timeout)`
- [x] 6.3 Subscribe to `network.beforeRequestSent`, `network.responseCompleted`, `network.fetchError` on `BidiClient` creation
- [x] 6.4 Route incoming BiDi network events through the waiter registry to resolve matching waiters
- [x] 6.5 Run `cargo check --features "bidi,firefox"` to validate

## 7. Firefox BiDi-based wait_for_request & wait_for_response

- [x] 7.1 Add optional `bidi_client: Option<Arc<BidiClient>>` to `FirefoxPageEngine` (behind `#[cfg(feature = "bidi")]`)
- [x] 7.2 Connect BiDi when creating `FirefoxPageEngine` (if `bidi` feature is enabled)
- [x] 7.3 Implement `wait_for_request(url)`: use BiDi `UrlWaiter` with `network.beforeRequestSent` event; if BiDi unavailable, fall back to polling
- [x] 7.4 Implement `wait_for_response(url)`: use BiDi `UrlWaiter` with `network.responseCompleted` event; if BiDi unavailable, fall back to polling
- [x] 7.5 Add fallback polling implementation using evaluate + `performance.getEntriesByType('resource')` for when BiDi is not available
- [x] 7.6 Run `cargo check --features "firefox,bidi"` to validate

## 8. Firefox BiDi-based expose_function & register_binding

- [x] 8.1 Implement `BidiClient::add_preload_script(js) -> ScriptId` using BiDi `script.addPreloadScript` command
- [x] 8.2 Implement `BidiClient::remove_preload_script(script_id)` using BiDi `script.removePreloadScript`
- [x] 8.3 Implement `BidiClient::call_function(function_declaration, args) -> Result` using BiDi `script.callFunctionOn`
- [x] 8.4 Implement `FirefoxPageEngine::expose_function(name, js_body)`: add preload script that creates `window[name]` calling the JS body
- [x] 8.5 Implement `FirefoxPageEngine::register_binding(name)`: add preload script that exposes `window[name]` as a callable binding
- [x] 8.6 Run `cargo check --features "firefox,bidi"` to validate

## 9. WebKit Polling-based Waiters & Script Features

- [x] 9.1 Implement `WebKitPageEngine::wait_for_request(url)`: poll `performance.getEntriesByType('resource')` every 200ms for 30s
- [x] 9.2 Implement `WebKitPageEngine::wait_for_response(url)`: same polling approach as 9.1
- [x] 9.3 Implement `WebKitPageEngine::expose_function(name, js_body)`: inject via evaluate (`window[name] = ...`)
- [x] 9.4 Implement `WebKitPageEngine::register_binding(name)`: inject via evaluate
- [x] 9.5 Run `cargo check` to validate

## 10. Integration & Testing

- [x] 10.1 Add unit tests for `BidiClient` command/response matching (mock WebSocket)
- [x] 10.2 Add unit tests for `UrlWaiter` registration, matching, and timeout cleanup
- [x] 10.3 Add integration test: launch Firefox via turbosheet, call `goto`, `set_content`, `url()`, assert URL is correct
- [x] 10.4 Add integration test: Firefox `set_input_files` with a temp file
- [x] 10.5 Add integration test: Firefox `wait_for_request` with a fetch from evaluated JS
- [x] 10.6 Verify full crate compiles with `--features "chromium,bidi,video,traces,visual"` — pre-existing `visual` feature errors (unrelated)
- [x] 10.7 Verify crate compiles with `--no-default-features` (minimal) ✅
- [x] 10.8 Verify crate compiles with `--features "webkit"` (WebKit only, no BiDi) ✅
