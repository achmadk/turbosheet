## Why

TurboSheet's test executor returns hardcoded "Simulated Test" results and all assertions (`toBeVisible()`, `toHaveText()`, etc.) return pass without calling real CDP methods. This makes the entire test runner unusable - tests appear to pass even when they don't execute. This is Priority 1 because without real test execution, TurboSheet cannot compete with Playwright, Cypress, or any other E2E tool.

## What Changes

1. **Connect assertion matchers to real CDP methods** - `expect(locator).toBeVisible()` will call `locator.is_visible()` which calls `page.is_visible()` which executes CDP's `document.querySelector()` + `getComputedStyle()`
2. **Implement real JS test file execution** - Parse TypeScript/JavaScript test files using SWC orisolated-vm, extract `test()`, `test.describe()`, hooks, execute via napi-rs threadpool
3. **Connect test runner to real page/locator APIs** - Tests run against actual Chromium via CDP
4. **Add real test result collection** - Capture screenshots on failure, traces, proper duration measurement

## Capabilities

### New Capabilities

- `assertion-engine`: Assertion matchers that call real CDP/page methods with exponential backoff polling
- `test-executor`: Real JS/TS test file parsing and execution via isolated runtime
- `fixture-runtime`: Fixture lifecycle management (test-level, worker-level, global) with proper cleanup
- `hook-runtime`: beforeAll/afterAll/beforeEach/afterEach execution with proper scoping

### Modified Capabilities

- `page-api`: No requirement changes - implementation already supports needed methods
- `locator-api`: No requirement changes - implementation already supports needed methods

## Impact

**Code Changes:**

- `src/assertions/matchers.rs` - Replace all hardcoded mock values with real CDP calls
- `src/test_runner/executor.rs` - Replace simulated results with real test execution
- `src/test_runner/discovery.rs` - May need enhancement for TS parsing
- `src/test_runner/fixtures.rs` - Real fixture lifecycle implementation
- `src/test_runner/hooks.rs` - Real hook execution

**Dependencies:**

- Need JS runtime for executing user test code: QuickJS or isolated-vm
- May need SWC for TypeScript parsing

**Risk:** Breaking change for anyone who relies on current mock behavior (unlikely - mock behavior is unusable)
