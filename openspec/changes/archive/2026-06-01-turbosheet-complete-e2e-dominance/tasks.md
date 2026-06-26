## 1. Foundation — Cross-Browser Engine

- [x] 1.1 Design and implement `BrowserEngine` Rust trait as browser abstraction layer
- [x] 1.2 Implement Firefox support via `geckodriver` + WebDriver BiDi protocol
- [x] 1.3 Implement WebKit support via `webkit2gtk` (Linux) + Safari remote WebDriver (macOS)
- [x] 1.4 Extend existing BiDi stream pre-processor to handle Firefox and WebKit event schemas
- [x] 1.5 Implement `tsheet install [browser]` CLI for downloading and managing all three browser binaries
- [ ] 1.6 Implement cross-browser selector compatibility tests (CSS, text, testId selectors on all three engines)
- [x] 1.7 Add `browser` option to launch config API: `tsheet.launch({ browser: 'chromium' | 'firefox' | 'webkit' })`

## 2. Foundation — Test Runner Core

- [x] 2.1 Implement test file discovery with glob pattern matching (`**/*.tsheet.ts`, `**/*.tsheet.spec.ts`)
- [x] 2.2 Implement Rust test executor with napi-rs bridge for orchestrating JS test callbacks
- [x] 2.3 Implement Tokio-based worker pool for parallel test execution
- [x] 2.4 Implement `test.describe()`, `test()`, `test.only()`, `test.skip()`, `test.fixme()`, `test.fail()`, `test.slow()` API
- [x] 2.5 Implement fixture system with `test.extend()` and DAG dependency resolution
- [x] 2.6 Implement fixture scoping (test-level, worker-level, global-level) with Rust LRU caching for worker-scoped fixtures
- [x] 2.7 Implement `test.beforeAll` / `test.afterAll` / `test.beforeEach` / `test.afterEach` hooks
- [x] 2.8 Implement `--shard=N/M` for CI test splitting
- [x] 2.9 Implement `--retries=N` for flaky test auto-retry
- [x] 2.10 Implement `--workers=N` for setting parallel worker count
- [x] 2.11 Implement configuration file: `tsheet.config.ts` (project, testDir, workers, retries, timeout, projects)

## 3. Foundation — Auto-Retrying Assertions

- [x] 3.1 Implement Rust-side assertion polling engine that runs in Tokio runtime (not Node.js event loop)
- [x] 3.2 Implement `expect(locator).toBeVisible()` with configurable timeout and polling interval
- [x] 3.3 Implement `expect(locator).toHaveText(text)` with text matching options (exact, substring, regex)
- [x] 3.4 Implement `expect(locator).toHaveAttribute(name, value)`
- [x] 3.5 Implement `expect(locator).toBeEnabled()` / `toBeDisabled()`
- [x] 3.6 Implement `expect(locator).toHaveValue(value)` for input fields
- [x] 3.7 Implement `expect(page).toHaveURL(url)` with glob/regex matching
- [x] 3.8 Implement `expect(page).toHaveTitle(title)`
- [x] 3.9 Implement `expect.poll()` for custom async polling assertions
- [x] 3.10 Implement timeout option per assertion and default in config

## 4. Foundation — Network Interception

- [x] 4.1 Implement in-process Tokio-based HTTP proxy that intercepts browser traffic
- [x] 4.2 Implement glob pattern matching engine in Rust for URL route matching
- [x] 4.3 Implement `page.route(pattern, handler)` API with handler callbacks across napi-rs
- [x] 4.4 Implement `route.fulfill({ status, headers, body })` for mock responses
- [x] 4.5 Implement `route.continue({ headers, method, postData })` for request modification (implemented as continue_request due to Rust reserved keyword conflict)
- [x] 4.6 Implement `route.abort()` for blocking requests (ads, analytics, fonts)
- [x] 4.7 Implement `page.unrouteAll()` for removing all route handlers
- [x] 4.8 Implement WebSocket interception with `page.routeWebSocket()` API
- [x] 4.9 Implement network throttling: `page.throttle({ download, upload, latency })` and `page.setOffline(bool)`

## 5. Phase 2 — Reporter System & CI Infrastructure

