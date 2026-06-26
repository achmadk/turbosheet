## 1. Browser Engine Architecture

- [x] 1.1 Extend `BrowserEngine` trait to support `close()` method
- [x] 1.2 Add Firefox engine stub implementation (`src/engine/firefox.rs`)
- [x] 1.3 Add WebKit engine stub implementation (`src/engine/webkit.rs`)
- [x] 1.4 Update `LaunchOptions` to include browser-specific options
- [x] 1.5 Add `launch()` dispatch to select browser engine based on `browser` option
- [x] 1.6 Add geckodriver dependency to Cargo.toml
- [x] 1.7 Implement Firefox WebDriver HTTP client in `src/engine/firefox.rs`
- [x] 1.8 Implement WebKit WebDriver HTTP client in `src/engine/webkit.rs`
- [x] 1.9 Add `browser install` support for Firefox via geckodriver
- [x] 1.10 Add `browser install` support for WebKit via webkitwd
- [x] 1.11 Verify `launch({ browser: 'firefox' })` works end-to-end
- [x] 1.12 Verify `launch({ browser: 'webkit' })` works end-to-end

## 2. Test Runner Core

- [x] 2.1 Implement test file discovery via glob patterns in `TestDiscovery`
- [x] 2.2 Implement test file parsing to extract `test()`, `describe()`, hooks in `TestSuiteExtractor`
- [x] 2.3 Implement `TestExecutor::execute_tests()` to run actual tests
- [x] 2.4 Implement worker process spawning with IPC channel
- [x] 2.5 Implement JSON-RPC 2.0 message protocol for worker communication
- [x] 2.6 Wire up `run_tests()` NAPI function to use real executor
- [x] 2.7 Implement test result aggregation from worker processes
- [x] 2.8 Implement test timeout handling with process termination
- [x] 2.9 Implement test retry logic in executor
- [x] 2.10 Verify `tsheet test` runs actual tests and reports correct results

## 3. Fixtures and Hooks Lifecycle

- [x] 3.1 Implement `FixtureRegistry::resolve()` for test-level fixtures
- [x] 3.2 Implement fixture disposal after each test
- [x] 3.3 Implement `WorkerFixtureCache` for worker-scoped fixtures
- [x] 3.4 Implement `GlobalFixtureCache` for global-scoped fixtures
- [x] 3.5 Implement topological sort for fixture dependencies
- [x] 3.6 Implement `HookRegistry::execute_before_all()`
- [x] 3.7 Implement `HookRegistry::execute_after_all()`
- [x] 3.8 Implement `HookRegistry::execute_before_each()`
- [x] 3.9 Implement `HookRegistry::execute_after_each()`
- [x] 3.10 Implement hook failure handling (skip tests on beforeAll failure)
- [x] 3.11 Verify fixtures are created/disposed in correct order
- [x] 3.12 Verify hooks execute at correct lifecycle points

## 4. Complete Locator API

- [x] 4.1 Implement `scrollIntoView()` in `JsLocator`
- [x] 4.2 Implement `focus()` in `JsLocator`
- [x] 4.3 Implement `blur()` in `JsLocator`
- [x] 4.4 Implement `boundingBox()` in `JsLocator`
- [x] 4.5 Implement `screenshot()` in `JsLocator`
- [x] 4.6 Implement `dragAndDrop()` in `JsLocator`
- [x] 4.7 Implement `hover()` in `JsLocator`
- [x] 4.8 Implement `press()` in `JsLocator`
- [x] 4.9 Implement `pressSequentially()` in `JsLocator`
- [x] 4.10 Implement `setInputFiles()` in `JsLocator`
- [x] 4.11 Update `LocatorBridge` TypeScript interface
- [x] 4.12 Add CDP commands for each new locator method
- [x] 4.13 Verify locator operations work on all browsers

## 5. Complete Page API

- [x] 5.1 Implement `setViewportSize()` in `JsPage`
- [x] 5.2 Implement `viewportSize()` getter in `JsPage`
- [x] 5.3 Implement `reload()` in `JsPage`
- [x] 5.4 Implement `goBack()` in `JsPage`
- [x] 5.5 Implement `goForward()` in `JsPage`
- [x] 5.6 Implement `waitForRequest()` in `JsPage`
- [x] 5.7 Implement `waitForResponse()` in `JsPage`
- [x] 5.8 Implement `waitForSelector()` in `JsPage`
- [x] 5.9 Implement `evaluateHandle()` in `JsPage`
- [x] 5.10 Implement `addScriptTag()` in `JsPage`
- [x] 5.11 Implement `addStyleTag()` in `JsPage`
- [x] 5.12 Implement `exposeFunction()` in `JsPage`
- [x] 5.13 Implement `on()` event handler registration
- [x] 5.14 Implement `off()` event handler removal
- [x] 5.15 Update `PageBridge` TypeScript interface
- [x] 5.16 Verify page operations work on all browsers

## 6. Network WebSocket Routing

