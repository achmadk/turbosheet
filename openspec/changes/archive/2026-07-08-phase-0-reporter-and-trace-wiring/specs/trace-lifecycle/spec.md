## ADDED Requirements

### Requirement: Clear recorder at test start

The system SHALL clear the trace recorder (`GLOBAL_RECORDER.clear()` or equivalent) at the start of each test's page operations in the Worker process.

#### Scenario: Recorder is empty before test

- **WHEN** a new test begins executing page operations in the Worker
- **THEN** the `GLOBAL_RECORDER` SHALL be cleared of any prior trace events

### Requirement: Serialize recorder at test completion

The system SHALL serialize the trace recorder state after each test's page operations complete, producing a JSON string of all recorded trace events.

#### Scenario: Recorder serialized after test actions

- **WHEN** a test's page operations finish and before the test result is sent via IPC
- **THEN** the `GLOBAL_RECORDER` SHALL be serialized via `trace_stop_and_serialize()`
- **AND** the resulting JSON string SHALL contain all trace events recorded during that test

### Requirement: Trace data sent via IPC

The Worker process SHALL include the serialized trace data in the JSON-RPC `test_result` message sent to the host process.

#### Scenario: test_result message includes trace_data

- **WHEN** a Worker sends a `test_result` JSON-RPC message
- **THEN** the message SHALL include a `trace_data` field containing the serialized trace JSON string (or `null` if trace was not active)

### Requirement: WorkerTestResult receives trace_data

The host process SHALL deserialize the `trace_data` field from the IPC message and populate `WorkerTestResult.trace_data`.

#### Scenario: trace_data populated in WorkerTestResult

- **WHEN** the host processes a `test_result` IPC message with a non-null `trace_data` field
- **THEN** the corresponding `WorkerTestResult` SHALL have its `trace_data` field set to that JSON string

### Requirement: TestResult receives trace_data

The system SHALL propagate `trace_data` from `WorkerTestResult` to `TestResult.trace_data` when constructing final test results.

#### Scenario: trace_data flows to TestResult

- **WHEN** a `WorkerTestResult` with populated `trace_data` is converted to a `TestResult`
- **THEN** the resulting `TestResult` SHALL have `trace_data` set to the same JSON string

### Requirement: Trace data available to reporters

When a `TestResult` contains `trace_data`, the reporter SHALL have access to it via `TestResult.trace_data` during `on_test_result()`.

#### Scenario: Reporter can read trace data

- **WHEN** a reporter's `on_test_result()` receives a `TestResult` with non-null `trace_data`
- **THEN** `result.trace_data` SHALL contain the full trace JSON
