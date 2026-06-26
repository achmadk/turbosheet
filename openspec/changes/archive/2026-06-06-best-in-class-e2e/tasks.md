## 1. Trust Foundation (Stub Elimination & Error Context)

- [ ] 1.1 Implement Chromium CDP stubs for `set_viewport_size`, `reload`, `go_back`, and `go_forward` in `src/engine/chromium.rs`
- [ ] 1.2 Replace remaining unsupported stubs (e.g., `tap`, `swipe`, `pinch`, `route_web_socket`) with explicit `NotImplementedError` throws
- [ ] 1.3 Enhance `TurbosheetError` with context fields (selector, expected/actual values)
- [ ] 1.4 Wire dialog handling via `Page.javascriptDialogOpening` CDP event
- [ ] 1.5 Implement `emulate_media`, `set_offline`, and `throttle` using CDP Emulation and Network domains

## 2. Selector Engine & Locator Composition

- [ ] 2.1 Build custom selector parser in `injected-core.ts` to support `text=`, `role=`, and `css=` prefixes
- [ ] 2.2 Implement `getByRole` using ARIA role mapping and accessible name calculation
- [ ] 2.3 Implement `getByText`, `getByLabel`, `getByTestId`, and `getByPlaceholder` factories
- [ ] 2.4 Update `JsLocator` in `src/locator.rs` to support compositional chaining (`.locator()`)
- [ ] 2.5 Implement `.filter()`, `.first()`, `.last()`, `.nth()`, and `.count()` on locators
- [ ] 2.6 Enforce strict mode across all locator actions, throwing an error on ambiguous matches

## 3. Network Layer Maturity

- [ ] 3.1 Implement CDP Fetch domain interception for Chromium in `src/network/route.rs`
- [ ] 3.2 Bridge CDP `Network.requestWillBeSent` and `Network.responseReceived` events to Tokio channels
- [ ] 3.3 Implement `waitForRequest` and `waitForResponse` using the new event channels
- [ ] 3.4 Wire `page.on('request')` and `page.on('response')` bindings to N-API

## 4. Assertion Engine Completeness

- [ ] 4.1 Implement `toHaveCount`, `toBeVisible`, `toBeAttached`, and `toBeInViewport` matchers in `src/assertions/matchers.rs`
- [ ] 4.2 Implement `toHaveClass`, `toHaveCSS`, `toBeFocused`, `toBeChecked`, `toBeEditable`, and `toBeEmpty` matchers
- [ ] 4.3 Add logical negation (`.not`) support to the `AssertionEngine`
- [ ] 4.4 Update all matchers to incorporate `MatcherConfig.message` in error output
- [ ] 4.5 Enhance assertion error formatting to include clear expected/actual visual diffs

## 5. Test Runner DX Polish

- [ ] 5.1 Create a config loader via `tsx` worker to parse `turbosheet.config.ts` into a `TestConfig` struct
- [ ] 5.2 Build the `npx turbosheet test` CLI entry point in the `cli/` directory
- [ ] 5.3 Implement test discovery filtering using grep pattern matching
- [ ] 5.4 Wire hook execution (`beforeAll`, `beforeEach`, `afterEach`, `afterAll`) in `HookExecutor::execute_hook`

## 6. Cross-Browser Parity

- [ ] 6.1 Extract duplicated WebDriver logic from `firefox.rs` and `webkit.rs` into a shared `WebDriverClient`
- [ ] 6.2 Implement dynamic, collision-free port allocation for parallel WebDriver instances
- [ ] 6.3 Transition Firefox implementation to utilize WebDriver BiDi for console events and script preloading
- [ ] 6.4 Implement robust context isolation (new window/tab per context) for Firefox and WebKit
