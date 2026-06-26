## Context

TurboSheet currently has 12/43 features working vs Playwright's 39/43. Three critical subsystems are broken (not just missing — they actively fail for users):

1. **CDP Event Dispatcher**: The `_handler_task` in `ChromiumEngine::launch()` iterates over `handler` events but discards them all — no dispatching, no routing, no processing. This blocks 6+ downstream features including injected.js bindings, dialog handling, console capture, network interception, URL tracking, and multi-page support.

2. **Network Proxy (`src/network/interceptor.rs`)**: The proxy is a stub with event bus infrastructure but no actual proxy implementation. Three critical failures: `route.continue()` returns HTTP 501, HTTPS CONNECT requests are no-ops, POST bodies are never read.

3. **Chromium Stubs**: Four methods (`evaluate_handle`, `add_script_tag`, `add_style_tag`, `expose_function`) return `Ok(())` doing nothing. 8+ more methods have incomplete implementations relying on synthetic `dispatchEvent` JS.

All three engines (Chromium, Firefox, WebKit) duplicate ~1500 lines of inline JS via `format!("(function() {{ ... }})()")` patterns, creating a 3x maintenance burden and fragile string-construction anti-patterns.

The codebase is Rust-native using `chromiumoxide` crate for CDP, NAPI bindings for Node.js, with a 416MB binary. There is no existing test suite for these subsystems.

## Goals / Non-Goals

**Goals:**

- Implement a reactive CDP event dispatcher that subscribes to and routes 10+ CDP event types to registered handlers
- Implement the injected-script-engine (Phase 0-2): build pipeline, `injected-core.js` (≤2KB), `injected-actions.js` (≤15KB), `InjectionManager` in Rust, `BindingRegistry` with oneshot correlation
- Fix the network proxy: HTTPS CONNECT tunneling, `route.continue()` passthrough, POST body capture + add CDP Fetch-based interception for Chromium
- Wire the 12 Chromium stub methods with proper CDP protocol commands
- Replace synthetic `dispatchEvent(new MouseEvent(...))` with CDP `Input.dispatchMouseEvent` / `Input.dispatchKeyEvent`
- Export missing NAPI functions (`run_tests`, `expect`, `test`, etc.) and wire `page.on()` event handlers

**Non-Goals:**

- NOT migrating Firefox/WebKit engines to use injected.js (Phase 2 deferred — this change focuses on Chromium + build infrastructure)
- NOT implementing stealth mode (deferred to Phase 3)
- NOT adding Firefox/WebKit-specific network proxy implementations
- NOT implementing the full auto-wait system overhaul
- NOT adding test suite for existing features (only new infrastructure)
- NOT reducing the 416MB binary size (injected scripts add ≤20KB)

## Decisions

### Decision 1: CDP Fetch.enable vs Proxy Server for Network Interception

**Option A (proxy server)**: Continue with the current `hyper`-based proxy at `src/network/interceptor.rs`. Implement HTTPS CONNECT tunneling, `Continue` passthrough, POST body capture.
**Option B (CDP Fetch domain)**: Use `Fetch.enable` + `Fetch.requestPaused` for Chromium, eliminating the proxy entirely. Simpler, more reliable, but Chromium-only.
**Decision: Both.** Use CDP `Fetch.enable` as the primary path for Chromium (eliminates proxy complexity for >80% of users). Keep the proxy server for Firefox/WebKit and for Chromium users who need it. Implement the proxy fixes (HTTPS tunnel, Continue, POST body) but at lower priority.

**Rationale**: CDP Fetch domain is battle-tested by Playwright, eliminates the TCP proxy server entirely for Chromium, and integrates naturally with the event dispatcher. The proxy remains as a fallback for non-CDP engines.

### Decision 2: Injection Timing for Core Script

**Option A**: `addScriptToEvaluateOnNewDocument` (CDP) — auto-injects into every new document/frame before any page script runs.
**Option B**: `Runtime.evaluate` on page creation — manual injection after page navigates.
**Decision: Option A** for core script, Option B for actions payload.

