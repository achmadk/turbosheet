## Why

TurboSheet is a Rust-powered browser automation framework with a strong foundation — native N-API module, Chromium CDP engine, test runner, visual regression, trace viewer, component testing, and swarm grid. However, multiple critical gaps prevent it from being a production-ready Playwright/Cypress competitor: Firefox/WebKit engines are partial stubs (34 no-ops each), there is zero TypeScript API surface (0-byte `index.d.ts`), migration tools output reference guides instead of transformed code, the test-runner DX specification is empty, WASM edge runtime is untouched, and core swarm features (auto-scaling, health monitoring, multi-tenant isolation) remain incomplete. Without closing these gaps, TurboSheet cannot credibly claim parity with incumbents.

## What Changes

- **Complete Firefox and WebKit engine parity** — eliminate all 34 no-op `Ok(())` returns per engine, implement native WebDriver actions for drag-and-drop, file upload, viewport control, script injection, binding registration, element stability checks, and network event waiting
- **Generate full TypeScript declaration surface** — create comprehensive `index.d.ts` with typed interfaces for all public APIs (JsPage, JsLocator, JsBrowser, JsContext, assertions, test runner config, migration tools), enabling IDE autocomplete and type checking
- **Build real AST-based migration tools** — replace hardcoded API mapping tables with actual JavaScript/TypeScript source transformation using SWC, converting Playwright/Cypress/Puppeteer test files into working TurboSheet tests with structural transforms
- **Complete swarm grid features** — implement auto-scaling (queue-depth-based), worker health monitoring with dead-worker reassignment, and Docker-based multi-tenant isolation
- **Implement WASM edge runtime** — add wasm32-wasi compilation target, edge test orchestrator, tree-shaking for minimal binary size, and edge-to-grid binary protocol
- **Write test-runner DX specification** — fill the empty spec directory with detailed requirements for test filtering, --grep, project references, global setup/teardown, timeouts, sharding, retries, and reporter configuration
- **Add test coverage across all modules** — create tests for CLI commands, migration tools, visual comparison, trace viewer/serializer, swarm grid (unit), component testing, network interception, and the TypeScript runtime layer
- **Complete documentation** — getting-started tutorial, comprehensive API reference, migration guides, CI/CD integration setup templates

## Capabilities

### New Capabilities

- `firefox-webkit-parity`: Complete Firefox and WebKit engine implementations matching Chromium's functionality, eliminating all no-op stubs through native WebDriver protocol commands and injected script fallbacks
- `typescript-declarations`: Full type-safe public API surface via comprehensive `index.d.ts` covering all JsBrowser, JsContext, JsPage, JsLocator, assertion, test runner, and migration APIs
- `ast-migration-tools`: Real SWC-based AST transformation pipeline that converts Playwright/Cypress/Puppeteer test files into valid TurboSheet tests, not just reference mappings
- `swarm-completion`: Auto-scaling, health monitoring with dead-worker reassignment, and Docker-based multi-tenant isolation for the swarm grid
- `wasm-edge-runtime`: WASM-compiled test orchestrator for edge deployment (Cloudflare Workers), with tree-shaken binary and edge-to-grid protocol
- `test-runner-dx`: Detailed test-runner developer experience specification covering filtering, grep, project references, global setup/teardown, timeouts, sharding, retries, reporter configuration
- `test-coverage`: Comprehensive test suite across all modules — CLI, migration, visual, trace, swarm, component, network, runtime
- `documentation-completion`: Getting-started tutorial, full API reference, migration guides, CI/CD setup templates with working workflow files

### Modified Capabilities

_(No existing spec-level behavior changes — all gaps are new capabilities or implementation work)_

## Impact

- **Affected code**: `src/engine/firefox.rs`, `src/engine/webkit.rs` (full rewrite of 30+ methods each), `src/migrate/` (replacement of mapping tables with AST transforms), `src/swarm/autoscaler.rs`, `src/swarm/health.rs`, `src/swarm/tenant.rs` (completion of stubbed features), new `wasm/` directory tree, new `index.d.ts` at root, new `__test__/` files across all modules
- **API surface**: No breaking changes — all new TypeScript declarations describe existing behavior; API signatures unchanged
- **Dependencies**: SWC parser (`@swc/core`) for AST migration; wasm32-wasi compilation toolchain for edge runtime; no new production runtime dependencies
- **Build system**: WASM target added to build matrix (currently 6 targets → 7+); JS bundling for TypeScript declarations; SWC plugin for migration
- **Documentation**: New docs and CI workflow templates shipped with the package
