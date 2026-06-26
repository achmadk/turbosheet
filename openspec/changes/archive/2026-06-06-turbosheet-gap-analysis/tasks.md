## 1. Firefox Engine Parity

- [ ] 1.1 Implement `FirefoxPageEngine::set_input_files()` using WebDriver element upload with local file path detection
- [ ] 1.2 Implement `FirefoxPageEngine::wait_for_request()` with polled network log matching and configurable timeout
- [ ] 1.3 Implement `FirefoxPageEngine::wait_for_response()` with polled network log matching and configurable timeout
- [ ] 1.4 Implement `FirefoxPageEngine::evaluate_handle()` returning opaque JS handle reference
- [ ] 1.5 Implement `FirefoxPageEngine::expose_function()` by injecting named function into window scope
- [ ] 1.6 Implement `FirefoxPageEngine::set_content()` by writing HTML to document via evaluate
- [ ] 1.7 Implement `FirefoxPageEngine::inject_core_script()` by evaluating and caching the injected core script
- [ ] 1.8 Implement `FirefoxPageEngine::register_binding()` with WebDriver-compatible message passing
- [ ] 1.9 Fix `FirefoxPageEngine::url()` to return actual current page URL (not hardcoded "about:blank")
- [ ] 1.10 Fix `FirefoxContextEngine::pages()` to return actual open pages list
- [ ] 1.11 Replace `FirefoxPageEngine::hover()` raw evaluate() with `invoke_action("hover")` for consistency
- [ ] 1.12 Replace `FirefoxPageEngine::drag_and_drop()` raw evaluate() with `invoke_action("dragAndDrop")`
- [ ] 1.13 Implement cross-browser selector compatibility tests for Firefox (CSS, text, testId selectors)
- [ ] 1.14 Verify all 57 PageEngine methods return real results on Firefox end-to-end

## 2. WebKit Engine Parity

- [ ] 2.1 Implement `WebKitPageEngine::set_input_files()` using WebDriver element upload
- [ ] 2.2 Implement `WebKitPageEngine::wait_for_request()` with polled network log matching
- [ ] 2.3 Implement `WebKitPageEngine::wait_for_response()` with polled network log matching
- [ ] 2.4 Implement `WebKitPageEngine::evaluate_handle()` returning opaque JS handle reference
- [ ] 2.5 Implement `WebKitPageEngine::expose_function()` by injecting named function into window scope
- [ ] 2.6 Implement `WebKitPageEngine::set_content()` by writing HTML to document via evaluate
- [ ] 2.7 Implement `WebKitPageEngine::inject_core_script()` by evaluating and caching the injected core script
- [ ] 2.8 Implement `WebKitPageEngine::register_binding()` with WebDriver-compatible message passing
- [ ] 2.9 Fix `WebKitPageEngine::url()` to return actual current page URL
- [ ] 2.10 Fix `WebKitContextEngine::pages()` to return actual open pages list
- [ ] 2.11 Replace `WebKitPageEngine::hover()` raw evaluate() with `invoke_action("hover")`
- [ ] 2.12 Replace `WebKitPageEngine::drag_and_drop()` raw evaluate() with `invoke_action("dragAndDrop")`
- [ ] 2.13 Implement cross-browser selector compatibility tests for WebKit (CSS, text, testId selectors)
- [ ] 2.14 Verify all 57 PageEngine methods return real results on WebKit end-to-end

## 3. TypeScript Declarations

