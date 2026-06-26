## Context

TurboSheet has a well-architected Rust core with `PageEngine` trait supporting Chromium via chromiumoxide CDP. However:

1. **Test executor returns "Simulated Test"** - All tests pass without executing real code
2. **Firefox/WebKit are empty stubs** - Only Chromium actually works
3. **Multiple PageEngine methods return `Ok(())` or `None`** - bounding_box, drag_and_drop, press, set_input_files
4. **No auto-wait intelligence** - Basic polling with fixed intervals
5. **No trace viewer** - Only basic screenshot capture

The goal is to make TurboSheet a **complete, reliable E2E testing platform** that can genuinely compete with Playwright.

## Goals / Non-Goals

**Goals:**

- Real test execution with isolated-v8 runtime
- Complete Firefox and WebKit automation
- Smart auto-wait system (visibility, stability, actionability)
- Time-travel trace debugging with 10x compression
- Automated migration from Playwright/Cypress
- 10,000+ concurrent browser contexts via swarm architecture

**Non-Goals:**

- Building new JS runtime (use isolated-v8 which is already integrated)
- Implementing TypeScript type-checking (transpile only, skip type errors)
- Supporting Deno or Bun as test runners (Node.js only initially)
- Full Visual Studio integration (VS Code extension only)

## Decisions

### Decision 1: Firefox Integration - WebDriver BiDi over CDP

**Choice:** Use WebDriver BiDi protocol via geckodriver for Firefox automation

**Rationale:**

- CDP is Chrome/Edge-only (not available in Firefox)
- WebDriver BiDi is the W3C standard for cross-browser automation
- TurboSheet already has `bidi_preprocessor.rs` for BiDi event processing
- geckodriver provides WebDriver-compatible API with BiDi extensions

**Alternatives considered:**

- Direct Firefox DevTools Protocol: Firefox supports it but geckodriver is the maintained path
- Marionette (legacy): Deprecated in favor of BiDi

### Decision 2: WebKit Integration - webkit2gtk over Safari Technology Preview

**Choice:** Use webkit2gtk via `wry` crate for WebKit automation

**Rationale:**

- webkit2gtk provides full WebKit API including automation
- `wry` crate provides safe Rust bindings
- Cross-platform (Linux primary, macOS via WebKitGTK)
- Apple only provides limited automation via Remote Inspector

**Alternatives considered:**

- Safari WebDriver: Only works on macOS, requires Safari Developer mode
- Safari Technology Preview: Not suitable for production

### Decision 3: Test Execution - isolated-v8 with Async Runtime Bridge

**Choice:** Use existing isolated-v8 integration with tokio task bridge for async CDP calls

```
┌─────────────────────────────────────────────────────────────────┐
│                    Test Execution Flow                           │
├─────────────────────────────────────────────────────────────────┤
│                                                                  │
│  test.tsheet.ts ──► SWC Transpile ──► JS String                │
│                          │                                       │
│                          ▼                                       │
│  ┌────────────────────────────────────────────────────────────┐  │
│  │              isolated-v8 Isolate                            │  │
│  │  ┌──────────┐  ┌──────────┐  ┌──────────┐                │  │
│  │  │   page   │  │  expect  │  │   test   │                │  │
│  │  │  (sync)  │  │ (sync)  │  │  (sync)  │                │  │
│  │  └────┬─────┘  └────┬─────┘  └────┬─────┘                │  │
│  │       │             │             │                       │  │
│  │       └─────────────┴─────────────┘                       │  │
│  │                     │                                       │  │
│  │                     ▼                                       │  │
│  │         ┌─────────────────────┐                            │  │
│  │         │  Tokio Channel      │                            │  │
│  │         │  Bridge (sync→async)│                            │  │
│  │         └──────────┬──────────┘                            │  │
│  └───────────────────┼───────────────────────────────────────┘  │
│                      │                                            │
│                      ▼                                            │
│  ┌────────────────────────────────────────────────────────────┐  │
│  │              Rust Core (tokio runtime)                      │  │
│  │  ┌──────────┐  ┌──────────┐  ┌──────────┐                  │  │
│  │  │ PAGES    │  │  CDP     │  │  Assert  │                  │  │
│  │  │  Map     │  │  calls   │  │  Engine  │                  │  │
│  │  └──────────┘  └──────────┘  └──────────┘                  │  │
│  └────────────────────────────────────────────────────────────┘  │
│                                                                  │
└─────────────────────────────────────────────────────────────────┘
```

