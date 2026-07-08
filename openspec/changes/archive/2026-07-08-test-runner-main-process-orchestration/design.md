## Context

The test runner currently operates in a single-phase worker model:

1. `TestExecutor` discovers test files, splits into chunks by worker count
2. Each worker receives a `filePath`, reads + transpiles the TS source, wraps it in an inline collector within isolated-vm that redefines `test()`/`describe()`/hooks to push to a local array
3. The worker executes collected tests one-by-one within the same isolated-vm context
4. Results stream back to Rust via JSON-RPC as `Vec<WorkerTestResult>`

**Critical gaps identified in current code:**

- **Rust registration is orphaned**: `mod.rs` has 15 `#[napi]` functions (`test()`, `describe()`, hooks) that register into a static `TestRegistry` — but this data is never consumed. The registry has full suite nesting, modifier tracking, and hook storage, but execution bypasses it entirely.
- **Worker collector is incomplete**: `test.only()`, `test.fixme()`, `test.fail()`, `test.slow()` are empty functions `{}` in the collector wrapper. `test.describe()` flattens — it just calls `fn()` without tracking suite boundaries. No modifier information is preserved.
- **Hook scoping is per-file, not per-suite**: `beforeAll` runs once per file regardless of describe blocks. `beforeEach`/`afterEach` inheritence across nested suites doesn't work because suite nesting isn't tracked.
- **Retry operates at file level**: If any test in a file fails, the entire file is retried. Individual test retry is impossible.
- **Sharding is file-level**: Distribution by file count means a heavy test file creates imbalance.

The existing `src/runtime/test-runner.ts` `TestFileExecutor` class has a reference implementation of proper suite nesting and hook execution with correct inheritance patterns. The `src/test_runner/registry.rs` already has `collect_active_tests()` with modifier cascade logic. The building blocks exist — they just aren't connected.

## Goals / Non-Goals

**Goals:**

