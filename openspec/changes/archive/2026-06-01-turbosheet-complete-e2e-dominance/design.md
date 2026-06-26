## Context

TurboSheet (Rashiq) is currently a Rust-powered browser automation library with a Node.js API via napi-rs. Its v1/v2 architecture provides: dual-core engine (Chromiumoxide + Rustenium), native FFI bridge, Tokio async runtime, and AI-native DOM extraction. However, it lacks the full testing framework, debugging tools, cross-browser support, and infrastructure that make Playwright/Cypress complete E2E solutions.

This design covers the architecture for transforming TurboSheet from a browser automation library into a comprehensive E2E testing platform with 17 new capabilities spanning test runner, debugging, cross-browser, CI/CD, and unique moats.

## Goals / Non-Goals

**Goals:**

- Define a modular architecture where all capabilities build on the existing Rust core and napi-rs bridge
- Design the test runner as a Rust-native execution engine with Node.js orchestration (not the reverse)
- Ensure every capability either matches or surpasses Playwright/Cypress/Puppeteer equivalents
- Preserve and leverage existing moats (swarm threading, zero-copy FFI, BiDi preprocessing, AI-native DOM)
- Provide clear extension points for community contributions

**Non-Goals:**

- Full implementation details or line-by-line code (delegated to specs and tasks)
- Kubernetes-native orchestration (swarm grid gets basic deployment first)
- Native mobile app support (iOS/Android apps via Appium — out of scope for browser E2E)

## Decisions

### D1: Layered Architecture — Core, Capabilities, Surface

```
┌─────────────────────────────────────────────────────────────┐
│                     SURFACE LAYER (Node.js)                  │
│  CLI         VS Code Ext    Reporter API    Plugin API       │
│  (test,       (test exp,     (HTML/JSON/     (custom          │
│   codegen,    trace viewer,   JUnit/GitHub)   reporters,      │
│   debug,      inline results)                  matchers)      │
│   migrate)                                                    │
├─────────────────────────────────────────────────────────────┤
│                  CAPABILITIES LAYER (Hybrid)                  │
│  ┌──────────┬──────────┬──────────┬──────────┬──────────┐    │
│  │Test Runner│ Network  │  Trace   │  Visual  │Component │    │
│  │Fixtures,  │Intercept │ Viewer   │Regression│ Testing  │    │
│  │Assertions,│ Mock,    │ Snapshots│ SIMD     │ Mount,   │    │
│  │Sharding   │ Throttle │ Recording│ Diff, AI │Interact  │    │
│  ├──────────┼──────────┼──────────┼──────────┼──────────┤    │
│  │ Codegen  │  Debug   │ Mobile   │Accessi-  │ Migration│    │
│  │ Recorder │  Runner  │ Emulation│ bility   │ Tooling  │    │
│  └──────────┴──────────┴──────────┴──────────┴──────────┘    │
├─────────────────────────────────────────────────────────────┤
│                      CORE LAYER (Rust)                       │
│  ┌──────────┬──────────┬──────────┬──────────┬──────────┐    │
│  │ CDP      │ BiDi     │ Selector │ Network  │ Trace    │    │
│  │ Engine   │ Engine   │ Engine   │ Proxy    │ Recorder │    │
│  │(Chromium)│(All brws)│ (Rust    │ (Tokio   │ (Zero-   │    │
│  │          │          │  polling)│  in-proc)│  copy)   │    │
│  ├──────────┼──────────┼──────────┼──────────┼──────────┤    │
│  │ Swarm    │ AI DOM   │ Fixture  │ WASM     │ Binary   │    │
│  │ Threading│ Snapshot │ Resolver │ Compiler │ Protocol │    │
│  └──────────┴──────────┴──────────┴──────────┴──────────┘    │
└─────────────────────────────────────────────────────────────┘
```

**Rationale**: This keeps the Rust core focused on performance-critical operations (browser protocol, network, selectors, threading) while allowing Node.js to handle developer-facing ergonomics (CLI, editor integration, plugin system). The hybrid capability layer means some capabilities are pure Rust (visual diff), some are pure JS (reporters), and some are both (test runner).

---

### D2: Test Runner — Rust Execution Engine, JS Fixture API

