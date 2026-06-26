## Why

TurboSheet aims to be the most complete and reliable E2E testing tool, but currently suffers from critical gaps compared to Playwright and Cypress. Over 42 core API methods are stubbed (silently doing nothing), the selector engine only supports basic CSS, network interception cannot inspect HTTPS bodies, and the test runner lacks essential DX features like a CLI and config file loader. Addressing these gaps is required to build user trust and achieve competitive parity.

## What Changes

- **BREAKING**: Replace silently succeeding stub methods with actual implementations or explicit `NotImplementedError` throws.
- Implement a comprehensive Selector Engine supporting smart selectors (`text=`, `role=`, `data-testid=`) and shadow DOM piercing.
- Introduce Locator Composition with chaining, filtering (`.filter()`, `.first()`), and strict mode.
- Overhaul the Network Layer to use the CDP Fetch domain for Chromium, enabling true HTTPS interception without a proxy.
- Expand the Assertion Engine with over 20 missing matchers (e.g., `toHaveCount`, `toBeVisible`) and `.not` negation support.
- Enhance the Test Runner with a `turbosheet.config.ts` loader, a dedicated CLI, and proper hook execution.
- Implement structured error types with detailed context (expected/actual diffs, selector used).
- Standardize cross-browser support, moving Firefox towards WebDriver BiDi and unifying driver clients.

## Capabilities

### New Capabilities

- `stub-elimination`: Replacing 42+ silent stubs with real CDP implementations or explicit errors.
- `selector-engine`: A new JS-based selector parser supporting `text=`, `role=`, and advanced filtering.
- `network-layer`: Full CDP Fetch domain interception, request/response events, and HTTPS body access.
- `assertion-engine`: Expanded matchers, `.not` negation, and context-rich error reporting.
- `test-runner-dx`: Config file loader, CLI runner, test filtering (`grep`), and hook execution.
- `cross-browser-parity`: Refactoring driver clients, dynamic ports, and context isolation for Firefox/WebKit.

### Modified Capabilities

- _(None currently tracked in existing spec files that overlap directly; however, this proposal supersedes some aspects of the next-gen-architecture proposal)_

## Impact

- **Core Engine**: `src/engine/chromium.rs`, `src/page.rs`, `src/locator.rs`, and injected JS will be heavily modified.
- **Assertions**: `src/assertions/matchers.rs` will expand significantly.
- **Test Runner**: `src/test_runner/mod.rs` and `worker.rs` will be refactored to support config loading and proper test registration.
- **API Surface**: New locator factories (`getByRole`, etc.) and event subscriptions (`page.on()`) will be introduced.
