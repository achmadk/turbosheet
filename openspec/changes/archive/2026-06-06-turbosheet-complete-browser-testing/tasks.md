## 1. Foundation: Real Test Execution

- [ ] 1.1 Wire isolated-v8 JsRuntime to test executor
- [ ] 1.2 Implement test file discovery for `**/*.tsheet.ts` and `**/*.tsheet.spec.ts`
- [ ] 1.3 Create SWC transpilation pipeline for TypeScript parsing
- [ ] 1.4 Implement TestSuiteExtractor to parse describe/test/hooks structure
- [ ] 1.5 Inject TurboSheet globals (page, expect, test) into isolated-v8 context
- [ ] 1.6 Implement sync-to-async bridge using tokio channels for CDP calls
- [ ] 1.7 Execute real test functions within isolate and capture results
- [ ] 1.8 Implement timeout handling with isolate termination
- [ ] 1.9 Collect TestResult with real duration, status, and error messages
- [ ] 1.10 Add screenshot capture on test failure

## 2. Foundation: Complete Missing PageEngine Methods

- [ ] 2.1 Implement bounding_box() in ChromiumPageEngine using CDP DOM.getBoxModel
- [ ] 2.2 Implement real drag_and_drop() using CDP Input.dispatchDragEvent
- [ ] 2.3 Implement press() using CDP Input.dispatchKeyEvent
- [ ] 2.4 Implement press_sequentially() with character-by-character dispatch
- [ ] 2.5 Implement set_input_files() using CDP Page.setInputFiles
- [ ] 2.6 Implement wait_for_selector() in ChromiumPageEngine
- [ ] 2.7 Verify all methods work via locator.rs delegation

## 3. High Priority: Firefox Automation

- [ ] 3.1 Implement FirefoxEngine with geckodriver WebDriver client
- [ ] 3.2 Implement FirefoxContextEngine for context management
- [ ] 3.3 Implement FirefoxPageEngine with WebDriver BiDi commands
- [ ] 3.4 Wire Firefox engine to browser.rs launch factory
- [ ] 3.5 Implement Firefox locator API (CSS, text, filter)
- [ ] 3.6 Implement Firefox network interception via WebDriver
- [ ] 3.7 Implement Firefox route_web_socket for WebSocket interception
- [ ] 3.8 Test Firefox automation end-to-end

## 4. High Priority: WebKit Automation

- [ ] 4.1 Integrate wry crate for webkit2gtk bindings
- [ ] 4.2 Implement WebKitEngine with webkit2gtk webview
- [ ] 4.3 Implement WebKitContextEngine for context management
- [ ] 4.4 Implement WebKitPageEngine with WebKitAutomation API
- [ ] 4.5 Wire WebKit engine to browser.rs launch factory
- [ ] 4.6 Implement WebKit locator API (CSS, text, filter)
- [ ] 4.7 Implement WebKit network interception
- [ ] 4.8 Implement WebKit touch events (tap, swipe, pinch)
- [ ] 4.9 Test WebKit automation end-to-end

## 5. High Priority: Smart Auto-Wait System

- [ ] 5.1 Implement visibility wait condition in AssertionEngine
- [ ] 5.2 Implement stability wait with MutationObserver for DOM changes
- [ ] 5.3 Implement actionability check (visible + enabled + stable + not covered)
- [ ] 5.4 Add exponential backoff with jitter to polling
- [ ] 5.5 Implement waitForFunction() for custom conditions
- [ ] 5.6 Add configurable timeout per action and globally
- [ ] 5.7 Implement force option to bypass auto-wait
- [ ] 5.8 Add covered element detection via elementFromPoint

## 6. Medium Priority: Visual Regression Engine

- [ ] 6.1 Implement fullPage screenshot with scrolling and stitching
- [ ] 6.2 Implement element screenshot using bounding box crop
- [ ] 6.3 Add pixel diff comparison with configurable threshold
- [ ] 6.4 Implement toHaveScreenshot assertion with baseline management
- [ ] 6.5 Implement ignoreRegions for dynamic content
- [ ] 6.6 Add UPDATE_VISUAL_BASELINES env var support
- [ ] 6.7 Implement semantic diff with image crate integration
- [ ] 6.8 Add layout shift (CLS) detection

