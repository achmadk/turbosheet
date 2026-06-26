## Phase 0: Foundation (Weeks 1-4)

### 0.1 CDP Event Dispatcher

- [x] 0.1.1 Create `src/events/mod.rs` with `EventDispatcher` struct containing channel-based handler registries for 12+ CDP event types
- [x] 0.1.2 Implement `EventDispatcher::handle_event(event: CdpEvent)` with match arms for: `Page.frameNavigated`, `Page.frameDetached`, `Page.domContentEventFired`, `Page.loadEventFired`, `Page.javascriptDialogOpening`, `Runtime.consoleAPICalled`, `Runtime.bindingCalled`, `Network.requestWillBeSent`, `Network.responseReceived`, `Network.loadingFinished`, `Target.targetCreated`, `Browser.downloadWillBegin`, `Page.screencastFrame`, `Page.fileChooserOpened`
- [x] 0.1.3 Replace the current handler task in `chromium.rs:55-62` that drops all events with one that feeds into `EventDispatcher`
- [x] 0.1.4 Add `url_cache: RwLock<String>` to `ChromiumPageEngine` that updates reactively from `Page.frameNavigated` events — eliminates `evaluate("window.location.href")` round-trip
- [x] 0.1.5 Add channel-based subscriber API: `event_dispatcher.subscribe::<FrameNavigated>(channel)` with typed unsubscribe
- [x] 0.1.6 Wire `page.on('request')` and `page.on('response')` to CDP `Network.requestWillBeSent` / `Network.responseReceived` via EventDispatcher channels
- [x] 0.1.7 Add tests: CDP events are received and dispatched correctly

### 0.2 Hierarchical Ownership Model

- [x] 0.2.1 Replace `BROWSERS: DashMap<String, Arc<ChromiumEngine>>` with proper `Browser { id, engine: Arc<dyn BrowserEngine>, contexts: RwLock<Vec<Arc<Context>>> }`
- [x] 0.2.2 Replace `CONTEXTS: DashMap<String, Context>` with `Context { id, browser: Weak<Browser>, pages: RwLock<Vec<Arc<Page>>> }`
- [x] 0.2.3 Replace `PAGES: DashMap<String, Page>` with `Page { id, context: Weak<Context>, engine: Arc<dyn PageEngine> }`
- [x] 0.2.4 Implement `Drop` for Browser: closes all contexts. Implement `Drop` for Context: closes all pages.
- [x] 0.2.5 Implement `context.pages()` returning real page list (not empty `vec![]`)
- [x] 0.2.6 Implement `browser.contexts()` returning real context list
- [x] 0.2.7 Add tests: cascade close, memory leak prevention, concurrent access

### 0.3 Network Proxy Rewrite

- [x] 0.3.1 Fix HTTPS CONNECT handler: spawn bidirectional TCP tunnel between upgraded connection and target host
- [x] 0.3.2 Fix `RouteAction::Continue`: forward request to original destination via `reqwest` client, apply `ContinueOptions` (URL rewrite, header modification, POST data override)
- [x] 0.3.3 Fix POST body capture: call `req.collect().await.to_bytes()` before routing, populate `post_data` field
- [x] 0.3.4 Add MitM certificate generation (self-signed CA + per-domain certs) for HTTPS interception when using proxy mode
- [x] 0.3.5 Add `useProxy: bool | ProxySettings` config option to toggle between proxy and direct CDP Fetch mode
- [x] 0.3.6 Add tests: HTTPS CONNECT authority parsing, MitM cert generation (3 tests), route-matching via proxy (4 tests)

### 0.4 Real Input Dispatch

- [x] 0.4.1 Replace `dispatchEvent(new MouseEvent(...))` for click/dblclick/right_click with CDP `Input.dispatchMouseEvent` with `type: mousePressed` + `type: mouseReleased`
- [x] 0.4.2 Implement `hover` via CDP `Input.dispatchMouseEvent(type: mouseMoved)`
- [x] 0.4.3 Implement `press` and `press_sequentially` via CDP `Input.dispatchKeyEvent` with `rawKeyDown`, `char`, `keyUp`
- [x] 0.4.4 Add touch event dispatch (`Input.dispatchTouchEvent`) for mobile emulation
- [x] 0.4.5 Remove `dispatchEvent`-based synthetic event code from user interaction methods in `chromium.rs`
- [x] 0.4.6 Verify `event.isTrusted === true` on all dispatched interactions (CDP Input domain always dispatches trusted events by design; well-established in CDP spec)
- [x] 0.4.7 Add tests: trusted event flag (click, hover, dblclick, rightClick, keydown, tap all verified isTrusted=true), CSS mousedown/mouseenter triggers (active/hover equivalents)

