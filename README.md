<p align="center">
  <picture>
    <source media="(prefers-color-scheme: dark)" srcset="">
    <img alt="TurboSheet" height="120" src="">
  </picture>
</p>

<h1 align="center">TurboSheet</h1>

<p align="center">
  Rust-native browser automation for Node.js<br />
  A high-performance alternative to Playwright, Puppeteer, and Cypress.
</p>

<p align="center">
  <a href="#installation"><img src="https://img.shields.io/badge/node-%3E%3D18-brightgreen" alt="Node 18+" /></a>
  <a href="#building"><img src="https://img.shields.io/badge/rust-%3E%3D1.75-orange" alt="Rust 1.75+" /></a>
  <a href="LICENSE"><img src="https://img.shields.io/badge/license-MIT-blue" alt="MIT License" /></a>
  <a href="https://www.npmjs.com/package/turbosheet"><img src="https://img.shields.io/npm/v/turbosheet" alt="npm" /></a>
  <a href="#features"><img src="https://img.shields.io/badge/status-pre--release-yellow" alt="Pre-release" /></a>
</p>

---

## Quick Start

```typescript
import { test, expect } from "turbosheet";

test("homepage loads", async ({ page }) => {
  await page.goto("https://example.com");
  await expect(page.locator("h1")).toHaveText("Example Domain");
  await expect(page).toHaveTitle(/Example/);
});

test("interaction works", async ({ page }) => {
  await page.goto("https://example.com");
  await page.locator("a").click();
  await expect(page).toHaveURL(/iana\.org/);
});
```

```bash
npx tsheet test
```

---

## Installation

TurboSheet is in **pre-release** (v0.1.0). The package is not yet published to npm — install via npm/yarn will be available after v1.0.

### Build from Source

**Prerequisites:** Rust 1.75+, Node.js 18+, pnpm

```bash
# Clone the repository
git clone https://github.com/your-org/turbosheet.git
cd turbosheet

# Install JS dependencies
pnpm install

# Build the native binary
npx napi build --platform --release

# Verify the native module loads
node -e "const t = require('./index.js'); console.log(t.version());"
```

### Docker

Pre-configured Dockerfiles are available:

| Architecture                       | Dockerfile                                                             |
| ---------------------------------- | ---------------------------------------------------------------------- |
| Full (Chromium + Firefox + WebKit) | [`docker/Dockerfile`](./docker/Dockerfile)                             |
| ARM64                              | [`docker/Dockerfile.arm64`](./docker/Dockerfile.arm64)                 |
| Chromium-only (lightweight)        | [`docker/Dockerfile.chromium-only`](./docker/Dockerfile.chromium-only) |

---

## Features

| Feature                 | Status          | Description                                                                                                                      |
| ----------------------- | --------------- | -------------------------------------------------------------------------------------------------------------------------------- |
| **Chromium Automation** | ✅ Mature       | Full CDP protocol via chromiumoxide. Navigation, locators, clicks, fills, screenshots, JavaScript evaluation.                    |
| **Firefox**             | 🚧 In Progress  | WebDriver BiDi protocol via geckodriver. Basic navigation and page interaction.                                                  |
| **WebKit**              | 🚧 In Progress  | WebDriver protocol. Basic navigation and page interaction.                                                                       |
| **Test Runner**         | 🚧 In Progress  | Built-in `describe`/`test`/`expect` framework with parallel workers, retries, timeouts, sharding, and grep filtering.            |
| **Component Testing**   | 🧪 Experimental | Mount React, Vue, Svelte, or vanilla components in isolation. ComponentLocator API with `getByRole`, `getByText`, `getByTestId`. |
| **Trace Viewer**        | 🧪 Experimental | Record and replay test traces. In-memory event store with HTML viewer. Commands: `show-trace`, `export-trace`.                   |
| **Visual Regression**   | 🧪 Experimental | Screenshot comparison with diff image generation and configurable thresholds.                                                    |
| **Video Recording**     | 🧪 Experimental | CDP screencast → FFmpeg pipeline with per-page and per-context controls, retention policies (on-failure, always, never).         |
| **Swarm / Parallel**    | 🧪 Experimental | Redis-backed distributed test execution across multiple machines.                                                                |
| **WASM Edge Runtime**   | 🧪 Experimental | Compile to `wasm32-wasi` for edge/functions environments.                                                                        |

Key capabilities:

