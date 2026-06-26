## Why

TurboSheet has a **structural advantage** — Rust-native core, NAPI-rs bridge, Tokio concurrency, swarm grid, 40+ device descriptors, component testing, migration tools, trace viewer, visual comparison, and a plugin system. The PRD v3 deep-dive codebase audit (1,390 lines) reveals that **39 of 43 competitive features are broken, stubbed, or missing** compared to Playwright's 39/43 score.

Four OpenSpec changes are in-flight (`best-in-class-e2e`, `next-gen-architecture`, `turbosheet-complete-browser-testing`, `turbosheet-gap-analysis`), but they miss the **foundational tier**: CDP event subscription system, hierarchical ownership model, network proxy fixes, real input dispatch with trusted events, and a proper injected script engine. Without these, every feature above them is built on sand.

Additionally, **30+ improvement opportunities** are not captured in any spec — AI-native differentiators (`getAgentSnapshot()`, self-healing selectors), developer experience (interactive `--ui` mode, watch mode, fixture system), observability (trend dashboards, Allure integration), and advanced testing paradigms (Core Web Vitals, API+UI integration, mobile cloud, clock mocking, test impact analysis).

This change consolidates everything missing into a single coordinated roadmap that bridges the gap from TurboSheet's current **12/43 feature score** to **43/43 — surpassing Playwright**.

## What Changes

### Foundation Layer (Tier 0) — Prerequisites, None in Any Spec

1. **CDP Event Dispatcher** — Replace the current event handler that drops all events with a proper `EventDispatcher` routing `Page.frameNavigated`, `Runtime.consoleAPICalled`, `Runtime.bindingCalled`, `Network.requestWillBeSent`, `Network.responseReceived`, `Page.javascriptDialogOpening`, `Target.targetCreated`, `Browser.downloadWillBegin` to registered handlers. This is the PREREQUISITE for injected.js binding bridge, dialog handling, console capture, network interception, popup detection, URL tracking, and download handling.

2. **Hierarchical Ownership Model** — Replace three global `DashMap` registries (`BROWSERS`, `CONTEXTS`, `PAGES`) with a proper tree: `Browser` owns `Context` owns `Page`, with `Weak` backlinks. Enables `context.pages()` to return real pages, automatic cleanup on close, and memory leak prevention.

3. **Network Proxy Rewrite** — Fix three critical failures: HTTPS CONNECT tunnel that drops upgraded connections, `route.continue()` that returns HTTP 501, and POST body never captured (`post_data: None` always). Add proper MitM certificate handling for HTTPS interception when not using CDP Fetch.

4. **Real Input Dispatch** — Replace synthetic `dispatchEvent(new MouseEvent(...))` with CDP `Input.dispatchMouseEvent`, `Input.dispatchKeyEvent`, `Input.dispatchTouchEvent`. This is required for `event.isTrusted === true`, which payment forms, CAPTCHAs, and anti-bot systems depend on.

5. **Selector Escaping Security Fix** — Replace ad-hoc `selector.replace("'", "\\'")` with `serde_json` serialization across all three engines, eliminating injection vectors.

### Injected Script Engine (PRD v3 §1-11) — Partial, Needs Completion

6. **Injected Core Script** — Complete the `injected-core.ts` module (binding establishment, MutationObserver for stability detection, querySelector wrappers, frame tracking) and wire it into all three engines via `Page.addScriptToEvaluateOnNewDocument` (CDP), `script.addPreloadScript` (BiDi), and `UserContentManager.addScript` (WebKit).

7. **Injected Actions Script** — Complete `injected-actions.ts` with full actionability engine (visibility, stability, enabled, pointer-events), geometry checks, smart auto-wait with MutationObserver + exponential backoff, drag-and-drop, file input, and AI agent DOM snapshot generation.

8. **Binding Bridge** — Implement `BindingRegistry` with request-response correlation via UUIDs and `DashMap<UUID, oneshot::Sender>`. Wire `Runtime.addBinding` for CDP, protocol-equivalent for Firefox/WebKit.

### Cross-Cutting Gaps (Tier 1-4 Enhancements) — Missing or Partial in Specs

9. **Frame/IFrame Support** — `page.frame(name)`, `page.frames()`, `page.frameLocator(selector)`, per-frame execution context tracking, auto-injection per frame.

10. **Multi-Page/Popup Support** — `context.waitForEvent('page')`, popup detection via `Target.targetCreated`, `page.opener()`.