### 0.5 Selector Escaping Security Fix

- [x] 0.5.1 Audit all `selector.replace("'", "\\'")` patterns across `chromium.rs`, `firefox.rs`, `webkit.rs`
- [x] 0.5.2 Replace all instances with `serde_json::to_string(&selector)` → `document.querySelector({})` (JSON string → valid JS string literal)
- [x] 0.5.3 Add tests: selectors with quotes, backslashes, template literal backticks, Unicode

## Phase 0.5: Current Code Correctness (Weeks 4-6, parallel with Phase 1)

### 0.5.1 Migration Adapter Bugfixes

#### Puppeteer

- [ ] 0.5.1.1 Add missing API mappings to `migrate_puppeteer()`: `page.emulateNetworkConditions()`, `page.setExtraHTTPHeaders()`, `page.screencast()`, `page.setCookie()`, `page.deleteCookie()`, `page.cookies()`, `page.workers()`, `page.frames()`
- [ ] 0.5.1.2 Fix `migrate_puppeteer_file()`: scoped replacements for `type: 'png'`, `type: 'jpeg'`, `width:`, `height:`, `deviceScaleFactor:` — change from global `.replace()` to context-aware replacements that only fire inside `screenshot()` and `setViewport()` blocks respectively
- [ ] 0.5.1.3 Fix `migrate_puppeteer()`: `page.emulate()` mapping marked `automated: true` but no string replacement exists in `migrate_puppeteer_file()` — set to `automated: false` with a todo, or add the replacement

#### Cypress

- [ ] 0.5.1.4 Fix `migrate_cypress_file()`: `cy.get(` and `cy.contains(` replacements produce mismatched quotes when original uses single quotes — replace `cy.get(` → `await page.locator(` (no added quotes) and wrap the argument in backticks or preserve the original quote style
- [ ] 0.5.1.5 Fix `migrate_cypress_file()`: `cy.wait(number)` → `await page.waitForTimeout(number)` conversion with auto-wait comment for selector-based waits
- [ ] 0.5.1.6 Fix `migrate_cypress_file()`: `cy.intercept()` callback replacement `}, (req, res) => {` → `, (route) => {\n  route.fulfill({` produces mismatched braces and breaks syntax — add proper function body wrapping

#### Playwright

- [ ] 0.5.1.7 Fix `migrate_playwright_file()`: `page.$$eval` → `page.evaluate` is an oversimplification ($$eval returns array over all matches, evaluate has different signature) — add appropriate wrapper or comment
- [ ] 0.5.1.8 Add missing API mappings to `migrate_playwright()`: `page.addInitScript()`, `page.addStyleTag()`, `page.addScriptTag()`, `page.pause()`
- [ ] 0.5.1.9 Fix `migrate_playwright_file()`: `from 'playwright'` → `from 'turbo-sheet'` should be `from 'tsheet'` for consistency

### 0.5.2 Stub API Completion — Eliminate Silent No-Ops

- [ ] 0.5.2.1 Implement `ChromiumEngine::close()`: send CDP `Browser.close`, terminate child process (SIGTERM → SIGKILL after 5s), reap zombie, mark engine as closed
- [ ] 0.5.2.2 Implement `ChromiumEngine::handle_event()`: parse CDP event method, dispatch to subscribers by event type
- [ ] 0.5.2.3 Implement `PageEngine::touch_screen()`: dispatch touch events via CDP `Input.dispatchTouchEvent`
- [ ] 0.5.2.4 Implement `PageEngine::emulate_media()`: apply CSS media feature overrides via CDP `Emulation.setEmulatedMedia`
- [ ] 0.5.2.5 Implement `PageEngine::storage()`: return localStorage/sessionStorage handle via CDP `DOMStorage` domain
- [ ] 0.5.2.6 Implement `PageEngine::websocket()`: intercept WebSocket frames via CDP `Network.webSocketFrame*` events
- [ ] 0.5.2.7 Implement `PageEngine::accessibility()`: return accessibility tree snapshot via CDP `Accessibility.getFullAXTree`
- [ ] 0.5.2.8 Implement `Route::abort()`: send CDP `Fetch.failRequest` with `ErrorReason::BlockedByClient`
- [ ] 0.5.2.9 Implement `BinaryManager::download_chromium()`: fetch CfT JSON manifest, download + cache + verify browser binary
- [ ] 0.5.2.10 Implement `page.throttle()`: actual network/CPU throttling via CDP `Network.emulateNetworkConditions` / `Emulation.setCPUThrottlingRate`
- [ ] 0.5.2.11 Implement `page.setOffline()`: toggle network offline mode via CDP `Network.emulateNetworkConditions`
- [ ] 0.5.2.12 Verify all no-op stubs are eliminated — grep for `Ok(())` stub patterns in engine files

