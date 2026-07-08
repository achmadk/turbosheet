## ADDED Requirements

### Requirement: Reporter trait definition

The system SHALL define a `Reporter` trait in `src/reporters/mod.rs` with two methods:

- `on_test_result(&self, result: &TestResult)` — called after each test completes
- `on_complete(&self, summary: &AggregatedTestResult)` — called after all tests finish, with the aggregate summary

#### Scenario: Trait is defined

- **WHEN** the compiler resolves `src/reporters/mod.rs`
- **THEN** the `Reporter` trait SHALL exist with `on_test_result` and `on_complete` methods

### Requirement: All 7 reporters implement Reporter

The system SHALL implement the `Reporter` trait for all 7 existing reporters: Dot, Line, List, Json, Junit, Html, Github.

Each implementation SHALL delegate to the reporter's existing internal methods so output is unchanged.

#### Scenario: Dot reporter implements Reporter

- **WHEN** `DotReporter` is constructed and passed as `&dyn Reporter`
- **THEN** `on_test_result()` SHALL update the progress dots identically to the current behavior
- **AND** `on_complete()` SHALL print the final summary identically to the current behavior

#### Scenario: Json reporter implements Reporter

- **WHEN** `JsonReporter` is constructed and passed as `&dyn Reporter`
- **THEN** `on_test_result()` SHALL buffer the result
- **AND** `on_complete()` SHALL write the JSON output identically to the current behavior

#### Scenario: All reporters produce identical output

- **WHEN** the same test suite runs with the old and new reporter wiring
- **THEN** each reporter's output SHALL be identical between old and new wiring

### Requirement: TestExecutor accepts Reporter trait

The system SHALL refactor `TestExecutor::execute()` to accept `&dyn Reporter` instead of a closure parameter.

#### Scenario: Executor calls on_test_result

- **WHEN** a test completes inside `TestExecutor::execute()`
- **THEN** the executor SHALL call `reporter.on_test_result(&result)`

#### Scenario: Executor calls on_complete

- **WHEN** all tests in a batch finish inside `TestExecutor::execute()`
- **THEN** the executor SHALL call `reporter.on_complete(&summary)`

### Requirement: run_tests constructs reporters via trait

The system SHALL refactor `run_tests()` in `src/test_runner/mod.rs` to construct reporters as `Box<dyn Reporter>` and pass them to the executor.

#### Scenario: CLI reporter selection

- **WHEN** `--format dot` is passed on the CLI
- **THEN** `run_tests()` SHALL construct a `DotReporter` and pass it as `Box<dyn Reporter>` to `TestExecutor::execute()`

### Requirement: ReporterWithProgress trait (optional)

Reporters that support live progress (Dot, Line, List) SHALL implement an optional `ReporterWithProgress` trait with method `on_progress(elapsed: Duration, completed: usize, total: usize)`.

The executor SHALL attempt to downcast and call `on_progress` periodically during test execution.

#### Scenario: Dot reporter shows progress

- **WHEN** tests are running with the Dot reporter
- **THEN** `on_progress()` SHALL be called periodically to show live progress dots

#### Scenario: Json reporter does not implement progress

- **WHEN** tests are running with the Json reporter
- **THEN** no progress callbacks SHALL be invoked (no-op downcast failure handled gracefully)
