## Context

TurboSheet is a Rust-native browser automation framework with a napi-rs FFI bridge to Node.js. The test runner (`TestExecutor` in `src/test_runner/executor.rs`) spawns JS worker subprocesses via JSON-RPC over stdin/stdout. The browser engine (Chromium via `chromiumoxide`) lives in the Rust process. Three subsystems — reporters, video recording, and trace recording — have complete data models and implementations but are not wired end-to-end from test execution through to output. Two other areas — keyboard key codes and assertion engine tests — have implementation that needs minor fixes and test coverage, respectively.

The architecture already has the correct abstractions:

- `PageEngine` trait methods for `start_recording()`, `stop_recording()`, `press()`, `press_sequentially()`
- `VideoRecorder` with CDP screencast integration
- `TraceRecorder` with a `GLOBAL_RECORDER` singleton
- 7 reporter implementations all reading from `AggregatedTestResult`
- `AssertionEngine` with polling/timeout logic

The work is integration, not new subsystem creation.

## Goals / Non-Goals

**Goals:**

- Wire `trace_data` through `WorkerTestResult` → `TestResult` → `TestCaseResult` → reporters (HTML trace viewer)
- Wire `GLOBAL_RECORDER.record_action()` calls into page action methods so traces capture real events
- Wire video recording start/stop around test execution so `--video-on-failure` produces actual video files
- Stream test results to dot/line/list reporters in real-time during execution
- Add key-code mapping table for special keys in Chromium `press()`/`press_sequentially()`
- Write passing tests for `AssertionEngine::poll()`, `poll_with_timeout()`, jitter, and backoff

**Non-Goals:**

- No changes to Firefox/WebKit keyboard implementations (they already work via WebDriver element value)
- No changes to the reporter data model (`AggregatedTestResult`, `TestCaseResult`, `TestSuiteResult`)
- No changes to the `VideoRecorder`/`VideoEncoder` internals
- No changes to the `TraceRecorder` data model
- No new reporter types
- No changes to the assertion engine logic itself

## Decisions

### 1. Trace data pipeline: Worker JSON-RPC → Rust serialization → Reporters

**Decision**: Add `trace_data: Option<String>` to `WorkerTestResult`. The JS worker will serialize trace events to a JSON string during test execution. After test completes, the worker includes this string in the JSON-RPC response. `TestExecutor` passes it through to `TestResult.trace_data`.
**Rationale**: Least invasive approach. The worker already produces structured results with `WorkerTestResult`. Adding a field is minimal. The alternative — having Rust capture traces via `GLOBAL_RECORDER` — would require the CDP session to be accessible from the executor, which is complicated by the worker process boundary.
**Alternative considered**: Rust-side trace recording by injecting `GLOBAL_RECORDER` calls into `ChromiumPageEngine` methods. Rejected because the worker process boundary means the Rust engine doesn't directly know which test is running.

### 2. Video recording: Rust-side lifecycle hooks in TestExecutor

**Decision**: `ChromiumPageEngine::start_recording()` and `stop_recording()` already exist and are complete. The gap is that `TestExecutor::execute_tests()` needs to call them. Since the JS worker manages page lifecycle, we need the worker to send JSON-RPC notifications at test start/end that trigger the Rust side to call `start_recording()`/`stop_recording()`.
**Rationale**: The recording implementation (CDP `Page.startScreencast` → JPEG frames → FFmpeg pipe) is complete and tested. Only the integration trigger is missing.
**Alternative considered**: Having the executor spawn video recording as a separate task tied to worker start/end. Rejected — the current worker protocol approach is simpler.

### 3. Real-time reporter output: mpsc channel callback

**Decision**: Pass reporter types into `TestExecutor::execute_tests()` so dot/line/list reporters can print results as they arrive through the mpsc channel. The final-report reporters (HTML, JUnit, JSON) still run after all tests finish.
**Rationale**: Minimal change. The mpsc channel already streams `TestResult` from workers. We just add a callback or channel listener that feeds arriving results to real-time reporters.
**Alternative considered**: Restructuring `run_tests()` to interleave reporter output with test execution. Rejected — the current structure is clean; we just need to hook into the stream.

### 4. Key-code mapping: Static lookup table in ChromiumPageEngine

**Decision**: Add a `lazy_static` HashMap mapping special key names (Enter, Escape, Tab, Arrow keys, etc.) to their Windows virtual key codes. Use `windows_virtual_key_code` from the table when the key matches; fall back to current `chars().next() as i64` for regular characters.
**Rationale**: CDP's `Input.dispatchKeyEvent` requires correct `windowsVirtualKeyCode` for proper handling by the renderer. The current code only works for printable ASCII characters.
**Alternative considered**: Omitting `windows_virtual_key_code` entirely. CDP docs mark it optional but Chromium behavior varies — some keys need it.

### 5. Assertion engine tests: Rewrite from scratch

**Decision**: Remove the empty test module comment and write focused tests for each method of `AssertionEngine` — `poll()` with success, `poll_with_timeout()` with failure, `compute_jitter()` determinism, backoff doubling, and edge cases (zero timeout, immediate success).
**Rationale**: The original tests were removed due to compilation errors with no record of what they tested. Starting fresh is cleaner than trying to reconstruct.
**Alternative considered**: Digging through git history to find the original tests. Rejected — the errors were apparently structural enough to warrant removal.

## Risks / Trade-offs

- **[Trace data size]** Large traces could bloat JSON-RPC messages. → Cap at 100K events in `TraceRecorder` (already implemented via `max_events`)
- **[Video files on CI]** Video recording requires ffmpeg in PATH. → The encoder already returns a clear error if ffmpeg is not found
- **[Worker protocol changes]** Adding `trace_data` to `WorkerTestResult` changes the JSON contract. → The field is `Option<String>` and `#[serde(default)]` — fully backward compatible
- **[Key code gaps]** The key-code table might miss uncommon keys. → Initial implementation covers the 20+ most common keys; missing keys fall back to the current behavior
- **[Test flakiness]** Assertion engine tests involve timing. → Use `tokio::time::pause()` for deterministic timeout tests

## Migration Plan

Apply artifacts in order independently (no cross-cutting deployment concerns):

1. Assertion engine tests (standalone, lowest risk)
2. Key-code mapping (standalone, isolated to chromium.rs)
3. Trace data pipeline (requires: worker.rs → executor.rs changes)
4. Reporter streaming (requires: executor.rs changes)
5. Video recording trigger (requires: worker protocol + executor.rs)

Each can be verified independently by running `cargo test` and `vp check`.

## Open Questions

- Does the JS worker entry (`src/runtime/worker-entry.ts`) have access to the page engine's `startRecording()`/`stopRecording()` methods? This determines whether the worker can trigger video recording.
- What was the specific compilation error that caused the assertion engine tests to be disabled? (May inform the rewrite approach.)
- Should trace recording be opt-in (via config flag) or always-on? Current `GLOBAL_RECORDER` is always-on.
