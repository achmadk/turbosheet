## ADDED Requirements

### Requirement: Test file discovery

TurboSheet SHALL automatically discover test files matching `**/*.tsheet.ts` or `**/*.tsheet.spec.ts` patterns.

#### Scenario: Default discovery

- **WHEN** user runs `tsheet test`
- **THEN** TurboSheet SHALL search the project root for all matching `*.tsheet.ts` files
- **THEN** TurboSheet SHALL exclude `node_modules` and build output directories

#### Scenario: Custom test pattern

- **WHEN** user sets `testMatch: ['**/e2e/**/*.ts']` in configuration
- **THEN** TurboSheet SHALL use the custom pattern instead of default

### Requirement: Test organization

TurboSheet SHALL support `test.describe`, `test()`, `test.only`, `test.skip`, `test.fixme`, `test.fail`, `test.slow`.

#### Scenario: Describe block

- **WHEN** user wraps tests in `test.describe('suite', () => { ... })`
- **THEN** tests SHALL be grouped under the suite name in reports

#### Scenario: Skip a test

- **WHEN** user marks `test.skip('broken feature', ...)`
- **THEN** the test SHALL not execute and SHALL appear as skipped in reports

### Requirement: Fixture system

TurboSheet SHALL support dependency-injected fixtures via `test.extend()` with automatic scope management.

#### Scenario: Simple fixture

- **WHEN** user defines `const test = tsheet.extend({ page: ... })` with a fixture factory
- **THEN** each test worker SHALL receive a fresh fixture instance

#### Scenario: Fixture dependency chain

- **WHEN** fixture A depends on fixture B (e.g., `page` depends on `context`)
- **THEN** TurboSheet SHALL resolve the dependency graph automatically

#### Scenario: Worker-scoped fixture

- **WHEN** user marks fixture `scope: 'worker'`
- **THEN** the fixture SHALL be shared across all tests in the same worker

### Requirement: Hooks

TurboSheet SHALL support `test.beforeAll`, `test.afterAll`, `test.beforeEach`, `test.afterEach` hooks.

#### Scenario: beforeEach hook

- **WHEN** user defines `test.beforeEach(async ({ page }) => { await page.goto('/'); })`
- **THEN** the hook SHALL execute before every test in the describe block

### Requirement: Auto-retrying assertions

TurboSheet SHALL provide auto-retrying assertion matchers that poll until timeout or success.

#### Scenario: Element visibility assertion

- **WHEN** user calls `await expect(page.locator('.toast')).toBeVisible()`
- **THEN** TurboSheet SHALL poll the selector engine in Rust until element is visible or timeout (default 5s)

#### Scenario: Text content assertion

- **WHEN** user calls `await expect(page.locator('.title')).toHaveText('Welcome')`
- **THEN** TurboSheet SHALL retry until element text matches exactly

#### Scenario: Assertion timeout

- **WHEN** element never matches the expected condition
- **THEN** assertion SHALL throw a descriptive timeout error after configurable timeout

### Requirement: Test execution control

TurboSheet SHALL support parallel execution, serial execution, sharding, and retries.

#### Scenario: Parallel execution

- **WHEN** user runs tests without `--serial`
- **THEN** test files SHALL execute in parallel using the Rust Tokio worker pool

#### Scenario: Serial execution

- **WHEN** user marks `test.describe.serial('suite', ...)`
- **THEN** tests within that describe block SHALL execute sequentially

#### Scenario: CI sharding

- **WHEN** user runs `tsheet test --shard=1/4`
- **THEN** TurboSheet SHALL only run the first quarter of tests

#### Scenario: Flaky retry

- **WHEN** a test fails and `--retries=2` is configured
- **THEN** TurboSheet SHALL retry the failed test up to 2 times before reporting final failure

### Requirement: Rust-based assertion engine

The assertion polling loop SHALL run in the Rust Tokio runtime, not the Node.js event loop.

#### Scenario: Zero JS blocking during assertion wait

- **WHEN** an assertion is retrying (e.g., waiting 3s for element to appear)
- **THEN** the Node.js event loop SHALL NOT be blocked — polling runs in Rust

#### Scenario: High-frequency assertions

- **WHEN** 100+ tests simultaneously run `expect(...).toBeVisible()`
- **THEN** all assertion polling SHALL execute in Tokio tasks without contention