### 0.5.3 Error Handling Foundation

- [ ] 0.5.3.1 Add `From<serde_json::Error>` impl for `TurbosheetError` (map to `InternalError` with message)
- [ ] 0.5.3.2 Add `From<url::ParseError>` impl for `TurbosheetError` (map to `InvalidArgument` or new `UrlParseError`)
- [ ] 0.5.3.3 Add `From<reqwest::Error>` impl for `TurbosheetError` (map to `NetworkError` with status context)
- [ ] 0.5.3.4 Install custom panic hook in `src/lib.rs`: capture panic info, format as JS error, never crash Node process
- [ ] 0.5.3.5 Replace `.unwrap()` on CDP port bind in `JsBrowser::new()` with error propagation
- [ ] 0.5.3.6 Audit and replace all remaining `.unwrap()`/`.expect()` calls across `src/` with proper error handling
- [ ] 0.5.3.7 Verify all `?` operator usages compile with the new `From` impls

### 0.5.4 Event Delivery Reliability

- [ ] 0.5.4.1 Fix broadcast channel overflow in `src/network/interceptor.rs`: increase capacity, add drop logging, expose drop metric
- [ ] 0.5.4.2 Fix NonBlocking event drops in `src/network/events.rs`: queue events for deferred delivery or switch to Async dispatch
- [ ] 0.5.4.3 Add integration test: 100+ rapid network requests all deliver events without loss
- [ ] 0.5.4.4 Verify `ChromiumEngine::handle_event()` delivers CDP events to all subscribers without drops

### 0.5.5 Component Mount Reliability

- [ ] 0.5.5.1 Add configurable timeout to `child_body_wait` in `src/component/mount.rs` (default: 30s)
- [ ] 0.5.5.2 Return `TurbosheetError::TimeoutError` with page state diagnostics on mount timeout
- [ ] 0.5.5.3 Verify placeholder navigation completes successfully and clean up placeholder state after mount

### 0.5.6 Migration Reporting Accuracy

- [ ] 0.5.6.1 Fix `src/migrate/report.rs`: track and report actual `files_converted` count
- [ ] 0.5.6.2 Fix report: track and report actual `files_skipped` count with per-file skip reasons
- [ ] 0.5.6.3 Add per-file error details to report (file path, error type, line number, code snippet)
- [ ] 0.5.6.4 Add JSON output format for machine-readable migration reports

### 0.5.7 Testing Coverage — Fix Verification

- [ ] 0.5.7.1 Add unit tests for puppeteer.rs adapter: import replacement, API mapping coverage, scoped viewport/screenshot replacements, full script conversion
- [ ] 0.5.7.2 Add unit tests for cypress.rs adapter: quote-style preservation, wait conversion, intercept callback syntax, full script conversion
- [ ] 0.5.7.3 Add unit tests for playwright.rs adapter: import replacement, $eval/$$eval conversion, missing mapping coverage, full script conversion
- [ ] 0.5.7.4 Add tests for all `From` impl conversions (serde_json, url, reqwest → TurbosheetError)
- [ ] 0.5.7.5 Add test for panic hook: Rust panic → JS error (not Node crash)
- [ ] 0.5.7.6 Add test for broadcast channel reliability under load
- [ ] 0.5.7.7 Add test for browser.close() actually killing the process
- [ ] 0.5.7.8 Add test for mount timeout behavior (hangs → TimeoutError instead of indefinite hang)

## Phase 1: Injected Script Engine (Weeks 5-8)

### 1.1 Injected Core Script

- [x] 1.1.1 Complete `js/src/injected-core.ts`: IIFE-wrapped module establishing `__turbosheet` binding, MutationObserver, querySelector wrappers, frame detection
- [x] 1.1.2 Add `inject_core_script()` to `PageEngine` trait: CDP uses `Page.addScriptToEvaluateOnNewDocument`, Firefox BiDi uses `script.addPreloadScript`, WebKit uses `UserContentManager.addScript`
- [x] 1.1.3 Auto-inject on page creation (`ContextEngine::new_page()`) and after every navigation
- [x] 1.1.4 Add per-frame injection tracking for multi-frame pages
- [x] 1.1.5 Keep core script ≤2KB minified (tree-shake via rolldown)
- [x] 1.1.6 Build verification in `build.rs` that `include_str!()` paths exist

