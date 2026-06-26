## Context

The landing page at `landing-page/index.html` is a standalone HTML+CSS+JS file with no build step. It was generated using the huashu-design skill as a futuristic dark-themed promotional page. However, the content was written without codebase awareness — it contains fabricated metrics, wrong API examples, and omits the project's strongest features.

The codebase has evolved significantly with a full test runner, component testing, trace viewer, visual regression, and network interception all implemented in Rust with napi-rs bindings. The landing page needs to be rewritten to match this reality.

## Goals / Non-Goals

**Goals:**

- Single-file HTML rewrite that accurately represents the project
- Remove all false/unverifiable claims
- Add real feature sections for: test runner, component testing, locator API, trace viewer, visual testing, video recording, network interception, CLI, cross-platform support
- Show a realistic code example using actual API shapes
- Preserve the existing visual design, animations, and responsive layout

**Non-Goals:**

- No build tooling or framework migration (remains standalone HTML+CSS+JS)
- No new CSS design system or visual overhaul
- No content beyond what the codebase actually supports
- No benchmark numbers until real benchmarks exist in CI

## Decisions

### Decision: Keep standalone HTML architecture (no framework)

The landing page will remain a single self-contained HTML file with inline CSS and JS. The huashu-design starter has no build step, no npm dependencies, and can be opened via `file://` or any static server. Adding a framework (React/Vue) would require a build step for zero functional gain.

### Decision: Remove all quantitative benchmarks

The current page claims "67% faster", "50% less memory", "100ms checks", "40MB footprint". None of these are backed by any benchmark in the repository. Replacing them with different unverified numbers is equally dishonest. Instead, the performance section will discuss architectural advantages qualitatively (Rust core, zero-copy FFI, no JSON-RPC overhead) and point to a benchmark suite that can be built separately.

### Decision: Lead with test runner + component testing as primary features

The two strongest differentiators vs Playwright/Cypress/Puppeteer are: (1) the built-in test runner with Rust-native execution, and (2) the component testing framework with `mount()` and full locator API. These should be the hero features.

### Decision: Code example shows 3 real capabilities

The example will show: `launch()` + page navigation with locator, `mount()` for component testing, and `test()`/`expect()` for the test runner — all using real exported API shapes from `index.js`.

## Risks / Trade-offs

- **[Risk] Stale content** → The page is a single file with no automated sync to the codebase. Mitigation: Add a comment header noting which source files to check when updating.
- **[Risk] Missing new features** → As development continues, new features won't automatically appear. Mitigation: The tasks include adding a `/* Last validated against: src/lib.rs, index.js, index.d.ts */` comment.
- **[Trade-off] No benchmark numbers** → Makes the page less punchy for top-of-funnel visitors, but avoids dishonesty. The architectural advantage narrative (Rust, no JSON-RPC, zero-copy) is compelling to the actual target audience (engineering teams evaluating test frameworks).
- **[Trade-off] Existing visual design is generic** → The dark cyber aesthetic doesn't communicate "test runner" or "testing tool" specifically. However, a full visual redesign is out of scope — the content realignment is the priority.
