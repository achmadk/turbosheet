## Context

TurboSheet has a structural advantage (Rust-native core, Tokio concurrency, NAPI-rs bridge) but sits at **12/43 feature parity** with Playwright (39/43). Analysis of the 41K LOC codebase reveals:

1. **Zero CDP event subscription** — the handler task consumes events but processes none, breaking 8+ features (dialog handling, console capture, URL tracking, network interception, popup detection, download handling, injected.js binding bridge, frame tracking)

2. **Three global DashMap registries** with no parent-child relationships — `context.pages()` returns empty lists, closing leaks memory, no cascade cleanup

3. **Network proxy has three critical failures** — HTTPS CONNECT drops tunnels, `route.continue()` returns HTTP 501, POST body is never captured

4. **All user interactions use synthetic `dispatchEvent`** — CSS `:active`/`:hover` don't trigger, `event.isTrusted` is false, anti-bot systems reject interactions

5. **Cross-engine JS duplication** — ~1,500 lines of inline JavaScript duplicated across all three `PageEngine` implementations

6. **Ad-hoc selector escaping** — `selector.replace("'", "\\'")` is an incomplete escaping strategy with injection vectors

7. **Missing 30+ improvements with competitive moats** — AI-native features, developer experience, observability, advanced testing paradigms not in any spec

Four OpenSpec changes cover surface-level improvements but miss the foundational layer. This change adds the foundational fixes, completes the injected script engine architecture, and adds 30+ unique differentiators that no other E2E tool offers.

## Goals / Non-Goals

**Goals:**

- Build CDP Event Dispatcher that routes 12+ event types to registered handlers
- Replace global DashMap registries with hierarchical Browser→Context→Page ownership
- Fix all three network proxy failures (CONNECT tunnel, Continue passthrough, POST body)
- Replace synthetic dispatchEvent with CDP Input.dispatch\* commands
- Complete the injected script engine (core + actions + binding bridge) across all 3 engines
- Implement frame/iframe, multi-page/popup, cookie/storage, download, video, HAR, codegen
- Build AI-native differentiators (getAgentSnapshot, self-healing selectors, flake diagnosis)
- Add developer experience (--ui mode, watch mode, typed fixtures, clock mocking)
- Add observability (Core Web Vitals, accessibility, flaky dashboard, test impact analysis)
- Reach 43/43 feature parity score (surpassing Playwright)

**Non-Goals:**