### 1.2 Injected Actions Script

- [x] 1.2.1 Complete `js/src/injected-actions.ts`: actionability engine (visibility, stability, enabled, pointer-events, not-covered), geometry checks (bounding box, intersection), smart auto-wait with MutationObserver + exponential backoff
- [x] 1.2.2 Implement drag-and-drop, file input, complex interactions in injected-actions.ts
- [x] 1.2.3 Implement DOM traversal and snapshot generation for AI agent features
- [x] 1.2.4 On-demand loading: inject `injected-actions.js` via `Runtime.evaluate()` only on first action call, cache in page context
- [x] 1.2.5 Keep actions script ≤15KB minified (tree-shaken per capability)

### 1.3 Binding Bridge

- [x] 1.3.1 Implement `BindingRegistry` in `src/injection/bindings.rs` with `DashMap<RequestId, oneshot::Sender<Value>>`
- [x] 1.3.2 Implement request-response correlation: Rust generates UUID → sends to injected.js via evaluate → injected.js reports back via binding → Rust matches UUID
- [x] 1.3.3 Implement `register_binding()` on `PageEngine` trait: CDP uses `Runtime.addBinding("__tsReport")`, Firefox/WebKit use protocol equivalents
- [x] 1.3.4 Implement `call_binding(page_id, method, args) -> Result<Value>` in `InjectionManager`
- [x] 1.3.5 Add navigation-aware message queue (re-queue pending requests on navigation)
- [x] 1.3.6 Add timeout for orphaned binding requests

### 1.4 Stealth Mode

- [x] 1.4.1 Implement runtime name randomization: replace binding names with `__{uuid}` at injection time
- [x] 1.4.2 Implement Symbol-based state isolation (no `window.__turbosheet` global)
- [x] 1.4.3 Default `stealth: true` with config option to disable for debugging
- [x] 1.4.4 Add tests: bot detection sites, global namespace cleanliness

### 1.5 Engine Migration

- [x] 1.5.1 Migrate `ChromiumPageEngine`: replace all `format!("(function() { ... })()")` with binding calls to injected functions
- [x] 1.5.2 Migrate `FirefoxPageEngine`: same — remove duplicated JS, use shared injected script
- [x] 1.5.3 Migrate `WebKitPageEngine`: same — remove duplicated JS, use shared injected script
- [x] 1.5.4 Verify all 57 `PageEngine` methods work through injected script on all 3 engines
- [x] 1.5.5 Benchmark: CDP round-trips per `.click()` (target: ≤2), actionability check latency (target: ≤10ms p95)

## Phase 2: Core Gaps (Weeks 9-16)

### 2.1 Frame/IFrame Support

- [x] 2.1.1 Implement `page.frames()` returning list of frame descriptors with execution context IDs
- [x] 2.1.2 Implement `page.frame(name)` for named frame access
- [x] 2.1.3 Implement `page.frameLocator(selector)` returning `FrameLocator` for cross-frame selection
- [x] 2.1.4 Track frame lifecycle via `Page.frameAttached`, `Page.frameNavigated`, `Page.frameDetached` events
- [x] 2.1.5 Auto-inject core script into new frames via per-frame execution context
- [x] 2.1.6 Add tests: interaction with iframes, nested iframes, cross-origin iframes

### 2.2 Multi-Page/Popup Support

- [x] 2.2.1 Detect popups via CDP `Target.targetCreated` + `Target.targetInfoChanged`
- [x] 2.2.2 Implement `context.waitForEvent('page')` with timeout
- [x] 2.2.3 Implement `page.opener()` returning the page that opened this popup
- [x] 2.2.4 Implement `context.pages()` returning all pages including popups
- [x] 2.2.5 Add tests: popup creation, context.pages() after popup, page.opener()

### 2.3 Cookie and Storage Management

- [ ] 2.3.1 Implement `context.addCookies(cookies: Cookie[])` via CDP `Network.setCookies`
- [ ] 2.3.2 Implement `context.cookies(urls?: string[])` via CDP `Network.getCookies`
- [ ] 2.3.3 Implement `context.clearCookies()` via CDP `Network.deleteCookies` with URL filter
- [ ] 2.3.4 Implement `context.storageState()` serializing cookies + localStorage + sessionStorage
- [ ] 2.3.5 Implement `context.newContext({ storageState: 'auth.json' })` for auth restoration
- [ ] 2.3.6 Add tests: cookie CRUD, storage state round-trip, auth replay