- [ ] 3.1 Create type hierarchy: Browser, Context, Page, Locator interfaces
- [ ] 3.2 Create LaunchOptions, ContextOptions, ScreenshotOptions type declarations with JSDoc
- [ ] 3.3 Create LocatorOptions interface with chainable method overloads
- [ ] 3.4 Create AssertionOptions and matcher interfaces for expect()
- [ ] 3.5 Create BrowserType union ('chromium' | 'firefox' | 'webkit')
- [ ] 3.6 Create ViewportSize, Rect, Cookie, Geolocation data interfaces
- [ ] 3.7 Create Route, RouteHandler, WebSocketRoute types for network interception
- [ ] 3.8 Create TestRunner config types: TsheetConfig, ProjectConfig, ReporterConfig
- [ ] 3.9 Create MigrationOptions types for migration tools
- [ ] 3.10 Create SwarmConfig types for grid configuration
- [ ] 3.11 Create DeviceDescriptor type with 50+ device preset signatures
- [ ] 3.12 Create EventCallback types for page.on() / page.off()
- [ ] 3.13 Add JSDoc documentation to all declarations with parameter descriptions and examples
- [ ] 3.14 Wire index.d.ts into package.json "types" field and verify
- [ ] 3.15 Test declarations compile correctly with `tsc --noEmit` against a sample .ts file
- [ ] 3.16 Verify TypeScript autocomplete works in VS Code with the local package

## 4. AST Migration Tools

- [ ] 4.1 Add SWC parser dependency to the migration tool chain
- [ ] 4.2 Implement SWC visitor for Playwright import statement replacement
- [ ] 4.3 Implement SWC visitor for Playwright API call transformation (locator, page methods)
- [ ] 4.4 Implement SWC visitor for Playwright assertion transformation
- [ ] 4.5 Implement SWC visitor for Playwright fixture and hook transformation
- [ ] 4.6 Implement SWC visitor for Cypress chain-to-async/await unwrapping
- [ ] 4.7 Implement SWC visitor for Cypress command transformation (cy.get, cy.visit, cy.contains)
- [ ] 4.8 Implement SWC visitor for Cypress assertion transformation (.should)
- [ ] 4.9 Implement SWC visitor for Puppeteer API transformation
- [ ] 4.10 Implement SWC visitor for Puppeteer $eval/$$eval pattern unwrapping
- [ ] 4.11 Implement batch directory processing with output structure mirroring
- [ ] 4.12 Implement migration report output with coverage percentage and unsupported patterns
- [ ] 4.13 Update `tsheet migrate` CLI command to use AST transforms
- [ ] 4.14 Create test fixtures: Playwright input → expected TurboSheet output for all transform patterns
- [ ] 4.15 Create test fixtures: Cypress input → expected TurboSheet output
- [ ] 4.16 Create test fixtures: Puppeteer input → expected TurboSheet output
- [ ] 4.17 Verify end-to-end migration produces valid, runnable TurboSheet tests

## 5. Swarm Grid Completion

- [ ] 5.1 Implement queue-depth watcher in GridController for auto-scaling trigger
- [ ] 5.2 Implement worker spawn/kill logic in response to scale-up/down signals
- [ ] 5.3 Add minWorkers, maxWorkers, scaleUpThreshold, scaleDownIdleDuration config
- [ ] 5.4 Implement graceful worker termination with in-progress job completion
- [ ] 5.5 Implement periodic heartbeat check in HealthMonitor with configurable interval
- [ ] 5.6 Implement dead worker detection after N missed heartbeats
- [ ] 5.7 Implement shard reassignment from dead workers to live workers
- [ ] 5.8 Implement job re-queuing with retry counter on worker death
- [ ] 5.9 Implement health events emission to metrics system
- [ ] 5.10 Add `bollard` Docker client dependency for tenant isolation
- [ ] 5.11 Implement Tenant container creation with CPU/memory resource limits
- [ ] 5.12 Implement Tenant container cleanup on deactivation
- [ ] 5.13 Implement Docker volume and network management per tenant
- [ ] 5.14 Write unit tests for auto-scaling logic
- [ ] 5.15 Write unit tests for health monitoring with simulated worker failures
- [ ] 5.16 Write integration tests for Docker tenant lifecycle

## 6. WASM Edge Runtime

