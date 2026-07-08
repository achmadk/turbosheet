## ADDED Requirements

### Requirement: beforeAll runs once per suite before its tests

The `beforeAll` hook SHALL execute exactly once per suite, before any test in that suite or its descendant suites. If a suite's beforeAll has already run for a previous test, it SHALL NOT re-run for subsequent tests in the same suite.

#### Scenario: beforeAll at root level runs once

- **WHEN** a test file has a root-level `test.beforeAll()` and 3 root-level tests
- **THEN** the beforeAll hook SHALL execute before the first test only
- **AND** SHALL NOT execute before the second or third tests

#### Scenario: Nested beforeAll scoping

- **WHEN** `describe("Outer")` has `test.beforeAll()` and `describe("Inner")` also has `test.beforeAll()`
- **THEN** the outer beforeAll SHALL execute before the first test in either suite
- **AND** the inner beforeAll SHALL execute before the first test in the inner suite (after outer beforeAll)

#### Scenario: beforeAll failure skips suite

- **WHEN** a suite's beforeAll hook throws an error
- **THEN** all tests in that suite and all descendant suites SHALL be marked as skipped
- **AND** the error message SHALL be recorded as the skip reason for each test

### Requirement: afterAll runs once per suite after its tests

The `afterAll` hook SHALL execute exactly once per suite, after all tests in that suite and its descendant suites have completed.

#### Scenario: afterAll at root level runs once

- **WHEN** a test file has root-level `test.afterAll()` and 3 root-level tests
- **THEN** the afterAll hook SHALL execute after the last test completes

#### Scenario: afterAll runs even on test failures

- **WHEN** a test in a suite fails
- **THEN** the suite's afterAll SHALL still execute
- **AND** afterAll errors SHALL NOT change test statuses

### Requirement: beforeEach runs before each test with inheritance

The `beforeEach` hook SHALL execute before each test in its suite. Hooks from parent suites SHALL execute before hooks from child suites.

#### Scenario: Inherited beforeEach order

- **WHEN** `describe("Parent")` has `test.beforeEach()` and `describe("Child")` also has `test.beforeEach()`
- **THEN** before each test in Child suite, the Parent beforeEach runs first, then the Child beforeEach

#### Scenario: beforeEach failure marks test as failed

- **WHEN** a beforeEach hook throws an error
- **THEN** the current test SHALL be marked as failed with the hook error
- **AND** subsequent tests in the same suite SHALL NOT be affected

### Requirement: afterEach runs after each test with inheritance

The `afterEach` hook SHALL execute after each test in its suite, regardless of test outcome. Hooks from child suites SHALL execute before parent suite hooks (reverse order of beforeEach).

#### Scenario: afterEach runs on test failure

- **WHEN** a test throws an error
- **THEN** the afterEach hooks SHALL still execute in reverse inheritance order

#### Scenario: afterEach errors do not affect test status

- **WHEN** an afterEach hook throws an error
- **THEN** the test result SHALL remain unchanged (pass or fail from the test itself)
- **AND** the afterEach error MAY be logged or recorded as supplementary information