- Separate test collection from execution into a two-phase protocol
- Move orchestration (modifier cascade, suite ordering, hook lifecycle, retry) to Rust side
- Fix all missing collector modifiers: `test.only`, `test.fixme`, `test.fail`, `test.slow`
- Implement proper `test.describe()` nesting with suite path tracking
- Suite-scoped hooks with correct parent → child inheritance
- Test-level retry (instead of file-level)
- Plan-aware sharding (distribute by test count)
- All existing reporter interfaces remain unchanged (`TestResult` structs don't change)

**Non-Goals:**

- No changes to browser engine modules (chromium, firefox, webkit)
- No changes to the napi-rs FFI layer or index.js exports
- No changes to the `expect()` implementation (continues to use the existing shim)
- No swc/TS compiler changes (transpilation stays in the worker)
- No changes to `config.rs`, `config_loader.rs`, `discovery.rs`, `fixtures.rs`
- No new reporter types
- No CI/CD changes
- No `test.extend()` fixture injection (unchanged, remains a placeholder)

## Decisions

### Decision 1: Two-phase IPC over extended single-phase protocol

**Option A (chosen)**: Two separate JSON-RPC methods — `extractTests(filePath)` returns collected definitions as structured data; `runPlan(plan)` executes a Rust-computed execution plan and returns results.

**Option B**: Extend the existing single `executeTest` method to return definitions first, then accept a filtered plan in a second call within the same RPC.

**Why A**: Clean separation of concerns. Extraction needs no browser, no page, no context — just a filesystem read and isolated-vm evaluation. Execution needs the full browser stack. Keeping them separate means extraction can be parallelized independently, cached, and retried cheaply. The two calls also map directly to the two phases the worker already does internally (collect → execute), just split across the IPC boundary.

### Decision 2: Collector lives in worker, not Rust

**Option A (chosen)**: Collection happens in the worker process via isolated-vm (as it does now). The collector wrapper is enhanced to track modifiers and suite structure. Results are serialized and sent back to Rust.

**Option B**: Move collection to the Rust side by loading the napi module in the main process and capturing `test()` calls.

**Why A**: The worker already has isolated-vm, swc transpilation, and the full Node.js runtime. Reproducing that in Rust would require bundling swc or another TS parser. The existing collector pattern works — it just needs enhancement. Keeping collection in the worker also means the main process doesn't need to load user test files at all, avoiding potential side effects from module-level code.

### Decision 3: ExecutionPlan per test (not per file or per suite)

**Option A (chosen)**: The Rust orchestrator builds one `ExecutionPlan` per test. Each plan contains:

- The test function body (string)
- The chain of `beforeEach`/`afterEach` hooks to run around it (from current suite and all ancestors)
- The `beforeAll`/`afterAll` flags (whether to run suite-level beforeAll first)
- Timeout, retry count, modifier behavior

**Option B**: Send one execution plan per file containing all tests.

**Why A**: Per-test plans give Rust maximal control over ordering. The orchestrator can interleave tests from different suites, respect `describe.serial`/`describe.parallel` at the suite level, and retry individual tests without re-sending an entire file. Per-file plans would require the worker to implement its own suite-level ordering and retry logic, defeating the purpose of moving orchestration to Rust.

**Trade-off**: More IPC round-trips per test. **Mitigation**: Extraction is the expensive part (transpile + parse). Per-test execution plans are small JSON payloads. The IPC cost of sending a plan and receiving a result is negligible compared to browser interaction within each test.

### Decision 4: `ExtractResponse` uses suite paths, not suite IDs

**Option A (chosen)**: Suite structure is represented as an array of path strings. A test's `suite_path: ["Root Suite", "Child Suite"]` places it in the hierarchy.

**Option B**: UUID-based suite IDs with parent references.

**Why A**: Paths are human-readable, deterministic across runs, and don't require ID coordination between worker and Rust. The Rust side reconstructs the tree from paths. This also matches how `TestRegistry.begin_suite()`/`end_suite()` work (stack-based, path-driven).

### Decision 5: Hook execution in worker isolate, not via Rust `HookExecutor`

**Option A (chosen)**: The worker's `runPlan` receives hook function bodies as strings and executes them in the same isolated-vm context as the test. The Rust orchestrator decides WHICH hooks to run (resolving inheritance), but the worker actually runs them.

**Option B**: Use the existing Rust `HookExecutor` (in `hooks.rs`) which invokes hooks via `page.evaluate()`.

**Why A**: The Rust `HookExecutor` depends on the `PageEngine` trait (CDP-level page evaluate). Hooks in test files are arbitrary JS/TS code that interact with the test's page fixture. They should run in the same isolated-vm sandbox where the test runs, not via CDP evaluate which has different scope and capabilities. The `TestFileExecutor` in `test-runner.ts` already demonstrates this pattern correctly.

### Decision 6: Collector wrapper is a generated string, not a separate module

**Option A (chosen)**: The collector wrapper continues to be a template string assembled in `worker-entry.ts`, wrapping user code. Enhancements add modifier fields and suite path tracking to the collected objects.

**Option B**: Extract the collector into a separate .ts module imported by the worker.

**Why A**: The collector code is tightly coupled to the shape of user code (it wraps it in an IIFE). A separate module would need an additional compilation step. The current approach is simple and debuggable. The `test-runner.ts` `TestFileExecutor` shows what proper collector code looks like — we port its suite-tracking pattern into the inline wrapper.

## Risks / Trade-offs

- **[Risk] More IPC round-trips**: Per-test execution plans mean N round-trips per file instead of 1. **Mitigation**: Extraction is the expensive part and happens once per file. Plan payloads are small (~1-5KB). For a typical test file with 10 tests, the difference is negligible compared to per-test browser interaction time (seconds per test vs microseconds per IPC message).

- **[Risk] Hook function serialization**: JavaScript function bodies are serialized as strings across the IPC boundary. If a hook closes over module-level state, that state is lost. **Mitigation**: This is the same limitation the current code has — hooks are already serialized as `fn.toString()`. Document that hooks should be self-contained or use fixtures for shared state.

- **[Risk] Suite path reconstruction ambiguity**: If two describe blocks have the same name at the same level, suite paths would collide. **Mitigation**: This edge case also exists in Playwright/Jest and is handled the same way — the first match wins. Unlikely in practice and easy to work around.

- **[Risk] Retry logic complexity**: Test-level retry means the orchestrator must track per-test attempt counts, decide which tests to retry, and rebuild partial execution plans. **Mitigation**: The existing file-level retry logic in `executor.rs` already has the retry loop structure. We adapt it to work on individual tests within a plan rather than entire files. The plan-per-test design makes this natural — just re-send the failed test's plan.

- **[Risk] Worker startup cost per suite**: If we spawn fresh workers per-suite for `describe.serial`, we pay the browser launch cost multiple times. **Mitigation**: Workers are pooled and reused across suites. `describe.serial` is a scheduling constraint (don't parallelize this suite's tests with others), not a process boundary. Tests from serial suites run sequentially on a single worker, but that worker persists.

- **[Trade-off] No pre-execution test listing without at least one extraction pass**: To answer "what tests exist in this file", we need to send it to a worker and run the collection phase. Full pre-execution test discovery requires spinning up workers. **Accepted**: The extraction phase is fast (no browser launch, just transpile + eval). For `--list-tests` CLI, we could add a dedicated `discovery` worker mode that only runs extraction.