**Rationale**: `addScriptToEvaluateOnNewDocument` ensures the binding bridge exists before page JS executes, preventing race conditions. The actions payload (15KB) should be lazy-loaded only when first action is invoked.

### Decision 3: Binding Bridge Protocol

**Option A**: Streaming channel-based protocol with multiple binding names.
**Option B**: Request-response UUID correlation via single binding (`__tsReport`).
**Decision: Option B** — one-shot UUID correlation with `DashMap<RequestId, oneshot::Sender>`.

**Rationale**: Simpler to implement and debug. Streaming adds ordering complexity. Each action is inherently request-response (click → result), so oneshot channels map cleanly. For streaming events (console, dialogs), separate dedicated bindings can be added later.

### Decision 4: Error Handling for Stubs

**Option A**: Return `TurbosheetError::NotImplemented` for all stubs (fail fast).
**Option B**: Implement minimal CDP calls that work for common cases.
**Decision: Option B** — implement with proper CDP commands, each 3-10 lines. `evaluate_handle` needs `Runtime.callFunctionOn`, `add_script_tag` needs `Page.addScriptToEvaluateOnNewDocument`, `expose_function` uses `Runtime.addBinding`, etc.

**Rationale**: Users hitting these already get `Ok(())` — silent failures. Proper implementations are small and prevent wasted debugging time. Each method maps 1:1 to a CDP command.

### Decision 5: Synthetic Events → CDP Input Dispatch

**Option A**: Replace all synthetic `dispatchEvent` calls with CDP `Input.dispatchMouseEvent` / `Input.dispatchKeyEvent`.
**Option B**: Keep synthetic events but try to make `isTrusted` true via CDP workarounds.
**Decision: Option A**. CDP Input dispatch is the correct approach — it produces trusted events that behave identically to user input.

**Rationale**: Synthetic events with `isTrusted=false` fail on real-world websites that check `event.isTrusted` (anti-fraud, analytics, React synthetic event delegation). CDP Input dispatch is Playwright's approach and is reliable.

### Decision 6: JS Build Pipeline

**Option A**: Use `rolldown` (native bundler, already in Vite+ toolchain).
**Option B**: Use `esbuild` (well-known but external dependency).
**Option C**: Manual minification (no dependency, error-prone).
**Decision: Option A** — `rolldown` via existing `vp` toolchain.

**Rationale**: The Vite+ toolchain (`vp`) is already configured for this project. Adding `rolldown` via the existing workflow avoids introducing new dependency management overhead. Build outputs go to `js/dist/` and are verified by `build.rs`.

## Risks / Trade-offs

| Risk                                                                    | Severity | Mitigation                                                                                                                                  |
| ----------------------------------------------------------------------- | -------- | ------------------------------------------------------------------------------------------------------------------------------------------- |
| CDP Fetch domain conflicts with existing proxy code                     | Medium   | Feature-gate: use Fetch by default, fall back to proxy if Fetch.enable fails                                                                |
| Binding message lost during navigation                                  | High     | Re-register binding after each `frameNavigated` event. Add timeout to oneshot receiver with user-visible warning                            |
| `addScriptToEvaluateOnNewDocument` not supported on all Chrome versions | Low      | Fall back to `Runtime.evaluate` on page creation — detect via CDP version negotiation                                                       |
| Event dispatcher memory leak from unregistered handlers                 | Medium   | Use `WeakHashMap` or explicit `unregister()` method on `PageEngine::close()`                                                                |
| Injected script breaks on pages with strict CSP                         | Medium   | Test against CSP-locked pages. Use nonce injection if available, or fall back to inline `Runtime.evaluate`                                  |
| Network proxy changes break existing traffic                            | High     | Implement new CDP Fetch path in parallel, keep proxy untouched until CDP path is verified. No changes to existing proxy until CDP is stable |