- **Locator API** — Chainable `locator()` queries with `getByRole`, `getByText`, `getByLabel`, `getByPlaceholder`, `getByAltText`, `getByTitle`, `getByTestId`, plus `filter()`, `first()`, `last()`, `nth()`.
- **Assertions** — `expect(page).toHaveTitle()`, `toHaveURL()`, `toHaveText()`, `toBeVisible()`, `toBeEnabled()`, `toHaveAttribute()`, and more.
- **Network Interception** — `page.route()` for request/response mocking, blocking, and modification.
- **Test Hooks** — `beforeAll`, `afterAll`, `beforeEach`, `afterEach`.
- **Multiple Reporters** — dot, line, list, JSON, HTML, JUnit, GitHub annotations.
- **Configuration** — `tsheet.config.ts` with `defineConfig()` for test directory, workers, timeouts, retries, projects, sharding, and global setup/teardown.
- **Device Emulation** — Built-in presets for mobile devices, tablets, and viewport configurations.

---

## CLI

The `tsheet` command-line tool provides the primary interface:

```
Usage: tsheet <command> [options]

Commands:
  test [files...]        Run tests (default: all .tsheet.ts files)
  install [browser]     Install browser (chromium, firefox, webkit)
  devices                List available device presets
  show-trace <file>     Open trace file in viewer
  export-trace          Export trace data
  codegen [--url <url>]  Record test actions to file
  debug [--url <url>]    Interactive debug session
  migrate <from>        Migrate tests from (playwright|cypress|puppeteer)
  init [--ci <type>]    Initialize project (github|gitlab|jenkins)
  help                  Show this help

Test Options:
  --reporter <type>     Reporter: dot, line, list, json, html, github, junit
  --workers <n>         Number of parallel workers
  --retries <n>         Retry failed tests n times
  --timeout <ms>        Test timeout in milliseconds
  --grep <pattern>      Only run tests matching pattern
  --shard <n>/<m>       Run shard n of m (for CI)
  --headed              Run tests in headed mode
  --browser <type>      Browser engine: chromium, firefox, webkit
  --output <dir>        Output directory for results and screenshots
  --update-snapshots    Update expected snapshot files
  --project <name>      Run tests matching the specified project

Examples:
  tsheet test
  tsheet test --reporter html --workers 4
  tsheet install chromium
  tsheet codegen --url https://example.com
  tsheet init --ci github
```

---

## Architecture

TurboSheet's architecture eliminates the traditional JSON-RPC serialization bottleneck found in other browser automation frameworks.

```
┌─────────────────────────────────────────────────┐
│                  Node.js Process                 │
│                                                   │
│   ┌─────────────┐         ┌───────────────────┐  │
│   │   User Code  │  CJS    │   index.js        │  │
│   │  (test file) │ ──────► │  (JS wrapper)     │  │
│   └──────┬──────┘         └────────┬──────────┘  │
│          │                         │             │
│          │           napi-rs FFI   │             │
│          │           (zero-copy)   │             │
│          ▼                         ▼             │
│   ┌──────────────────────────────────────────┐   │
│   │        Rust Native Module (turbosheet)    │   │
│   │                                           │   │
│   │  ┌──────────┐  ┌──────────┐ ┌──────────┐ │   │
│   │  │ Browser  │  │ Context  │ │  Page    │ │   │
│   │  │ Manager  │  │ Manager  │ │ Manager  │ │   │
│   │  └────┬─────┘  └────┬─────┘ └────┬─────┘ │   │
│   │       │              │            │        │   │
│   │  ┌────▼──────────────▼────────────▼─────┐  │   │
│   │  │       Chromiumoxide (CDP Client)      │  │   │
│   │  │  ┌─────────┐ ┌────────┐ ┌─────────┐  │  │   │
│   │  │  │ Session │ │ Events │ │  DOM    │  │  │   │
│   │  │  │ Manager │ │ Stream │ │ Query   │  │  │   │
│   │  │  └─────────┘ └────────┘ └─────────┘  │  │   │
│   │  └──────────────────────────────────────┘  │   │
│   └──────────────────────────────────────────┘   │
│                                                   │
│    ┌──────────┐ ┌──────────┐ ┌────────────────┐  │
│    │ Tokio    │ │  Feature │ │   Platform     │  │
│    │ Async    │ │  Flags   │ │   Native       │  │
│    │ Runtime  │ │ (cfg)    │ │  (6 triples)   │  │
│    └──────────┘ └──────────┘ └────────────────┘  │
└───────────────────┬───────────────────────────────┘
                    │
                    │ CDP / WebDriver Protocol
                    ▼
         ┌─────────────────────┐
         │      Browser        │
         │  (Chromium/Firefox  │
         │      /WebKit)       │
         └─────────────────────┘
```

