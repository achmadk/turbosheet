## Why

`ComponentLocator` (the JS-side locator used in component-testing mode) currently reimplements basic element queries using inline `document.querySelector()` evaluations. It bypasses the Rust `JsLocator` system entirely, which already has full Playwright-compatible locator logic — actionability checks, `get_by_role/text/label/placeholder`, chaining, filtering, frame scoping, and proper wait-for-actionability semantics. This creates a fragmented developer experience: `page.locator()` uses the full Rust system, but `ComponentLocator` uses a deprecated path. Users get inconsistent behavior between page-level and component-level locators.

## What Changes

- Add `.locator(selector)` to `ComponentLocator` returning a napi-bound Rust `JsLocator` instance
- Add `.frameLocator(selector)` to `ComponentLocator`
- Add `.getByRole()`, `.getByText()`, `.getByLabel()`, `.getByPlaceholder()`, `.getByAltText()`, `.getByTitle()`, `.getByTestId()` to `ComponentLocator`
- Route all existing `ComponentLocator` action methods (`click`, `fill`, `evaluate`, etc.) through the Rust `JsLocator` for unified actionability checks
- Expose `JsLocator`/`FrameLocator` from the napi module so JS `page.locator()` already uses the same path
- Expose the get-by and filter capabilities at the top-level `page` level (already partially done)

## Capabilities

### New Capabilities

- `component-locator`: Locator API for component-level element selection and interaction in component-testing mode, matching Playwright semantics

### Modified Capabilities

_(None — no existing specs change behavior)_

## Impact

**JS API** — `ComponentLocator` grows locator sub-API; all existing methods rerouted through Rust `JsLocator`
**Rust napi layer** — `JsLocator` already fully implemented; may need minor `napi` tweaks for JS interop
**napi exports** — New exports for `componentLocator`, `frameLocator` and their action/query methods may supersede some existing `component*` napi bindings