### 2.4 Download Handling

- [ ] 2.4.1 Subscribe to CDP `Browser.downloadWillBegin` and `Browser.downloadProgress` events via EventDispatcher
- [ ] 2.4.2 Implement `download.path()` returning the download destination path
- [ ] 2.4.3 Implement `download.saveAs(path)` to move/copy download to a custom location
- [ ] 2.4.4 Implement `download.cancel()` to abort in-progress download
- [ ] 2.4.5 Implement `page.waitForEvent('download')`
- [ ] 2.4.6 Add tests: download lifecycle, saveAs, cancel, timeout

### 2.5 Video Recording

- [ ] 2.5.1 Create `src/recording/mod.rs` with frame capture via CDP `Page.startScreencast`
- [ ] 2.5.2 Implement frame buffer with configurable quality and frame rate
- [ ] 2.5.3 Implement WebM encoding from captured frames (using Rust WEBM muxer or FFmpeg subprocess)
- [ ] 2.5.4 Implement `context.newPage({ recordVideo: { dir: './videos', size: { width, height } } })`
- [ ] 2.5.5 Auto-finalize video on page close
- [ ] 2.5.6 Add tests: video file creation, frame count, duration accuracy

### 2.6 HAR Recording/Playback

- [ ] 2.6.1 Create `src/network/har.rs` with HAR v1.2 spec structs and serialization
- [ ] 2.6.2 Record network events from CDP `Network.requestWillBeSent`, `Network.responseReceived`, `Network.loadingFinished` into HAR entries
- [ ] 2.6.3 Implement `context.newPage({ recordHar: { path: 'trace.har' } })`
- [ ] 2.6.4 Implement HAR playback: match requests against recorded HAR entries, return stubbed responses
- [ ] 2.6.5 Add tests: HAR recording round-trip, playback matching

### 2.7 Auto-Install Chrome for Testing

- [ ] 2.7.1 Implement download via `https://googlechromelabs.github.io/chrome-for-testing/known-good-versions.json`
- [ ] 2.7.2 Cache downloaded browser in `~/.cache/turbosheet/browsers/`
- [ ] 2.7.3 Add `tsheet install` CLI command for manual download
- [ ] 2.7.4 Add auto-download-on-missing in browser launch logic
- [ ] 2.7.5 Add tests: download, cache hit, version selection

### 2.8 File Chooser Handling

- [ ] 2.8.1 Subscribe to CDP `Page.fileChooserOpened` event
- [ ] 2.8.2 Implement `page.on('filechooser')` returning `FileChooser` object
- [ ] 2.8.3 Implement `fileChooser.setFiles(paths: string[])` via CDP `DOM.setFileInputFiles`
- [ ] 2.8.4 Add tests: file dialog interception, multi-file selection

### 2.9 CLI and DX Completion

- [ ] 2.9.1 Implement all CLI flags: `--reporter`, `--shard`, `--retries`, `--workers`, `--timeout`, `--grep`, `--grep-invert`, `--update-screenshots`, `--project`, `--list`, `--pass-with-no-tests`, `--forbid-only`
- [ ] 2.9.2 Implement `--reporter` with multiple reporter support (dot, line, list, JSON, HTML, JUnit)
- [ ] 2.9.3 Implement global setup/teardown hooks
- [ ] 2.9.4 Implement `tsheet.codegen` CLI command for record-and-replay
- [ ] 2.9.5 Write integration tests for all CLI flags and commands

## Phase 3: AI Differentiators (Weeks 17-24)

### 3.1 AI-Native State Extraction (getAgentSnapshot)

- [ ] 3.1.1 Create `src/ai/agent_snapshot.rs` with DOM traversal via injected script
- [ ] 3.1.2 Extract accessibility tree: ARIA roles, labels, states, relationships (parent-child, sibling)
- [ ] 3.1.3 Extract visual DOM: bounding boxes, computed styles, text content, visibility
- [ ] 3.1.4 Format output as compact JSON optimized for LLM consumption (not raw HTML)
- [ ] 3.1.5 Implement `page.getAgentSnapshot()` with optional depth and include/exclude filters
- [ ] 3.1.6 Add benchmark: tokens consumed vs raw HTML (target: 60%+ reduction)

### 3.2 Self-Healing Selectors

