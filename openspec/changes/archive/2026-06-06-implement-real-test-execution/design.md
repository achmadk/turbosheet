## Context

TurboSheet has a well-architected browser automation layer:

- `ChromiumEngine` → `ChromiumContextEngine` → `ChromiumPageEngine` using chromiumoxide CDP
- `PageEngine` trait with methods: `goto`, `click`, `fill`, `evaluate`, `screenshot`, etc.
- `JsPage` (napi wrapper) delegates to `PAGES` map which holds `Arc<dyn PageEngine>`
- `JsLocator` delegates to page methods

However:

1. **Assertions (matchers.rs)** hardcode mock values: `let is_visible = true;`
2. **Test Executor (executor.rs)** returns `name: "Simulated Test"` with `status: Passed`
3. **No JS runtime** to actually execute user test code

The infrastructure is ready - we just need to wire it up.

## Goals / Non-Goals

**Goals:**

- `expect(locator).toBeVisible()` calls `locator.is_visible()` → `page.is_visible(selector)` → CDP
- `expect(page).toHaveURL()` calls `page.url()` → CDP
- Test executor loads and runs actual `.tsheet.ts` files
- Real test results: actual duration, real errors, real screenshots on failure

**Non-Goals:**

- Implementing Firefox or WebKit engines (those are separate work)
- Full TypeScript type checking (just transpile for execution)
- Building a new JS runtime (use existing solutions)

## Decisions

### Decision 1: JS Runtime - isolated-vm over QuickJS

**Choice:** Use `isolated-vm` for executing user test code

**Rationale:**

- `isolated-vm` provides v8 isolate execution within Node.js - same engine as Chromium
- Simpler integration with existing napi-rs architecture
- Better memory safety than native QuickJS bindings
- Can share context with napi-rs threadpool

**Alternatives considered:**

- QuickJS: Lighter but would require FFI complexity
- Deno: Too heavy, different security model
- Worker threads: More complex lifecycle management

### Decision 2: Assertion Implementation Pattern

**Pattern:** `expect()` returns `JsExpect` with target, matchers call CDP via existing infrastructure

```
expect(page.locator('.error')).toBeVisible()
         │                │
         │                └── creates JsExpect with ExpectTarget::Locator(JsLocator)
         │
         └── returns JsExpect {
              target: ExpectTarget::Locator(JsLocator {
                selector: ".error",
                page_id: "abc123"
              })
            }
```

Matchers need to:

1. Extract `page_id` from locator
2. Get `Arc<PageEngine>` from `PAGES` map
3. Call actual CDP method via `PageEngine::is_visible()`
4. Apply exponential backoff via `AssertionEngine::poll()`

### Decision 3: Test File Execution Flow

```
discovery::discover_tests() → Vec<TestFile>
            │
            ▼
test_executor.execute(files)
            │
            ▼
For each file:
  1. isolated-vm creates v8 context
  2. Inject TurboSheet globals (test, expect, page, etc.)
  3. Evaluate file contents (transpiled TS → JS)
  4. Collect test suite structure (describe blocks, tests, hooks)
  5. Execute tests via tokio task on napi-rs threadpool
  6. Report results via channel
```

### Decision 4: Fixture Scoping

**Pattern:** Worker-level fixtures via lazy_static, test-level via Arc<Mutex>

```
Global fixtures:   lazy_static! { static ref FIXTURE: Type = ... }
Worker fixtures:   static ref per-worker: HashMap<worker_id, Fixture>
Test fixtures:     Arc<Mutex<HashMap<String, Value>>>
```

## Risks / Trade-offs

| Risk                         | Mitigation                                               |
| ---------------------------- | -------------------------------------------------------- |
| isolated-vm memory leaks     | Implement explicit context.drop() and memory monitoring  |
| TS transpilation complexity  | Use SWC for fast transpilation, skip type checking       |
| Async/sync bridge complexity | All CDP calls are async via tokio, expose as async to JS |
| Test isolation failures      | Each test gets fresh page/context, cleanup in afterEach  |

## Migration Plan

1. **Phase 1:** Wire assertions to real CDP (lowest risk, highest visibility)
   - Modify matchers.rs to call real methods
   - Test with existing Chromium integration

2. **Phase 2:** Add isolated-vm for test execution
   - Add dependency
   - Create basic test runner that evaluates JS
   - Verify "Simulated Test" → real test names

3. **Phase 3:** Full fixture/hook lifecycle
   - Implement worker-level fixtures
   - Implement test-level fixtures with cleanup
   - Implement beforeAll/afterAll hooks

## Open Questions

1. **Should we support ES modules import in test files?** Would require file loading and resolution logic
2. **How to handle test超时?** Currently no way to kill runaway test - isolated-vm has timeout support
3. **Page/context reuse across tests?** Currently each test gets fresh page - is pooling desired?