- Rewriting existing engine driver crates (chromiumoxide, geckodriver, webkit2gtk)
- Replacing the Rust core with JavaScript
- Supporting legacy browsers (IE11)
- Building a native mobile runtime (iOS/Android app automation — that's a separate product)

## Decisions

- **CDP Fetch over Proxy for Chromium**: Use CDP `Fetch.enable` + `Fetch.requestPaused` for Chromium network interception. Keep the TCP proxy as fallback for Firefox/WebKit where CDP Fetch isn't available. The proxy will still be fixed for those engines.
  - _Rationale_: CDP Fetch eliminates need for MitM CA, gives native HTTPS body access, has lower latency, and includes WebSocket interception by default.
- **Injected Script First, Engine Parity Second**: Implement the injected script engine before completing Firefox/WebKit engine parity. The shared JS eliminates 3x duplication and makes engine parity faster.
  - _Rationale_: As proven by PRD v3 analysis, injected.js unifies cross-engine JS, reducing Tier 1-4 implementation effort significantly.
- **AI Features as Rust Module, Not API Wrapper**: Build `getAgentSnapshot()`, self-healing selectors, and flake diagnosis as native Rust modules with NAPI bindings, not JavaScript wrappers.
  - _Rationale_: Rust gives 10-100x faster DOM traversal, tree analysis, and similarity computation vs JS. Also keeps the AI features working even without the injected script.
- **UI Mode as Separate Electron App**: The `--ui` mode will be a standalone Electron/React app that communicates with the Rust test runner via a WebSocket/pipe protocol, similar to Playwright's architecture.
  - _Rationale_: Separates the UI process from the test execution process, preventing UI lag from affecting test timing. Enables remote debugging (connect to tests running on a CI server).
- **Clock Mocking via CDP + Injected Script**: Override `Date.now()`, `performance.now()`, `setTimeout`, `setInterval` via injected script interception, combined with CDP `TimeDomain` for browser-level time control.
  - _Rationale_: JS-level mocking is reliable for most cases; CDP-level time control handles edge cases like `requestAnimationFrame` timing.
- **Test Impact Analysis via Coverage + Git Diff**: Parse `--coverage` output (lcov format) to build file→test mapping. Parse `git diff HEAD~1` for changed files. Run only tests that touch changed files.
  - _Rationale_: Simple heuristic that gives 60-80% reduction with minimal infrastructure. Can be refined with dependency graph in v2.

## Architecture

```
┌─────────────────────────────────────────────────────────────────────────┐
│                   TurboSheet Complete Architecture                        │
├─────────────────────────────────────────────────────────────────────────┤
│                                                                          │
│  ┌──────────────────────────────────────────────────────────────────┐   │
│  │                    NEW: Event Dispatcher                           │   │
│  │  ┌─────────────────────────────────────────────────────────────┐  │   │
│  │  │  chromiumoxide handler stream → EventDispatcher              │  │   │
│  │  │    ├── Page.frameNavigated       → url_cache, frame_tracker  │  │   │
│  │  │    ├── Runtime.consoleAPICalled  → page.on('console')       │  │   │
│  │  │    ├── Runtime.bindingCalled     → injected.js bridge       │  │   │
│  │  │    ├── Network.requestWillBeSent → page.on('request')       │  │   │
│  │  │    ├── Page.javascriptDialogOpening → dialog_handler         │  │   │
│  │  │    ├── Target.targetCreated      → popup_detector           │  │   │
│  │  │    ├── Browser.downloadWillBegin → download_handler         │  │   │
│  │  │    └── ... (5 more event types)                              │  │   │
│  │  └─────────────────────────────────────────────────────────────┘  │   │
│  └──────────────────────────────────────────────────────────────────┘   │
│                                                                          │
│  ┌──────────────────────────────────────────────────────────────────┐   │
│  │             NEW: Hierarchical Ownership Model                      │   │
│  │                                                                   │   │
│  │  Browser { id, engine, contexts: Vec<Context> }                   │   │
│  │    └── Context { id, browser: Weak<Browser>, pages: Vec<Page> }  │   │
│  │          └── Page { id, context: Weak<Context>, engine }          │   │
│  │                                                                   │   │
│  │  Benefits: context.pages() returns real pages, Drop cascade       │   │
│  └──────────────────────────────────────────────────────────────────┘   │
│                                                                          │
│  ┌──────────────────────────────────────────────────────────────────┐   │
│  │          COMPLETED: Injected Script Engine                         │   │
│  │                                                                   │   │
│  │  injected-core.ts (≤2KB) → pre-injected at document start        │   │
│  │    └── Binding establishment, MutationObserver, selectors         │   │
│  │                                                                   │   │
│  │  injected-actions.ts (≤15KB) → on-demand, cached                  │   │
│  │    └── Actionability, geometry, auto-wait, drag-drop, AI snapshot │   │
│  │                                                                   │   │
│  │  BindingRegistry (Rust)                                           │   │
│  │    └── DashMap<UUID, oneshot::Sender> + request-response bridge   │   │
│  └──────────────────────────────────────────────────────────────────┘   │
│                                                                          │
│  ┌──────────────────────────────────────────────────────────────────┐   │
│  │             NEW: AI-Native Features Module                         │   │
│  │                                                                   │   │
│  │  src/ai/                                                          │   │
│  │  ├── agent_snapshot.rs  → getAgentSnapshot() (accessibility tree) │   │
│  │  ├── self_heal.rs       → fallback selector engine                │   │
│  │  └── flake_diagnosis.rs → failure classification engine           │   │
│  └──────────────────────────────────────────────────────────────────┘   │
│                                                                          │
│  ┌──────────────────────────────────────────────────────────────────┐   │
│  │           NEW: Developer Experience + Observability                │   │
│  │                                                                   │   │
│  │  packages/ui-mode/     → Electron/React interactive test runner   │   │
│  │  src/clock/            → fake timers (injected + CDP)            │   │
│  │  src/vitals/           → Core Web Vitals via PerformanceObserver   │   │
│  │  src/accessibility/    → axe-core integration                     │   │
│  │  src/impact/           → test impact analysis (coverage + git)    │   │
│  │  src/snapshot/         → toMatchSnapshot with storage             │   │
│  └──────────────────────────────────────────────────────────────────┘   │
│                                                                          │
└─────────────────────────────────────────────────────────────────────────┘
```

## Phasing Strategy

```
Phase 0 (Foundation) — Weeks 1-4
  ├── CDP Event Dispatcher      (build once, enables 8+ features)
  ├── Hierarchical Ownership    (memory safety, context.pages())
  ├── Network Proxy Fix         (CONNECT, Continue, POST body)
  ├── Real Input Dispatch       (trusted events, isTrusted=true)
  └── Selector Escaping Fix     (security)

Phase 1 (Injected Engine) — Weeks 5-8
  ├── injected-core.ts + injection into all 3 engines
  ├── injected-actions.ts + BindingRegistry
  └── Migrate Chromium/Firefox/WebKit engines to use injected scripts

Phase 2 (Core Gaps) — Weeks 9-16
  ├── Frame/IFrame + Multi-Page/Popup
  ├── Cookie/Storage + Download + Video
  ├── HAR Recording + Code Generation
  ├── Auto-Install Browser + File Chooser
  └── Complete test runner CLI (--ui, --watch, fixtures)

Phase 3 (AI Differentiators) — Weeks 17-24
  ├── getAgentSnapshot() + Self-Healing Selectors
  ├── BiDi Stream Pre-Processor
  ├── AI-Powered Flake Diagnosis
  └── Core Web Vitals + Accessibility + Clock Mocking

Phase 4 (Ecosystem) — Weeks 25+
  ├── Test Impact Analysis + Flaky Dashboard
  ├── Mobile Cloud Integration
  ├── Snapshot Testing + API+UI Integration
  └── Plugin Marketplace + Kubernetes Operator
```

## Risks / Trade-offs

- **[Risk] CDP Event Dispatcher adds complexity to chromium.rs** → Mitigation: Extract into dedicated `src/events/` module. The dispatcher is a one-time build that enables 8+ features.
- **[Risk] Injected script breaks on specific websites** → Mitigation: Strict IIFE isolation, runtime name randomization, comprehensive test suite against top 100 websites.
- **[Risk] Self-healing selectors could mask real bugs** → Mitigation: Log all healing events prominently. Fail the test if healing rate exceeds threshold (configurable, default: 3). Report healing count in test summary.
- **[Risk] --ui mode Electron app is heavy** → Mitigation: Make it an optional install (`npx tsheet install-ui`). The CLI works without it. Consider a lighter web-based version for CI.
- **[Risk] Clock mocking may not work with all frameworks** → Mitigation: Start with `page.clock.setFixedTime()` (simplest, most compatible). Add `install()` + `fastForward()` as opt-in advanced features.
- **[Risk] AI features require ML dependencies** → Mitigation: Use heuristic-based approaches first (Levenshtein distance for self-healing, fast failure pattern matching for flake diagnosis). ML can be added as an optional enhancement later.