**Rationale:**

- isolated-v8 provides v8 isolate within Node.js (same engine as Chromium)
- Async CDP calls need sync wrapper in v8 context
- Tokio channels provide the bridge between sync isolate and async Rust

### Decision 4: Auto-Wait System - Condition-Based with Exponential Backoff

**Choice:** Implement wait conditions as composable predicates with exponential backoff

```
┌─────────────────────────────────────────────────────────────────┐
│                    Auto-Wait Conditions                          │
├─────────────────────────────────────────────────────────────────┤
│                                                                  │
│  Visibility Condition:                                           │
│  ┌────────────────────────────────────────────────────────────┐  │
│  │ element !== null                                           │  │
│  │ display !== 'none'                                         │  │
│  │ visibility !== 'hidden'                                     │  │
│  │ opacity !== '0'                                             │  │
│  └────────────────────────────────────────────────────────────┘  │
│                                                                  │
│  Stability Condition:                                            │
│  ┌────────────────────────────────────────────────────────────┐  │
│  │ No DOM mutations for 2 consecutive checks (100ms apart)       │  │
│  │ Using MutationObserver                                      │  │
│  └────────────────────────────────────────────────────────────┘  │
│                                                                  │
│  Actionability Condition:                                       │
│  ┌────────────────────────────────────────────────────────────┐  │
│  │ isVisible() && isEnabled() && isStable()                   │  │
│  │ + boundingBox !== null (element not clipped)               │  │
│  └────────────────────────────────────────────────────────────┘  │
│                                                                  │
│  Backoff Strategy:                                              │
│  ┌────────────────────────────────────────────────────────────┐  │
│  │  Initial: 100ms                                            │  │
│  │  Max: 5000ms                                                │  │
│  │  Multiplier: 1.5x                                           │  │
│  │  Jitter: ±10%                                               │  │
│  └────────────────────────────────────────────────────────────┘  │
│                                                                  │
└─────────────────────────────────────────────────────────────────┘
```

### Decision 5: Trace Compression - zstd + ciborium (CBOR)

**Choice:** Use zstd for compression and ciborium for CBOR serialization

**Rationale:**

- zstd provides 10-20x compression ratio at high speed
- CBOR is more compact than JSON (30-40% smaller)
- ciborium is a pure Rust CBOR encoder/decoder
- Traces can be 100MB+ for complex test runs

**Trace File Format:**

```
┌─────────────────────────────────────────────────────────────────┐
│                        TurboTrace File                           │
├─────────────────────────────────────────────────────────────────┤
│  Header (16 bytes):                                             │
│  - Magic: "TTRC" (4 bytes)                                      │
│  - Version: u16                                                 │
│  - Compression flags: u16                                       │
│  - Entry count: u32                                             │
│  - Start timestamp: u64                                         │
├─────────────────────────────────────────────────────────────────┤
│  Entry (repeated):                                               │
│  - Type: u8 (action, event, network, console, dom)              │
│  - Timestamp delta: u32 (ms since last)                          │
│  - Data (CBOR-encoded):                                          │
│    - action: { type, selector, duration, result }                │
│    - dom: { html, selector, boundingBox }                        │
│    - network: { url, method, status, timing }                    │
│    - console: { level, message, args }                          │
└─────────────────────────────────────────────────────────────────┘
```

### Decision 6: Migration Tooling - AST-Based Conversion

**Choice:** Parse source files with SWC and transform AST to TurboSheet format

**Rationale:**