**Key design points:**

- **Zero-copy FFI** — Data passes between JS and Rust through napi-rs buffers without JSON serialization. No JSON-RPC overhead.
- **Direct CDP** — Chromium integration talks Chrome DevTools Protocol directly via chromiumoxide, not through a WebSocket proxy.
- **Feature-gated engines** — Browser backends are conditional Cargo features (`chromium`, `firefox`, `webkit`), allowing minimal builds for specific use cases.
- **Tokio async** — All I/O runs on tokio's multi-threaded runtime, with napi-rs async tasks bridging to JS Promises.
- **Cross-platform binaries** — Prebuilt for 6 platform triples via napi-rs CI: Linux (x64, arm64), macOS (x64, arm64), Windows (x64, arm64).
- **Feature flags** — `visual` (screenshot comparison), `video` (recording), `swarm`/`redis` (distributed), `edge`/`wasm` (edge runtime) are all optional.

---

## Comparison

TurboSheet takes a different architectural approach than existing browser automation frameworks:

|                     | TurboSheet             | Playwright      | Puppeteer          | Cypress            |
| ------------------- | ---------------------- | --------------- | ------------------ | ------------------ |
| **Core**            | Rust (napi-rs)         | Node.js + C++   | Node.js + C++      | Electron + Node.js |
| **Protocol**        | Direct CDP             | WebSocket CDP   | WebSocket CDP      | Custom (proxy)     |
| **Serialization**   | napi-rs zero-copy      | JSON (CDP)      | JSON (CDP)         | JSON               |
| **Test Runner**     | Built-in (Rust-native) | Built-in        | External           | Built-in           |
| **Languages**       | JS/TS (Node.js)        | JS/TS (Node.js) | JS/TS (Node.js)    | JS/TS (Node.js)    |
| **Component Tests** | ✅ Mount + Locator     | ✅              | ✅ (via 3rd party) | ✅                 |

**Architectural advantages of TurboSheet:**

- **No JSON-RPC bottleneck** — The traditional CDP proxy setup (Chrome ↔ WebSocket ↔ JSON ↔ Node.js) adds serialization overhead for every interaction. TurboSheet's napi-rs FFI moves data as native buffers.
- **Built-in test runner** — `test`/`expect`/`describe` live in the Rust native layer alongside browser automation. No separate test framework to configure.
- **Single binary** — The entire framework compiles to a single `.node` binary. No chain of npm dependencies for protocol handling.
- **Feature modularity** — Compile only what you need (e.g., `chromium + traces` or `firefox + wasm`), keeping the footprint minimal.

**Current gaps (honest assessment):**

- **Firefox and WebKit are partial** — Chromium is the mature engine (~65% complete). Firefox and WebKit support is in early stages.
- **Not yet published to npm** — Installation requires building from source or Docker. npm/yarn install will be available post-v1.0.
- **Smaller ecosystem** — Fewer integrations, plugins, and community resources compared to Playwright or Cypress.
- **No official cloud offering** — No SaaS dashboard or cloud-hosted runner (yet).

---

## Contributing

### Prerequisites

- Rust 1.75+ (`rustup install 1.75`)
- Node.js 18+ and pnpm
- A browser binary (Chromium is default)

### Building

```bash
# Install JS dependencies
pnpm install

# Build the native binary (release)
npx napi build --platform --release

# Build for WASM edge runtime
cargo build --target wasm32-wasi --release
```

### Testing

```bash
# Run JS/TS tests (Vitest)
pnpm test

# Run Rust unit tests
cargo test

# Full check (format + lint + type-check + test)
pnpm exec vp check
```

### Documentation

- [Getting Started Guide](./docs/getting-started.md) — 5-minute quickstart
- [API Reference](./docs/api-reference.md) — Complete API documentation
- [Migration Guide](./docs/migration-guide.md) — Migrate from Playwright, Cypress, or Puppeteer
- [AGENTS.md](./AGENTS.md) — Vite+ toolchain conventions and project setup

### Project Status

TurboSheet is in active development. See [`PROGRESS.md`](./PROGRESS.md) for the full feature audit, completion estimates by engine, and improvement roadmap.

---

## License

MIT — see [LICENSE](./LICENSE) for details.

**v0.1.0 — Pre-release.** The API is subject to change before the stable v1.0 release.
