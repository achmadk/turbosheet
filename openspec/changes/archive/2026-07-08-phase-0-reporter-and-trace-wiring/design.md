## Context

The project has 7 reporters (Dot, Line, List, Json, Junit, Html, Github) each implementing their own public methods with different signatures. They are currently wired in `run_tests()` via manual match arms that construct each reporter and pass its methods as closures to `TestExecutor::execute()`. This means:

- No way to pass a reporter to the executor without knowing all 7 variants at the call site
- No trait object dispatch — reporters can't be swapped dynamically
- Custom/third-party reporters require code changes inside `run_tests()`

Trace recording works via `GLOBAL_RECORDER` which captures page actions (click, type, navigate, press) inside every `PageEngine` method. The NAPI layer exposes `trace_start_recording`, `trace_clear`, `trace_stop_and_serialize`, `trace_events_to_json`. However, trace data is never connected to the test lifecycle — `WorkerTestResult.trace_data` and `TestResult.trace_data` are `None` in practice.

## Goals / Non-Goals

**Goals:**

- Define a `Reporter` trait with `on_test_result()` and `on_complete()` methods
- Implement the trait for all 7 existing reporters (backward-compatible)
- Refactor `TestExecutor::execute()` to accept `&dyn Reporter` instead of a closure
- Refactor `run_tests()` to construct reporters via the trait
- Wire trace lifecycle: clear recorder at test start, serialize at test end, populate `trace_data` in `WorkerTestResult`
- Handle the NAPI-to-worker IPC path so trace data reaches `TestResult`
- Update `PROGRESS.md` to reflect current state

**Non-Goals:**

- Reporter output formatting changes (existing output stays identical)
- New reporter types (custom reporters from external config is future work)
- Trace viewer integration (existing viewer works, just needs data)
- Firefox/WebKit video recording (not in scope)
- Refactoring the assertion engine or screenshot handling

## Decisions

### Decision 1: Reporter trait design — two-method trait vs event enum

**Chosen**: Two-method trait (`on_test_result`, `on_complete`).

**Alternatives considered:**

- **Event enum** (`ReporterEvent::TestResult(..) | ReporterEvent::Complete(..)`): More extensible but adds an enum variant layer with no current benefit. The two-method trait is simpler and directly callable.
- **Type-erased reporter (`Box<dyn Fn(ReporterEvent)>`)**: Loses the structured interface and composability.

**Rationale**: Two named methods provide clear semantics, are easy to implement, and keep the trait simple. A single `on_test_result()` receives each result as tests finish; `on_complete()` is called once with the aggregate summary. This matches how all 7 reporters already work internally — they all have a "process result" step and a "finalize" step.

### Decision 2: Trait location

**Chosen**: Define the trait in `src/reporters/mod.rs` alongside `ReporterConfig`, `ReporterType`, and the aggregate result types.

**Rationale**: All reporter-related types already live here. The trait is the natural abstraction boundary. Implementations stay in their respective files.

### Decision 3: Duration-based live progress

**Chosen**: Add a secondary trait `ReporterWithProgress` for reporters that support live progress (Dot, Line, List). It has one method: `on_progress(elapsed: Duration, completed: usize, total: usize)`.

**Rationale**: Json, Junit, Html, and Github reporters are output-at-end and don't need live progress. Separating concerns avoids forcing every reporter to implement unused methods. The executor checks `if let Some(progress) = reporter.downcast_ref::<ReporterWithProgress>()` to call progress updates.

### Decision 4: Trace lifecycle — approach

**Chosen**: Simple approach — clear the `GLOBAL_RECORDER` at the start of each test's execution in worker, serialize at the end, and attach via IPC.

**Alternatives considered:**

- **Per-page TraceRecorder instance**: Cleaner architecture but requires injecting a recorder into each PageEngine at creation time rather than using a global. Too invasive for Phase 0.
- **NAPI-level hooks**: Could call `trace_clear()` / `trace_stop_and_serialize()` from the JS runner, but that duplicates logic across all users of the library.

**Rationale**: The `GLOBAL_RECORDER` already works and data races are avoided because each test runs in its own Worker under the hood. Clearing at test start and serializing at test end is minimal, correct, and unblocks the trace data pipeline. A per-PageEngine recorder can be refactored later.

### Decision 5: IPC path for trace data

**Chosen**: The Worker process serializes the recorder after each test's page operations and sends the JSON string as a new field in the existing JSON-RPC `test_result` message.

**Rationale**: The IPC channel between Worker and host already sends test results as JSON-RPC. Adding a `trace_data` string field to the existing message is the lowest-risk path. The host deserializes it into `WorkerTestResult.trace_data` which is then mapped to `TestResult.trace_data`.

## Risks / Trade-offs

- **GLOBAL_RECORDER race on parallel tests**: Currently mitigated because each test runs in its own OS process (Worker). If that changes, the per-PageEngine refactor becomes urgent.
- **Reporter trait change is backward-compatible**: All existing public methods remain; the trait impl delegates to them. External code using direct reporter structs continues to work.
- **Trace data memory for long test suites**: Each test's trace is serialized and sent after the test completes, so memory is bounded per-test. No unbounded accumulation.
- **ReporterWithProgress uses downcast**: This is an anti-pattern but acceptable because the executor is the only place that calls it, and the alternative (pulling `on_progress` into the base trait) is worse — it forces every reporter to handle a no-op method.
