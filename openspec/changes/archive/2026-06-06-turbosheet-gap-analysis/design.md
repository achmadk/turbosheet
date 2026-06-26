## Context

TurboSheet's architecture separates browser engines behind the `PageEngine` trait (57 methods in `src/engine/mod.rs`). Chromium uses full CDP via `chromiumoxide` (1,223 lines, all methods implemented). Firefox and WebKit use WebDriver HTTP protocol via a shared `WebDriverClient` (449 lines each, ~30 no-op methods). The migration module uses hardcoded HashMap mappings, not AST transforms. TypeScript declarations are nonexistent (0-byte `index.d.ts`). The swarm module has real infrastructure but 3 incomplete features. WASM edge runtime is entirely unimplemented (7 tasks). Test coverage is sparse (6 test files for a project with 50+ Rust source files). Documentation exists in partial form.

Stakeholders: E2E test developers migrating from Playwright/Cypress/Puppeteer, CI/CD pipeline users, VS Code extension users, enterprise teams needing multi-browser and parallel execution.

## Goals / Non-Goals

**Goals:**

- Achieve full Firefox/WebKit engine parity with Chromium — no no-op stubs remaining
- Ship comprehensive TypeScript declarations for the entire public API surface
- Replace migration API mapping tables with real SWC-based AST transformation
- Complete swarm grid: auto-scaling, health monitoring, Docker tenant isolation
- Implement WASM edge runtime compilation target with tree-shaken binary
- Define test-runner DX specification (currently empty directory)
- Add test coverage across all uncovered modules
- Complete shipping documentation

**Non-Goals:**

- No new public API or breaking changes — all work is implementation-complete of already-specified interfaces
- No new browser engine support (no IE11, no Safari iOS, no Appium)
- No cloud service or SaaS layer — swarm remains self-hosted
- No migration from non-JS test frameworks (Ruby, Python, Java)

## Decisions

### D1: Firefox/WebKit Parity — Extend Existing Pattern, Don't Rewrite

**Decision:** Complete every no-op and JS-eval-fallback method in Firefox and WebKit PageEngine using the existing WebDriver protocol + injected action fallback pattern already established (e.g., `invoke_action`, `find_element`, direct WebDriver HTTP calls).

**Rationale:** Firefox and WebKit already have the correct structure — WebDriver session creation, element finding, `invoke_action` for injected JS, and several working methods (click, fill, screenshot, goto). The remaining 34 no-ops each need to be categorized and implemented:

- **WebDriver-native** (set_viewport, reload, go_back, go_forward, cookies, url) — use existing WebDriver HTTP endpoints
- **JS eval** (drag_and_drop, check, uncheck, select, focus, blur, scroll_into_view, right_click, dblclick, hover, is_visible, is_enabled, is_disabled) — implement via invoke_action which already calls the injected actions script
- **Stubbed no-ops** (set_input_files, wait_for_request, wait_for_response, evaluate_handle, expose_function, set_content, inject_core_script, register_binding) — each needs proper WebDriver or eval-based implementation

**Alternatives Considered:**

- BiDi protocol for Firefox: Rejected — geckodriver WebDriver is stable, BiDi would require separate protocol implementation
- Shared cross-browser engine: Rejected — each browser's WebDriver dialect differs enough (element keys, supported endpoints) to justify separate files

### D2: TypeScript Declarations — Hand-Written > Generated

**Decision:** Create hand-written `index.d.ts` covering all public APIs by extracting types from N-API exports in `src/lib.rs`, `src/page.rs`, `src/locator.rs`, `src/browser.rs`, `src/context.rs`, `src/assertions/mod.rs`, `src/migrate/mod.rs`, `src/test_runner/mod.rs`, plus the TypeScript runtime layer in `runtime/`.

**Rationale:** Automatic generation from Rust N-API bindings is possible but produces verbose, non-idiomatic TypeScript. Hand-written declarations give control over documentation, deprecation markers, generic overloads, and union types. The Rust N-API surface is stable and bounded (~40 exported functions).

**Alternatives Considered:**

- napi-rs `--js-mode declaration`: Produces JS-only output, not .d.ts
- napi-rs type generation: Generated types lack JSDoc, overloads, and ergonomic design
- Manual generation: Selected — best developer experience

### D3: AST Migration — SWC Visitor Pattern

**Decision:** Implement migration as SWC visitor plugins that match Playwright/Cypress/Puppeteer API call patterns and rewrite them to TurboSheet equivalents. The existing mapping tables become test fixtures for expected input→output pairs.

**Rationale:** SWC is already a direct dependency (listed in `package.json`). AST-level transformation handles imports, method calls, chaining, and async/await patterns correctly — impossible with string-based or mapping-table approaches.

**Migration flow:**

```
Input file → SWC parse → Visitor matcher → AST rewrite → Codegen → Output file
                              ↕
                    Mapping tables (config-driven)
```

### D4: Swarm Completion — Add to Existing Infrastructure

**Decision:** Complete the three missing swarm features by extending the existing `GridController` and `SwarmOrchestrator`:

- **Auto-scaling**: Add queue-depth watcher in controller that spawns/kills worker processes (Tokio tasks), respecting min/max config bounds
- **Health monitoring**: Add periodic heartbeat check in controller, mark workers dead after N missed heartbeats, reassign their shards to live workers
- **Docker tenant**: Implement `Tenant::spawn_container()` using `bollard` Rust Docker client, mount config volumes, expose ports

### D5: WASM Edge — New Compilation Target

**Decision:** Create `wasm/` directory with a tree-shaken build configuration that excludes browser engine implementations (since edge nodes delegate to grid), keeping only the test orchestrator, scheduler, retry logic, and result aggregation. The binary protocol uses MessagePack over WebSocket.

**Rationale:** Edge deployment is TurboSheet's unique moat — no other E2E testing tool can run test orchestration at the edge. The tree-shaken WASM binary targets <3MB.

### D6: Capability Specs Structure

**Decision:** Each new capability gets its own `specs/<name>/spec.md` file describing detailed requirements. This aligns with the existing OpenSpec workflow.

## Risks / Trade-offs

- **[Risk] Firefox WebDriver endpoint gaps**: The W3C WebDriver spec doesn't support drag-and-drop, hover, or right-click natively — these must use JavaScript injection fallbacks which may behave differently than CDP. → Mitigation: Use injected script actions (already implemented via `invoke_action`) as primary implementation, WebDriver endpoints as fallback.
- **[Risk] SWC version compatibility**: SWC is fast-moving; visitor plugin API may shift. → Mitigation: Pin SWC version in `package.json`, use stable visitor patterns.
- **[Risk] WASM binary size exceeds 3MB target**: Tree-shaking may not remove enough code. → Mitigation: Use `wasm-pack` with `--no-default-features`, feature-gate all engine code behind `chromium`/`firefox`/`webkit` features, create separate `wasm-orchestrator` crate.
- **[Risk] Health monitoring false positives**: Network jitter could cause unnecessary worker reassignment. → Mitigation: Configurable grace period (default 3 missed heartbeats), exponential backoff on reconnection.
