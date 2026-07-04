## ADDED Requirements

### Requirement: TestRunner attaches trace data to test results

When a test completes, the system SHALL attach serialized trace events to the `TestResult.trace_data` field so that reporters (including the HTML trace viewer) can display them.

#### Scenario: Worker returns trace_data in JSON-RPC response

- **WHEN** a test file finishes execution in the JS worker
- **THEN** the worker SHALL include a `trace_data` field (JSON-serialized trace events) in the `WorkerTestResult`
- **AND** `TestExecutor` SHALL pass this value through to `TestResult.trace_data`

#### Scenario: Trace data appears in HTML report viewer

- **WHEN** a test has `trace_data` populated
- **THEN** the HTML reporter SHALL render the trace viewer component with the test's events
- **AND** clicking "View Trace" SHALL open the trace viewer timeline

#### Scenario: Absent trace_data does not break report

- **WHEN** a test has no trace data (`trace_data` is `None`)
- **THEN** the HTML report SHALL render normally without trace viewer for that test
- **AND** no error SHALL be raised

### Requirement: Page actions record trace events

Page action methods (click, fill, press, goto, etc.) SHALL call `GLOBAL_RECORDER.record_action()` to capture trace events during test execution.

#### Scenario: Click action is recorded

- **WHEN** `page.click(selector)` is called
- **THEN** a `TraceEvent::Action` SHALL be recorded with `action_type = "click"` and the selector

#### Scenario: Fill action is recorded

- **WHEN** `page.fill(selector, value)` is called
- **THEN** a `TraceEvent::Action` SHALL be recorded with `action_type = "fill"`, the selector, and the value

#### Scenario: Network request is recorded

- **WHEN** a CDP `Network.requestWillBeSent` event fires
- **THEN** a `TraceEvent::Network` SHALL be recorded with the URL, method, and headers

#### Scenario: Console message is recorded

- **WHEN** a CDP `Runtime.consoleAPICalled` event fires
- **THEN** a `TraceEvent::Console` SHALL be recorded with the level and message

### Requirement: Trace events are cleared between tests

The system SHALL clear the `GLOBAL_RECORDER` events between test executions to prevent cross-test trace contamination.

#### Scenario: Clear before next test

- **WHEN** a new test begins execution
- **THEN** `GLOBAL_RECORDER.clear()` SHALL be called
- **AND** the previous test's events SHALL NOT appear in the new test's trace data

### Requirement: Trace serialization produces valid JSON

The `serialize_trace()` function SHALL produce a byte sequence that can be deserialized back into the original trace events.

#### Scenario: Round-trip serialize/deserialize

- **WHEN** a set of `TraceEvent`s is serialized via `serialize_trace()`
- **THEN** `deserialize_trace()` on the resulting bytes SHALL return events identical to the original
