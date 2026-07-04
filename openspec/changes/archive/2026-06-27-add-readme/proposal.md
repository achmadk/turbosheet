## Why

TurboSheet is a Rust-native browser automation framework with a substantial codebase (28+ Rust modules, JS bindings, CLI, docker, docs), but it has **no README.md** at the project root. This is the first thing developers, contributors, and evaluators see — and currently they see an empty repo page. The landing page and docs/ directory have good content, but they're not discoverable from the GitHub repo entry point. A README is essential for adoption, contribution, and project credibility.

## What Changes

- Create `README.md` at the project root with:
  - Project identity and one-liner
  - Installation and quick-start code example
  - Key differentiators (Rust-native, direct CDP, built-in test runner)
  - Feature overview with honest maturity indicators
  - CLI commands reference
  - Architecture summary
  - Links to existing documentation (`docs/getting-started.md`, `docs/api-reference.md`)
  - Comparison context vs Playwright/Puppeteer/Cypress (honest, no fake benchmarks)
  - Build/contribution instructions
  - License and project status

## Capabilities

### New Capabilities

- `readme`: Project README.md covering identity, quick start, features (with maturity), CLI, architecture, comparison, and contribution guide

### Modified Capabilities

_(none — this is a new capability)_

## Impact

- **Single new file**: `README.md` at project root
- **No API, code, or dependency changes**
- **No breaking changes**
- Existing `docs/getting-started.md` and `docs/api-reference.md` remain the canonical deep-dive docs — README links to them
- Landing page content (already accurate) serves as reference for feature descriptions
