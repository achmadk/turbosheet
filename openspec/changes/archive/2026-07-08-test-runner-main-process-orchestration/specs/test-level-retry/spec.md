## ADDED Requirements

### Requirement: Retry operates at individual test granularity

The Rust orchestrator SHALL retry individual tests within a plan rather than retrying entire files. Each `ExecutionPlan` SHALL have its own `attempt_count` and `max_retries`.

#### Scenario: Single test retry on failure

- **WHEN** an `ExecutionPlan` with `max_retries: 2` fails on first attempt
- **THEN** the orchestrator SHALL re-send the same plan to a worker (or the same worker) for a second attempt
- **AND** other tests in the same file that passed SHALL NOT be re-executed

#### Scenario: Retry count exhausted

- **WHEN** a test has failed `max_retries + 1` times
- **THEN** the test SHALL be reported with its latest failure status
- **AND** the `retries` field in the `TestResult` SHALL reflect the number of retry attempts

#### Scenario: Retry does not re-run beforeAll

- **WHEN** a test is retried after its suite's beforeAll has already run
- **THEN** the retry SHALL NOT re-execute beforeAll
- **AND** the `ExecutionPlan` for retry SHALL have `run_before_all: false`

#### Scenario: Flaky test passes on retry

- **WHEN** a test fails on first attempt but passes on retry
- **THEN** the test SHALL be reported as passed
- **AND** the `retries` field SHALL indicate 1 retry occurred
