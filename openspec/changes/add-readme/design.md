## Context

TurboSheet is a Rust-native browser automation framework (v0.1.0, pre-release) with a mature codebase — 28+ Rust modules via napi-rs, JS bindings, CLI, Dockerfiles, and existing documentation (`docs/getting-started.md`, `docs/api-reference.md`). However, the project has **zero README** at the repository root. The landing page (`landing-page/index.html`) was recently corrected to accurately describe features, and the `docs/` directory has detailed guides — but none of this is discoverable from the GitHub repo page.

The project has one commit on `dev` branch, is not published to npm, and several features are partial (Firefox ~25%, WebKit ~20%, test runner ~35%). Any README must be honest about maturity.

## Goals / Non-Goals

**Goals:**

- Create a README.md that serves as the front door for developers evaluating TurboSheet
- Include a working code example in the first screenful (quick-start pattern)
- Present features with accurate maturity indicators (not marketing claims)
- Document the CLI interface
- Explain the architecture at a high level (napi-rs → Rust → CDP)
- Link to existing docs/ for deep dives (don't duplicate)
- Provide build-from-source and Docker instructions
- Include honest comparison context vs Playwright/Puppeteer/Cypress

**Non-Goals:**

- Replacing `docs/getting-started.md` or `docs/api-reference.md` — those remain canonical
- Writing installation instructions for npm publish (package is not published yet)
- Manufacturing fake benchmarks or performance claims
- Documenting every napi export (that's `docs/api-reference.md`)
- Internal development tracking (that's `PROGRESS.md`)

## Decisions

1. **Structure: inverted pyramid — most important first**
   - Top: identity + code example + install
   - Middle: features + CLI + architecture
   - Bottom: comparison + contributing + license
   - Rationale: Developers scan READMEs vertically. The code example is the highest-value element.

2. **Maturity table: explicit per-feature status**
   - Use emoji/glyph indicators (✅ mature, 🧪 experimental, 🚧 in progress)
   - Rationale: Prevents over-promising. Evaluators appreciate honesty. Matches the PROGRESS.md audit.

3. **Linking over duplicating**
   - README links to `docs/getting-started.md` for setup, `docs/api-reference.md` for API
   - Rationale: Keeps README scannable. Single source of truth in docs/.

4. **Comparison section: honest bullets, no table**
   - Avoid fake precision (no "2.3x faster" without benchmarks)
   - Focus on architectural differentiators (Rust native, direct CDP, test runner built-in)
   - Explicitly acknowledge gaps (Firefox/WebKit partial, fewer integrations)

5. **API code example: use real signatures**
   - Use actual API from `index.d.ts` / `src/lib.rs` (launch, locator, test/expect)
   - Rationale: The landing page rewrite taught us that fake API examples erode trust.

6. **Build instructions: from source + Docker**
   - Since the package isn't published, the main path is `napi build` + `cargo build`
   - Include Docker-based workflow as alternative
   - Rationale: Reflects current reality, avoids broken `npm install` instructions

## Risks / Trade-offs

- **[Staleness]** README drifts from actual API → _Mitigation: README links to docs/ for canonical API reference; README focuses on concepts, not exhaustive docs_
- **[Over-promising]** Listing features that aren't production-ready → _Mitigation: Explicit maturity table with 🚧/🧪 indicators; linked PROGRESS.md for full audit_
- **[Under-selling]** Too conservative tone discourages adoption → _Mitigation: Balance honesty with enthusiasm — emphasize architectural advantages (Rust-native, direct CDP) that are real, not aspirational_
- **[Maintenance burden]** README needs updates as APIs change → _Mitigation: Keep it high-level; detailed API surface lives in docs/api-reference.md_
