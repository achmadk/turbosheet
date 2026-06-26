## Why

TurboSheet is a Rust-based browser automation framework with significant architectural advantages over existing tools (Puppeteer, Cypress, Playwright): native CDP performance, isolated VM pooling for test execution, and Rust's memory efficiency. However, it's currently incomplete—Chromium works but Firefox/WebKit are stubbed, and critical E2E testing features (real test execution, proper locators, network automation, visual testing, migration tools) are missing or non-functional. This change completes TurboSheet into a production-ready E2E testing platform.

## What Changes

### Browser Support

- **Complete Firefox implementation** via GeckoDriver/WebDriver protocol
- **Complete WebKit implementation** via safaridriver or webkitwd
- **Cross-browser test execution** with single API

### Test Execution Pipeline

- **Real test runner** - wire up `TestExecutor` to parse and run actual test files
- **Full fixture lifecycle** - test-level, worker-level, and global fixtures with proper disposal
- **Complete hook execution** - beforeAll/afterAll/beforeEach/afterEach with proper scoping for nested describes
- **Worker process isolation** - parallel execution with proper process forking and result aggregation
- **Global setup/teardown** - project-level lifecycle hooks

### Locator API Completeness

- `scrollIntoView()`, `focus()`, `blur()`
- `boundingBox()` returning element rect
- `screenshot()` for element capture
- `dragAndDrop()`, `hover()`, `press()`, `pressSequentially()`
- `setInputFiles()` for file upload

### Page API Completeness

- `setViewportSize()`, `reload()`, `goBack()`, `goForward()`
- `waitForRequest()`, `waitForResponse()`, `waitForSelector()`
- `evaluateHandle()` returning JSHandle
- `addScriptTag()`, `addStyleTag()`
- `exposeFunction()` for bidirectional JS↔Rust communication
- Full event handling: `on()`, `off()`

### Network Automation

- WebSocket interception via `routeWebSocket()`
- Request/Response body capture
- Fetch interception (page.route should intercept fetch)
- Service worker handling

### Visual Testing

- `toHaveScreenshot()` matcher with automatic polling
- Screenshot comparison with pixel diff
- Configurable threshold
- Automatic baseline management
- CI screenshot storage integration

### Configuration

- `tsheet.config.ts` project configuration (like playwright.config.ts)
- GlobalSetup/GlobalTeardown
- FullyParallel vs Serial execution modes
- Timeout overrides per test
- Reporter configuration per test

### Migration Tools

- Playwright → TurboSheet test converter
- Cypress → TurboSheet test converter
- Puppeteer → TurboSheet test converter
- Batch migration with reporting

## Capabilities

### New Capabilities

- `multi-browser-launch`: Cross-browser launch abstraction (Firefox, WebKit) with unified API
- `firefox-automation`: Firefox automation via GeckoDriver WebDriver protocol
- `webkit-automation`: WebKit automation via safaridriver
- `real-test-execution`: Complete test runner with worker process isolation
- `complete-locator-api`: Full locator operations (scroll, focus, boundingBox, screenshot, drag, press)
- `complete-page-api`: Full page operations (viewport, navigation, waits, evaluateHandle, exposeFunction)
- `network-websocket`: WebSocket interception and routing
- `visual-screenshot-testing`: toHaveScreenshot matcher with baseline management
- `project-configuration`: tsheet.config.ts with global setup/teardown
- `test-migration`: Playwright/Cypress/Puppeteer test conversion tools
- `auto-wait`: Automatic waiting for elements and conditions

### Modified Capabilities

- (none - new capabilities only)

## Impact

### Affected Code

- `src/engine/` - add Firefox and WebKit engine implementations
- `src/test_runner/` - complete executor, add worker process spawning
- `src/locator.rs` - add missing locator methods
- `src/page.rs` - add missing page methods
- `src/network/` - add WebSocket routing
- `src/assertions/` - add toHaveScreenshot matcher
- `src/migrate/` - implement migration converters
- `cli/` - add tsheet.config.ts loading
- `package.json` - add new dependencies (geckodriver, webkitdriver)

### New Dependencies

- `geckodriver` npm package (or system installation)
- `safaridriver` (system)
- Configuration file parsing (cosmiconfig or tsconfig-paths)

### Breaking Changes

- None (completing stubbed functionality)
