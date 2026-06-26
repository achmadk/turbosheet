# TurboSheet PRD v3: Injected Script Engine Architecture

> **Version:** 3.0
> **Date:** 2026-06-04
> **Status:** Draft
> **Authors:** TurboSheet Core Team
> **Supersedes:** PRD v2 (Browser Testing Platform Expansion)

---

## Executive Summary

TurboSheet v3 introduces a **pre-injected script engine** (`injected.js`) as the foundational layer for all browser-side DOM interaction, replacing the current ad-hoc `page.evaluate()` pattern. This architectural shift—modeled after Playwright's proven approach via CDP's `Page.addScriptToEvaluateOnNewDocument` and WebDriver BiDi's script pinning—resolves three critical bottlenecks: **duplicated cross-engine JS logic**, **excessive CDP round-trips per action**, and **fragile inline JavaScript string construction**.

The injected script will be compiled, minified, and embedded directly into the Rust binary via `include_str!()`, preserving TurboSheet's zero-config distribution model while dramatically reducing per-action latency.

---

## 1. Problem Statement

### 1.1 Current Architecture Deficiencies

TurboSheet's current `PageEngine` implementations construct JavaScript strings inline for every browser interaction. Analysis of the codebase reveals:

- **50+ instances** of `format!("(function() {{ ... }})()")` calls across `chromium.rs`, `firefox.rs`, and `webkit.rs`
- **Duplicated logic** — The same visibility/actionability/bounding-box JavaScript appears in all three engine implementations
- **3-10+ CDP round-trips per user action** — Each `.click()` involves separate calls for actionability checking, element finding, and action dispatch
- **Fragile string construction** — Selector escaping (`replace("'", "\\\\'")`) is done ad-hoc, creating injection risk and maintenance burden
- **No pre-injection** — All JavaScript is evaluated on demand, missing the performance benefits of `addScriptToEvaluateOnNewDocument`
- **No binding bridge** — Communication relies solely on `evaluate()` return values, preventing streaming events or async callbacks

### 1.2 Quantified Impact

| Metric                                | Current State                         | Target (with injected.js)     |
| ------------------------------------- | ------------------------------------- | ----------------------------- |
| CDP round-trips per `.click()`        | 3-10+ (with retries)                  | **1** (binding callback)      |
| JS logic duplication across engines   | 3x (chromium/firefox/webkit)          | **1x** (shared `injected.js`) |
| Actionability check latency           | ~50ms per poll cycle (fixed interval) | **<5ms** (in-page observer)   |
| Inline JS string allocations per test | Hundreds                              | **0** (pre-injected)          |

---

## 2. Proposed Architecture: Hybrid Injection Engine

### 2.1 Core Design

