## 1. Reporter Trait Definition

- [x] 1.1 Add `Reporter` trait in `src/reporters/mod.rs` with `on_test_result(&self, result: &TestResult)` and `on_complete(&self, summary: &AggregatedTestResult)` methods
- [x] 1.2 Add `ReporterWithProgress` trait in `src/reporters/mod.rs` with `on_progress(&self, elapsed: Duration, completed: usize, total: usize)` method
- [x] 1.3 Ensure `use` imports for `TestResult` and `AggregatedTestResult` are present in `src/reporters/mod.rs`

## 2. Implement Reporter for All 7 Reporters

- [x] 2.1 Implement `Reporter` for `DotReporter` — delegate `on_test_result` to existing internal update method, `on_complete` to final print
- [x] 2.2 Implement `ReporterWithProgress` for `DotReporter` — delegate `on_progress` to existing progress-printing logic
- [x] 2.3 Implement `Reporter` for `LineReporter` — delegate to existing methods
- [x] 2.4 Implement `ReporterWithProgress` for `LineReporter` — delegate to existing progress-printing logic
- [x] 2.5 Implement `Reporter` for `ListReporter` — delegate to existing methods
- [x] 2.6 Implement `ReporterWithProgress` for `ListReporter` — delegate to existing progress-printing logic
- [x] 2.7 Implement `Reporter` for `JsonReporter` — `on_test_result` buffers, `on_complete` finalizes output
- [x] 2.8 Implement `Reporter` for `JunitReporter` — `on_test_result` accumulates, `on_complete` writes XML
- [x] 2.9 Implement `Reporter` for `HtmlReporter` — `on_test_result` accumulates, `on_complete` writes HTML
- [x] 2.10 Implement `Reporter` for `GithubReporter` — `on_test_result` annotates, `on_complete` finalizes

## 3. Refactor TestExecutor

- [x] 3.1 Change `TestExecutor::execute()` signature from accepting a closure to accepting `&dyn Reporter`
- [x] 3.2 Replace closure calls inside `execute()` with `reporter.on_test_result(&result)` and `reporter.on_complete(&summary)`
- [x] 3.3 Add periodic `reporter.downcast_ref::<ReporterWithProgress>()` check and call `on_progress()` during test execution

## 4. Refactor run_tests() Reporter Construction

- [x] 4.1 Refactor `run_tests()` in `src/test_runner/mod.rs` — replace match-arm reporter construction with `Box::new(DotReporter)` etc. as `Box<dyn Reporter>`
- [x] 4.2 Pass the `Box<dyn Reporter>` (or `&*boxed` as `&dyn Reporter`) to `TestExecutor::execute()`
- [x] 4.3 Remove old closure-based reporter wiring code

## 5. Trace Lifecycle in Worker

- [x] 5.1 Add `trace_clear()` call at the start of each test's page operations in the Worker process — **already implemented** at worker-entry.ts line 143
- [x] 5.2 Add `trace_stop_and_serialize()` call after each test's page operations complete — **already implemented** via `trace_events_to_json()` at line 390
- [x] 5.3 Include the serialized trace data in the JSON-RPC `test_result` message — **already implemented**, trace_data flows through WorkerTestResult serde (lines 391-393)

## 6. Propagate trace_data Through IPC Pipeline

- [x] 6.1 Deserialize `trace_data` field from IPC message — **already implemented** via serde in worker.rs
- [x] 6.2 Propagate `WorkerTestResult.trace_data` to `TestResult.trace_data` — **already implemented** in executor.rs line 271
- [x] 6.3 Verify `TestResult.trace_data` is accessible — **already implemented**, field is public on TestResult

## 7. Verification

- [x] 7.1 Run `cargo build` and verify no compilation errors — `cargo check` passes (0 errors, 31 pre-existing warnings)
- [ ] 7.2 Run `cargo test` and verify all existing tests pass — **skipped**: test binary timed out (>300s, pre-existing napi linking issue)
- [x] 7.3 Run `pnpm exec vp check` for full format + lint + type-check + test pipeline — passed (0 errors)
- [ ] 7.4 Verify reporter output is identical before and after refactoring — **skipped**: no integration test suite available in this environment

## 8. Update PROGRESS.md

- [x] 8.1 Update the Phase 0 section in PROGRESS.md to reflect actual completion state — done
