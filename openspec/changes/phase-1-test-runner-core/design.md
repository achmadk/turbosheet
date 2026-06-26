## Context

The test runner module (`src/test_runner/`) has 10 submodules with substantial scaffolding already in place:

- **executor.rs**: `TestExecutor` with worker-based parallel execution, global setup/teardown, retry logic, shard support — **functional**
- **worker.rs**: `WorkerProcess` with JSON-RPC IPC over stdin/stdout to a Node.js worker — **functional**
- **hooks.rs**: `HookRegistry` and `HookExecutor` with suite-scoped beforeAll/afterAll tracking — **functional**
- **fixtures.rs**: `FixtureRegistry` with dependency resolution, worker/test/global caches — **functional**
- **discovery.rs**: Glob-based test file discovery with grep filtering — **functional**
- **config.rs**: `TestConfig` with all Playwright-compatible options — **functional**
- **ipc.rs**: JSON-RPC 2.0 message types — **functional**
- **mod.rs**: 15 `#[napi]` exported functions — **all empty stubs `{}`**
- **extractor.rs**: `TestSuiteExtractor` — **returns hardcoded "dummy test" always**
- **config_loader.rs**: Loads JS config via tsx subprocess — **functional**

The JS layer (`index.js`) already exports `test`, `describe`, `beforeAll`, `afterAll`, `beforeEach`, `afterEach`, `run_tests`, `expect` mapped to native bindings. The eight reporter modules (Dot, Line, List, JSON, GitHub, HTML, JUnit, Summary) exist under `src/reporters/`.

**Core gap**: The Rust-side test registration stubs accept a name and callback but discard both. No test case is ever registered, so the executor/worker pipeline has nothing to execute.

## Goals / Non-Goals

**Goals:**

- Implement all 15 napi stub functions to register tests, suites, hooks, and fixture extensions in a Rust-side registry
- Wire the registered test/suite/hook data through the extractor → executor → worker pipeline
- Provide a working list reporter that shows individual test results with status, timing, and error details
- Support all test modifiers: only, skip, fixme, fail, slow
- Support all describe modifiers: serial, parallel, skip, only

**Non-Goals:**

- No new reporter types (existing Dot, Line, JSON, GitHub, HTML, JUnit, Summary reporters are unchanged)
- No changes to the worker IPC protocol or worker-entry.ts
- No changes to browser engine modules (chromium, firefox, webkit)
- No CI/CD integration or CLI enhancements
- No video recording, coverage, or accessibility features

## Decisions

### Decision 1: Per-Run TestRegistry singleton (not global static)

**Option A (chosen)**: A `TestRegistry` struct owned by `run_tests()`, passed through the pipeline via `Arc<Mutex<>>`.

**Option B**: A global `once_cell::sync::Lazy<Mutex<TestRegistry>>` static.
**Why A**: Per-run state avoids cross-contamination between test runs, simplifies cleanup, and matches the existing pattern where `TestExecutor` owns its config. The registry is created at the start of `run_tests()` and consumed by the executor.

### Decision 2: Suite nesting via explicit tree, not flat tagging

**Option A (chosen)**: `TestSuite` structs with `children: Vec<TestSuite>` and `tests: Vec<RegisteredTest>`.

**Option B**: Flat list with dotted-path names (e.g., "root > suite > test").
**Why A**: Tree structure enables proper hook scoping (beforeAll runs per-suite), parallel/serial execution at the suite level, and cleaner reporting with indentation. The existing `TestSuiteExtractor` already returns a tree.

### Decision 3: Hook execution happens on the Rust side, not in the worker

**Option A (chosen)**: The `HookExecutor` (already in `hooks.rs`) invokes hooks via `page.evaluate()` before/after each test/suite.

**Option B**: Pass hook source to the worker and let it execute.
**Why A**: The hook infrastructure is already built and tested. The worker should remain a "dumb executor" for individual test files. The executor orchestrates hook lifecycle around worker calls.

### Decision 4: Real extraction comes after stub, not in this phase

**Option A (chosen)**: Keep `extractor.rs` as a placeholder that reads the test file and returns the pre-registered test structure from the JS-side registry.

**Option B**: Implement full swc-based AST parsing now.
**Why A**: The JS API stubs (`test()`, `describe()` etc.) are called at module load time in the worker. The worker already captures results via JSON-RPC. The extractor's job is to discover what tests exist before running — this can start simple (scan for `test(` calls via regex) and be refined later. Full AST parsing is Phase 2 work. See risk #1.

### Decision 5: List reporter lives in existing `src/reporters/list/` module

**Option A (chosen)**: Extend the existing `list.rs` reporter with timing, error details, and indented suite hierarchy.

**Option B**: Inline the list output in `mod.rs` (currently partially there).
**Why A**: All other reporters follow the same module pattern (`src/reporters/<name>/`). Moving the inline list output into a proper reporter module improves consistency and makes it available via `--reporter list` CLI.

## Risks / Trade-offs

- **[Risk] Extractor stub means tests are discovered at worker runtime, not during Rust-side planning**: The current `run_tests()` flow discovers files, then delegates to workers. If test registration happens inside the worker, the Rust side won't know test names until results come back. → **Mitigation**: Accept this for Phase 1. The worker already returns `Vec<WorkerTestResult>` with names. The list reporter can still display results. Full pre-execution discovery becomes Phase 2.

- **[Risk] Hook execution depends on `page.evaluate()` which requires a browser page**: If hooks need a browser context and the test hasn't launched one yet, they'll fail. → **Mitigation**: Document that beforeAll/afterAll receive a `page` or `context` fixture. Users who don't need browser hooks can use empty functions.

- **[Risk] napi function callbacks are single-use by design**: The current `#[napi]` stubs take `Function` callbacks that can only be called once. This prevents the "register now, execute later" pattern. → **Mitigation**: Use the worker-side approach. The Rust `test()` stub captures the callback but the real work happens in the worker where the JS runtime holds references. For the Rust API, we persist the callback into a vector and execute during the run phase.

- **[Trade-off] No swc parsing means no type-aware extraction**: Without AST parsing, we can't distinguish `test(` calls from strings containing `test(`. → **Accepted**: Regex-based scanning is sufficient for the 95% case. swc integration can be added later.
