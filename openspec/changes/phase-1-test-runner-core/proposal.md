## Why

Turbosheet's test runner has all the infrastructure scaffolding — executor, worker pool, discovery, hook registry, config — but the JS-facing API (`test()`, `describe()`, hooks) are all empty stubs `{}` that do nothing. Tests can be discovered and workers can execute files, but there is no way to register individual test cases, define suites, or attach lifecycle hooks from user code. This makes the framework unusable for end-to-end testing.

## What Changes

- Implement `test()`, `test.only()`, `test.skip()`, `test.fixme()`, `test.fail()`, `test.slow()` with proper test case registration
- Implement `describe()`, `describe.serial()`, `describe.parallel()`, `describe.skip()`, `describe.only()` with suite nesting
- Implement `beforeAll()`, `afterAll()`, `beforeEach()`, `afterEach()` hooks with scope-aware lifecycle
- Wire the registration pipeline through `extractor.rs` (AST parsing) and `executor.rs` (test execution)
- Add a working list reporter with pass/fail/skip/timeout output similar to Playwright's list reporter
- Implement `test.extend()` for fixture injection

## Capabilities

### New Capabilities

- `test-registration`: Test case registration via `test()`, `test.only()`, `test.skip()`, `test.fixme()`, `test.fail()`, `test.slow()` with napi-rs bindings and a Rust-side test registry
- `test-hooks`: Lifecycle hooks (`beforeAll`, `afterAll`, `beforeEach`, `afterEach`) with suite-scoped tracking via the existing `HookRegistry`
- `test-reporter-list`: Console list reporter showing individual test results with status symbols, timing, error details, and summary line

### Modified Capabilities

- `injected-script-engine`: The `extractor.rs` module needs to be upgraded from a stub (returns "dummy test") to real AST-based test extraction using swc

## Impact

- `src/test_runner/mod.rs`: Replace all empty `{}` napi function bodies with real implementations backed by a `TestRegistry` singleton
- `src/test_runner/extractor.rs`: Implement real AST-based test suite extraction from TypeScript test files using swc
- `src/test_runner/executor.rs`: Wire extracted test cases through the existing worker pipeline instead of dummy fallback
- `src/reporters/`: Add or refine list reporter in the existing reporter system
- `Cargo.toml`: May need `swc` dependencies for AST parsing if not already present
- `index.js`: Re-export pattern already in place — verify bindings work end-to-end
