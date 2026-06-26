## 1. Test Registry — Core Data Structures

- [ ] 1.1 Create `RegisteredTest` struct with fields: name, fn_ptr, modifiers (only, skip, fixme, fail, slow), timeout multiplier, parent suite ID
- [ ] 1.2 Create `SuiteType` enum: Default, Serial, Parallel, Skip, Only
- [ ] 1.3 Create `RegisteredSuite` struct with fields: name, suite_type, children suites, child tests, parent ID
- [ ] 1.4 Create `TestRegistry` struct with: suites map, test map, hooks map, builder pattern for add_test / add_suite / add_hook
- [ ] 1.5 Implement `TestRegistry::new()`, `add_test()`, `add_suite()`, `add_hook()`, `get_tests()`, `get_suites()`, `clear()`
- [ ] 1.6 Add `TestRegistry` to `run_tests()` flow: create at start, pass to executor, clear at end

## 2. Test Registration — napi Stub Implementation

- [ ] 2.1 Implement `test(name, fn_callback)` to register a basic test case in the registry
- [ ] 2.2 Implement `test_only(name, fn_callback)` to register a focused test
- [ ] 2.3 Implement `test_skip(name, fn_callback)` to register a skipped test
- [ ] 2.4 Implement `test_fixme(name, fn_callback)` to register a fixme test
- [ ] 2.5 Implement `test_fail(name, fn_callback)` to register an expected-failure test
- [ ] 2.6 Implement `test_slow(name, fn_callback)` to register a slow test with timeout multiplier
- [ ] 2.7 Implement `test_describe(name, fn_callback)` to create a default suite and execute callback for children
- [ ] 2.8 Implement `test_describe_serial(name, fn_callback)` to create a serial-execution suite
- [ ] 2.9 Implement `test_describe_parallel(name, fn_callback)` to create a parallel-execution suite
- [ ] 2.10 Implement `test_describe_skip(name, fn_callback)` to create a skipped suite
- [ ] 2.11 Implement `test_describe_only(name, fn_callback)` to create a focused suite
- [ ] 2.12 Implement `test_extend(fixtures)` to create a new test function with extended fixture bindings

## 3. Hook Registration — napi Stub Implementation

- [ ] 3.1 Implement `test_before_all(fn_callback)` to register a beforeAll hook in the registry
- [ ] 3.2 Implement `test_after_all(fn_callback)` to register an afterAll hook
- [ ] 3.3 Implement `test_before_each(fn_callback)` to register a beforeEach hook
- [ ] 3.4 Implement `test_after_each(fn_callback)` to register an afterEach hook
- [ ] 3.5 Ensure hooks are scoped to their enclosing suite (tracked via current suite context)

## 4. Executor Wiring — Connect Registry to Execution Pipeline

- [ ] 4.1 Modify `TestExecutor::execute()` to accept or access the `TestRegistry` instead of relying solely on file-level discovery
- [ ] 4.2 Implement suite-level execution loop: iterate suites, respect skip/only modifiers
- [ ] 4.3 Wire `HookExecutor::execute_before_all()` and `execute_after_all()` around each suite
- [ ] 4.4 Wire `HookExecutor::execute_before_each()` and `execute_after_each()` around each test
- [ ] 4.5 Handle `describe.only` / `test.only` modifier cascade: if any only exists, skip non-only items
- [ ] 4.6 Handle `describe.skip` / `test.skip` modifier: mark all children as skipped without execution
- [ ] 4.7 Handle `test.fixme` modifier: skip by default, mark with fixme annotation
- [ ] 4.8 Handle `test.fail` modifier: invert pass/fail for expected failures
- [ ] 4.9 Handle `test.slow` modifier: apply timeout multiplier to the test
- [ ] 4.10 Ensure `WorkerProcess::execute_test()` is called with the correct file path and timeout per test

## 5. List Reporter — Dedicated Module

- [ ] 5.1 Create or extend `src/reporters/list/mod.rs` with a `ListReporter` struct following existing reporter pattern
- [ ] 5.2 Implement suite hierarchy output: print suite names, indent child tests
- [ ] 5.3 Implement per-test output: status symbol + name + duration in ms
- [ ] 5.4 Implement error detail output: indented error message and stack trace for failures
- [ ] 5.5 Implement timeout display: special symbol and "(timeout: Xs)" annotation
- [ ] 5.6 Implement summary line: pass/fail/skip/timeout counts with colors
- [ ] 5.7 Wire `ListReporter` into the reporter dispatch in `run_tests()` (already partially done in mod.rs lines 61-76)
- [ ] 5.8 Verify `--reporter list` config option selects the list reporter

## 6. Verification & Cleanup

- [ ] 6.1 Run `cargo check` and fix any compilation errors
- [ ] 6.2 Run `cargo test` and verify existing tests still pass
- [ ] 6.3 Verify all 15 napi function signatures match their CJS exports in index.js
- [ ] 6.4 Create a minimal end-to-end test: write a .tsheet.ts file with test/describe/hooks, run via `run_tests()`, verify list output