Instead of a monolithic injected script (like Playwright's thousands of lines), TurboSheet will use a **two-module hybrid injection philosophy** optimized for the v2.0 concurrency and AI goals:

```
┌─────────────────────────────────────────────────────────────────────┐
│                   TurboSheet Injection Architecture                   │
├─────────────────────────────────────────────────────────────────────┤
│                                                                      │
│  ┌─ Phase 1: Document Start ──────────────────────────────────────┐  │
│  │                                                                 │  │
│  │  core.js  (~2KB minified)                              │  │
│  │  ┌──────────────────────────────────────────────────────────┐   │  │
│  │  │  • Establish __turbosheet binding (IIFE-wrapped)         │   │  │
│  │  │  • Register MutationObserver for stability detection     │   │  │
│  │  │  • Set up event listeners for navigation/frame events    │   │  │
│  │  │  • Expose lightweight querySelector utilities            │   │  │
│  │  └──────────────────────────────────────────────────────────┘   │  │
│  │                                                                 │  │
│  │  Injected via: Page.addScriptToEvaluateOnNewDocument (CDP)      │  │
│  │                script.addPreloadScript (WebDriver BiDi)         │  │
│  └─────────────────────────────────────────────────────────────────┘  │
│                                                                      │
│  ┌─ Phase 2: On-Demand Payload ───────────────────────────────────┐  │
│  │                                                                 │  │
│  │  actions.js  (loaded only when needed)                 │  │
│  │  ┌──────────────────────────────────────────────────────────┐   │  │
│  │  │  • Actionability engine (visibility, stability, enabled) │   │  │
│  │  │  • Geometry checking (bounding box, intersection)        │   │  │
│  │  │  • Smart auto-wait with exponential backoff              │   │  │
│  │  │  • Agent snapshot / DOM traversal for AI features         │   │  │
│  │  │  • Drag-and-drop, file input, complex interactions       │   │  │
│  │  └──────────────────────────────────────────────────────────┘   │  │
│  │                                                                 │  │
│  │  Injected via: Runtime.evaluate() on first action invocation    │  │
│  └─────────────────────────────────────────────────────────────────┘  │
│                                                                      │
│  ┌─ Rust Runtime Bridge ──────────────────────────────────────────┐  │
│  │                                                                 │  │
│  │  Runtime.addBinding("__tsReport") ──► Tokio channel ──► Result │  │
│  │  Runtime.addBinding("__tsLog")    ──► Tracing subscriber       │  │
│  │                                                                 │  │
│  └─────────────────────────────────────────────────────────────────┘  │
│                                                                      │
└─────────────────────────────────────────────────────────────────────┘
```

### 2.2 Module Breakdown

#### Module 1: Core Traversal (`core.js`)

**Purpose:** Minimal footprint script injected at document start to establish the native Rust binding and DOM observation infrastructure.

**Size target:** ≤2KB minified

**Responsibilities:**

- Establish the `Runtime.addBinding` bridge to Rust (via a randomized global name for stealth)
- Register a `MutationObserver` on `document.documentElement` for stability detection
- Provide lightweight `querySelector` / `querySelectorAll` wrappers with error handling
- Track frame/iframe context for multi-frame pages
- Self-cleanup on page unload

**Injection point:** `Page.addScriptToEvaluateOnNewDocument` (CDP) / `script.addPreloadScript` (WebDriver BiDi)

#### Module 2: On-Demand Payload (`actions.js`)

**Purpose:** Heavy actionability, geometry, and interaction functions loaded only when a Node.js user actually invokes an action like `.click()` or `.getAgentSnapshot()`.

**Size target:** ≤15KB minified (tree-shaken per capability)

**Responsibilities:**

- Full actionability engine (visibility, stability, enabled, not-covered)
- Bounding box / intersection observer calculations
- Smart auto-wait with MutationObserver + exponential backoff
- Complex interactions (drag-and-drop, file input simulation, keyboard sequences)
- DOM traversal and snapshot generation for AI agent features
- Visual regression helpers (layout shift detection, element highlighting)

**Injection point:** `Runtime.evaluate()` — loaded on first action call, cached in page context

### 2.3 Rust-Side Architecture Changes

```
┌─────────────────────────────────────────────────────────────────────┐
│                   Rust Module: src/injection/                        │
├─────────────────────────────────────────────────────────────────────┤
│                                                                      │
│  mod.rs                                                              │
│  ├── InjectionManager                                                │
│  │   ├── inject_core(page: &dyn PageEngine)                          │
│  │   ├── inject_actions(page: &dyn PageEngine)                       │
│  │   ├── call_binding(page_id, method, args) -> Result<Value>        │
│  │   └── on_binding_called(name, payload) -> dispatch to channel     │
│  │                                                                   │
│  scripts.rs                                                          │
│  ├── CORE_SCRIPT: &str = include_str!("../../js/core.mjs")│
│  ├── ACTIONS_SCRIPT: &str = include_str!("../../js/actions.mjs")│
│  └── obfuscate_entry_points(script: &str) -> String                  │
│                                                                      │
│  bindings.rs                                                         │
│  ├── BindingRegistry                                                 │
│  │   ├── register(name, callback)                                    │
│  │   ├── dispatch(name, payload)                                     │
│  │   └── pending_results: DashMap<RequestId, oneshot::Sender>        │
│  │                                                                   │
│  stealth.rs                                                          │
│  ├── randomize_globals(script: &str) -> (String, NameMap)            │
│  ├── wrap_iife(script: &str) -> String                               │
│  └── detect_bot_checks(page: &dyn PageEngine) -> Vec<BotCheck>      │
│                                                                      │
└─────────────────────────────────────────────────────────────────────┘
```

### 2.4 Integration with Existing PageEngine Trait

The `PageEngine` trait (`src/engine/mod.rs`) will gain two new methods:

```rust
#[async_trait]
pub trait PageEngine: Send + Sync {
    // ... existing methods ...

    /// Pre-inject the core traversal script into the page context.
    /// Called automatically on page creation and after navigation.
    async fn inject_core_script(&self, script: &str) -> Result<(), TurbosheetError>;

    /// Register a binding that allows injected JS to call back into Rust.
    async fn register_binding(&self, name: &str) -> Result<(), TurbosheetError>;
}
```

**Engine-specific implementations:**

| Engine       | `inject_core_script`                    | `register_binding`                         |
| ------------ | --------------------------------------- | ------------------------------------------ |
| **Chromium** | `Page.addScriptToEvaluateOnNewDocument` | `Runtime.addBinding`                       |
| **Firefox**  | `script.addPreloadScript` (BiDi)        | `script.callFunction` + WebSocket          |
| **WebKit**   | `WebKitUserContentManager.addScript`    | `WebKitUserContentManager.registerHandler` |

---

## 3. Positive Impacts (Advantages)

### 3.1 Simplified Distribution

The injected scripts will be compiled, minified, and embedded directly into the Rust binary:

```rust
// src/injection/scripts.rs
pub const CORE_SCRIPT: &str = include_str!("../../js/dist/core.mjs");
pub const ACTIONS_SCRIPT: &str = include_str!("../../js/dist/actions.mjs");
```

**Impact:** No Chrome Extension directory to unpack, no manifest.json to maintain, no absolute file path resolution inside the npm package. The `turbosheet.linux-x64-gnu.node` binary (currently ~416MB) gains negligible size from the embedded scripts.

### 3.2 Main World Context Access

A native script evaluation tool runs directly inside the page's execution context, allowing it to:

- Hook into the DOM and capture elements without Isolated World restrictions
- Listen to internal browser events (MutationObserver, IntersectionObserver, PerformanceObserver)
- Access and manipulate the page's JavaScript objects directly
- Bridge to Rust via `Runtime.addBinding` for zero-copy result transfer

**Impact:** Eliminates the need for the current `page.evaluate(format!(...))` pattern that constructs JavaScript strings per call. The injected script's functions are already available in the page context.

### 3.3 Compatibility with Headless Modes

The `addScriptToEvaluateOnNewDocument` approach works across all Chrome variations:

- Legacy `--headless` mode
- New `--headless=new` mode
- Headed mode (for debugging)
- Docker containers (TurboSheet's CI/CD target per proposal.md)

**Impact:** Maximum infrastructure flexibility for the Swarm Scale Architecture (10,000+ concurrent contexts) without mode-specific workarounds.

### 3.4 Cross-Engine JS Unification

**This is the most significant win specific to TurboSheet.**

Currently, the same actionability/visibility/interaction JavaScript is duplicated across three engine files:

- `src/engine/chromium.rs` — 705 lines, majority is inline JS
- `src/engine/firefox.rs` — ~450 lines, duplicated JS logic
- `src/engine/webkit.rs` — similar duplication

With `injected.js`, the browser-side logic exists **once** and is shared across all engines. The Rust `PageEngine` implementations become thin transport layers that only differ in _how_ they inject and _how_ they receive binding callbacks.

---

## 4. Technical Challenges & Mitigations

### 4.1 The Event-Loop Block & Execution Latency

**Challenge:** If the injected script is massive, executing it on every frame, iframe, and page navigation introduces a CPU tax. A page with 20 nested iframes means the script runs 21 times.

**Mitigation (Hybrid Approach):**

- The **Core Traversal** module is kept under 2KB — its execution cost is negligible
- The **On-Demand Payload** is only injected when a user actually invokes an action
- Tree-shaking ensures only required capabilities are included per test scenario
- The build pipeline will use `rolldown` for aggressive minification

**TurboSheet-Specific Context:** The current `wait_for_actionability()` in `locator.rs` already evaluates ~40 lines of JavaScript on every action call via `page.evaluate()`. The injected approach would evaluate this _once_ (on-demand load), then call cached functions — a net reduction in execution cost.

### 4.2 Cross-Framework Binding Maintenance

**Challenge:** Without a Chrome Extension's background service worker for message streaming, the injected script must bridge the gap between the page's JavaScript context and the Rust runtime via `Runtime.addBinding`.

**Mitigation:**

```
┌─────────────────────────────────────────────────────────────────────┐
│               Binding Bridge: JS ↔ Rust                              │
├─────────────────────────────────────────────────────────────────────┤
│                                                                      │
│  Page Context (injected.js)                                          │
│  ┌──────────────────────────────────────────────────────────────┐    │
│  │  function performAction(selector, action) {                  │    │
│  │    const requestId = crypto.randomUUID();                    │    │
│  │    const result = checkActionability(selector);              │    │
│  │    if (!result.ok) {                                         │    │
│  │      __tsReport(JSON.stringify({                             │    │
│  │        id: requestId,                                        │    │
│  │        error: result.reason                                  │    │
│  │      }));                                                    │    │
│  │      return;                                                 │    │
│  │    }                                                         │    │
│  │    executeAction(selector, action);                          │    │
│  │    __tsReport(JSON.stringify({                               │    │
│  │      id: requestId,                                          │    │
│  │      ok: true,                                               │    │
│  │      data: { boundingBox: result.rect }                      │    │
│  │    }));                                                      │    │
│  │  }                                                           │    │
│  └──────────────────────────────────────────────────────────────┘    │
│                          │                                           │
│                          ▼ Runtime.addBinding("__tsReport")          │
│                                                                      │
│  Rust Runtime (src/injection/bindings.rs)                            │
│  ┌──────────────────────────────────────────────────────────────┐    │
│  │  BindingRegistry {                                           │    │
│  │    pending: DashMap<RequestId, oneshot::Sender<Value>>,       │    │
│  │                                                              │    │
│  │    fn on_binding_called(&self, payload: &str) {              │    │
│  │      let msg: BindingMessage = serde_json::from_str(payload);│    │
│  │      if let Some((_, tx)) = self.pending.remove(&msg.id) {   │    │
│  │        tx.send(msg.data).ok();                               │    │
│  │      }                                                       │    │
│  │    }                                                         │    │
│  │  }                                                           │    │
│  └──────────────────────────────────────────────────────────────┘    │
│                                                                      │
└─────────────────────────────────────────────────────────────────────┘
```

**Key Design Decision:** Use request-response correlation via UUIDs, not streaming. Each Rust-initiated action generates a UUID, sends it to `injected.js` via `Runtime.evaluate`, and waits on a `oneshot::Receiver`. The injected script reports results back via the binding with the same UUID. This avoids the complexity of a full event streaming system while maintaining correctness.

### 4.3 DOM Pollution Risk

**Challenge:** The script executes inside the page's global scope, risking pollution or breakage of the page's native JavaScript. Malicious pages can detect automation globals.

**Mitigation:**

1. **Strict IIFE Wrapping:** All injected code runs inside `(function() { ... })()` — zero globals leak
2. **Runtime Name Randomization:** The binding name (e.g., `__tsReport`) is replaced with a random string at injection time:
   ```rust
   // src/injection/stealth.rs
   fn randomize_globals(script: &str) -> (String, NameMap) {
       let random_name = format!("__{}", uuid::Uuid::new_v4().simple());
       let modified = script.replace("__tsReport", &random_name);
       (modified, NameMap { report_binding: random_name })
   }
   ```
3. **Symbol-Based Isolation (Advanced):** For maximum stealth, use `Symbol()` for internal state rather than named properties
4. **No `window.__turbosheet`:** Unlike naive implementations, TurboSheet will never expose a detectable global object

---

## 5. Strategic Framework Evaluation

| Metric                     | Chrome Extension               | Ad-Hoc Evaluate (Current) | **Injected Script (Proposed)** |
| -------------------------- | ------------------------------ | ------------------------- | ------------------------------ |
| **npm Package Footprint**  | Heavy (files/manifests)        | Embedded ✅               | **Embedded ✅**                |
| **Execution Overhead**     | Low (browser-isolated)         | High (per-call eval)      | **Low (pre-injected)**         |
| **Security/Stealth**       | Medium (extension fingerprint) | Medium (eval detectable)  | **High (obfuscatable)**        |
| **Maintenance Complexity** | Medium (browser API updates)   | High (3x duplication)     | **Medium (single source)**     |
| **Cross-Engine Support**   | ❌ Chrome only                 | ✅ All (duplicated)       | **✅ All (unified)**           |
| **CDP Round-Trips/Action** | 1-2                            | 3-10+                     | **1**                          |
| **Auto-Wait Quality**      | Service worker based           | Fixed-interval polling    | **MutationObserver native**    |
| **AI Agent Compatibility** | Limited                        | Limited                   | **Full (DOM traversal)**       |

---

## 6. Implementation Plan

### Phase 0: Build Infrastructure (Week 1)

| Task                           | Description                                                      |
| ------------------------------ | ---------------------------------------------------------------- |
| Create `js/` directory         | Source for `core.ts` and `actions.ts`                            |
| Add build pipeline             | Rolldown for minification, output to `js/dist/`                  |
| Add `build.rs` integration     | Verify `include_str!()` paths at compile time                    |
| Create `src/injection/` module | Rust-side injection manager, binding registry, stealth utilities |

### Phase 1: Core Traversal Injection (Weeks 2-3)

| Task                                     | Description                                                       |
| ---------------------------------------- | ----------------------------------------------------------------- |
| Implement `core.ts`                      | Binding establishment, MutationObserver, querySelector utilities  |
| Add `inject_core_script()` to PageEngine | CDP: `addScriptToEvaluateOnNewDocument`, BiDi: `addPreloadScript` |
| Add `register_binding()` to PageEngine   | CDP: `Runtime.addBinding`, BiDi/WebKit: equivalent                |
| Implement `BindingRegistry`              | Request-response correlation, `DashMap<UUID, oneshot::Sender>`    |
| Wire injection into page creation        | Auto-inject on `ContextEngine::new_page()` and after navigation   |

### Phase 2: Action Payload Migration (Weeks 4-6)

| Task                               | Description                                                            |
| ---------------------------------- | ---------------------------------------------------------------------- |
| Implement `actions.ts`             | Port actionability checks from `locator.rs` inline JS                  |
| Migrate `ChromiumPageEngine`       | Replace inline `format!()` JS with binding calls to injected functions |
| Migrate `FirefoxPageEngine`        | Same — remove duplicated JS, use shared injected script                |
| Migrate `WebKitPageEngine`         | Same — remove duplicated JS, use shared injected script                |
| Migrate `wait_for_actionability()` | Replace locator.rs polling with MutationObserver-based wait            |

### Phase 3: Advanced Features (Weeks 7-8)

| Task                      | Description                                                 |
| ------------------------- | ----------------------------------------------------------- |
| Implement stealth mode    | Runtime name randomization, IIFE wrapping, Symbol isolation |
| Add on-demand loading     | Lazy-load `actions.js` only on first action call            |
| Implement agent snapshot  | DOM traversal for AI features via injected script           |
| Add frame/iframe tracking | Multi-frame injection management                            |
| Performance benchmarking  | Compare CDP round-trips before/after                        |

### Phase 4: Integration with Existing Roadmap (Weeks 9-12)

| Task                             | Description                                                       |
| -------------------------------- | ----------------------------------------------------------------- |
| Integrate with Auto-Wait System  | Replace Task 5.x polling with injected MutationObserver           |
| Integrate with Component Testing | Use injection for framework-agnostic component interaction        |
| Integrate with TurboTrace        | Record injection events in trace files                            |
| Integrate with Visual Regression | Use injected script for element highlighting and layout detection |

---

## 7. Build Pipeline

```
┌─────────────────────────────────────────────────────────────────────┐
│                   JS → Rust Embedding Pipeline                       │
├─────────────────────────────────────────────────────────────────────┤
│                                                                      │
│  js/src/                                                             │
│  ├── core.ts          TypeScript source                     │
│  ├── actions.ts       TypeScript source                     │
│  ├── utils/                    Shared utilities                      │
│  │   ├── selector.ts           querySelector wrappers                │
│  │   ├── actionability.ts      Visibility/stability/enabled checks   │
│  │   └── geometry.ts           Bounding box / intersection           │
│  └── types.ts                  Shared type definitions               │
│           │                                                          │
│           ▼                                                          │
│  Rolldown (tree-shaken, minified)                             │
│           │                                                          │
│           ▼                                                          │
│  js/dist/                                                            │
│  ├── core.mjs      ≤2KB                                  │
│  └── actions.mjs   ≤15KB                                 │
│           │                                                          │
│           ▼                                                          │
│  build.rs verifies files exist                                       │
│           │                                                          │
│           ▼                                                          │
│  src/injection/scripts.rs                                            │
│  pub const CORE: &str = include_str!("../../js/dist/core.mjs");│
│  pub const ACTIONS: &str = include_str!("../../js/dist/actions.mjs");│
│                                                                      │
└─────────────────────────────────────────────────────────────────────┘
```

---

## 8. Risk Analysis

| Risk                                                  | Severity | Likelihood | Mitigation                                                                                       |
| ----------------------------------------------------- | -------- | ---------- | ------------------------------------------------------------------------------------------------ |
| Injected script breaks on specific websites           | High     | Medium     | Strict IIFE isolation, no prototype pollution, comprehensive test suite against top 100 websites |
| Bot detection fingerprints injection                  | Medium   | Medium     | Runtime name randomization, Symbol-based state, no global exposure                               |
| Performance regression on iframe-heavy pages          | Medium   | Low        | Core module is ≤2KB, lazy-load actions module, benchmark against Playwright                      |
| Binding message lost during page navigation           | High     | Medium     | Navigation-aware message queue, auto-re-inject on `frameNavigated` event                         |
| Memory leak in long-running test suites               | Medium   | Medium     | Explicit cleanup on page close, WeakRef for DOM references, periodic GC hints                    |
| Build pipeline adds npm dependency for JS compilation | Low      | High       | Use existing `vp` toolchain (already in project), add to CI validation                           |

---

## 9. Success Metrics

| Metric                              | Current Baseline       | Target                         | Measurement Method        |
| ----------------------------------- | ---------------------- | ------------------------------ | ------------------------- |
| CDP round-trips per `.click()`      | 3-10+                  | ≤2                             | CDP protocol logging      |
| Actionability check latency (p95)   | ~50ms                  | ≤10ms                          | TurboTrace timing         |
| JS logic duplication across engines | ~1500 lines duplicated | 0 (shared injected.js)         | LOC count in engine files |
| Bundle size increase                | 0                      | ≤20KB                          | Binary size comparison    |
| Stealth score (bot detection tests) | Not measured           | Pass 10/10 bot detection sites | Automated test suite      |

---

## 10. Relationship to PRD v2 Roadmap

The Injected Script Engine is **not a replacement** for the PRD v2 capabilities — it is **foundational infrastructure** that makes them better:

| PRD v2 Capability              | How Injected Script Engine Improves It                                      |
| ------------------------------ | --------------------------------------------------------------------------- |
| **Smart Auto-Wait (5.x)**      | MutationObserver runs in-page instead of Rust-side polling loops            |
| **Complete Locator API (2.x)** | Actions execute via single binding call instead of multiple CDP round-trips |
| **Firefox Automation (3.x)**   | Shared `injected.js` eliminates need to rewrite JS for BiDi transport       |
| **WebKit Automation (4.x)**    | Same shared JS, webkit2gtk injects via `UserContentManager`                 |
| **Visual Regression (6.x)**    | In-page layout shift detection via PerformanceObserver                      |
| **Swarm Scale (9.x)**          | Reduced per-page overhead enables higher concurrency at same CPU budget     |
| **Component Testing (10.x)**   | Unified injection for framework-agnostic component interaction              |

---

## 11. Open Questions

1. **Should the binding bridge use CDP `Runtime.addBinding` or `Runtime.evaluate` with a polling callback?**
   - `addBinding` is cleaner but requires CDP event subscription management
   - Polling is simpler but adds latency
   - **Recommendation:** `addBinding` for Chromium, protocol-equivalent for Firefox/WebKit

2. **Should the injected script support source maps for debugging?**
   - Source maps aid development but could expose internals if shipped
   - **Recommendation:** Source maps in dev builds only, strip in production

3. **How should multi-frame pages handle the binding namespace?**
   - Each frame gets its own injected context
   - The binding name must be unique per frame to avoid collisions
   - **Recommendation:** Append frame ID to binding name

4. **Should the stealth mode be enabled by default or opt-in?**
   - Default stealth adds slight complexity
   - Opt-in risks users forgetting to enable it for production
   - **Recommendation:** Default on, with `stealth: false` config option for debugging

---

## Appendix A: Playwright Reference Architecture

For context, Playwright's injection engine:

- Injects ~3000 lines of JavaScript per page
- Uses `Page.addScriptToEvaluateOnNewDocument` for CDP
- Uses `script.addPreloadScript` for WebDriver BiDi
- Communicates via `Runtime.addBinding("__playwright")`
- Handles multi-frame via per-frame execution contexts
- Uses source maps for internal debugging

TurboSheet's hybrid approach (2KB core + on-demand payload) is designed to be **lighter than Playwright's monolithic injection** while achieving equivalent functionality.

## Appendix B: Current Codebase References

Key files affected by this change:

- `src/engine/mod.rs` — PageEngine trait extension
- `src/engine/chromium.rs` — Replace inline JS with binding calls
- `src/engine/firefox.rs` — Replace inline JS with binding calls
- `src/engine/webkit.rs` — Replace inline JS with binding calls
- `src/locator.rs` — Replace `wait_for_actionability()` inline JS
- `src/component/mount.rs` — Use injection for component interaction
- `src/assertions/engine.rs` — Integrate with MutationObserver-based waits
- `build.rs` — Add JS dist file verification
- `Cargo.toml` — No new dependencies required (JS embedding is free)

---

# Part II: Comprehensive Competitive Improvement Roadmap

> **Added:** 2026-06-04 — Deep-dive codebase audit covering all 20+ modules, 3 engine
> implementations, and the full public API surface. This section transforms TurboSheet from
> a 12/43 feature score to a roadmap that surpasses Playwright (39/43).

---

## 12. Deep Dive: The Three Critical Subsystems

### 12.1 The CDP Event System (Missing Foundation)

**Current state:** Zero CDP event subscriptions exist anywhere in the codebase.

The `chromiumoxide` handler task in `chromium.rs:55-62` consumes events, but only to detect fatal errors:

```rust
// chromium.rs:55-62 — the ONLY event handling
let _handler_task = tokio::spawn(async move {
    while let Some(event) = futures::StreamExt::next(&mut handler).await {
        if let Err(e) = event {
            tracing::error!("Browser handler error: {:?}", e);
            break;
        }
    }
});
```

No event is ever processed. This is the root cause of 8+ broken features:

```
┌──────────────────────────────────────────────────────────────────────┐
│                Missing CDP Event Subscriptions                        │
├───────────────────────────┬──────────────────────────────────────────┤
│ CDP Event                 │ Feature It Would Enable                  │
├───────────────────────────┼──────────────────────────────────────────┤
│ Page.frameNavigated       │ URL tracking (eliminates evaluate poll)  │
│ Page.frameDetached        │ Frame lifecycle tracking                 │
│ Page.domContentEventFired │ DomContentLoaded wait (currently fake)   │
│ Page.loadEventFired       │ Load state detection                    │
│ Page.javascriptDialog     │ Dialog accept/dismiss/getText            │
│ Runtime.consoleAPICalled  │ Console message capture                  │
│ Runtime.bindingCalled     │ Injected.js → Rust communication         │
│ Network.requestWillBeSent │ Request interception without proxy       │
│ Network.responseReceived  │ Response capture without proxy           │
│ Target.targetCreated      │ Popup/new-tab detection                  │
│ Browser.downloadWillBegin │ File download handling                   │
│ Page.screencastFrame      │ Video recording                          │
└───────────────────────────┴──────────────────────────────────────────┘
```

**What's needed — Event Dispatcher Architecture:**

```
┌──────────────────────────────────────────────────────────────────────┐
│               Proposed: CDP Event Dispatcher                          │
├──────────────────────────────────────────────────────────────────────┤
│                                                                       │
│  ChromiumPageEngine {                                                 │
│    page: Mutex<Option<Page>>,                                         │
│    current_url: RwLock<String>,                                       │
│    event_dispatcher: Arc<EventDispatcher>,    ◄── NEW                 │
│  }                                                                    │
│                                                                       │
│  struct EventDispatcher {                                             │
│    dialog_handlers: RwLock<Vec<oneshot::Sender<DialogAction>>>,       │
│    console_handlers: RwLock<Vec<mpsc::Sender<ConsoleMessage>>>,      │
│    request_handlers: DashMap<String, mpsc::Sender<NetworkRequest>>,   │
│    navigation_handlers: broadcast::Sender<NavigationEvent>,          │
│    download_handlers: mpsc::Sender<DownloadEvent>,                   │
│  }                                                                    │
│                                                                       │
│  // Handler task becomes:                                             │
│  tokio::spawn(async move {                                            │
│    while let Some(event) = handler.next().await {                    │
│      match event {                                                    │
│        CdpEvent::Dialog(d) => dispatcher.handle_dialog(d),           │
│        CdpEvent::Console(c) => dispatcher.handle_console(c),         │
│        CdpEvent::Navigation(n) => {                                  │
│          url_cache.update(n.url);                     ◄── eliminates  │
│          dispatcher.handle_navigation(n);                 evaluate    │
│        },                                                 URL poll   │
│        CdpEvent::BindingCalled(b) =>                                 │
│          dispatcher.handle_binding(b),      ◄── injected.js bridge   │
│        _ => {}                                                        │
│      }                                                                │
│    }                                                                  │
│  });                                                                  │
│                                                                       │
└──────────────────────────────────────────────────────────────────────┘
```

**Impact on `page.url()` synchronous problem:** Currently `chromium.rs:244-263` makes an extra `evaluate("window.location.href")` call after every `evaluate()` to update the URL cache — doubling round-trips. With `Page.frameNavigated` event subscription, the URL updates reactively and `page.url()` returns the cached value with zero extra CDP calls.

**Impact on injected.js:** The `Runtime.bindingCalled` event is the **mandatory foundation** for the injected.js binding bridge described in Section 2.4. Without this event subscription, the entire injected.js architecture cannot work.

---

### 12.2 The Network Proxy (Broken Foundation)

**Current state:** The proxy has three critical failures:

**Failure 1: `route.continue()` returns HTTP 501**

```rust
// proxy.rs:166-176 — RouteAction::Continue does NOTHING
RouteAction::Continue(_opts) => {
    // empty block — falls through to 501
}

// Falls through to:
Ok(Response::builder()
    .status(StatusCode::NOT_IMPLEMENTED)
    .body(Full::new(Bytes::from("Passthrough not fully implemented in this mock")))
    .unwrap())
```

When a user calls `route.continue()`, the proxy literally returns a 501 error page. The `ContinueOptions` (URL rewrite, header modification, POST data override) are accepted but completely ignored.

**Failure 2: HTTPS CONNECT is a no-op**

```rust
// proxy.rs:88-97 — CONNECT handler does nothing
if req.method() == Method::CONNECT {
    tokio::task::spawn(async move {
        match hyper::upgrade::on(req).await {
            Ok(_upgraded) => {
                // NOTHING HAPPENS WITH THE UPGRADED CONNECTION
            }
            Err(e) => tracing::error!("upgrade error: {}", e),
        }
    });
    return Ok(Response::new(Full::new(Bytes::new())));
}
```

HTTPS requests are never proxied. The CONNECT tunnel is upgraded but the upgraded connection is dropped immediately.

**Failure 3: POST body is never captured**

```rust
// proxy.rs:120-125 — post_data is always None
let route_req = RouteRequest {
    url: url.clone(),
    method,
    headers,
    post_data: None,  // ◄── ALWAYS NONE
};
```

The request body from `Incoming` is never read — `req.collect().await` is never called. Users see empty POST data in their route handlers.

**What's needed — Real Transparent Proxy:**

```
┌──────────────────────────────────────────────────────────────────────┐
│              Proposed: Real Transparent Proxy                         │
├──────────────────────────────────────────────────────────────────────┤
│                                                                       │
│  1. HTTPS CONNECT: Tunnel TCP stream to target host                  │
│     ┌─────────┐      ┌─────────┐      ┌─────────┐                   │
│     │ Browser  │─CONNECT──►│ Proxy   │──TCP──►│ Server  │             │
│     └─────────┘      │ (tunnel) │      └─────────┘                   │
│                       └─────────┘                                     │
│     For route interception: MitM with self-signed CA                 │
│     (like Playwright's internal proxy)                                │
│                                                                       │
│  2. Continue: Forward request to original destination                │
│     RouteAction::Continue(opts) => {                                 │
│       let mut req = original_request;                                │
│       if let Some(url) = opts.url { req.set_url(url); }             │
│       if let Some(headers) = opts.headers { req.merge_headers(h); } │
│       let resp = reqwest::Client::execute(req).await;                │
│       return Ok(resp.into_hyper_response());                         │
│     }                                                                 │
│                                                                       │
│  3. POST body: Read Incoming stream before routing                   │
│     let body_bytes = req.collect().await?.to_bytes();                │
│     route_req.post_data = Some(body_bytes);                          │
│                                                                       │
└──────────────────────────────────────────────────────────────────────┘
```

**Alternative: CDP-based interception (eliminates proxy entirely):**

Instead of routing traffic through a local HTTP proxy (which requires browser proxy configuration), use CDP's `Fetch.enable` + `Fetch.requestPaused` for Chromium. This is what Playwright does — it intercepts at the protocol level, not the network level. Benefits:

- Works with HTTPS without a MitM CA
- No proxy configuration on browser launch
- Lower latency (no TCP hop through proxy)
- WebSocket interception for free

---

### 12.3 The Chromium Stub Surface (Silent Failures)

**Current state:** 15 methods in `chromium.rs` are no-ops that return `Ok(())`:

```
┌──────────────────────────────────────────────────────────────────────┐
│           Chromium PageEngine: Stub Audit                             │
├──────────────────────┬─────────────┬─────────────────────────────────┤
│ Method               │ Lines       │ CDP Command Needed               │
├──────────────────────┼─────────────┼─────────────────────────────────┤
│ set_viewport_size    │ 648         │ Emulation.setDeviceMetrics...    │
│ viewport_size        │ 649         │ read from cached state           │
│ reload               │ 650         │ Page.reload                      │
│ go_back              │ 651         │ Page.navigateToHistoryEntry(-1)  │
│ go_forward           │ 652         │ Page.navigateToHistoryEntry(+1)  │
│ wait_for_request     │ 653         │ Network.requestWillBeSent        │
│ wait_for_response    │ 654         │ Network.responseReceived         │
│ evaluate_handle      │ 676         │ Runtime.evaluate (returnByValue) │
│ add_script_tag       │ 677         │ Page.addScriptTag               │
│ add_style_tag        │ 678         │ Page.addStyleTag                 │
│ expose_function      │ 679         │ Runtime.addBinding               │
│ set_input_files      │ 642-645     │ DOM.setFileInputFiles            │
├──────────────────────┼─────────────┼─────────────────────────────────┤
│ TOTAL STUBS          │ 12          │ All are 1-10 line CDP calls      │
└──────────────────────┴─────────────┴─────────────────────────────────┘
```

**Contrast with Firefox/WebKit engines:** Interestingly, Firefox and WebKit engines (both using WebDriver) already implement `set_viewport_size`, `viewport_size`, `reload`, `go_back`, and `go_forward` properly via WebDriver commands. Only the Chromium engine (the primary engine!) has these as stubs.

**What's needed — concrete implementation example:**

```rust
// set_viewport_size — currently: Ok(())
// Proposed implementation (5 lines):
async fn set_viewport_size(&self, width: u32, height: u32) -> Result<(), TurbosheetError> {
    let mut page_guard = self.page.lock().await;
    if let Some(page) = page_guard.as_mut() {
        let params = chromiumoxide::cdp::browser_protocol::emulation::SetDeviceMetricsOverrideParams::builder()
            .width(width)
            .height(height)
            .device_scale_factor(1.0)
            .mobile(false)
            .build();
        page.execute(params).await.map_err(|e| {
            TurbosheetError::Other(format!("Failed to set viewport: {}", e))
        })?;
    }
    Ok(())
}

// reload — currently: Ok(())
// Proposed implementation (3 lines):
async fn reload(&self) -> Result<(), TurbosheetError> {
    let mut page_guard = self.page.lock().await;
    if let Some(page) = page_guard.as_mut() {
        let params = chromiumoxide::cdp::browser_protocol::page::ReloadParams::default();
        page.execute(params).await.map_err(|e| {
            TurbosheetError::Other(format!("Failed to reload: {}", e))
        })?;
        page.wait_for_navigation().await.ok();
    }
    Ok(())
}
```

**Effort estimate:** Each of these 12 stubs is a 3-10 line CDP call implementation. The entire stub surface can be eliminated in ~2-3 days of focused work.

---

### 12.4 Additional Deep-Dive: Synthetic Events vs Real Input

**Current state:** All mouse and keyboard interactions in `chromium.rs` use synthetic JavaScript events:

```rust
// chromium.rs:355-368 — dblclick uses synthetic dispatchEvent
let js = format!("(function() {{
    var el = document.querySelector('{}');
    var evt = new MouseEvent('dblclick', {{ bubbles: true, ... }});
    el.dispatchEvent(evt);
    return true;
}})()", selector);
```

This pattern is used for: `dblclick`, `right_click`, `hover`, `check`, `uncheck`, `press`, `press_sequentially`, `drag_and_drop`.

**Why synthetic events are unreliable:**

```
┌──────────────────────────────────────────────────────────────────────┐
│       Synthetic Events vs CDP Input.dispatch* Commands                │
├─────────────────────────────┬──────────────────┬─────────────────────┤
│ Behavior                    │ Synthetic        │ CDP Input.*         │
├─────────────────────────────┼──────────────────┼─────────────────────┤
│ Triggers CSS :active        │ ❌ No            │ ✅ Yes              │
│ Triggers CSS :hover         │ ❌ No            │ ✅ Yes              │
│ Fires beforeinput on        │ ❌ No            │ ✅ Yes              │
│   contenteditable           │                  │                     │
│ Trusted isTrusted flag      │ ❌ false          │ ✅ true             │
│ Works with shadow DOM       │ ⚠️ Partial        │ ✅ Yes              │
│ Triggers native autocomplete│ ❌ No            │ ✅ Yes              │
│ Works across iframes        │ ❌ No            │ ✅ Yes              │
│ Detectable by page JS       │ ✅ Yes (untrusted)│ ❌ No (trusted)     │
└─────────────────────────────┴──────────────────┴─────────────────────┘
```

**Impact:** Pages that check `event.isTrusted` (common in payment forms, CAPTCHAs, and anti-bot systems) will reject TurboSheet's synthetic events. Playwright and Puppeteer use CDP `Input.dispatchMouseEvent` and `Input.dispatchKeyEvent` to produce trusted events.

---

### 12.5 Deep-Dive: Selector Engine Limitation

**Current state:** All three engines use only CSS selectors via `document.querySelector()`.

TurboSheet's selector resolution flow:

```
User Code:  page.locator('button.submit')
               │
               ▼
JsLocator { selector: "button.submit", page_id: "..." }
               │
               ▼
PageEngine::click(selector)  ──►  format!("document.querySelector('{}')", selector)
               │
               ▼
CDP Runtime.evaluate(js_string)
```

**What Playwright supports (10+ selector engines):**

```
┌──────────────────────────────────────────────────────────────────────┐
│              Selector Engine Comparison                                │
├───────────────────────┬──────────────┬────────────────────────────────┤
│ Selector Type         │ TurboSheet   │ Playwright                     │
├───────────────────────┼──────────────┼────────────────────────────────┤
│ CSS: div.class        │ ✅           │ ✅                             │
│ text=Submit           │ ❌           │ ✅ (text content matching)     │
│ text="exact match"    │ ❌           │ ✅ (exact text matching)       │
│ role=button[name=OK]  │ ❌           │ ✅ (ARIA role matching)        │
│ data-testid=submit    │ ❌           │ ✅ (getByTestId shorthand)     │
│ xpath=//button        │ ❌           │ ✅ (XPath evaluation)          │
│ has=.child            │ ❌           │ ✅ (structural filtering)      │
│ hasText=Hello         │ ❌           │ ✅ (text content filter)       │
│ nth=2                 │ ❌           │ ✅ (index-based selection)      │
│ visible=true          │ ❌           │ ✅ (visibility filter)         │
│ internal:chain        │ ❌           │ ✅ (locator chaining)          │
│ id=my-id              │ ❌           │ ✅ (id shorthand)              │
└───────────────────────┴──────────────┴────────────────────────────────┘
```

**Proposed: Selector Engine Registry in injected.js**

The injected.js core module should include a pluggable selector engine:

```
User Code:  page.locator('text=Submit')
               │
               ▼
SelectorParser::parse("text=Submit")
  → SelectorType::Text("Submit")
               │
               ▼
core.js::__ts.query({ type: "text", value: "Submit" })
  → Uses TreeWalker to find text nodes matching "Submit"
  → Returns element handle
```

This maps perfectly to the injected.js architecture: the selector engines run **inside the page** as pre-injected functions, eliminating the need for multiple CDP round-trips per selector resolution.

---

## 13. Complete Feature Parity Matrix

```
┌──────────────────────────────────────────────────────────────────────────────┐
│                    FEATURE COMPLETENESS AUDIT                                │
├──────────────────────────┬──────────┬─────────┬───────────┬─────────────────┤
│ Category                 │Playwright│ Cypress │ Puppeteer │   TurboSheet    │
├──────────────────────────┼──────────┼─────────┼───────────┼─────────────────┤
│ Browser Launch           │   ✅     │   ✅    │    ✅     │   ✅            │
│ Page Navigation          │   ✅     │   ✅    │    ✅     │   ✅            │
│ CSS Selectors            │   ✅     │   ✅    │    ✅     │   ✅            │
│ Text/Role Selectors      │   ✅     │   ✅    │    ❌     │   ❌            │
│ Click/Fill/Type          │   ✅     │   ✅    │    ✅     │   ✅ (synthetic)│
│ Real Input Events        │   ✅     │   ❌    │    ✅     │   ❌            │
│ Auto-Wait                │   ✅     │   ✅    │    ❌     │   ⚠️ (basic)    │
│ Assertions (30+ types)   │   ✅     │   ✅    │    ❌     │   ⚠️ (12 types) │
│ Network Interception     │   ✅     │   ✅    │    ✅     │   ❌ (broken)   │
│ Dialog Handling          │   ✅     │   ✅    │    ✅     │   ❌            │
│ File Download            │   ✅     │   ✅    │    ✅     │   ❌            │
│ File Upload              │   ✅     │   ✅    │    ✅     │   ❌ (stub)     │
│ Frame/IFrame             │   ✅     │   ✅    │    ✅     │   ❌            │
│ Multi-Page/Popups        │   ✅     │   ⚠️    │    ✅     │   ❌            │
│ Console Capture          │   ✅     │   ✅    │    ✅     │   ❌            │
│ Geolocation              │   ✅     │   ⚠️    │    ✅     │   ❌ (stub)     │
│ Permissions              │   ✅     │   ❌    │    ✅     │   ❌ (stub)     │
│ Media Emulation          │   ✅     │   ❌    │    ✅     │   ❌ (stub)     │
│ Mobile Touch Events      │   ✅     │   ❌    │    ✅     │   ❌ (stub)     │
│ Viewport Control         │   ✅     │   ✅    │    ✅     │   ❌ (stub)     │
│ Cookie Management        │   ✅     │   ✅    │    ✅     │   ❌ (stub)     │
│ Storage State            │   ✅     │   ⚠️    │    ❌     │   ❌            │
│ CDP Event System         │   ✅     │   ❌    │    ✅     │   ❌            │
│ Config File              │   ✅     │   ✅    │    ❌     │   ❌            │
│ CLI Tool                 │   ✅     │   ✅    │    ❌     │   ❌            │
│ Code Generation          │   ✅     │   ✅    │    ❌     │   ❌            │
│ Screenshot on Failure    │   ✅     │   ✅    │    ❌     │   ❌ (TODO)     │
│ Video Recording          │   ✅     │   ✅    │    ❌     │   ❌            │
│ HAR Recording            │   ✅     │   ❌    │    ❌     │   ❌            │
│ Trace Viewer             │   ✅     │   ❌    │    ❌     │   ⚠️ (basic)    │
│ Visual Regression        │   ✅     │   ⚠️    │    ❌     │   ⚠️ (basic)    │
│ Component Testing        │   ✅     │   ✅    │    ❌     │   ✅            │
│ Migration Tools          │   ❌     │   ❌    │    ❌     │   ✅ (unique!)  │
│ Multi-Browser            │   ✅     │   ✅    │    ❌     │   ⚠️ (stubs)    │
│ Parallel Execution       │   ✅     │   ✅    │    ❌     │   ✅            │
│ Swarm/Grid               │   ❌     │   ✅    │    ❌     │   ✅ (unique!)  │
│ Plugin System            │   ❌     │   ✅    │    ❌     │   ✅            │
│ Accessibility Testing    │   ✅     │   ⚠️    │    ❌     │   ❌ (stub)     │
│ Auto-Install Browser     │   ✅     │   ✅    │    ✅     │   ❌ (partial)  │
│ Device Descriptors       │   ✅     │   ⚠️    │    ✅     │   ✅ (40+!)     │
│ Reporters (HTML/JSON/etc)│   ✅     │   ✅    │    ❌     │   ✅            │
│ Flaky Test Detection     │   ✅     │   ✅    │    ❌     │   ❌            │
│ TypeScript Support       │   ✅     │   ✅    │    ✅     │   ⚠️ (partial)  │
│ not Assertion Negation   │   ✅     │   ✅    │    ❌     │   ❌            │
│ Soft Assertions          │   ✅     │   ❌    │    ❌     │   ❌            │
├──────────────────────────┼──────────┼─────────┼───────────┼─────────────────┤
│ Score (✅ count)         │  39/43   │  29/43  │   21/43   │   12/43         │
└──────────────────────────┴──────────┴─────────┴───────────┴─────────────────┘
```

---

## 14. Full Improvement Roadmap (28 Items, 4 Tiers)

### Tier 1: "Stop Being Broken" (Weeks 1-4)

> These items fix features that are currently exposed in the public API but silently do nothing.
> Every fix here is a trust issue — users calling these APIs get `Ok(())` with zero effect.

#### 14.1 — Implement Chromium Stubs via CDP Commands

**Files:** `src/engine/chromium.rs`
**Effort:** 2-3 days

| Method              | CDP Command                                            | Lines of Rust |
| ------------------- | ------------------------------------------------------ | ------------- |
| `set_viewport_size` | `Emulation.setDeviceMetricsOverrideParams`             | ~8            |
| `viewport_size`     | Read from cached Emulation state                       | ~5            |
| `reload`            | `Page.reload` + `wait_for_navigation`                  | ~6            |
| `go_back`           | `Page.getNavigationHistory` + `navigateToHistoryEntry` | ~10           |
| `go_forward`        | Same as go_back with index+1                           | ~10           |
| `add_script_tag`    | `Page.addScriptTag`                                    | ~8            |
| `add_style_tag`     | `Page.addStyleTag` (CSS injection)                     | ~8            |
| `set_input_files`   | `DOM.querySelector` → `DOM.setFileInputFiles`          | ~12           |

#### 14.2 — Build CDP Event Subscription System

**Files:** `src/engine/chromium.rs`, new `src/engine/events.rs`
**Effort:** 5-7 days

Create the `EventDispatcher` as described in Section 12.1. This is the **prerequisite** for:

- Dialog handling (14.7)
- Console capture (14.8)
- Network interception via CDP (14.3)
- URL tracking without polling
- The injected.js binding bridge (Section 2)

#### 14.3 — Fix Network Proxy OR Switch to CDP Fetch

**Files:** `src/network/proxy.rs` or new `src/network/cdp_intercept.rs`
**Effort:** 5-7 days

**Option A:** Fix the proxy (implement HTTPS tunneling, `Continue` passthrough, POST body capture).
**Option B (recommended):** Implement CDP `Fetch.enable` + `Fetch.requestPaused` for Chromium (eliminates proxy entirely, works with HTTPS, lower latency). Keep the proxy as a fallback for Firefox/WebKit where CDP Fetch isn't available.

#### 14.4 — Replace Synthetic Events with CDP Input Dispatch

**Files:** `src/engine/chromium.rs`
**Effort:** 3-4 days

Replace `dispatchEvent(new MouseEvent(...))` with:

- `Input.dispatchMouseEvent` for click, dblclick, right_click, hover
- `Input.dispatchKeyEvent` for press, press_sequentially
- `Input.dispatchTouchEvent` for mobile touch gestures

#### 14.5 — Export Missing Functions from index.js

**Files:** `index.js`, `index.d.ts`
**Effort:** 1 day

Currently missing exports: `run_tests`, `expect`, `test`, `describe`, `beforeAll`, `afterAll`, `beforeEach`, `afterEach`.

#### 14.6 — Fix page.on() / page.off() Event Handlers

**Files:** `src/page.rs:420-427`
**Effort:** 2-3 days (requires 14.2)

Wire `page.on('dialog')`, `page.on('request')`, `page.on('console')`, `page.on('response')` to the CDP Event Dispatcher. Currently all no-ops.

---

### Tier 2: "Be Usable" (Weeks 5-8)

> These items add features that users expect to exist in any E2E testing tool.

#### 14.7 — Dialog Handling (alert/confirm/prompt)

**Files:** `src/engine/chromium.rs`, `src/page.rs`
**Effort:** 2-3 days (requires 14.2)
**CDP:** `Page.javascriptDialogOpening` event → `Page.handleJavaScriptDialog` command

Needed: Auto-dismiss for unhandled dialogs, `dialog.accept(text?)`, `dialog.dismiss()`, `dialog.message()`, `dialog.type()`.

#### 14.8 — Console Message Capture

**Files:** `src/engine/chromium.rs`, `src/page.rs`
**Effort:** 1-2 days (requires 14.2)
**CDP:** `Runtime.consoleAPICalled` event

Needed: `page.on('console', msg => { msg.type(), msg.text(), msg.args() })`.

#### 14.9 — Frame/IFrame Support

**Files:** `src/engine/chromium.rs`, new `src/frame.rs`, `src/locator.rs`
**Effort:** 5-7 days
**CDP:** `Page.getFrameTree`, `Page.frameAttached`, `Page.frameNavigated`, `Page.frameDetached`

Needed: `page.frame(name)`, `page.frames()`, `page.frameLocator(selector)`, per-frame execution context tracking. Critical for the injected.js multi-frame injection described in Section 2.

#### 14.10 — Cookie and Storage Management

**Files:** `src/context.rs`, `src/engine/chromium.rs`
**Effort:** 2-3 days
**CDP:** `Network.setCookies`, `Network.getCookies`, `Network.deleteCookies`

Needed: `context.addCookies([...])`, `context.cookies(urls?)`, `context.clearCookies()` (real impl), `context.storageState()` for auth serialization.

#### 14.11 — Screenshot on Failure

**Files:** `src/test_runner/executor.rs:215-222`
**Effort:** 1-2 days

The TODO already exists in code. Implementation: after detecting `TestStatus::Failed`, capture screenshot via the page's CDP session before cleanup.

#### 14.12 — Config File Support (turbosheet.config.ts)

**Files:** new `src/config/mod.rs`, `src/test_runner/executor.rs`
**Effort:** 3-5 days

Needed: Config file discovery (`turbosheet.config.ts` / `.js` / `.mjs`), project configuration (multiple browser configs), `webServer` auto-start, `baseURL`, global timeout, global `use` options.

---

### Tier 3: "Compete with Playwright" (Weeks 9-16)

> These items bring TurboSheet to feature parity with Playwright on core capabilities.

#### 14.13 — Text/Role/XPath Selector Engines

**Files:** new `src/selectors/mod.rs`, `js/src/selectors/`, `src/locator.rs`
**Effort:** 5-7 days

Integrate with injected.js architecture. Implement:

- `text=` — TreeWalker-based text content matching
- `role=` — ARIA role matching via `element.getAttribute('role')` + implicit roles
- `xpath=` — `document.evaluate()` XPath evaluation
- `data-testid=` — shorthand for `[data-testid="..."]`

#### 14.14 — Multi-Page/Popup Support

**Files:** `src/engine/chromium.rs`, `src/context.rs`
**Effort:** 3-5 days (requires 14.2)
**CDP:** `Target.targetCreated`, `Target.targetInfoChanged`

Needed: `context.waitForEvent('page')`, popup detection, `page.opener()`.

#### 14.15 — `not` Negation + Missing Assertion Matchers

**Files:** `src/assertions/matchers.rs`
**Effort:** 3-5 days

Add `not` modifier to `JsExpect` (invert the polling condition). Add matchers:

- `toBeChecked()` / `toBeUnchecked()`
- `toBeFocused()`
- `toBeEditable()`
- `toBeEmpty()`
- `toHaveCount(n)`
- `toHaveCSS(property, value)`
- `toHaveClass(className)`
- `toHaveId(id)`
- `toHaveRole(role)`

#### 14.16 — Download Handling

**Files:** `src/engine/chromium.rs`, `src/page.rs`
**Effort:** 2-3 days (requires 14.2)
**CDP:** `Browser.downloadWillBegin`, `Browser.downloadProgress`

Needed: `download.path()`, `download.saveAs(path)`, `download.cancel()`, `page.waitForEvent('download')`.

#### 14.17 — Video Recording

**Files:** new `src/recording/mod.rs`, `src/engine/chromium.rs`
**Effort:** 5-7 days
**CDP:** `Page.startScreencast` → collect frames → encode to WebM/MP4

Needed: `context.newPage({ recordVideo: { dir: './videos' } })`, frame-by-frame capture, video finalization on page close.

#### 14.18 — CLI Tool

**Files:** new `cli/` crate or `src/cli/mod.rs`
**Effort:** 5-7 days

Needed: `npx tsheet test`, `npx tsheet show-report`, `npx tsheet install`, `npx tsheet codegen`. Use `clap` for argument parsing.

#### 14.19 — Chromium Auto-Install

**Files:** `src/binary_manager.rs`
**Effort:** 3-5 days

Replace the stub at `binary_manager.rs:69-73`. Download Chrome for Testing (CfT) via the JSON API at `https://googlechromelabs.github.io/chrome-for-testing/known-good-versions.json`.

#### 14.20 — Geolocation/Permissions/Media Emulation

**Files:** `src/engine/chromium.rs`, `src/context.rs`, `src/page.rs`
**Effort:** 2-3 days
**CDP:** `Emulation.setGeolocationOverride`, `Browser.grantPermissions`, `Emulation.setEmulatedMedia`

These are all 5-10 line CDP command implementations — the stubs in `context.rs:463-470` and `page.rs:207-228` just need the CDP calls wired in.

---

### Tier 4: "Surpass Playwright" (Weeks 17+)

> These items create competitive advantages no other tool offers.

#### 14.21 — Code Generation (Record & Replay)

**Files:** new `src/codegen/mod.rs`
**Effort:** 10-15 days

Use CDP event stream + injected.js to record user interactions and generate test code. Differentiation: TurboSheet can generate code in multiple test framework syntaxes (TurboSheet, Playwright, Cypress) using the migration engine in reverse.

#### 14.22 — Soft Assertions

**Files:** `src/assertions/matchers.rs`, `src/assertions/engine.rs`
**Effort:** 2-3 days

`expect.soft(locator).toBeVisible()` — collect failures without stopping the test, report all at the end.

#### 14.23 — HAR Recording

**Files:** new `src/network/har.rs`
**Effort:** 3-5 days (requires 14.2 + 14.3)

Record HTTP Archive files from CDP network events. Needed: `context.newPage({ recordHar: { path: 'trace.har' } })`.

#### 14.24 — Flaky Test Detection + Per-Test Retry

**Files:** `src/test_runner/executor.rs`, `src/test_runner/mod.rs`
**Effort:** 3-5 days

Needed: `test.retries(3)` annotation, per-test retry (currently only file-level at `executor.rs:199`), flaky test marking in reports, first-failure vs retry screenshot comparison.

#### 14.25 — Storage State Serialization

**Files:** `src/context.rs`
**Effort:** 2-3 days (requires 14.10)

`context.storageState({ path: 'auth.json' })` → serialize cookies + localStorage + sessionStorage. `context.newContext({ storageState: 'auth.json' })` → restore.

#### 14.26 — ESM Support

**Files:** `index.mjs` (new), `package.json`
**Effort:** 1-2 days

Add ESM entry point alongside CJS. Update `package.json` `exports` field.

#### 14.27 — TypeScript Type Generation

**Files:** `index.d.ts`, build pipeline
**Effort:** 2-3 days

Generate proper TypeScript types from NAPI bindings. Currently, the `Component` and `ComponentLocator` classes in `index.js` don't have corresponding type definitions.

#### 14.28 — WebSocket Interception

**Files:** `src/network/proxy.rs`, `src/page.rs:76-92`
**Effort:** 3-5 days
**CDP:** `Network.webSocketCreated`, `Network.webSocketFrameReceived`, `Network.webSocketFrameSent`

Currently `route_web_socket`, `unroute_web_socket`, `unroute_all_web_sockets` in `page.rs:76-92` are all no-ops.

---

## 15. Architectural Recommendations

### 15.1 Replace Global DashMap with Hierarchical Ownership

**Current:** Three global `DashMap` registries (`BROWSERS`, `CONTEXTS`, `PAGES`) with no parent-child relationship.

```
Current:
  BROWSERS: { "b1" => ChromiumEngine, "b2" => FirefoxEngine }
  CONTEXTS: { "c1" => Context, "c2" => Context }     ← No link to browser
  PAGES:    { "p1" => Page, "p2" => Page }            ← No link to context
```

**Proposed:** Hierarchical ownership with parent backlinks:

```
Proposed:
  Browser {
    id: "b1",
    engine: Arc<dyn BrowserEngine>,
    contexts: RwLock<Vec<Arc<Context>>>,    ← Owned by browser
  }
  Context {
    id: "c1",
    browser: Weak<Browser>,                 ← Backlink
    pages: RwLock<Vec<Arc<Page>>>,          ← Owned by context
  }
  Page {
    id: "p1",
    context: Weak<Context>,                 ← Backlink
    engine: Arc<dyn PageEngine>,
  }
```

**Benefits:**

- Closing a context automatically closes all its pages
- Closing a browser automatically closes all its contexts
- `context.pages()` returns real pages, not empty `vec![]`
- Memory leak prevention — orphaned pages are cleaned up via `Drop`

### 15.2 Use RwLock Instead of Mutex for Read-Heavy Operations

**Current:** `chromium.rs:151` uses `Mutex<Option<Page>>` for all operations, serializing reads and writes.

**Proposed:** Use `RwLock` for read-only operations (`evaluate`, `content`, `title`, `is_visible`, `text_content`) and `Mutex` only for state-changing operations (`click`, `fill`, `goto`, `close`). This enables concurrent read operations on the same page.

### 15.3 Enrich Error Types with Context

**Current:** `error.rs` has 8 error variants with only string context. No selector info, no retry info, no expected-vs-actual in timeout errors.

**Proposed:**

```rust
#[derive(Debug, thiserror::Error)]
pub enum TurbosheetError {
    #[error("Element not found: selector={selector}")]
    ElementNotFound {
        selector: String,
        page_url: String,
        timeout_ms: u64,
        frame_id: Option<String>,
    },

    #[error("Assertion timeout after {timeout_ms}ms ({poll_count} polls)")]
    AssertionTimeout {
        timeout_ms: u64,
        poll_count: u32,
        selector: Option<String>,
        expected: String,
        actual: String,
        last_error: String,
    },

    // ... etc
}
```

### 15.4 Selector Escaping Security Fix

**Current:** All engines use ad-hoc `selector.replace("'", "\\'")` — this is an incomplete escaping strategy that can be bypassed with selectors containing `\`, newlines, or template literal backticks.

**Proposed:** Use JSON serialization for all selector values passed into `evaluate()`:

```rust
// Instead of:
let js = format!("document.querySelector('{}')", selector.replace("'", "\\\\'"));

// Use:
let selector_json = serde_json::to_string(&selector).unwrap();
let js = format!("document.querySelector(JSON.parse({}))", selector_json);
```

This eliminates all escaping edge cases.

---

## 16. TurboSheet's Unique Competitive Moats

Despite the gaps, TurboSheet has real advantages that should be preserved and amplified:

| Moat                       | Description                                                                       | How to Amplify                                             |
| -------------------------- | --------------------------------------------------------------------------------- | ---------------------------------------------------------- |
| **Rust-Native Core**       | 10-100x faster than JS for parsing, comparison, binary operations                 | Move more logic to Rust (selector parsing, HAR generation) |
| **Swarm Grid**             | Built-in distributed execution with autoscaler + health monitoring (`src/swarm/`) | Add Kubernetes operator, cloud-native worker pools         |
| **40+ Device Descriptors** | More devices than Playwright's registry (`src/context.rs:37-423`)                 | Auto-update from real device databases                     |
| **Migration Tools**        | Playwright/Cypress/Puppeteer → TurboSheet AST conversion (`src/migrate/`)         | Publish as standalone CLI tool for adoption                |
| **SIMD Visual Comparison** | Hardware-accelerated image diff (`src/visual/compare.rs`)                         | Add perceptual diff, anti-aliasing tolerance               |
| **Component Testing**      | React/Vue/Svelte with real browser (`src/component/`)                             | Add framework-specific dev server integration              |
| **6 Report Formats**       | dot, line, list, JSON, GitHub Actions, HTML                                       | Add Allure, JUnit XML, TestRail integration                |
| **Edge WASM Runtime**      | Serverless test execution (`src/edge/`)                                           | Deploy to Cloudflare Workers, AWS Lambda                   |
| **CLS Score API**          | Native Core Web Vitals measurement (`src/page.rs:276-307`)                        | Add LCP, FID, INP, TTFB measurement                        |
| **Plugin System**          | Both JS and Rust plugin support (`src/plugin/`)                                   | Publish plugin marketplace                                 |

---

## 17. Roadmap Timeline Summary

```
┌─────────────────────────────────────────────────────────────────────┐
│                  TurboSheet Improvement Timeline                     │
├──────────────────┬──────────────────────────────────────────────────┤
│                  │                                                   │
│  Weeks 1-4       │  ████████ Tier 1: Fix Broken Features             │
│  (Critical)      │  • CDP event system          (5-7 days)          │
│                  │  • Chromium stub fixes        (2-3 days)          │
│                  │  • Network proxy / CDP Fetch  (5-7 days)          │
│                  │  • Real input dispatch        (3-4 days)          │
│                  │  • index.js exports           (1 day)             │
│                  │  • page.on() handlers         (2-3 days)          │
│                  │                                                   │
│  Weeks 5-8       │  ████████ Tier 2: Core E2E Features               │
│  (High)          │  • Dialog handling            (2-3 days)          │
│                  │  • Console capture            (1-2 days)          │
│                  │  • Frame/IFrame support       (5-7 days)          │
│                  │  • Cookie/storage mgmt        (2-3 days)          │
│                  │  • Screenshot on failure      (1-2 days)          │
│                  │  • Config file support        (3-5 days)          │
│                  │                                                   │
│  Weeks 9-16      │  ████████ Tier 3: Playwright Parity               │
│  (Medium)        │  • Selector engines           (5-7 days)          │
│                  │  • Multi-page/popup           (3-5 days)          │
│                  │  • Assertion matchers + not   (3-5 days)          │
│                  │  • Download handling           (2-3 days)          │
│                  │  • Video recording             (5-7 days)          │
│                  │  • CLI tool                    (5-7 days)          │
│                  │  • Auto-install browser        (3-5 days)          │
│                  │  • Emulation fixes             (2-3 days)          │
│                  │                                                   │
│  Weeks 17+       │  ████████ Tier 4: Surpass Playwright              │
│  (Differentiation)│ • Code generation            (10-15 days)        │
│                  │  • Soft assertions             (2-3 days)          │
│                  │  • HAR recording               (3-5 days)          │
│                  │  • Flaky test detection        (3-5 days)          │
│                  │  • Storage state serialization (2-3 days)          │
│                  │  • ESM + TypeScript types      (3-5 days)          │
│                  │  • WebSocket interception      (3-5 days)          │
│                  │                                                   │
│  Score Projection:                                                   │
│  After Tier 1:  12/43 → 18/43                                        │
│  After Tier 2:  18/43 → 26/43 (surpasses Puppeteer)                  │
│  After Tier 3:  26/43 → 36/43 (approaching Playwright)               │
│  After Tier 4:  36/43 → 43/43 (surpasses Playwright)                 │
│                                                                      │
└──────────────────────────────────────────────────────────────────────┘
```

---

## 18. Cross-Reference: Injected.js Enables Multiple Improvements

The injected.js architecture from Sections 1-11 is not isolated — it **accelerates** many Tier 2-3 items:

| Tier Item                      | How Injected.js Helps                                                      |
| ------------------------------ | -------------------------------------------------------------------------- |
| 14.7 — Dialog Handling         | Injected.js can detect `beforeunload` dialogs before CDP event             |
| 14.8 — Console Capture         | Injected.js can hook `console.*` before `Runtime.consoleAPICalled` fires   |
| 14.9 — Frame/IFrame            | Core module auto-injects per frame via `addScriptToEvaluateOnNewDocument`  |
| 14.13 — Selector Engines       | Text/role/XPath engines run inside `actions.js` — zero extra CDP calls     |
| 14.15 — Assertion Matchers     | `toHaveCSS()`, `toBeFocused()` etc. run as injected functions — 1 CDP call |
| 14.17 — Video Recording        | Injected.js can annotate frames with action metadata for trace correlation |
| 14.21 — Code Generation        | Injected.js captures all user interactions as structured events            |
| 14.28 — WebSocket Interception | Injected.js can hook `WebSocket` constructor in main world                 |

**Recommendation:** Implement Tier 1 items 14.1 (stubs) and 14.2 (CDP events) first, then the injected.js architecture from Sections 1-11, then proceed to Tier 2-4 items which will be significantly easier with the injection foundation in place.
