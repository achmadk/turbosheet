## Why

The Phase 0 "Quick Wins" launch plan identified 5 items. Items (2), (4), and (5) — video recording, keyboard press/pressSequentially, and assertion engine tests — are already fully implemented. The two remaining items form the foundation for reliable test output: reporters lack a unified trait abstraction making them inaccessible to external consumers, and trace data is collected but never attached to test results. Completing these closes Phase 0 and unblocks reporter assertions, custom reporting, and trace-based debugging in user workflows.

## What Changes

1. **Extract `Reporter` trait** — Define a `Reporter` trait with `on_test_result()` and `on_complete()` methods, implement it for all 7 reporters (Dot, Line, List, Json, Junit, Html, Github), refactor `TestExecutor::execute()` to accept a `Reporter` impl, and restructure `run_tests()` to build reporters via the trait instead of raw match arms.
2. **Wire per-test trace lifecycle** — Connect trace recording to the test lifecycle so captured actions appear in `WorkerTestResult.trace_data` and propagate through IPC to `TestResult.trace_data`, making traces consumable by reporters and external tools.
3. **Update PROGRESS.md** — Reflect current true state: 2 of 5 Phase 0 items remain.

## Capabilities

### New Capabilities

- `reporter-trait`: Unifies 7 reporters under a common `Reporter` trait with `on_test_result()` and `on_complete()` methods, plus a duration-based live progress system.
- `trace-lifecycle`: Connects trace recording into the per-test lifecycle so trace data flows from engine actions through NAPI and IPC into final test results.

### Modified Capabilities

<!-- No existing capabilities have spec-level behavior changes -->

## Impact

- **`src/reporters/`** — New `Reporter` trait definition in `mod.rs`; all 7 reporters implement the trait; no existing `pub fn` signatures removed
- **`src/test_runner/executor.rs`** — `TestExecutor::execute()` signature changes to accept `&dyn Reporter` instead of a raw closure; call sites updated
- **`src/test_runner/mod.rs`** — `run_tests()` reporter construction refactored to use the trait
- **`src/trace/recorder.rs`** — Per-test lifecycle hooks added (clear/serialize at test boundaries)
- **`src/test_runner/worker.rs`** — IPC message handler populates `WorkerTestResult.trace_data` from recorder state
- **`src/lib.rs`** — Possibly minor NAPI wiring if new lifecycle API is needed
- **`PROGRESS.md`** — Updated to reflect actual completion state
