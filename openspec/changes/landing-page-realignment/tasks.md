## 1. Content Audit

- [x] 1.1 Read `index.js` to confirm all exported APIs for code example accuracy
- [x] 1.2 Read `index.d.ts` to confirm TypeScript signatures for code example accuracy
- [x] 1.3 Read `src/lib.rs` for feature list cross-reference (all napi exports)
- [x] 1.4 Read `Cargo.toml` feature flags to validate supported-engine claims
- [x] 1.5 Run `git log --oneline -30` to surface any recent feature additions not yet accounted for

## 2. Hero Section Rewrite

- [x] 2.1 Rewrite tagline and value proposition to focus on Rust-native E2E testing + component testing
- [x] 2.2 Remove "three browser engines" claim; replace with accurate engine support note
- [x] 2.3 Remove "zero runtime dependencies" claim
- [x] 2.4 Remove fabricated benchmark numbers (67%, 50%, 100ms, 40MB)
- [x] 2.5 Keep existing visual design (particle animation, grid, glow orbs, typography)

## 3. Feature Grid Rewrite

- [x] 3.1 Add "Test Runner" feature card (built-in `test()`/`expect()`/`describe()` with Rust-native execution)
- [x] 3.2 Add "Component Testing" feature card (`mount()` with locator API)
- [x] 3.3 Add "Locator API" feature card (`getByRole`, `getByText`, `getByTestId`, etc.)
- [x] 3.4 Add "Trace Viewer" feature card (record, serialize, HTML viewer)
- [x] 3.5 Add "Visual Regression" feature card (screenshot comparison & diff)
- [x] 3.6 Add "Video Recording" feature card (session recording)
- [x] 3.7 Add "Network Interception" feature card (`page.route()`)
- [x] 3.8 Add "Swarm Mode" feature card (Redis-backed parallel execution)
- [x] 3.9 Add "CLI & Cross-Platform" feature card (npx binary + prebuilt ARM/x64)
- [x] 3.10 Remove fake features (multi-engine, page.act, etc.)

## 4. Code Example Rewrite

- [x] 4.1 Replace code example with real API: `launch()` + page nav + locator interaction
- [x] 4.2 Extend example to show component testing with `mount()`
- [x] 4.3 Extend example to show test runner with `test()`/`expect()`
- [x] 4.4 Verify all used API signatures match `index.js` and `index.d.ts` exactly

## 5. Performance & Architecture Section Rewrite

- [x] 5.1 Replace fake benchmarks with qualitative architectural advantages (Rust core, napi-rs zero-copy FFI, no JSON-RPC overhead, chromiumoxide direct protocol)
- [x] 5.2 Remove any "vs Playwright" side-by-side comparison table if it uses fabricated numbers
- [x] 5.3 Add note about feature-gated engines (bidi/WASM) with honest maturity status

## 6. Footer & Metadata

- [x] 6.1 Update package name and repo links to match `package.json`
- [x] 6.2 Add comment header: `<!-- Last validated against: src/lib.rs, index.js, index.d.ts -->`
- [x] 6.3 Verify all external links resolve correctly

## 7. Final Validation

- [x] 7.1 Proofread entire page — no remaining false claims, fabricated APIs, or fake benchmarks
- [x] 7.2 Confirm code example compiles mentally (each API call matches a real export)
- [x] 7.3 Open page in browser to confirm visual layout is intact
- [x] 7.4 Run `lsp_diagnostics` on landing page (HTML — no fixable diagnostics)
