## Why

TurboSheet's v1 and v2 PRDs establish a powerful Rust-native browser automation core with unique advantages (swarm threading, zero-copy FFI, BiDi preprocessing, AI-native DOM extraction). However, to surpass Playwright, Cypress, and Puppeteer as the **best complete reliable end-to-end testing tool**, TurboSheet must expand from a "browser automation library" into a **full E2E testing framework** with matching developer experience, debugging, cross-browser support, CI/CD infrastructure, and unique moats that competitors cannot replicate.

Without these capabilities, TurboSheet remains a niche scraping/AI-agent tool. With them, it dominates the entire E2E testing landscape by offering what no competitor can: **Rust-native speed + complete testing framework + WASM edge execution + swarm-scale concurrency**.

## What Changes

- **Cross-Browser Engine**: Add Firefox (via `geckodriver`/BiDi) and WebKit (via `webkit2gtk`/BiDi) support alongside existing Chromium engine
- **Full Test Runner**: Built-in test framework with fixtures (DI), hooks, auto-retrying assertions, test discovery, sharding, retries, and execution control
- **Network Interception Engine**: Rust-native request/response interception, mocking, throttling, and WebSocket interception via in-process Tokio proxy
- **TurboTrace Debugger**: Time-travel debug viewer with Rust-compressed trace files, DOM snapshots, console logs, and network waterfall
- **Visual Regression Engine**: Rust-native pixel comparison with SIMD acceleration, AI-powered semantic diff, and inline diff reports
- **Codegen Test Recorder**: Interactive test recording that generates `.tsheet.ts` spec files from user browser actions
- **VS Code Extension**: Test explorer, run/debug buttons, trace viewer integration, inline pass/fail decorations
- **Interactive Debug Runner**: Cypress-style command log with step-by-step DOM snapshots, built on Rust shared-memory snapshots
- **Component Testing**: Mount framework components (React, Vue, Svelte) in lightweight Rust headless context without full page load
- **CI/CD Infrastructure**: Official Docker images, GitHub Actions/GitLab CI templates, multi-architecture builds
- **Reporter System**: HTML (with embedded trace viewer), JSON, JUnit XML, GitHub Annotations, terminal reporters
- **Plugin System**: Rust-backable plugin architecture with JS fallback for community extensions
- **Mobile & Device Emulation**: Device presets, geolocation, touch, orientation, dark mode, reduced motion via CDP
- **Accessibility Testing**: Rust-native aXe rule engine integration for WCAG compliance scanning
- **Swarm Cloud Grid**: Multi-tenant execution grid leveraging Tokio thread-per-context architecture for 25-40x cost reduction vs Playwright
- **WASM Edge Runtime**: Run automation logic at the edge (Cloudflare Workers, Deno) for geo-distributed testing
- **Migration Tooling**: `tsheet migrate from playwright` and `tsheet migrate from cypress` CLI commands with automated spec conversion

## Capabilities

### New Capabilities

- `cross-browser-engine`: Firefox and WebKit support via BiDi protocol with Rust-native stream preprocessing
- `test-runner`: Complete test framework with fixtures, hooks, auto-retrying assertions, sharding, retries
- `network-interception`: Rust-native request mocking, interception, blocking, and throttling engine
- `trace-viewer`: Time-travel debugging with Rust-compressed trace files and DOM snapshots
- `visual-regression`: Rust SIMD-accelerated screenshot comparison with AI semantic diff
- `codegen`: Interactive test recorder that generates test files from browser actions
- `vscode-extension`: VS Code extension for test management, debugging, and trace viewing
- `debug-runner`: Interactive step-through debug runner with command log and DOM snapshots
- `component-testing`: Framework-agnostic component mounting in lightweight Rust headless context
- `ci-infrastructure`: Docker images, CI templates, and multi-platform build system
- `reporter-system`: Pluggable reporters (HTML, JSON, JUnit, GitHub Annotations)
- `plugin-system`: Extension architecture with Rust-native and JS plugin support
- `mobile-emulation`: Device presets, touch events, geolocation, orientation APIs
- `accessibility-testing`: WCAG compliance scanning with Rust-native aXe engine integration
- `swarm-cloud-grid`: Distributed execution grid using Tokio swarm threading architecture
- `wasm-edge-runtime`: WebAssembly-compiled runtime for edge environment test execution
- `migration-tooling`: Automated migration from Playwright, Cypress, and Puppeteer test suites

### Modified Capabilities

<!-- No existing specs to modify - this is the first spec set for TurboSheet -->

## Impact

- **Core Rust Engine**: Significant expansion of `chromiumoxide` and `rustenium` integrations; new BiDi event processor for Firefox/WebKit; new network proxy layer
- **napi-rs Bridge**: Expanded FFI surface for trace data, network events, and test runner callbacks; new shared-memory snapshot buffers
- **Node.js Package**: New CLI commands (`test`, `codegen`, `debug`, `migrate`, `run-trace-viewer`); new public APIs for interception, assertions, fixtures, component mounting
- **Build System**: Additional binary targets per browser engine; WASM compilation target; Docker multi-stage builds
- **Documentation**: Complete rewrite of all docs to reflect testing framework usage, not just automation library
- **Pricing/Business Model**: Swarm grid enables disruptive pricing vs Playwright/ Cypress cloud offerings
