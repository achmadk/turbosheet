## 1. IPC Protocol — New Data Types

- [ ] 1.1 Add `CollectedTest` struct to `ipc.rs` with fields: name, modifier (string), suite_path (Vec<String>), fn_body (String)
- [ ] 1.2 Add `CollectedHook` struct to `ipc.rs` with fields: hook_type (string), suite_path (Vec<String>), fn_body (String)
- [ ] 1.3 Add `ExtractRequest` and `ExtractResponse` structs to `ipc.rs` with tests, hooks, suites fields
- [ ] 1.4 Add `ExecutionPlan` struct to `ipc.rs` with fields: test_fn_body, before_all/after_all/before_each/after_each hook bodies, timeout_ms, is_fail, is_fixme, is_slow, run_before_all (bool)
- [ ] 1.5 Add `PlanResponse` struct (wraps single `TestResultMsg`) to `ipc.rs`
- [ ] 1.6 Add serialization tests for all new IPC types

## 2. Worker — Collector Wrapper Enhancement

- [ ] 2.1 Fix `test.only(name, fn)` in collector wrapper to push `{ name, fn, modifier: "only" }` instead of being a no-op
- [ ] 2.2 Fix `test.fixme(name, fn)` to push `{ name, fn, modifier: "fixme" }`
- [ ] 2.3 Fix `test.fail(name, fn)` to push `{ name, fn, modifier: "fail" }`
- [ ] 2.4 Fix `test.slow(name, fn)` to push `{ name, fn, modifier: "slow" }`
- [ ] 2.5 Implement `test.describe(name, fn)` with proper nesting: track current suite path via a stack, push suite name on enter, pop on exit
- [ ] 2.6 Implement `describe.serial`/`describe.parallel`/`describe.skip`/`describe.only` with suite_type field in collected suite data
- [ ] 2.7 Ensure each collected test includes its full `suite_path` array from the describe stack
- [ ] 2.8 Ensure each collected hook includes its `suite_path` from the describe stack at registration point
- [ ] 2.9 Remove the old flat `__collected_tests.push` from the root-level `test()` definition — now handled via modifier-aware variant

## 3. Worker — Split Extraction from Execution

- [ ] 3.1 Create `handleExtractTests(request)` in worker-entry.ts that reads file, transpiles, runs collector wrapper (no browser), returns `ExtractResponse`
- [ ] 3.2 Create `handleRunPlan(request)` in worker-entry.ts that accepts `ExecutionPlan`, launches browser page, executes hooks + test in isolated-vm, returns `PlanResponse`
- [ ] 3.3 Ensure `extractTests` method does NOT launch a browser, create context, or start video recording
- [ ] 3.4 Ensure `runPlan` method receives all hook bodies as strings and executes them in the correct order: beforeAll (if flagged) → beforeEach → test → afterEach → afterAll (if flagged)
- [ ] 3.5 Add beforeAll execution guard: skip beforeAll if `run_before_all: false` in the plan
- [ ] 3.6 Add beforeAll failure handling: skip test with "beforeAll hook failed" message if beforeAll throws
- [ ] 3.7 Add afterEach execution: run afterEach hooks even if test fails; log errors but don't change test status
- [ ] 3.8 Add afterAll execution: run afterAll after all tests in suite complete; log errors but don't change test status
- [ ] 3.9 Handle `test.fail` modifier in runPlan: if test passes (no error), return "failed" status with "Expected to fail, but passed" message. If test fails, return "passed"
- [ ] 3.10 Handle `test.fixme` modifier in runPlan: if test fails, return "fixme" status instead of "failed"

## 4. WorkerProcess — New Rust Methods