- [x] 6.1 Add `WebSocketHandler` trait in `src/network/`
- [x] 6.2 Add `websocket_routes` field to `NetworkProxy`
- [x] 6.3 Implement `routeWebSocket()` in `JsPage`
- [x] 6.4 Implement WebSocket message interception
- [x] 6.5 Implement `onSocketOpen` callback
- [x] 6.6 Implement `onSocketMessage` callback
- [x] 6.7 Implement `onSocketClose` callback
- [x] 6.8 Implement `unrouteWebSocket()` method
- [x] 6.9 Implement `unrouteAllWebSockets()` method
- [x] 6.10 Update TypeScript interfaces for WebSocket routing
- [x] 6.11 Verify WebSocket interception works end-to-end

## 7. Visual Testing (toHaveScreenshot)

- [x] 7.1 Add `VisualComparisonOptions` struct in `src/assertions/`
- [x] 7.2 Implement `toHaveScreenshot()` matcher in `matchers.rs`
- [x] 7.3 Implement screenshot baseline loading/saving
- [x] 7.4 Implement pixel diff comparison using `image` crate
- [x] 7.5 Add threshold comparison logic
- [x] 7.6 Add `ignoreRegions` support
- [x] 7.7 Implement diff image generation
- [x] 7.8 Add CLI flag `--update-screenshots` for baseline updates
- [x] 7.9 Add screenshot directories configuration
- [x] 7.10 Implement CI screenshot bucket upload/download
- [x] 7.11 Update TypeScript interface for `toHaveScreenshot` options
- [x] 7.12 Verify visual comparison works with known baselines

## 8. Project Configuration (tsheet.config.ts)

- [x] 8.1 Add `cosmiconfig` dependency for config loading
- [x] 8.2 Create `TsheetConfig` struct in `src/test_runner/config.rs`
- [x] 8.3 Implement `load_config()` to find and parse config file
- [x] 8.4 Support `tsheet.config.ts`, `.tsheetrc`, `package.json` lookup
- [x] 8.5 Apply `testDir` configuration to test discovery
- [x] 8.6 Apply `testMatch` configuration to test discovery
- [x] 8.7 Apply `timeout` configuration to test execution
- [x] 8.8 Apply `retries` configuration to test execution
- [x] 8.9 Apply `workers` configuration to parallel execution
- [x] 8.10 Implement `globalSetup` execution
- [x] 8.11 Implement `globalTeardown` execution
- [x] 8.12 Implement projects configuration (multiple browser/viewport)
- [x] 8.13 Update CLI to load and merge config
- [x] 8.14 Verify config loading works from project root

## 9. Migration Tools

- [x] 9.1 Create `src/migrate/playwright.rs` with AST parser
- [x] 9.2 Create `src/migrate/cypress.rs` with AST parser
- [x] 9.3 Create `src/migrate/puppeteer.rs` with AST parser
- [x] 9.4 Create API mapping tables for each framework
- [x] 9.5 Implement `migratePlaywright()` NAPI function
- [x] 9.6 Implement `migratePlaywrightFile()` NAPI function
- [x] 9.7 Implement `migrateCypress()` NAPI function
- [x] 9.8 Implement `migrateCypressFile()` NAPI function
- [x] 9.9 Implement `migratePuppeteer()` NAPI function
- [x] 9.10 Implement `migratePuppeteerFile()` NAPI function
- [x] 9.11 Implement migration report generation
- [x] 9.12 Add `tsheet migrate` CLI command
- [x] 9.13 Add migration batch processing for directories
- [x] 9.14 Verify migration produces working TurboSheet tests

## 10. Auto-Wait

- [x] 10.1 Add `auto_wait` configuration option to `LocatorOptions`
- [x] 10.2 Implement wait for element visibility before actions
- [x] 10.3 Implement wait for element attachment before actions
- [x] 10.4 Implement wait for element enabled state before actions
- [x] 10.5 Implement wait for element stability (not animating)
- [x] 10.6 Implement wait for navigation after click
- [x] 10.7 Add `noWaitAfter` option to disable auto-wait per action
- [x] 10.8 Update locator methods to use auto-wait by default
- [x] 10.9 Add timeout configuration for auto-wait
- [x] 10.10 Verify auto-wait improves test reliability

## 11. Integration Testing

- [x] 11.1 Run full test suite on Chromium
- [x] 11.2 Run full test suite on Firefox
- [x] 11.3 Run full test suite on WebKit (macOS)
- [x] 11.4 Verify visual screenshot tests with CI
- [x] 11.5 Verify migration tools convert Playwright tests correctly
- [x] 11.6 Verify migration tools convert Cypress tests correctly
- [x] 11.7 Verify migration tools convert Puppeteer tests correctly
- [x] 11.8 Performance benchmark: TurboSheet vs Playwright

## 12. Documentation

- [x] 12.1 Update README with multi-browser support
- [x] 12.2 Document tsheet.config.ts options
- [x] 12.3 Document migration CLI usage
- [x] 12.4 Document visual testing setup
- [x] 12.5 Add examples directory with test samples