- [x] 5.1 Implement streaming HTML reporter with embedded trace viewer, test summary, and failure screenshots
- [x] 5.2 Implement JSON reporter for CI ingestion with structured output
- [x] 5.3 Implement JUnit XML reporter for Jenkins/GitLab CI compatibility
- [x] 5.4 Implement GitHub Actions Annotations reporter for inline PR comments
- [x] 5.5 Implement terminal reporters: `dot` (compact) and `line` (verbose)
- [x] 5.6 Support multiple simultaneous reporters: `--reporter html --reporter json`
- [x] 5.7 Create official Docker images: `tsheet/tsheet`, `tsheet/tsheet:chromium-only`, `tsheet/tsheet:arm64`
- [x] 5.8 Implement `tsheet init --ci github|gitlab` template generators
- [x] 5.9 Implement CI browser binary caching in generated workflows
- [x] 5.10 Implement screenshots-on-failure capture with auto-attachment to reporters

## 6. Phase 2 — Mobile Emulation & Accessibility Testing

- [x] 6.1 Implement 50+ device presets (iPhone, iPad, Pixel, Galaxy, Surface, etc.)
- [x] 6.2 Implement `page.emulate()` and `test.use({ ...devices['iPhone 15'] })` API
- [x] 6.3 Implement touch event emulation (tap, swipe, pinch, long-press)
- [x] 6.4 Implement geolocation override: `context.setGeolocation({ lat, lng })`
- [x] 6.5 Implement `page.emulateMedia({ colorScheme, reducedMotion, forcedColors })`
- [x] 6.6 Implement aXe accessibility engine integration: `page.checkAccessibility()`
- [x] 6.7 Implement `expect(page).toHaveNoAccessibilityViolations()` assertion
- [x] 6.8 Implement accessibility report output (HTML, JSON)
- [x] 6.9 Implement WCAG level configuration (A, AA, AAA) and rule filtering

## 7. Phase 2 — Component Testing

- [x] 7.1 Implement lightweight headless Chromium context for component mounting (no full page load)
- [x] 7.2 Implement React rendering pipeline with `mount(<Component />)` API
- [x] 7.3 Implement Vue rendering pipeline
- [x] 7.4 Implement Svelte rendering pipeline
- [x] 7.5 Implement framework auto-detection from `package.json`
- [x] 7.6 Implement pre-warmed context pool for fast component rendering cycles
- [x] 7.7 Ensure all standard `expect` assertions work on mounted component output

## 8. Phase 2 — Plugin System & Migration Tooling

- [x] 8.1 Implement JS plugin discovery from `node_modules/tsheet-plugin-*`
- [x] 8.2 Implement lifecycle hooks API: `onTestStart`, `onTestEnd`, `onTestFailed`
- [x] 8.3 Implement custom reporter plugin support
- [x] 8.4 Implement custom matcher plugin support
- [x] 8.5 Implement Rust plugin (.so/.dylib) loading from `~/.tsheet/plugins/`
- [x] 8.6 Implement `tsheet migrate from playwright` with spec-to-spec conversion
- [x] 8.7 Implement `tsheet migrate from cypress` with cy.\* → TurboSheet API mapping
- [x] 8.8 Implement `tsheet migrate from puppeteer` with Puppeteer API mapping
- [x] 8.9 Implement migration report generation (files converted, manual todos, coverage %)

## 9. Phase 3 — TurboTrace Debugger

- [x] 9.1 Implement trace recording engine in Rust with CBOR encoding and zstd compression
- [x] 9.2 Capture action log (timestamp, type, selector, value, duration, result) for each browser action
- [x] 9.3 Capture network events (request, response, timing, headers, body) in trace
- [x] 9.4 Capture console output (log, warn, error, info) with timestamps
- [x] 9.5 Capture DOM snapshots using AI-native `getAgentSnapshot()` format at configurable intervals
- [x] 9.6 Implement trace file writer with `.tsheet-trace` extension
- [x] 9.7 Build web-based TurboTrace viewer with timeline navigation
- [x] 9.8 Implement step inspection in viewer (DOM snapshot, console, network at each step)
- [x] 9.9 Implement `tsheet show-trace <file>` CLI command
- [x] 9.10 Embed trace viewer in HTML reporter output

## 10. Phase 3 — Visual Regression Testing

- [x] 10.1 Implement full-page, element-level, and viewport screenshot capture
- [x] 10.2 Implement SIMD-accelerated pixel comparison engine (SSE/AVX x86, NEON ARM)
- [x] 10.3 Implement baseline storage in `.tsheet-snap/` directories
- [x] 10.4 Implement `expect(page).toHaveScreenshot(name, options)` assertion
- [x] 10.5 Implement auto-baseline creation on first run
- [x] 10.6 Implement diff image generation with heatmap overlay (red/green/gray)
- [x] 10.7 Implement AI-powered semantic diff using `getAgentSnapshot()` data to ignore dynamic regions
- [x] 10.8 Implement pixel threshold configuration per snapshot
- [x] 10.9 Implement screenshot storage management (update, delete baselines)