- [ ] 4.1 Add `extract_tests(&mut self, file_path: &str, timeout_ms: u32) -> Result<ExtractResponse>` method to `WorkerProcess` that sends `extractTests` JSON-RPC and parses response
- [ ] 4.2 Add `run_plan(&mut self, plan: ExecutionPlan, video_on_failure: bool, video_dir: Option<&str>) -> Result<TestResultMsg>` method that sends `runPlan` and parses response
- [ ] 4.3 Keep existing `execute_test()` method as a backwards-compat wrapper (or remove if nothing depends on it)
- [ ] 4.4 Update error handling: distinguish extraction errors from execution errors with clear messages

## 5. Rust PlanBuilder — Test Plan Construction

- [ ] 5.1 Create `PlanBuilder` struct in executor.rs (or new plan_builder.rs module)
- [ ] 5.2 Implement `PlanBuilder::add_extracted_file(response: ExtractResponse)` to accumulate definitions per file
- [ ] 5.3 Implement `PlanBuilder::build() -> Vec<ExecutionPlan>` that produces flattened per-test plans
- [ ] 5.4 Implement modifier cascade: scan all extracted tests for `Only` modifier; if any found, exclude non-only tests
- [ ] 5.5 Implement suite-type cascade: if `describe.only` exists, exclude tests outside only suites; if `describe.skip`, exclude all children
- [ ] 5.6 Implement hook inheritance: for each test, walk its `suite_path` ancestry and collect `beforeEach`/`afterEach` hooks in order (parent → child for beforeEach, child → parent for afterEach)
- [ ] 5.7 Implement beforeAll/afterAll scheduling: determine if this test is the first in its suite (needs beforeAll) or last (needs afterAll)
- [ ] 5.8 Implement `test.slow` timeout tripling in plan construction
- [ ] 5.9 Handle empty test files: produce no execution plans, don't error

## 6. Executor — Two-Phase Orchestration

- [ ] 6.1 Modify `TestExecutor::execute()` to run extraction phase first: send `extract_tests()` for all files in parallel across worker pool
- [ ] 6.2 After all extractions complete, pass results to `PlanBuilder` to produce `Vec<ExecutionPlan>`
- [ ] 6.3 Implement execution dispatch loop: iterate `ExecutionPlan` entries, send `run_plan()` via worker pool, collect results
- [ ] 6.4 Preserve mpsc channel pattern: each `PlanResponse` → `TestResult` → `reporter.on_test_result()` as before
- [ ] 6.5 Implement test-level retry: if test fails and retries remain, re-insert its plan into the dispatch queue (with `run_before_all: false`)
- [ ] 6.6 Implement suite-level serial scheduling: group plans by serial suite path, execute those groups on single workers sequentially
- [ ] 6.7 Implement suite-level parallel scheduling (default): any plan not in a serial group can execute on any available worker
- [ ] 6.8 Update sharding: distribute by plan count (not file count) across shards

## 7. Hook Lifecycle — BeforeAll/AfterAll Guards

- [ ] 7.1 In `TestExecutor`, maintain `HashSet<String>` tracking which suite paths have had beforeAll executed
- [ ] 7.2 Before dispatching a plan with `run_before_all: true`, check the set; if already run, set flag to false
- [ ] 7.3 After executing a plan with `run_before_all: true`, add suite path to the set
- [ ] 7.4 After all plans for a suite complete (or on suite failure), dispatch afterAll if suite has one
- [ ] 7.5 Handle retry: do not re-run beforeAll for retried tests (suite is already marked)

## 8. Verification & Cleanup

- [ ] 8.1 Run `cargo check` and fix any compilation errors across changed Rust files
- [ ] 8.2 Run `cargo test` and verify all existing tests pass (registry tests, worker tests, IPC tests)
- [ ] 8.3 Verify worker-entry.ts compiles with TypeScript (`npx tsc --noEmit src/runtime/worker-entry.ts`)
- [ ] 8.4 Create a minimal end-to-end manual test: write a .tsheet.ts file with test.only, test.fixme, test.fail, test.describe nesting, and hooks; run via `run_tests()`, verify correct results
- [ ] 8.5 Verify reporter output unchanged: `TestResult` struct fields remain compatible
