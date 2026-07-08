## Why

The test runner's execution pipeline currently operates as a black box: workers receive a file path, transpile and collect tests internally, run them, and return results. The Rust side has a fully-functional `TestRegistry` with suite nesting, modifier tracking, and hook registration — but it's completely orphaned from actual execution. This means modifier cascade (`test.only`, `describe.skip`), suite-level parallel/serial execution, proper hook scoping, and pre-execution test planning are all impossible. Every orchestration decision is buried inside the worker's `worker-entry.ts` collector, which is a fragile inline script with critical gaps (`test.only`, `test.fixme`, `test.fail` are no-ops).

## What Changes

- **New IPC protocol**: Two-phase protocol separating test extraction from execution. Workers first `extractTests(filePath)` returning collected definitions, then `runPlan(plan)` executing a Rust-computed execution plan.
- **Worker-entry collector fix**: `test.only()`, `test.fixme()`, `test.fail()`, `test.slow()` are actual no-ops in the current collector wrapper — they get proper modifier registration.
- **Worker-entry `test.describe()` nesting**: Currently flattens all tests into one list. Changes to proper suite nesting with parent-child tracking for correct hook scoping.
- **Rust-side orchestrator**: New `PlanBuilder` in the executor that receives extracted definitions, applies modifier cascade, resolves hook inheritance, and produces per-test `ExecutionPlan` objects.
- **Hook lifecycle in Rust**: Suite-scoped `beforeAll`/`afterAll`/`beforeEach`/`afterEach` with correct inheritance (parent hooks apply to child suites). beforeAll failure cascading skips entire suite.
- **Test modifiers honored**: `test.fail` inverts pass/fail, `test.fixme` marks as fixme (doesn't fail CI), `test.slow` triples timeout, `test.only`/`describe.only` filters non-focused items.
- **Retry at test level, not file level**: Currently retries the entire file. After this change, retries happen per individual test within the plan.
- **Sharding with plan awareness**: Distribute by individual test count instead of file chunks.

## Capabilities

### New Capabilities

- `two-phase-ipc`: Two-phase worker protocol separating test collection from execution. Workers expose `extractTests` (collect definitions, no browser) and `runPlan` (execute specific tests with hooks).
- `main-process-orchestrator`: Rust-side test plan builder that receives extracted definitions, applies modifier cascade, resolves hook inheritance, and drives suite-level serial/parallel execution.
- `suite-scoped-hooks`: Proper hook lifecycle with nested suite scoping. `beforeAll`/`afterAll` run per-suite; `beforeEach`/`afterEach` inherit from parent suites. beforeAll failure skips all descendant tests.
- `test-modifier-execution`: Runtime enforcement of `test.only`, `test.fixme`, `test.fail`, `test.slow`, `test.skip` modifiers. Only filters non-only tests. Fixme marks as known-issue. Fail inverts pass/fail. Slow triples timeout.
- `test-level-retry`: Retry individual tests within a plan rather than retrying entire files. Each test in the plan has its own retry counter.

### Modified Capabilities

- `test-registration`: (from phase-1-test-runner-core) The existing napi `test()`/`describe()`/hook functions in `mod.rs` already register into the `TestRegistry` — but this data was never wired to execution. With the orchestrator, registry data becomes the authoritative test plan source.

## Impact

- `src/test_runner/ipc.rs`: Add `CollectedTest`, `CollectedHook`, `ExecutionPlan`, `ExtractRequest`, `ExtractResponse` types
- `src/test_runner/worker.rs`: Add `extract_tests()` method, modify `execute_test()` to `run_plan()`
- `src/test_runner/executor.rs`: Add `PlanBuilder` with modifier cascade, hook inheritance, suite ordering. Modify `execute()` to use two-phase flow.
- `src/test_runner/mod.rs`: Wire `run_tests()` to build plan from extracted definitions instead of passing raw file paths
- `src/test_runner/registry.rs`: Already has complete data structures — no changes needed
- `src/test_runner/hooks.rs`: Keep existing `HookExecutor` but note it uses `PageEngine` trait (CDP). Workers execute hooks in isolated-vm, not via CDP — this file serves as reference for hook lifecycle logic but execution happens in the worker plan.
- `src/runtime/worker-entry.ts`: Split `executeTestFile` into `extractTests` (read+parse, no browser) and `runPlan` (execute with browser). Fix collector for `test.only/fixme/fail/slow` and `test.describe` nesting.
- `src/runtime/js-runtime.ts`: No changes needed
- `src/runtime/test-runner.ts`: The `TestFileExecutor` class has working suite nesting and hook execution — this serves as reference for plan execution design
