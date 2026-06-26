## 1. README Header — Identity, Badges, One-Liner

- [x] 1.1 Write project title "TurboSheet" and subtitle "Rust-native browser automation for Node.js" with one-liner positioning vs Playwright/Puppeteer/Cypress
- [x] 1.2 Add badge row: npm version (v0.1.0), license (MIT), build status placeholder, Rust toolchain

## 2. Quick-Start Code Example

- [x] 2.1 Write a realistic code snippet showing `import { test, expect } from "turbosheet"`, `launch()`, `newContext()`, `newPage()`, `locator()`, `test()`, `expect()` using real API signatures from `index.d.ts`
- [x] 2.2 Add brief "run with `tsheet test`" instruction below the snippet

## 3. Installation Section

- [x] 3.1 Write "Build from Source" subsection: prerequisites (Rust 1.75+, Node.js 18+, pnpm), steps (`pnpm install`, `npx napi build --platform --release`), verify with `node -e "require('./index.js')"`
- [x] 3.2 Write "Docker" subsection linking to `docker/Dockerfile`, `docker/Dockerfile.arm64`, and `docker/Dockerfile.chromium-only`
- [x] 3.3 Add note: "npm/yarn install coming post-v1.0 — package not yet published"

## 4. Feature Overview with Maturity Table

- [x] 4.1 Create a feature table with columns: Feature, Status, Description
- [x] 4.2 Add rows: Chromium automation (✅ Mature), Firefox (🚧 In Progress), WebKit (🚧 In Progress), Test Runner (🚧 In Progress), Component Testing (🧪 Experimental), Trace Viewer (🧪 Experimental), Visual Regression (🧪 Experimental), Video Recording (🧪 Experimental), Swarm/Redis (🧪 Experimental), WASM Edge Runtime (🧪 Experimental)
- [x] 4.3 Add brief prose below the table describing key capabilities per feature

## 5. CLI Commands Reference

- [x] 5.1 List all `tsheet` commands: `test`, `install`, `codegen`, `debug`, `migrate`, `init`, `show-trace`, `export-trace`, `devices`, `help` — each with a one-line description matching the CLI help output from `cli/tsheet.js`

## 6. Architecture Summary

- [x] 6.1 Write architecture description: napi-rs FFI layer, Rust core (tokio), direct CDP via chromiumoxide, feature-gated engines, 6 platform triples for prebuilt binaries
- [x] 6.2 Add a simple ASCII/inline diagram or bullet flow showing JS ↔ napi-rs ↔ Rust ↔ CDP ↔ Browser

## 7. Comparison Section

- [x] 7.1 Write "vs Playwright, Puppeteer, Cypress" section with architectural differentiators (Rust-native, zero-copy FFI, built-in test runner, direct CDP)
- [x] 7.2 Acknowledge gaps honestly: partial Firefox/WebKit, package not yet published, smaller ecosystem, fewer integrations

## 8. Contribution / Build Guide

- [x] 8.1 Write prerequisites section (Rust 1.75+, Node.js 18+, pnpm)
- [x] 8.2 Write build commands: `pnpm install`, `npx napi build --platform --release`, `cargo build` for WASM
- [x] 8.3 Write test commands: `pnpm test` (Vitest), `cargo test` (Rust unit tests)
- [x] 8.4 Add link to `AGENTS.md` for Vite+ toolchain conventions
- [x] 8.5 Add link to `docs/getting-started.md` and `docs/api-reference.md`

## 9. Footer Section

- [x] 9.1 Add license block: MIT
- [x] 9.2 Add version note: "v0.1.0 — pre-release, breaking changes may occur"
- [x] 9.3 Add link to PROGRESS.md for full feature audit

## 10. Final Verification

- [x] 10.1 Verify all code example API names match actual exports in `index.d.ts` and `index.js`
- [x] 10.2 Verify all CLI commands match actual output from `cli/tsheet.js`
- [x] 10.3 Verify all doc links resolve correctly (docs/getting-started.md, docs/api-reference.md, AGENTS.md, PROGRESS.md)
- [x] 10.4 Verify README renders correctly on GitHub (no broken markdown, tables render, code blocks are fenced)
- [x] 10.5 Run `vp check` to ensure no formatting/lint issues in any files touched
