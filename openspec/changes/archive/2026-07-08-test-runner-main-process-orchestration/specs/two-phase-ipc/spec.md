## ADDED Requirements

### Requirement: Worker exposes `extractTests` method

The worker SHALL expose a JSON-RPC method `extractTests(filePath)` that reads, transpiles, and parses a test file without launching a browser or creating a page context. The method SHALL return the collected test definitions, hook definitions, and suite structure as structured data.

#### Scenario: Extract tests from a valid test file

- **WHEN** the Rust executor sends an `extractTests` request with a valid `.tsheet.ts` file path
- **THEN** the worker returns an `ExtractResponse` containing all `test()` calls with their modifiers, `describe()` blocks with nesting, and hook registrations (beforeAll, afterAll, beforeEach, afterEach)

#### Scenario: Extract tests from invalid TypeScript

- **WHEN** the worker receives an `extractTests` request with a file containing syntax errors
- **THEN** the worker returns an error response with the compilation error details
- **AND** no further execution is attempted on that file

#### Scenario: Extract from file with no test definitions

- **WHEN** the worker extracts a file that contains no `test()` or `describe()` calls
- **THEN** the response SHALL include an empty tests array and empty suites array
- **AND** the response SHALL NOT be treated as an error

### Requirement: Worker exposes `runPlan` method

The worker SHALL expose a JSON-RPC method `runPlan(plan)` that receives an `ExecutionPlan` object and executes the specified tests with their associated hooks in an isolated-vm sandbox with a browser page fixture.

#### Scenario: Execute a plan with one test

- **WHEN** the Rust executor sends a `runPlan` request with an `ExecutionPlan` containing one test function body and its hooks
- **THEN** the worker launches a browser context, creates a page, evaluates the test in isolated-vm, and returns a single `TestResultMsg`

#### Scenario: Execute a plan with beforeEach/afterEach hooks

- **WHEN** the `ExecutionPlan` includes `before_each_hooks` and `after_each_hooks`
- **THEN** the each `before_each_hook` SHALL run before the test function in the same isolate context
- **AND** each `after_each_hook` SHALL run after the test function, even if the test fails

#### Scenario: Execute a plan with beforeAll that hasn't run yet

- **WHEN** the `ExecutionPlan` has `run_before_all: true`
- **THEN** the worker SHALL execute the beforeAll hooks before the first test in that suite
- **AND** subsequent tests in the same suite SHALL NOT re-run beforeAll

### Requirement: ExtractResponse data format

The `ExtractResponse` SHALL contain structured fields for tests, hooks, and suite definitions.

#### Scenario: ExtractResponse includes collected tests with modifiers

- **WHEN** a test file contains `test("a", ...)`, `test.only("b", ...)`, `test.skip("c", ...)`, `test.fixme("d", ...)`, `test.fail("e", ...)`, `test.slow("f", ...)`
- **THEN** the `ExtractResponse.tests` array SHALL contain entries with `name`, `suite_path`, and `modifier` fields reflecting each variant

#### Scenario: ExtractResponse preserves suite nesting

- **WHEN** test file has `describe("Outer", () => { describe("Inner", () => { test("deep", ...) }) })`
- **THEN** the collected test for "deep" SHALL have `suite_path: ["Outer", "Inner"]`

#### Scenario: ExtractResponse includes hooks scoped to suites

- **WHEN** hooks are defined inside describe blocks at different nesting levels
- **THEN** each hook SHALL include its `suite_path` so the Rust side can resolve inheritance

### Requirement: ExecutionPlan data format

The `ExecutionPlan` SHALL contain all data needed for a single test execution run: the test function body, hooks to run around it, timeout, and modifier behavior flags.

#### Scenario: ExecutionPlan includes function bodies as strings

- **WHEN** building an ExecutionPlan for a test
- **THEN** the test function body and all hook function bodies SHALL be serialized as JavaScript source strings

#### Scenario: ExecutionPlan includes modifier flags

- **WHEN** a test has `test.fail` modifier
- **THEN** the `ExecutionPlan.is_fail` flag SHALL be true
- **WHEN** a test has `test.fixme` modifier
- **THEN** the `ExecutionPlan.is_fixme` flag SHALL be true
- **WHEN** a test has `test.slow` modifier
- **THEN** the `ExecutionPlan.timeout_ms` SHALL be triple the configured timeout