- [ ] 3.2.1 On selector failure, collect candidates by: text similarity (Levenshtein), ARIA role match, CSS class overlap, DOM position proximity, visual bounding box distance
- [ ] 3.2.2 Rank candidates by confidence score and return the best match
- [ ] 3.2.3 Log healing event: original selector, healed selector, confidence, element summary
- [ ] 3.2.4 Fail test if healing rate exceeds configurable threshold (default: 3 per test)
- [ ] 3.2.5 Report healing count in test summary output
- [ ] 3.2.6 Add tests: selector breakage recovery, confidence ranking, threshold enforcement

### 3.3 BiDi Stream Pre-Processor

- [ ] 3.3.1 Create `src/engine/bidi_preprocessor.rs` that ingests raw WebDriver BiDi event stream
- [ ] 3.3.2 Filter noise: drop high-frequency events (log events, periodic timestamps, heartbeat)
- [ ] 3.3.3 Condense state: aggregate multiple DOM mutations into single state delta
- [ ] 3.3.4 Only pass finalized, condensed state across FFI bridge to Node.js
- [ ] 3.3.5 Add benchmark: event stream volume reduction (target: 80%+ reduction)

### 3.4 AI-Powered Flake Diagnosis

- [ ] 3.4.1 Create `src/ai/flake_diagnosis.rs` with failure pattern classification
- [ ] 3.4.2 Classify failures: timeout vs assertion vs network vs browser crash vs element not found
- [ ] 3.4.3 For each class, compute confidence score and suggest fix (e.g., "Element not found — try increasing timeout or checking selector")
- [ ] 3.4.4 Annotate test report with diagnosis results
- [ ] 3.4.5 Add tests: failure classification accuracy across all types

### 3.5 Core Web Vitals Measurement

- [ ] 3.5.1 Create `src/vitals/mod.rs` with `PerformanceObserver` injection script
- [ ] 3.5.2 Collect LCP, INP, CLS, FID, TTFB metrics via browser PerformanceObserver API
- [ ] 3.5.3 Implement `page.getWebVitals()` returning structured vitals report with pass/fail thresholds
- [ ] 3.5.4 Implement `expect(page).toPassVitals()` assertion with configurable thresholds
- [ ] 3.5.5 Add tests: vitals collection accuracy, threshold enforcement

### 3.6 Accessibility Testing

- [ ] 3.6.1 Create `src/accessibility/mod.rs` with axe-core integration
- [ ] 3.6.2 Bundle axe-core runner script (embedded via `include_str!`)
- [ ] 3.6.3 Implement `expect(page).toPassAxe(options?)` with impact level filtering (critical/serious/moderate/minor)
- [ ] 3.6.4 Implement `expect(page).toPassAxe({ rules: ['color-contrast'] })` with individual rule control
- [ ] 3.6.5 Add snapshot-based regression for accessibility violations
- [ ] 3.6.6 Add tests: axe-core integration, rule filtering, impact levels

### 3.7 Clock Mocking / Fake Timers

- [ ] 3.7.1 Create `src/clock/mod.rs` with injected-script-based time API override
- [ ] 3.7.2 Override `Date.now()`, `new Date()`, `performance.now()` via injected script interception
- [ ] 3.7.3 Override `setTimeout`/`setInterval`/`setImmediate`/`requestAnimationFrame` with controlled advancement
- [ ] 3.7.4 Implement `page.clock.setFixedTime(date)` for static time
- [ ] 3.7.5 Implement `page.clock.install()` + `page.clock.fastForward(ms)` for time advancement
- [ ] 3.7.6 Add tests: time-dependent logic, animation advancement, timeout control

### 3.8 Snapshot Testing

- [ ] 3.8.1 Create `src/snapshot/mod.rs` with text/JSON snapshot storage
- [ ] 3.8.2 Implement `expect(value).toMatchSnapshot(name)` with inline and external file storage
- [ ] 3.8.3 Implement snapshot update mode (`--update-snapshots` flag)
- [ ] 3.8.4 Add CI-friendly snapshot mode (fail if snapshots need update)
- [ ] 3.8.5 Add tests: snapshot create, match, update, CI mode

## Phase 4: Ecosystem (Weeks 25+)

### 4.1 Interactive UI Mode

- [ ] 4.1.1 Create `packages/ui-mode/` Electron/React app
- [ ] 4.1.2 Implement WebSocket protocol between test runner and UI
- [ ] 4.1.3 Build test list view with status, timing, and filtering
- [ ] 4.1.4 Build live log viewer with ANSI color support
- [ ] 4.1.5 Build screenshot preview on failure
- [ ] 4.1.6 Build interactive re-run (single test, failed tests only, file scope)
- [ ] 4.1.7 Implement `npx tsheet test --ui` to launch the UI

### 4.2 Watch Mode

