## Why

TurboSheet has a solid Rust-native browser automation core with real CDP implementation for Chromium, but the test execution layer returns "Simulated Test" results and Firefox/WebKit engines are empty stubs. To become a **complete, reliable end-to-end testing tool** that competes with Playwright, Cypress, and Puppeteer, TurboSheet must implement real test execution, complete cross-browser support, and fill critical capability gaps.

## What Changes

1. **Real Test Execution** - Replace mock executor with actual JS/TS test file parsing and isolated-v8 runtime execution
2. **Complete Firefox Engine** - Implement real Firefox automation via geckodriver + WebDriver BiDi protocol
3. **Complete WebKit Engine** - Implement real WebKit automation via webkit2gtk
4. **Complete Missing PageEngine Methods** - Implement bounding_box, drag_and_drop, press, set_input_files, wait_for_selector with real CDP calls
5. **Smart Auto-Wait System** - Replace basic polling with intelligent wait conditions (visibility, stability, actionability)
6. **Visual Regression Engine** - Implement semantic diff with AI-powered ignore regions
7. **TurboTrace Viewer** - Time-travel debugging with Rust-compressed trace files (zstd + ciborium)
8. **Migration Tooling** - Automated Playwright/Cypress test conversion to TurboSheet format
9. **Swarm Scale Architecture** - True parallel execution with context pooling (10,000+ concurrent contexts)
10. **Component Testing** - Mount React/Vue/Svelte components in lightweight Rust headless context
11. **CI/CD Infrastructure** - Official Docker images, GitHub Actions template, multi-arch builds
12. **VS Code Extension** - Test explorer, run/debug buttons, trace viewer integration

## Capabilities

### New Capabilities

- `real-test-execution`: Full JS/TS test parsing and execution via isolated-v8 with fixtures, hooks, and proper lifecycle
- `firefox-automation`: Complete Firefox engine via geckodriver + WebDriver BiDi protocol
- `webkit-automation`: Complete WebKit engine via webkit2gtk
- `auto-wait`: Intelligent wait conditions with exponential backoff, stability detection, and actionability checks
- `complete-locator-api`: All locator methods (bounding_box, drag_and_drop, press, set_input_files) with real implementations
- `visual-semantic-diff`: AI-powered screenshot comparison with layout shift detection and ignore regions
- `turbo-trace-viewer`: Time-travel debugging with zstd-compressed traces and WASM-based viewer
- `migration-tooling`: Automated conversion from Playwright and Cypress test suites
- `swarm-execution`: Tokio-native parallel execution with context pooling for massive concurrency
- `component-testing`: Framework-agnostic component mounting in lightweight headless context
- `ci-infrastructure`: Docker images, GitHub Actions/GitLab CI templates, binary caching
- `vscode-extension`: Full IDE integration with test explorer, inline decorations, trace viewer

### Modified Capabilities

- `assertion-engine`: No requirement changes - already uses real CDP methods via page.is_visible() and page.text_content()
- `page-api`: No requirement changes - already delegates to PageEngine trait correctly
- `locator-api`: No requirement changes - already delegates to page methods

## Impact

**Core Rust Engine:**

- `src/test_runner/executor.rs` - Replace mock with real isolated-v8 execution
- `src/engine/firefox.rs` - Implement from stub to real geckodriver integration
- `src/engine/webkit.rs` - Implement from stub to real webkit2gtk integration
- `src/engine/chromium.rs` - Complete missing methods (bounding_box, drag_and_drop, etc.)
- `src/assertions/engine.rs` - Enhance with smart auto-wait conditions

**New Modules:**

- `src/runtime/` - Already has isolated-v8 wrapper, needs integration
- `src/visual/compare.rs` - Add semantic diff capabilities
- `src/trace/` - Implement TurboTrace compression and viewer
- `src/migrate/` - New migration tooling module
- `src/component/` - New component testing module
- `src/swarm/` - New swarm execution module
- `vscode-extension/` - New VS Code extension package

**Dependencies:**

- `geckodriver` crate - Firefox automation
- `webkit2gtk` sys crate - WebKit automation (Linux)
- `zstd` crate - Trace compression (already optional)
- `ciborium` crate - CBOR serialization for traces (already optional)
- `image` crate with `image` feature - Semantic diff (already optional)

**Binary Targets:**

- `turbosheet` - Main Node.js native module (existing)
- `turbosheet-wasm` - WASM edge runtime (new)
- `tsheet-migrate` - Migration CLI (new)
- `tsheet-trace` - Trace viewer CLI (new)
