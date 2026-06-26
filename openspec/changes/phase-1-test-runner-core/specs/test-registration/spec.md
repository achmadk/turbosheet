## ADDED Requirements

### Requirement: Register basic test case

The system SHALL provide a `test(name, fn)` function that registers a test case with a name and a callback function. The test case SHALL be stored in a Rust-side `TestRegistry` and executed when the test run begins.

#### Scenario: Register and execute a passing test

- **WHEN** user calls `test("should login", async () => { ... })` in a test file
- **THEN** the system registers the test case with name "should login"
- **THEN** the system executes the callback during the test run
- **THEN** the test result shows status "passed" if no error is thrown

### Requirement: Focus a test with test.only

The system SHALL provide `test.only(name, fn)` that registers a focused test. When any `test.only` is present in a file, only focused tests SHALL run; all others SHALL be skipped.

#### Scenario: Only focused tests run when test.only is used

- **WHEN** user registers `test("a", fn1)`, `test.only("b", fn2)`, and `test("c", fn3)`
- **THEN** only test "b" SHALL execute
- **THEN** tests "a" and "c" SHALL be marked as skipped

### Requirement: Skip a test with test.skip

The system SHALL provide `test.skip(name, fn)` that registers a test to be unconditionally skipped during execution.

#### Scenario: Skipped test does not execute

- **WHEN** user calls `test.skip("slow test", async () => { ... })`
- **THEN** the test SHALL NOT execute its callback
- **THEN** the result SHALL show status "skipped"

### Requirement: Mark a test as fixme with test.fixme

The system SHALL provide `test.fixme(name, fn)` that registers a test as known-broken. A fixme test SHALL be skipped by default but can be run with a modifier flag.

#### Scenario: Fixme test is skipped by default

- **WHEN** user calls `test.fixme("flaky test", async () => { ... })`
- **THEN** the test SHALL be skipped during normal execution
- **THEN** the result SHALL show status "skipped" with a fixme annotation

### Requirement: Mark a test as expected-failure with test.fail

The system SHALL provide `test.fail(name, fn)` that registers a test expected to fail. If it passes, the system SHALL report it as unexpected success.

#### Scenario: Expected failure test fails as anticipated

- **WHEN** user calls `test.fail("known bug", async () => { throw new Error("bug"); })`
- **THEN** the test SHALL execute its callback
- **THEN** if the callback throws, the result SHALL show status "passed" (expected failure)
- **THEN** if the callback succeeds, the result SHALL show status "failed" (unexpected pass)

### Requirement: Mark a test as slow with test.slow

The system SHALL provide `test.slow(name, fn)` that registers a test with an extended timeout multiplier.

#### Scenario: Slow test gets triple timeout

- **WHEN** user calls `test.slow("large render", async () => { ... })`
- **THEN** the timeout SHALL be 3x the configured default timeout
- **THEN** the test SHALL execute with the extended timeout

### Requirement: Group tests with describe

The system SHALL provide `describe(name, fn)` that creates a test suite containing nested tests and suites. The callback SHALL be executed immediately to register children.

#### Scenario: Describe creates a named suite

- **WHEN** user calls `describe("Login flows", () => { test("valid", fn1); test("invalid", fn2); })`
- **THEN** a suite named "Login flows" SHALL be created
- **THEN** tests "valid" and "invalid" SHALL be registered under that suite

### Requirement: Serial execution with describe.serial

The system SHALL provide `describe.serial(name, fn)` that creates a suite where tests execute one after another, not in parallel.

#### Scenario: Serial suite executes tests sequentially

- **WHEN** user calls `describe.serial("DB tests", () => { test("a", fn1); test("b", fn2); })`
- **THEN** tests within this suite SHALL execute one at a time
- **THEN** a test SHALL start only after the previous test completes

### Requirement: Parallel execution with describe.parallel

The system SHALL provide `describe.parallel(name, fn)` that creates a suite where tests execute concurrently.

#### Scenario: Parallel suite executes tests concurrently

- **WHEN** user calls `describe.parallel("API tests", () => { test("a", fn1); test("b", fn2); })`
- **THEN** tests within this suite MAY execute concurrently
- **THEN** the suite completion time SHALL be less than the sum of individual test times

### Requirement: Skip a suite with describe.skip

The system SHALL provide `describe.skip(name, fn)` that creates a suite where all child tests are skipped.

#### Scenario: Skipped suite skips all children

- **WHEN** user calls `describe.skip("Slow tests", () => { test("a", fn1); test("b", fn2); })`
- **THEN** all tests within this suite SHALL be marked as skipped
- **THEN** no test callback SHALL execute

### Requirement: Focus a suite with describe.only

The system SHALL provide `describe.only(name, fn)` that creates a focused suite. When any `describe.only` exists, only focused suites and their children SHALL run.

#### Scenario: Only focused suite runs when describe.only is present

- **WHEN** user registers `describe("a", ...)`, `describe.only("b", ...)`, and `describe("c", ...)`
- **THEN** only suite "b" and its children SHALL execute
- **THEN** suites "a" and "c" SHALL be skipped

### Requirement: Extend test context with test.extend

The system SHALL provide `test.extend(fixtures)` that creates a new test function with additional fixture bindings.

#### Scenario: Extend adds typed fixtures

- **WHEN** user calls `const test2 = test.extend({ page: async ({}, use) => { ... } })`
- **THEN** `test2` SHALL accept a callback with fixture objects as the second parameter
- **THEN** fixtures SHALL be resolved before the test runs and cleaned up after