```
                     ┌────────────────────────────┐
                     │  User's .tsheet.ts file     │
                     │  import { test, expect }   │
                     │  from 'tsheet'             │
                     │                            │
                     │  test('works', async({pg}) │
                     │    => expect(pg)...)        │
                     └──────────┬─────────────────┘
                                │
                    napi-rs     │  callback
                                ▼
              ┌──────────────────────────────────┐
              │     Rust Test Executor            │
              │  ┌────────────────────────────┐  │
              │  │ Test Discovery (glob)       │  │
              │  │ → Parse .tsheet.ts files    │  │
              │  │   (via napi-rs require())   │  │
              │  ├────────────────────────────┤  │
              │  │ Test Scheduler              │  │
              │  │ → Worker pool (Tokio)       │  │
              │  │ → Shard assignment          │  │
              │  │ → Retry logic               │  │
              │  ├────────────────────────────┤  │
              │  │ Fixture Resolver            │  │
              │  │ → Resolve DI graph in Rust  │  │
              │  │ → Cache browser contexts    │  │
              │  │   in LRU (avoid recreation) │  │
              │  ├────────────────────────────┤  │
              │  │ Assertion Engine            │  │
              │  │ → Auto-retry in Rust thread │  │
              │  │ → Zero JS event loop impact │  │
              │  └────────────────────────────┘  │
              └──────────────────────────────────┘
```

**Rationale**: Placing the test executor in Rust is the key differentiator. Playwright Test runs in Node.js — its parallel workers are separate Node.js processes. TurboSheet's test executor uses Tokio tasks for workers, which are ~1000x lighter than processes. This enables thousands of parallel tests on a single machine.

**Fixture Resolution Flow:**

```typescript
// User declares fixtures
const test = tsheet.extend<{ page: Page }>({
  page: async ({ browser }, use) => {
    const ctx = await browser.newContext();
    const page = await ctx.newPage();
    await use(page);
    await ctx.close();
  },
});
```

1. Rust resolves fixture dependency graph (DAG)
2. For each test worker (Tokio task):
   - Rust calls into Node.js via napi-rs to invoke fixture factory
   - Node.js returns the fixture value across FFI
   - Rust caches the fixture in the worker's scope
3. On `use()` in JS, control returns to Rust
4. Rust runs the test body in the same Tokio task

---

### D3: Cross-Browser — Strategy over Implementation

```
     ┌────────────────────────────────────────────────────┐
     │              TurboSheet Browser Abstraction         │
     │                                                      │
     │  Rust Trait: BrowserEngine                           │
     │  ┌──────────────────────────────────────────────┐   │
     │  │ fn launch(config) -> BrowserProcess          │   │
     │  │ fn new_context(options) -> BrowserContext    │   │
     │  │ fn new_page(context) -> Page                 │   │
     │  │ fn navigate(page, url)                       │   │
     │  │ fn query_selector(page, selector) -> Element │   │
     │  │ fn take_screenshot(page) -> Pixels           │   │
     │  │ fn evaluate(page, script) -> Value           │   │
     │  │ ...                                          │   │
     │  └──────────────────────────────────────────────┘   │
     │                                                      │
     │  Implementations:                                    │
     │  ┌──────────┐ ┌──────────┐ ┌──────────┐            │
     │  │Chromium  │ │ Firefox  │ │ WebKit   │            │
     │  │oxide (CDP)│ │(geckodrv)│ │(webkit2gt)            │
     │  │          │ │ BiDi     │ │ BiDi     │            │
     │  └──────────┘ └──────────┘ └──────────┘            │
     │                                                      │
     │  BiDi Reuse: Firefox and WebKit share BiDi engine   │
     │  Chromium uses CDP (faster for Chromium-specific)   │
     │  Stealth mode always uses BiDi (any browser)       │
     └──────────────────────────────────────────────────────┘
```

**Rationale**:

- **Firefox**: `geckodriver` crate exists and supports WebDriver BiDi. Firefox has contributed significantly to BiDi spec — it's the most mature non-Chromium BiDi implementation. Priority: HIGH.
- **WebKit**: `webkit2gtk` on Linux + Safari's remote WebDriver on macOS. WebKit's BiDi support is newer but functional. For CI use, `webkit2gtk` is sufficient. Priority: MEDIUM.
- **Shared BiDi engine**: Both Firefox and WebKit use WebDriver BiDi, so the BiDi stream pre-processor built for Rustenium applies to all three — TurboSheet's unique advantage (BiDi filtering in Rust) benefits cross-browser automatically.

