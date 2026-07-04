## Why

TurboSheet has 5 built-but-unwired subsystems costing us a 20-30% feature regression vs Playwright parity: reporters render empty `trace_data`, video recording isn't invoked from the test runner, trace events are recorded nowhere, press() for special keys uses the wrong virtual key codes, and the assertion engine tests are a comment placeholder. These represent <2 weeks of integration work for an outsized confidence gain in our core DX value prop.

## What Changes

### 1. Wire trace data through the test pipeline

- Add `trace_data: Option<String>` to `WorkerTestResult` (the JSON-RPC response from the JS worker)
- Populate `TestResult.trace_data` in `TestExecutor::execute_tests()` from worker results
- Wire `GLOBAL_RECORDER` record calls into page action methods (click, fill, press, etc.)
- Trigger `trace_stop_and_serialize()` at end of each test and attach to result

### 2. Wire video recording end-to-end

- Ensure `TestExecutor` calls `ChromiumPageEngine::start_recording()` before test execution and `stop_recording()` after
- The Rust-side implementation is complete (`VideoRecorder`, `VideoEncoder`, CDP screencast integration) — what's missing is the test runner calling into it

### 3. Wire reporters for real-time output

- Pipe `TestResult` through reporters as they arrive from the mpsc channel (currently dot/line/list print everything at the end)
- The final-report reporters (HTML, JUnit, JSON) already work correctly

### 4. Fix Chromium key-code mapping for special keys

- `press()` and `press_sequentially()` in `ChromiumPageEngine` use `chars().next() as i64` for `windows_virtual_key_code`
- Need a lookup table for non-character keys: Enter (0x0D), Tab (0x09), Escape (0x1B), Backspace (0x08), Arrow keys, etc.
- Firefox/WebKit WebDriver paths are functional — no changes needed

### 5. Restore assertion engine tests

- `src/assertions/engine.rs:78` has `// Tests disabled to fix compilation errors` as the entire test module
- Restore or rewrite tests for `AssertionEngine::poll()`, `poll_with_timeout()`, `compute_jitter()`, backoff timing, and timeout behavior

## Capabilities

### New Capabilities

- `trace-recording`: Record and surface trace events (actions, network, console, snapshots) through the test pipeline into HTML reporter and trace viewer
- `video-recording`: End-to-end video capture via CDP screencast, wired from test runner through JS worker into `ChromiumPageEngine`

### Modified Capabilities

- (None — existing specs (cdp-event-dispatcher, injected-script-engine, network-interception) have no requirement changes)

## Impact

- **`src/test_runner/worker.rs`**: Add `trace_data` to `WorkerTestResult` struct
- **`src/test_runner/executor.rs`**: Populate `trace_data` from worker results; wire video start/stop; stream results to reporters
- **`src/engine/chromium.rs`**: Add key-code mapping table for `press()`/`press_sequentially()`
- **`src/assertions/engine.rs`**: Write tests for `poll()`, `poll_with_timeout()`, jitter, backoff
- **`src/reporters/mod.rs`**: No changes (trace pipeline feeds existing fields)
- **JS worker entry (`src/runtime/worker-entry.ts`)**: May need to forward trace data back from page actions