11. **Cookie and Storage Management** — Real CDP implementations of `addCookies`, `cookies`, `clearCookies`, plus `storageState()` for auth serialization.

12. **Download Handling** — `download.path()`, `download.saveAs(path)`, `download.cancel()`, `page.waitForEvent('download')` via `Browser.downloadWillBegin` / `Browser.downloadProgress`.

13. **Video Recording** — `context.newPage({ recordVideo: { dir: './videos' } })` with frame-by-frame capture via `Page.startScreencast` and WebM/MP4 encoding.

14. **Auto-Install Chrome for Testing** — Download CfT via `https://googlechromelabs.github.io/chrome-for-testing/known-good-versions.json` JSON API.

15. **HAR Recording** — HTTP Archive file generation from CDP network events, `context.newPage({ recordHar: { path: 'trace.har' } })`.

16. **Code Generation (Record & Replay)** — Use CDP event stream + injected.js to record user interactions and generate test code in multiple framework syntaxes (TurboSheet, Playwright, Cypress).

17. **File Chooser Handling** — `page.on('filechooser')` event via CDP `Page.fileChooserOpened`, `fileChooser.setFiles()`.

### Unique Differentiators (G1-G30) — Not in Any Spec

18. **AI-Native State Extraction** — `page.getAgentSnapshot()` returning Rust-optimized accessibility tree / visual DOM instead of raw HTML, reducing LLM token consumption by 60%+.

19. **Self-Healing Selectors** — When a selector fails, use AI/heuristics to find the best alternative element (by proximity, text similarity, ARIA role, visual position). Log the healing event for user review.

20. **BiDi Stream Pre-Processor** — Rust-native event filter that ingests massive WebDriver BiDi streams, filters noise, condenses state, and only passes finalized data across the FFI bridge. Keeps Node.js event loop clear.

21. **AI-Powered Flake Diagnosis** — Auto-classify test failures (timeout, assertion, network, browser crash) and suggest fixes with confidence scores.

22. **Interactive `--ui` Mode** — Electron/React-based test runner UI with test list, live status, log streaming, screenshot preview, and interactive re-run.

23. **Watch Mode** — File watcher that auto-re-runs affected tests on source changes, with debounce and test-scope detection.

24. **Playwright-Style Typed Fixture System** — `test.extend<{ auth: AuthPage }>({ auth: async ({ page }, use) => { ... } })` with type-safe fixture injection and automatic setup/teardown.

25. **Core Web Vitals Measurement** — Native LCP, INP, CLS, FID, TTFB measurement via `PerformanceObserver` injection, exposed as `page.getWebVitals()`.

26. **API + UI Integration Testing** — Embedded HTTP client in the same test suite for testing API contracts alongside UI interactions (no separate SuperTest/Playwright setup).

27. **Accessibility Testing** — `expect(page).toPassAxe()` with axe-core integration, configurable impact levels, snapshot-based regression.

28. **Mobile Device Cloud Integration** — `BrowserStack` / `SauceLabs` / `LambdaTest` connector for running tests on real iOS/Android devices.

29. **Clock Mocking / Fake Timers** — `page.clock.setFixedTime(date)`, `page.clock.install()`, `page.clock.fastForward(ms)` for deterministic time-dependent tests.

30. **Test Impact Analysis** — Parse coverage data + git diff to determine which tests to run based on code changes, reducing CI time by 60%+ on large suites.

31. **Snapshot Testing** — `expect(value).toMatchSnapshot(name)` with inline and external snapshot storage, snapshot review CLI, and CI-friendly update mode.

32. **Flaky Test Dashboard** — Aggregate flakiness across CI runs, surface most-flaky tests, track flakiness trends over time, auto-retry with different strategies.

## Capabilities

### New Capabilities