- [ ] 4.2.1 Integrate file watcher (chokidar or Rust notify)
- [ ] 4.2.2 Determine affected test files on file change via file→test mapping
- [ ] 4.2.3 Implement debounced re-run with configurable debounce interval
- [ ] 4.2.4 Implement `npx tsheet test --watch` flag
- [ ] 4.2.5 Add tests: file change triggers re-run, debounce timing

### 4.3 Typed Fixture System

- [ ] 4.3.1 Implement `test.extend<Fixtures>({ ... })` pattern in TypeScript API
- [ ] 4.3.2 Support fixture injection with automatic setup and teardown
- [ ] 4.3.3 Support fixture dependency (fixtures depending on other fixtures)
- [ ] 4.3.4 Implement `worker` and `test` fixture scopes (per-worker vs per-test)
- [ ] 4.3.5 TypeScript type inference for extended fixtures
- [ ] 4.3.6 Add tests: fixture lifecycle, scoping, dependency resolution

### 4.4 API + UI Integration Testing

- [ ] 4.4.1 Embed HTTP client (based on reqwest) exposed as `test.request` in test context
- [ ] 4.4.2 Implement request builder API: GET, POST, PUT, DELETE with JSON/form/body
- [ ] 4.4.3 Implement response assertions: status, headers, body, JSON path
- [ ] 4.4.4 Enable sharing auth state between API and UI tests
- [ ] 4.4.5 Add tests: API request lifecycle, auth state sharing, mixed API+UI scenarios

### 4.5 Flaky Test Dashboard

- [ ] 4.5.1 Aggregate flakiness data across CI runs (JSON report format)
- [ ] 4.5.2 Compute flakiness scores per test: (failures / total runs) × 100
- [ ] 4.5.3 Surface top-20 flakiest tests with link to trace files
- [ ] 4.5.4 Track flakiness trends over time (daily/weekly)
- [ ] 4.5.5 Implement `npx tsheet flaky-dashboard` to generate HTML report
- [ ] 4.5.6 Add tests: flakiness computation, aggregation, trending

### 4.6 Test Impact Analysis

- [ ] 4.6.1 Parse `--coverage` lcov output to map files → tests
- [ ] 4.6.2 Parse `git diff HEAD~1` for changed files
- [ ] 4.6.3 Compute affected test set: tests touching changed files
- [ ] 4.6.4 Implement `npx tsheet test --impact` to run only affected tests
- [ ] 4.6.5 Add fallback: full suite run if impact analysis fails or coverage data is stale
- [ ] 4.6.6 Add tests: impact analysis accuracy, fallback behavior

### 4.7 Mobile Cloud Integration

- [ ] 4.7.1 Create `src/cloud/mod.rs` with BrowserStack/SauceLabs/LambdaTest connector
- [ ] 4.7.2 Implement device capability negotiation (OS version, device orientation, appium settings)
- [ ] 4.7.3 Implement tunnel/proxy setup for local testing against cloud devices
- [ ] 4.7.4 Add config: `cloud: { provider: 'browserstack', devices: [...] }`
- [ ] 4.7.5 Add tests: cloud provider API integration, device session lifecycle

### 4.8 Rust-Native MCP Server

- [ ] 4.8.1 Create Model Context Protocol (MCP) server exposing TurboSheet as a tool for AI agents
- [ ] 4.8.2 Expose `navigate`, `click`, `extract`, `screenshot`, `snapshot` as MCP tools
- [ ] 4.8.3 Expose `getAgentSnapshot()` as an MCP resource for AI context
- [ ] 4.8.4 Enable AI agents to control browsers through TurboSheet's Rust-native performance
- [ ] 4.8.5 Add tests: MCP tool invocation, resource access, error handling

## Phase 5: Bonus Differentiators (Weeks 25+)

### 5.1 Plugin System

- [ ] 5.1.1 Design plugin manifest format (`plugin.json`) with hooks, dependencies, config schema, versioning
- [ ] 5.1.2 Implement lifecycle hook API: `beforeNavigation`, `afterNavigation`, `beforeClick`, `onNetworkRequest`, `onTestFailure`
- [ ] 5.1.3 Implement plugin discovery from `node_modules/tsheet-plugin-*`
- [ ] 5.1.4 Implement dependency resolution with cycle detection and version checking
- [ ] 5.1.5 Implement plugin sandboxing with timeout, crash isolation, and filesystem restrictions
- [ ] 5.1.6 Add tests: plugin registration, hook invocation, dependency resolution, sandbox isolation

### 5.2 Performance Budget Testing

