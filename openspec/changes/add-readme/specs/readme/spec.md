## ADDED Requirements

### Requirement: Project identity and one-liner

The README SHALL clearly state the project name ("TurboSheet") and a one-liner describing it as a Rust-native browser automation framework for Node.js, positioned as an alternative to Playwright, Puppeteer, and Cypress.

#### Scenario: Reader understands project purpose

- **WHEN** a developer reads the first paragraph of the README
- **THEN** they can immediately identify it as a browser automation/testing framework
- **AND** they understand its differentiator (Rust-native, high-performance)

---

### Requirement: Working code example in first screenful

The README SHALL contain a runnable-looking code example within the first ~30 lines of content (before scrolling) that demonstrates: importing from `turbosheet`, launching a browser, navigating to a page, using a locator, and making an assertion. The example SHALL use real API signatures matching `index.d.ts` and `src/lib.rs`.

#### Scenario: Developer sees a realistic quick-start snippet

- **WHEN** a developer opens the README
- **THEN** they see a code block with import, launch, goto, locator, and expect calls
- **AND** the API names match `launch`, `newContext`, `newPage`, `locator`, `test`, `expect`

---

### Requirement: Installation instructions

The README SHALL include installation instructions reflecting the project's current pre-release state: build from source (via Rust toolchain + napi build) as the primary path, and Docker usage as an alternative. It SHALL NOT claim `npm install turbosheet` works unless the package is published.

#### Scenario: Developer can build from source

- **WHEN** a developer follows the build-from-source instructions
- **THEN** they can run `pnpm install`, `npx napi build --platform --release`, and verify the native module is built

#### Scenario: Developer can use Docker

- **WHEN** a developer reads the Docker section
- **THEN** they see references to `docker/Dockerfile`, `docker/Dockerfile.arm64`, and `docker/Dockerfile.chromium-only`

---

### Requirement: Feature table with maturity indicators

The README SHALL present the feature set in a table with per-feature maturity indicators. At minimum: Chromium automation (✅ mature), Firefox (🚧 in progress), WebKit (🚧 in progress), Test runner (🚧 in progress), Trace viewer (🧪 experimental), Visual regression (🧪 experimental), Video recording (🧪 experimental), Swarm/Redis (🧪 experimental), WASM edge (🧪 experimental).

#### Scenario: Developer sees feature maturity at a glance

- **WHEN** a developer reads the features section
- **THEN** they see which engines and subsystems are production-ready vs experimental
- **AND** the indicators are consistent with the PROGRESS.md audit

---

### Requirement: CLI commands reference

The README SHALL document the `tsheet` CLI commands: `test`, `install`, `codegen`, `debug`, `migrate`, `init`, `show-trace`, `export-trace`, `devices`, and `help`. Each command SHALL have a brief one-line description.

#### Scenario: Developer can discover CLI capabilities

- **WHEN** a developer reads the CLI section
- **THEN** they see all `tsheet` commands listed with descriptions matching the actual CLI help output

---

### Requirement: Architecture summary

The README SHALL include a high-level architecture summary describing: napi-rs FFI layer, Rust core with tokio async runtime, direct CDP protocol via chromiumoxide (for Chromium), feature-gated engines (feature flags in Cargo.toml), and cross-platform prebuilt binaries (6 platform triples).

#### Scenario: Developer understands the technical architecture

- **WHEN** a developer reads the architecture section
- **THEN** they understand the zero-copy FFI design, absence of JSON-RPC overhead, and the feature-flag engine system

---

### Requirement: Honest comparison context

The README SHALL include a "vs Playwright/Puppeteer/Cypress" section that honestly describes architectural differentiators (Rust-native, direct CDP, built-in test runner) without fabricated benchmarks or misleading claims. It SHALL acknowledge gaps (partial Firefox/WebKit, not yet published, smaller ecosystem).

#### Scenario: Evaluator can make an informed choice

- **WHEN** an evaluator reads the comparison section
- **THEN** they see factual architectural differences without fake numbers
- **AND** they see explicitly stated gaps and limitations

---

### Requirement: Build and contribution guide

The README SHALL include a contributing section with: build prerequisites (Rust 1.75+, Node.js 18+, pnpm), build commands (`pnpm install`, `napi build`, `cargo build`), test commands (`pnpm test`, `cargo test`), and a link to `AGENTS.md` for Vite+ toolchain conventions.

#### Scenario: New contributor can build the project

- **WHEN** a new contributor follows the build guide
- **THEN** they can successfully compile the native module and run the test suite

---

### Requirement: License and project status

The README SHALL include the project license (MIT, matching `package.json`), current version (v0.1.0, matching `package.json`), and a note about pre-release status.

#### Scenario: Reader sees license and version

- **WHEN** a reader scrolls to the bottom of the README
- **THEN** they see "MIT License" and "v0.1.0 — pre-release"