- [ ] 6.1 Add wasm32-wasi compilation target to Cargo.toml
- [ ] 6.2 Create `wasm-orchestrator` crate with feature-gated engine exclusion
- [ ] 6.3 Implement WASM-compatible test scheduler (queue, prioritize, dispatch)
- [ ] 6.4 Implement WASM-compatible retry logic and timeout management
- [ ] 6.5 Implement WASM-compatible result aggregation
- [ ] 6.6 Implement MessagePack binary protocol encoding/decoding
- [ ] 6.7 Implement WebSocket client for grid node communication (workerd-compatible)
- [ ] 6.8 Implement TCP fallback for deno runtime
- [ ] 6.9 Implement runtime detection: workerd vs deno vs native
- [ ] 6.10 Add tree-shaking config to exclude browser engine code from WASM build
- [ ] 6.11 Verify WASM binary size is under 3MB target
- [ ] 6.12 Write unit tests for WASM orchestrator scheduling and aggregation
- [ ] 6.13 Write integration test for WASM ↔ grid node binary protocol round-trip

## 7. Test Runner DX

- [ ] 7.1 Implement --grep flag for test name filtering
- [ ] 7.2 Implement --grep-invert flag
- [ ] 7.3 Implement project references in tsheet.config.ts with per-project browser/viewport config
- [ ] 7.4 Implement globalSetup file execution before any tests
- [ ] 7.5 Implement globalTeardown file execution after all tests
- [ ] 7.6 Implement global fixture data passing from setup to tests
- [ ] 7.7 Implement per-test timeout via test.setTimeout()
- [ ] 7.8 Implement timeout configuration hierarchy (global → project → test)
- [ ] 7.9 Implement multi-reporter configuration and simultaneous output
- [ ] 7.10 Implement all CLI flags: --reporter, --shard, --retries, --workers, --timeout, --grep, --grep-invert, --update-screenshots, --project, --list, --pass-with-no-tests, --forbid-only
- [ ] 7.11 Write integration tests for grep filtering
- [ ] 7.12 Write integration tests for project references
- [ ] 7.13 Write integration tests for global setup/teardown
- [ ] 7.14 Write integration tests for timeout configuration
- [ ] 7.15 Write integration tests for reporter configuration

## 8. Test Coverage Expansion

- [ ] 8.1 Write CLI integration tests: tsheet test, tsheet migrate, tsheet show-trace, tsheet devices, tsheet install
- [ ] 8.2 Write migration unit tests for all three source framework transformers
- [ ] 8.3 Write visual comparison tests: PixelDiff identical → 0%, different → >0%, diff image valid PNG
- [ ] 8.4 Write trace module tests: serialize/deserialize round-trip, HTML viewer validity
- [ ] 8.5 Write swarm controller unit tests: job queue FIFO, priority ordering
- [ ] 8.6 Write component testing unit tests: framework detection from package.json, iframe injection
- [ ] 8.7 Write network interception integration tests: route fulfillment, request modification, route abort
- [ ] 8.8 Write TypeScript runtime tests: test file parsing, fixture resolution, hook execution order
- [ ] 8.9 Write Rust in-module #[cfg(test)] unit tests for error types and utility functions
- [ ] 8.10 Run full test suite on all three browsers and verify all tests pass

## 9. Documentation Completion

- [ ] 9.1 Write getting-started tutorial: install → launch browser → write test → run → view results
- [ ] 9.2 Write comprehensive API reference organized by module
- [ ] 9.3 Write configuration reference for tsheet.config.ts options
- [ ] 9.4 Write Playwright migration guide with API comparison table and step-by-step checklist
- [ ] 9.5 Write Cypress migration guide with conceptual differences explanation
- [ ] 9.6 Write Puppeteer migration guide
- [ ] 9.7 Implement `tsheet init --ci github` template generator
- [ ] 9.8 Implement `tsheet init --ci gitlab` template generator
- [ ] 9.9 Implement `tsheet init --ci jenkins` template generator
- [ ] 9.10 Implement `tsheet init --ci circleci` template generator
- [ ] 9.11 Create .github/workflows/tsheet.yml with checkout, setup, install, test, upload-artifact steps
- [ ] 9.12 Create .github/ISSUE_TEMPLATE/ bug report and feature request templates