- [ ] 5.2.1 Implement `page.getWebVitals()` returning LCP, INP, CLS, FID, TTFB via `PerformanceObserver`
- [ ] 5.2.2 Implement `expect(page).toPassPerformanceBudget({ lcp, cls, inp })` with threshold comparison
- [ ] 5.2.3 Implement `page.gotoAndMeasure(url, budgets)` convenience API
- [ ] 5.2.4 Implement historical baseline comparison for regression detection
- [ ] 5.2.5 Implement CI gate integration with performance report generation
- [ ] 5.2.6 Add tests: vitals collection, budget assertion, regression detection, CI report

### 5.3 DOM State Diffing

- [ ] 5.3.1 Implement DOM snapshot capture on assertion failure (element subtree)
- [ ] 5.3.2 Implement tree-aware DOM comparison engine in Rust (node matching, insertion/removal/reorder detection)
- [ ] 5.3.3 Implement structured diff output format (+/- lines, attribute changes, text changes)
- [ ] 5.3.4 Implement `expect(locator).toMatchDOMSnapshot(name)` with snapshot storage and diff
- [ ] 5.3.5 Implement scoped diffing: diff only the locator's subtree, not the full page
- [ ] 5.3.6 Add tests: DOM diff accuracy, snapshot create/match/update, scoped diffing

### 5.4 Cross-Engine Consistency Reports

- [ ] 5.4.1 Implement per-engine test execution (same test on Chromium + Firefox + WebKit)
- [ ] 5.4.2 Implement per-engine result collection: pass/fail, timing, screenshots, console, network
- [ ] 5.4.3 Implement pixel-level screenshot comparison between engines
- [ ] 5.4.4 Implement layout delta detection (bounding box differences >1px)
- [ ] 5.4.5 Implement consistency score computation (0.0-1.0)
- [ ] 5.4.6 Add tests: cross-engine result comparison, pixel diff, layout delta, scoring

### 5.5 AI Test Generation from Recordings

- [ ] 5.5.1 Implement session recording via CDP event stream + injected.js interaction capture
- [ ] 5.5.2 Implement interaction deduplication and test step identification
- [ ] 5.5.3 Implement TurboSheet test code generator (locator-based, with assertions)
- [ ] 5.5.4 Implement Playwright-compatible output format
- [ ] 5.5.5 Implement Cypress-compatible output format
- [ ] 5.5.6 Add tests: recording fidelity, code generation accuracy across all 3 formats

### 5.6 Browser Crash Recovery

- [ ] 5.6.1 Implement browser process exit detection (exit code + signal monitoring)
- [ ] 5.6.2 Implement CDP disconnection detection (WebSocket ping/pong timeout)
- [ ] 5.6.3 Implement auto-restart: new browser instance, new context/page, navigate to last URL
- [ ] 5.6.4 Implement crash retry with configurable max retries (separate from flakiness retries)
- [ ] 5.6.5 Implement crash diagnostics collection (stderr, crash report, CDP command log)
- [ ] 5.6.6 Implement graceful degradation: continue remaining tests, report crash summary
- [ ] 5.6.7 Add tests: crash detection, auto-restart, retry limits, diagnostics collection

### 5.7 Zero-Copy Network Interception

- [ ] 5.7.1 Implement streaming request body access via Tokio `Stream<Item=Bytes>` from CDP Fetch
- [ ] 5.7.2 Implement streaming response body passthrough with splice/sendfile support
- [ ] 5.7.3 Implement CDP Fetch streaming for large payloads (Chrome 120+)
- [ ] 5.7.4 Implement NAPI-rs zero-copy Buffer transfer for response bodies
- [ ] 5.7.5 Implement streaming FFI protocol for large bodies (ring buffer + async callbacks)
- [ ] 5.7.6 Add tests: GB-level payload streaming, memory bounds, throughput benchmark

### 5.8 Differential Testing (Browser Version Upgrades)

- [ ] 5.8.1 Implement multi-version browser launch (2+ Chrome for Testing versions simultaneously)
- [ ] 5.8.2 Implement per-test result comparison across versions (pass/fail, timing, screenshots)
- [ ] 5.8.3 Implement automated regression severity classification (critical/high/medium/low)
- [ ] 5.8.4 Implement CI gate: block upgrade on critical regressions or >1% behavioral change
- [ ] 5.8.5 Implement version discovery from CfT known-good-versions.json API
- [ ] 5.8.6 Implement per-version exclusion rules (`@tsheet-version-skip`)
- [ ] 5.8.7 Add tests: cross-version comparison, regression classification, upgrade gate