---

### D4: Network Interception — In-Process Tokio Proxy

```
                    ┌────────────────────────┐
                    │     Browser             │
                    │  HTTP ──────┐           │
                    │  WS  ──────┤           │
                    └────────────┤───────────┘
                                 │
                    ┌────────────┴──────────┐
                    │  TurboSheet Proxy      │
                    │  (Tokio::spawn)         │
                    │                         │
                    │  ┌───────────────────┐ │
                    │  │ Pattern Matcher    │ │
                    │  │ (regex in Rust)    │ │
                    │  │ → unmatched pass   │ │
                    │  │   through (zero-   │ │
                    │  │   copy buffer)     │ │
                    │  │ → matched invoke   │ │
                    │  │   JS callback via  │ │
                    │  │   napi-rs channel   │ │
                    │  └───────────────────┘ │
                    │                         │
                    │  ┌───────────────────┐ │
                    │  │ Mock Response Cache│ │
                    │  │ (pre-serialized    │ │
                    │  │  HTTP responses    │ │
                    │  │  stored in Rust    │ │
                    │  │  Arc<Vec<u8>>)     │ │
                    │  └───────────────────┘ │
                    │                         │
                    │  ┌───────────────────┐ │
                    │  │ Throttle Controller│ │
                    │  │ (Tokio::time)      │ │
                    │  └───────────────────┘ │
                    └────────────────────────┘
                                 │
                          Internet
```