- SWC is 20x faster than Babel for TypeScript parsing
- AST allows precise code transformation vs regex
- Can handle complex cases (async/await, imports, decorators)

**Conversion Coverage:**

```
┌─────────────────────────────────────────────────────────────────┐
│                  Migration Conversion Matrix                     │
├─────────────────────────────────────────────────────────────────┤
│  Playwright → TurboSheet                                         │
│  ─────────────────────────────────────────────────────────────── │
│  ✓ page.goto()        → page.goto()                             │
│  ✓ page.locator()     → page.locator()                          │
│  ✓ locator.click()    → locator.click()                         │
│  ✓ expect().toBe()    → expect().to_be_()                        │
│  ✓ page.evaluate()    → page.evaluate()                          │
│  ⚠ webServer()       → requires manual setup                    │
│                                                                  │
│  Cypress → TurboSheet                                           │
│  ─────────────────────────────────────────────────────────────── │
│  ✓ cy.get()           → page.locator()                          │
│  ✓ cy.contains()      → page.locator().filter()                  │
│  ✓ cy.click()         → locator.click()                          │
│  ✓ cy.type()          → locator.fill()                           │
│  ⚠ cy.wait()         → page.waitForSelector()                   │
│  ⚠ cy.intercept()     → page.route()                             │
│  ⚠ cy.session()      → requires manual setup                    │
└─────────────────────────────────────────────────────────────────┘
```

## Risks / Trade-offs

| Risk                                                   | Mitigation                                                                                      |
| ------------------------------------------------------ | ----------------------------------------------------------------------------------------------- |
| isolated-v8 memory leaks with long-running test suites | Implement explicit isolate disposal after each test file; add memory monitoring to CI           |
| Firefox BiDi support incomplete in geckodriver         | Implement fallback to Chrome-only features where Firefox gaps exist; contribute upstream fixes  |
| Auto-wait race conditions with dynamic content         | Use MutationObserver for stability detection; allow explicit `waitForFunction()` for edge cases |
| Trace files grow too large for CI                      | Implement trace rotation (max size, max entries); offer trace upload to cloud storage           |
| Migration produces incorrect test logic                | Implement 80% automatic + 20% manual review flow; provide `tsheet migrate --dry-run`            |
| WASM edge runtime complexity                           | Start with Node.js only; WASM as v2 feature; use `wasm-bindgen` for FFI                         |

## Migration Plan

**Phase 1 - Foundation (Weeks 1-4):**

1. Wire isolated-v8 to real test execution
2. Implement Firefox WebDriver/BiDi integration
3. Complete missing PageEngine methods

**Phase 2 - Reliability (Weeks 5-8):** 4. Implement smart auto-wait system 5. Add flaky test detection and retry logic 6. Complete WebKit integration

**Phase 3 - DX (Weeks 9-12):** 7. Implement TurboTrace viewer 8. Add VS Code extension 9. Build migration tooling

**Phase 4 - Scale (Weeks 13-16):** 10. Implement swarm execution architecture 11. Add CI/CD templates and Docker images 12. Component testing module

**Deployment:**

- New npm package version for each phase
- Feature flags to disable incomplete features
- Migration guide documentation
- Breaking changes only in major versions

## Open Questions

1. **Should TurboSheet support test parallelization at the file or test level?**
   - Playwright: Files in parallel, tests within file serial
   - Cypress: Tests within file parallel (experimental)
   - Decision needed for swarm architecture

2. **How to handle page/context pooling for performance?**
   - Option A: Pool pages (faster, risk state leakage)
   - Option B: Fresh page per test (slower, safer)
   - Could offer both with config option

3. **Should isolated-v8 run in-process or as worker threads?**
   - In-process: Faster communication, but blocks event loop
   - Worker threads: Better isolation, but IPC overhead

4. **BiDi vs CDP for new protocol features?**
   - CDP has more features but is Chrome-only
   - BiDi is cross-browser but less mature
   - Could implement dual-protocol with feature detection
