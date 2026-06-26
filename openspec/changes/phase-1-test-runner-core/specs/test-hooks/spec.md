## ADDED Requirements

### Requirement: Run setup once per suite with beforeAll

The system SHALL provide a `beforeAll(fn)` function that registers a callback to run once before all tests in the enclosing suite. The callback SHALL receive a fixture context object.

#### Scenario: beforeAll runs once before suite tests

- **WHEN** user registers `beforeAll(async () => { await db.seed(); })` inside a `describe` block
- **THEN** the callback SHALL execute once before any test in that suite
- **THEN** the callback SHALL NOT execute again for subsequent tests in the same suite

#### Scenario: beforeAll failure skips suite tests

- **WHEN** user registers `beforeAll(async () => { throw new Error("setup failed"); })`
- **THEN** all tests in the suite SHALL be skipped
- **THEN** the test results SHALL show the beforeAll error message

### Requirement: Run teardown once per suite with afterAll

The system SHALL provide an `afterAll(fn)` function that registers a callback to run once after all tests in the enclosing suite complete, regardless of pass/fail status.

#### Scenario: afterAll runs after suite tests complete

- **WHEN** user registers `afterAll(async () => { await db.cleanup(); })` inside a `describe` block
- **THEN** the callback SHALL execute after all tests in that suite finish
- **THEN** the callback SHALL execute even if some tests failed

### Requirement: Run setup before each test with beforeEach

The system SHALL provide a `beforeEach(fn)` function that registers a callback to run before every test in the enclosing suite.

#### Scenario: beforeEach runs before each test

- **WHEN** user registers `beforeEach(async () => { page.goto("/"); })` and the suite has 3 tests
- **THEN** the callback SHALL execute 3 times, once before each test
- **THEN** each test SHALL see the effects of the beforeEach

#### Scenario: beforeEach failure skips the current test only

- **WHEN** user registers `beforeEach(async () => { throw new Error("precondition failed"); })` and the suite has 2 tests
- **THEN** the first test SHALL be skipped with the beforeEach error
- **THEN** the second test SHALL also attempt beforeEach (which may also skip it)

### Requirement: Run teardown after each test with afterEach

The system SHALL provide an `afterEach(fn)` function that registers a callback to run after every test in the enclosing suite, regardless of pass/fail.

#### Scenario: afterEach runs after each test

- **WHEN** user registers `afterEach(async () => { await page.screenshot(); })` and the suite has 2 tests
- **THEN** the callback SHALL execute 2 times, once after each test
- **THEN** the callback SHALL execute even if the test failed

### Requirement: Hooks inherit from parent suites

The system SHALL execute hooks from outer suites before inner suites. A `beforeEach` on an outer describe SHALL run before tests in a nested describe.

#### Scenario: Nested hooks compose hierarchically

- **WHEN** `describe("outer", () => { beforeEach(outerFn); describe("inner", () => { beforeEach(innerFn); test("t", fn); }); })`
- **THEN** before test "t": `outerFn` SHALL execute first, then `innerFn`
- **THEN** after test "t": `innerFn` SHALL run (if defined), then any outer afterEach

### Requirement: Hook timeout is configurable

The system SHALL respect the suite's timeout configuration for hooks, with a default of 30 seconds.

#### Scenario: Hook exceeding timeout is reported as failure

- **WHEN** a `beforeAll` hook takes longer than the configured timeout
- **THEN** the hook SHALL be terminated
- **THEN** the suite SHALL report the hook failure with a timeout error message