**Rationale**: In-process proxy (vs Playwright's out-of-process) eliminates context switching. The proxy runs as a Tokio task sharing the same Rust runtime, so data never crosses process boundaries until the JS callback is explicitly needed.

---

### D5: TurboTrace — Zero-Copy Recording Architecture

```
  Browser Event ──► Rust Trace Recorder ──► Shared Memory Buffer ──► napi-rs ──► Node.js
       │                  │                              │
       │                  ▼                              ▼
       │           Compressed via               JS reads pre-compressed
       │           zstd in real-time             chunks for viewer
       │                                         (decompressed via
       │                                          Rust WASM in browser)
       │
       └──► DOM Snapshot via AI-native
            getAgentSnapshot() — 90% smaller
            than raw HTML, semantic format
```

**TurboTrace File Format:**

```
  ┌─────────────────────────────────────────┐
  │  Trace Container (.tsheet-trace)         │
  │  ├─ Metadata (JSON header)              │
  │  ├─ Action Log (compressed CBOR)        │
  │  │  Each entry: { timestamp, type,      │
  │  │   selector, value, duration, result } │
  │  ├─ DOM Snapshots (compressed chunks)    │
  │  │  → AI-native format (accessibility   │
  │  │    tree, not raw HTML)                │
  │  ├─ Network Events (compressed CBOR)    │
  │  │  → Request, response, timing, body   │
  │  ├─ Console Log (compressed)            │
  │  └─ Screenshots (WebP, key frames only) │
  └─────────────────────────────────────────┘
```

**Rationale**: Playwright traces are JSON-based and can be large. TurboSheet uses CBOR (binary JSON) + zstd compression + AI-native DOM format for 60-80% smaller trace files. The trace viewer itself runs as a Rust-compiled WASM bundle in the browser — no server needed for local trace viewing.

---

### D6: WASM Edge Runtime — Execution on Distributed Edge

```
  ┌─────────────────────────────────────────────────────┐
  │  WASM Compilation Target                             │
  │                                                       │
  │  Rust Core ──► wasm32-wasi ──► Edge Runtime          │
  │       │                                                  │
  │       │  What runs at edge:                              │
  │       │  • Test orchestrator (schedule, retry, report)  │
  │       │  • Assertion engine (auto-retry)                │
  │       │  • BiDi stream pre-processor (filter events)    │
  │       │  • AI DOM extraction (snapshot creation)        │
  │       │                                                  │
  │       │  What stays on server:                          │
  │       │  • Browser process (launched on grid node)      │
  │       │  • Heavy screenshot processing                  │
  │       │  • Binary protocol between edge ↔ grid node    │
  │       └──────────────────────────────────────────────┘
```

**Rationale**: WASM allows TurboSheet's orchestration and assertion logic to run at CDN edge locations (Cloudflare Workers, Deno, etc.) while browser processes run on dedicated grid nodes. This enables geo-distributed testing where latency measurement is accurate and test execution happens close to users.

---

### D7: Plugin System — Rust-First, JS-Fallback

```
  Plugin Types:
  ┌──────────────┬───────────────────┬──────────────────┐
  │ Plugin Type  │ Rust (Fast Path)  │ JS (Easy Path)    │
  ├──────────────┼───────────────────┼──────────────────┤
  │ Reporter     │ ─                 │ ✅ (dynamic req)  │
  │ Matcher      │ ✅ (assert perf)  │ ✅ (fallback)     │
  │ Fixture      │ ✅ (context pool) │ ✅ (simple)       │
  │ Hook         │ ✅ (lifecycle)    │ ✅ (common)       │
  │ Transformer  │ ✅ (perf crit)    │ ✅ (easy)         │
  │ Protocol     │ ✅ (custom proto) │ ─                │
  └──────────────┴───────────────────┴──────────────────┘

  Plugin Resolution:
  1. Check for Rust plugin (.so/.dylib in ~/.tsheet/plugins/)
  2. Fall back to JS plugin (node_modules/tsheet-plugin-*)
  3. Rust plugins share the same Tokio runtime (zero overhead)
  4. JS plugins run in Node.js via napi-rs bridge (slight overhead)
```

**Rationale**: Performance-critical plugins (custom matchers, protocol handlers) benefit from Rust. Developer-friendly plugins (custom reporters, simple hooks) benefit from JS ease. Supporting both maximizes ecosystem growth.

---

### D8: Swarm Grid Architecture

```
  ┌──────────────────────────────────────────────────────────┐
  │                 TurboSheet Swarm Grid                     │
  │                                                            │
  │  Controller Node (Rust binary)                            │
  │  ┌──────────────────────────────────────────────────────┐ │
  │  │  REST API (test submission, result polling)          │ │
  │  │  Job Queue (Tokio channel, prioritized)              │ │
  │  │  Worker Pool Manager (tracks live nodes)             │ │
  │  │  Result Aggregator (merge sharded results)           │ │
  │  └──────────────────────────────────────────────────────┘ │
  │                                                            │
  │  Worker Nodes (Rust binary, 1 per physical server)        │
  │  ┌──────────────────────────────────────────────────────┐ │
  │  │  Tokio Thread Pool: 1 thread per core               │ │
  │  │  ┌────────────┐ ┌────────────┐ ┌────────────┐      │ │
  │  │  │ Context 1  │ │ Context 2  │ │ Context N  │ ...  │ │
  │  │  │ (task)     │ │ (task)     │ │ (task)     │      │ │
  │  │  │ ~2MB RAM   │ │ ~2MB RAM   │ │ ~2MB RAM   │      │ │
  │  │  └────────────┘ └────────────┘ └────────────┘      │ │
  │  │                                                      │ │
  │  │  Compare: Playwright = 1 context = 1 process        │ │
  │  │  (~80MB RAM, ~10s spawn)                            │ │
  │  │  TurboSheet = 1 context = 1 Tokio task              │ │
  │  │  (~2MB RAM, ~5ms spawn)                             │ │
  └──────────────────────────────────────────────────────────┘
```

**Rationale**: This is TurboSheet's biggest moat. Playwright cannot do this — Node.js processes are OS-level and heavy. Tokio tasks are Rust-level and lightweight. A single 32-core server can run **5000+ parallel contexts** with TurboSheet vs ~150 with Playwright.

---

### D9: Visual Regression — Rust SIMD Pipeline

```
  Screenshot Capture ──► Rust Buffer ──► SIMD Compare ──► Diff Output
       │                     │                │                │
       │                     │           ┌────┴────┐          │
       │                     │           │ x86:    │          │
       │                     │           │ SSE/AVX │          │
       │                     │           │ ARM:    │          │
       │                     │           │ NEON    │          │
       │                     │           └─────────┘          │
       ▼                     ▼                                ▼
  Baseline stored      PixelPipeline:                   Output:
  as WebP in           1. Per-channel diff               WebP diff with
  .tsheet-snap/        2. Threshold mask                  heatmap overlay
  directory            3. Anti-aliasing pass              JSON diff report
                       4. Semantic region                  (pass/fail, % diff,
                        detection (AI-enhanced)            pixel count)
```

**Rationale**: Rust's `image` crate + explicit SIMD (via `core::simd` or `wide`) provides 3-8x faster pixel comparison than JS-based solutions. The AI-enhanced semantic diff (leveraging the existing AI-native DOM extraction to identify "meaningful" vs "noise" pixel changes) is a unique differentiator.

---

### D10: Reporter System Architecture

```
  Test Results ──► Rust Trace Buffer
                         │
                    napi-rs channel
                         │
                         ▼
              ┌─────────────────────┐
              │ Reporter Manager (JS) │
              │ (processes results    │
              │  as they arrive, not  │
              │  after completion)    │
              └────┬──────┬──────┬───┘
                   │      │      │
         ┌─────────┘      │      └──────────┐
         ▼                ▼                  ▼
   ┌──────────┐    ┌──────────┐    ┌──────────────┐
   │ HTML     │    │ JSON     │    │ GitHub       │
   │ Reporter │    │ Reporter │    │ Annotations  │
   │ (with    │    │ (CI      │    │ (PR comments)│
   │ embedded │    │  ingest) │    │              │
   │ viewer)  │    │          │    │              │
   └──────────┘    └──────────┘    └──────────────┘
```

**Rationale**: Reporters run in JS (they're I/O-bound, not CPU-bound). The streaming architecture (results arrive in JS as tests complete, not batched at end) enables live-updating HTML reports and real-time CI annotations.

## Risks / Trade-offs

| Risk                                                             | Severity | Mitigation                                                                                                                     |
| ---------------------------------------------------------------- | -------- | ------------------------------------------------------------------------------------------------------------------------------ |
| **R1: napi-rs cross-FFI overhead for fixture resolution**        | Medium   | Batch fixture resolution; use shared memory for large objects; benchmark real-world scenarios before optimizing                |
| **R2: Firefox/WebKit BiDi maturity gaps vs Chromium CDP**        | High     | Start with Firefox (most mature BiDi); mark WebKit as beta initially; fallback to CDP for Chromium where BiDi is slow          |
| **R3: WASM compilation limits (no browser process, no threads)** | High     | WASM runs orchestration only; browser processes remain on server grid; clearly document WASM capabilities vs full runtime      |
| **R4: Swarm Grid security model (multi-tenant)**                 | High     | Use OS-level container (Docker) per tenant as safety boundary; Tokio tasks share process but each runs in its own cgroup       |
| **R5: Trace file size for long-running tests**                   | Low      | Configurable snapshot frequency; streaming compression; auto-purge of redundant snapshots                                      |
| **R6: Plugin ecosystem chicken-and-egg problem**                 | Medium   | Ship with 10+ built-in plugins (Allure, Percy, Slack, etc.) to bootstrap; provide clear migration path from Playwright plugins |
| **R7: Dual Rust/JS plugin API complexity**                       | Low      | Start with JS-only plugin API (fastest ecosystem growth); add Rust plugin API in Phase 2 for power users                       |

## Open Questions

1. **Test runner file format**: Should we use `.tsheet.ts` (Playwright-style with `test` and `expect` imports) or a custom format? Recommendation: `.tsheet.ts` — ecosystem familiarity matters.
2. **WASM target for grid nodes**: `wasm32-wasi` or `wasm32-unknown-unknown`? WASI gives us file system access needed for trace output. **Tentative**: `wasm32-wasi-preview2` once stable.
3. **Browser binary bundling**: Download on first use (like Playwright), or bundle in Docker images? **Tentative**: Both — CLI downloads via `tsheet install`, Docker images come pre-bundled.
4. **VS Code extension language**: TypeScript (faster development) or Rust via WASM (performance)? **Tentative**: TypeScript for MVP, Rust WASM for performance-critical parts (trace viewer rendering).
5. **Swarm Grid pricing model**: Per-context-hour, per-test-run, or flat-rate server license? Not decided — depends on market research.