## 11. Phase 3 — Codegen Test Recorder

- [x] 11.1 Implement `tsheet codegen --url <url>` CLI command
- [x] 11.2 Implement browser opening with recording overlay UI
- [x] 11.3 Implement click action recording with smart selector generation (getByRole > getByText > getByTestId > CSS)
- [x] 11.4 Implement form input recording with `fill()` action
- [x] 11.5 Implement navigation recording with URL assertion suggestions
- [x] 11.6 Implement assertion suggestion engine (infer expected state after each action)
- [x] 11.7 Implement `.tsheet.ts` file generation from recorded actions
- [x] 11.8 Implement stop/restart/undo controls in recording UI

## 12. Phase 3 — VS Code Extension

- [x] 12.1 Implement Test Explorer tree view showing all discovered `.tsheet.ts` tests
- [x] 12.2 Implement run/debug buttons per test and per file
- [x] 12.3 Implement inline pass/fail decorations (green check / red X) in source
- [x] 12.4 Implement debug session with breakpoint support and source map resolution
- [x] 12.5 Implement trace viewer as VS Code webview panel
- [x] 12.6 Implement "View Trace" button on failed test results
- [x] 12.7 Implement configuration UI for TurboSheet settings

## 13. Phase 3 — Interactive Debug Runner

- [x] 13.1 Implement `tsheet debug --url <url>` CLI command
- [x] 13.2 Implement command log UI with per-action status (pending, passed, failed)
- [x] 13.3 Implement step-through execution (pause after each action)
- [x] 13.4 Implement DOM snapshot display on pause using AI-native format
- [x] 13.5 Implement live browser context with ad-hoc command execution
- [x] 13.6 Implement "modify and continue" — edit selector, re-run from that step
- [x] 13.7 Implement snapshot exploration (click element in tree → view computed properties)

## 14. Phase 4 — Swarm Cloud Grid

- [x] 14.1 Implement grid controller with job queue (Tokio channel-based)
- [x] 14.2 Implement worker node that registers with controller and accepts jobs
- [x] 14.3 Implement Tokio task-based context creation (1000+ contexts per server)
- [x] 14.4 Implement REST API for test submission and result polling
- [ ] 14.5 Implement auto-scaling (scale up on queue depth, scale down on idle)
- [x] 14.6 Implement result aggregation from multiple workers into unified report
- [ ] 14.7 Implement worker health monitoring and dead worker re-assignment
- [ ] 14.8 Implement Docker-based multi-tenant isolation per client

## 15. Phase 4 — WASM Edge Runtime

- [ ] 15.1 Configure Rust build for `wasm32-wasi` compilation target
- [ ] 15.2 Implement WASM-compatible test orchestrator (scheduling, retries, aggregation)
- [ ] 15.3 Implement tree-shaking to exclude unused browser engine code (target: 3MB WASM binary)
- [ ] 15.4 Implement edge-to-grid binary protocol for proxying browser commands
- [ ] 15.5 Implement latency-optimized grid node routing
- [ ] 15.6 Implement edge-native assertion execution (assertion polling runs in WASM, not on grid node)
- [ ] 15.7 Implement runtime detection for `workerd` (Cloudflare Workers), `deno`, and bare metal

## 16. Polish — Configuration, CLI, and Documentation

- [x] 16.1 Implement `tsheet test` command with all flags (--reporter, --shard, --retries, --workers, --timeout, --grep)
- [x] 16.2 Implement `tsheet install`, `tsheet codegen`, `tsheet debug`, `tsheet show-trace`, `tsheet migrate`, `tsheet devices`, `tsheet init` CLI commands
- [x] 16.3 Implement configuration file support: `tsheet.config.ts` with typed configuration
- [ ] 16.4 Implement project references for monorepo support
- [ ] 16.5 Implement global setup/teardown files
- [x] 16.6 Implement `.env` file loading for test secrets
- [ ] 16.7 Write comprehensive migration guide from Playwright, Cypress, and Puppeteer
- [ ] 16.8 Write API reference documentation with examples
- [ ] 16.9 Write getting-started tutorial (5-minute quickstart)
- [ ] 16.10 Write CI/CD integration guides (GitHub Actions, GitLab CI, Jenkins, CircleCI)
