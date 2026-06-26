## 1. Setup Build Pipeline and JavaScript Payload

- [x] 1.1 Create `js/` directory structure with `injected-core.js` and `injected-actions.js` placeholders
- [x] 1.2 Implement the build pipeline (e.g., using Rolldown) to minify JS into `js/dist/`
- [x] 1.3 Add `build.rs` step to ensure minified JS files exist before Rust compilation

## 2. Core Injection and Binding Bridge

- [x] 2.1 Update `PageEngine` trait with `inject_core_script` and `register_binding` methods
- [x] 2.2 Implement CDP Event Dispatcher for Chromium to handle navigation and binding callbacks
- [x] 2.3 Implement the `BindingRegistry` in Rust to handle request-response correlation (UUIDs)
- [x] 2.4 Wire injection logic into page/context creation hooks

## 3. Migrate PageEngine Actions

- [x] 3.1 Port actionability checks from `locator.rs` inline JS to `injected-actions.js`
- [x] 3.2 Update `ChromiumPageEngine` to replace `evaluate()` calls with injected binding invocations
- [x] 3.3 Update `FirefoxPageEngine` and `WebKitPageEngine` to use injected bindings
- [x] 3.4 Refactor `wait_for_actionability()` to use MutationObserver instead of polling

## 4. Network Interception

- [x] 4.1 Implement HTTPS CONNECT tunneling in the transparent proxy
- [x] 4.2 Fix route continuation to forward requests with modified properties (URL, headers)
- [x] 4.3 Ensure POST payloads are fully read and exposed to route handlers before forwarding

## 5. Security and Stealth

- [x] 5.1 Add runtime randomization for the binding name to avoid detection
- [x] 5.2 Implement strict IIFE wrapping and isolate state in the JS payload
- [x] 5.3 Verify script stealth using bot-evasion detection pages