## 7. Medium Priority: TurboTrace Viewer

- [ ] 7.1 Define TurboTrace file format with TTRC header
- [ ] 7.2 Implement trace recording for actions, DOM, network, console
- [ ] 7.3 Implement zstd + ciborium compression
- [ ] 7.4 Create trace viewer web UI with timeline
- [ ] 7.5 Implement DOM snapshot viewer at action point
- [ ] 7.6 Implement network waterfall visualization
- [ ] 7.7 Add tsheet trace info CLI command
- [ ] 7.8 Add tsheet trace export CLI command
- [ ] 7.9 Implement trace server for real-time streaming
- [ ] 7.10 Add trace annotations (labels, screenshots)

## 8. Medium Priority: Migration Tooling

- [ ] 8.1 Create tsheet migrate CLI command
- [ ] 8.2 Implement Playwright test file detection
- [ ] 8.3 Implement Cypress test file detection
- [ ] 8.4 Implement Playwright-to-TurboSheet AST transformation
- [ ] 8.5 Implement Cypress-to-TurboSheet AST transformation
- [ ] 8.6 Add migration warnings for unsupported features
- [ ] 8.7 Implement --dry-run mode with stdout output
- [ ] 8.8 Implement file writing with directory preservation
- [ ] 8.9 Add conversion statistics summary
- [ ] 8.10 Create migration guide documentation

## 9. Lower Priority: Swarm Scale Architecture

- [ ] 9.1 Implement context pooling with reset
- [ ] 9.2 Create task-per-context model in tokio
- [ ] 9.3 Implement parallel test distribution across workers
- [ ] 9.4 Add shard distribution (--shard=N/M)
- [ ] 9.5 Implement worker parallelism (--workers=N)
- [ ] 9.6 Add load balancing with duration tracking
- [ ] 9.7 Implement swarm coordinator mode
- [ ] 9.8 Implement worker registration
- [ ] 9.9 Add resource limits (memory, CPU, global cap)
- [ ] 9.10 Test 1000+ concurrent contexts

## 10. Lower Priority: Component Testing

- [ ] 10.1 Create component mounting API
- [ ] 10.2 Implement React component mounting with jsdom
- [ ] 10.3 Implement Vue component mounting with Vue test utils
- [ ] 10.4 Implement Svelte component mounting
- [ ] 10.5 Implement component locator API for internal elements
- [ ] 10.6 Add props and state access API
- [ ] 10.7 Implement framework auto-detection from imports
- [ ] 10.8 Add component cleanup on unmount/timeout
- [ ] 10.9 Create component testing configuration options

## 11. Lower Priority: CI/CD Infrastructure

- [ ] 11.1 Create official Dockerfile with multi-stage build
- [ ] 11.2 Build multi-arch images (amd64, arm64)
- [ ] 11.3 Implement browser binary caching mechanism
- [ ] 11.4 Create GitHub Actions workflow template
- [ ] 11.5 Create GitLab CI template
- [ ] 11.6 Create Jenkins shared library
- [ ] 11.7 Add retry-on-failure (--retries=N) support
- [ ] 11.8 Implement trace upload on failure for CI

## 12. Lower Priority: VS Code Extension

- [ ] 12.1 Create VS Code extension project structure
- [ ] 12.2 Implement Test Explorer provider for tsheet tests
- [ ] 12.3 Add run/debug test buttons in editor gutter
- [ ] 12.4 Implement inline pass/fail decorations
- [ ] 12.5 Embed TurboTrace viewer web UI
- [ ] 12.6 Add navigation from trace to source
- [ ] 12.7 Provide TurboSheet API IntelliSense completions
- [ ] 12.8 Create configuration UI for settings
- [ ] 12.9 Package and publish VS Code extension
