## ADDED Requirements

### Requirement: Test File Discovery

The test executor SHALL discover test files matching glob patterns `**/*.tsheet.ts` and `**/*.tsheet.spec.ts` in the configured `testDir`.

#### Scenario: Single matching file

- **WHEN** testDir contains one file `tests/login.spec.ts`
- **THEN** discovery SHALL return a single TestFile with path `tests/login.spec.ts`

#### Scenario: Multiple matching files

- **WHEN** testDir contains `tests/a.spec.ts`, `tests/b.tsheet.ts`, `tests/c.ts`
- **THEN** discovery SHALL return TestFiles for `a.spec.ts` and `b.tsheet.ts` only (not `c.ts`)

#### Scenario: No matching files

- **WHEN** testDir contains no matching files
- **THEN** discovery SHALL return empty vector

### Requirement: Test File Execution

The test executor SHALL load, parse, and execute test files using an isolated JavaScript runtime.

#### Scenario: Execute single test file

- **WHEN** executor receives TestFile for `login.spec.ts`
- **THEN** executor SHALL evaluate the file contents in isolated-vm context
- **AND** SHALL extract test suite structure (describe blocks, tests, hooks)

#### Scenario: Extract test cases

- **WHEN** file contains `test.describe("suite", () => { test("case1", ...); test("case2", ...); })`
- **THEN** executor SHALL create TestCase entries for "case1" and "case2"

#### Scenario: Collect hooks

- **WHEN** file contains `test.beforeAll(() => { ... })` inside describe block
- **THEN** executor SHALL register hook to run before tests in that describe block

### Requirement: Test Result Reporting

The test executor SHALL report results via a channel with actual execution data.

#### Scenario: Passing test

- **WHEN** a test executes without errors
- **THEN** executor SHALL emit TestResult with status=Passed, real duration_ms, no error_message

#### Scenario: Failing test with error

- **WHEN** a test throws an error or assertion fails
- **THEN** executor SHALL emit TestResult with status=Failed, error_message containing the error

#### Scenario: Test timeout

- **WHEN** a test exceeds configured timeout
- **THEN** executor SHALL emit TestResult with status=Timeout

### Requirement: Parallel Worker Execution

The test executor SHALL distribute tests across configured number of workers using Tokio task spawning.

#### Scenario: Two workers process four tests

- **WHEN** workers=2 and 4 tests exist
- **THEN** worker 1 SHALL process 2 tests and worker 2 SHALL process 2 tests concurrently

#### Scenario: Worker receives chunk of files

- **WHEN** worker processes its assigned chunk
- **THEN** it SHALL execute tests within its chunk sequentially

### Requirement: Screenshot on Failure

The test executor SHALL capture screenshot when test fails if `screenshotOnFailure` is enabled.

#### Scenario: Capture screenshot on failure

- **WHEN** test fails and screenshotOnFailure=true
- **THEN** executor SHALL call `page.screenshot()` and attach path to TestResult.screenshot_paths

#### Scenario: No screenshot when passing

- **WHEN** test passes
- **THEN** executor SHALL NOT capture screenshot