- `cdp-event-dispatcher`: Full CDP event subscription system routing 12+ event types to registered handlers via `EventDispatcher` with channel-based dispatch
- `hierarchical-ownership`: Tree-based Browser→Context→Page ownership replacing global DashMap registries
- `network-proxy-rewrite`: Complete HTTPS CONNECT tunneling, route.continue passthrough, POST body capture
- `real-input-dispatch`: CDP Input.dispatchMouseEvent/KeyEvent/TouchEvent replacing synthetic dispatchEvent
- `injected-script-engine-complete`: Full injected-core + injected-actions + BindingRegistry across all 3 engines
- `frame-iframe-support`: Per-frame execution context tracking, auto-injection, frameLocator API
- `multi-page-popup`: Popup detection, context.waitForEvent('page'), page.opener()
- `cookie-storage-management`: Real CDP cookie/storage CRUD, storageState serialization
- `download-handling`: download lifecycle (path, saveAs, cancel, waitForEvent)
- `video-recording`: Page.startScreencast-based frame capture, WebM encoding
- `auto-install-browser`: Chrome for Testing auto-download via JSON API
- `har-recording-playback`: HTTP Archive file generation and replay
- `code-generation`: Record-and-replay test generation in multiple framework syntaxes
- `file-chooser-handling`: Page.on('filechooser'), fileChooser.setFiles()
- `ai-agent-snapshot`: page.getAgentSnapshot() with Rust-optimized accessibility tree
- `self-healing-selectors`: AI-driven fallback selectors with healing event logging
- `bidi-stream-preprocessor`: Rust-native WebDriver BiDi event filter/condenser
- `ai-flake-diagnosis`: Auto-classification of test failures with fix suggestions
- `interactive-ui-mode`: Electron/React-based --ui mode for test development
- `watch-mode`: File watcher with scope detection and auto re-run
- `typed-fixture-system`: test.extend-based typed fixtures with lifecycle
- `core-web-vitals`: LCP, INP, CLS, FID, TTFB measurement via PerformanceObserver
- `api-ui-integration-testing`: Embedded HTTP client in test suite
- `accessibility-testing`: axe-core integration with configurable impact levels
- `mobile-cloud-integration`: BrowserStack/SauceLabs/LambdaTest connector
- `clock-mocking`: Fake timers with install/fastForward/setFixedTime
- `test-impact-analysis`: Coverage + git diff → test selection
- `snapshot-testing`: toMatchSnapshot with inline/external storage
- `flaky-test-dashboard`: Cross-run flakiness aggregation and trending

### Modified Capabilities

- `selector-engine` (from best-in-class-e2e): Extend with self-healing fallback when primary selector fails
- `assertion-engine` (from best-in-class-e2e): Add soft assertions, toPassAxe, toMatchVitals, toMatchSnapshot
- `network-interception` (from next-gen-architecture): CDP Fetch domain replaces proxy; proxy kept as Firefox/WebKit fallback
- `trace-viewer-ui` (from next-gen-architecture): Add AI flake annotations and video timeline
- `swarm-execution` (from turbosheet-complete-browser-testing): Add health-monitoring-based auto-retry and tenant-aware scheduling
- `complete-locator-api` (from turbosheet-complete-browser-testing): Add frame-aware locators, self-healing fallback
- `test-runner-dx` (from turbosheet-gap-analysis): Add --ui flag, --watch flag, typed fixture system, test impact analysis
- `vscode-extension` (from turbosheet-complete-browser-testing): Integrate with --ui mode, flaky dashboard, live trace viewer

## Impact

- **Core Engine**: `src/engine/chromium.rs` — Major refactor: CDP event dispatcher, real input dispatch, injected script wiring, stub elimination. `src/engine/firefox.rs` + `src/engine/webkit.rs` — Injected script wiring, BiDi pre-processor integration.
- **Architecture**: New `src/injection/` module (manager, scripts, bindings, stealth), new `src/events/` module (EventDispatcher), new `src/ownership/` or refactor of global registries.
- **Network**: `src/network/proxy.rs` — Full rewrite of HTTPS tunnel and continue passthrough. New `src/network/cdp_intercept.rs` — CDP Fetch domain interception.
- **New Modules**: `src/ai/` (agent snapshot, self-healing selectors, flake diagnosis), `src/recording/` (video capture), `src/clock/` (fake timers), `src/vitals/` (Core Web Vitals), `src/accessibility/` (axe-core integration), `src/impact/` (test impact analysis), `src/snapshot/` (snapshot testing).
- **Frontend**: `packages/ui-mode/` — New Electron/React app for interactive `--ui`. `packages/trace-viewer/` — Enhanced with video timeline and flake annotations.
- **CLI**: New flags: `--ui`, `--watch`, `--impact`, `--update-snapshots`. New commands: `npx tsheet codegen`, `npx tsheet flaky-dashboard`.
- **Dependencies**: `axe-core` (accessibility), `fluent-ffmpeg` or Rust-native encoder (video), `chokidar` (watch mode), `electron` (UI mode).
- **Score Projection**: 12/43 → 43/43 (surpasses Playwright)
