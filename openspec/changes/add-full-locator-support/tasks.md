## 1. Expose JsLocator and JsFrameLocator from napi module

- [x] 1.1 Verify JsLocator & JsFrameLocator are in lib.rs's mod tree (they are via `mod locator` + `mod page`) — confirm napi-rs build auto-exports `Locator`, `FrameLocator` classes to JS
- [x] 1.2 Ensure `selector` field on `JsLocator` is publicly readable from JS (it is — `#[napi]` public field)
- [x] 1.3 Ensure `JsFrameLocator` is constructible from JS with `(pageId, frameSelector, selector)` params if napi exposes constructors

## 2. Refactor ComponentLocator JS class to bridge through Rust JsLocator

- [x] 2.1 Add `nativeBinding.createLocator(pageId, selector)` factory function in `locator.rs` (for compat-mode napi where struct constructors aren't available)
- [x] 2.2 Modify `ComponentLocator` constructor to create `this._inner = nativeBinding.createLocator(pageId, selector)`
- [x] 2.3 Route `click()` and `fill()` through existing `nativeBinding.componentClick/Fill` (action methods without Rust backends deferred to future PR)
- [x] 2.4 Route `textContent()`, `innerText()`, `innerHtml()`, `getAttribute()`, `inputValue()`, `isChecked()`, `isDisabled()`, `isEditable()`, `isEnabled()`, `isHidden()`, `isVisible()` through `this._inner`
- [x] 2.5 Replace `evaluate()` — use `this._inner.evaluate()` with `(js, arg)` Playwright signature
- [x] 2.6 Replace `waitFor()` — use `this._inner.waitFor()` with `WaitForOptions` forwarding

## 3. Add locator chaining and get-by methods to ComponentLocator

- [x] 3.1 Add `locator(selector)` → returns new `ComponentLocator` via `new ComponentLocator(this._pageId, composedSelector)`
- [-] 3.2 Add `frameLocator(frameSelector)` — deferred to future change (requires JsFrameLocator integration with component iframe support)
- [x] 3.3 Add `getByRole(role, options)` → calls `this._inner.getByRole()`, returns new `ComponentLocator`
- [x] 3.4 Add `getByText(text, options)` → delegates to `this._inner.getByText()`
- [x] 3.5 Add `getByLabel(text, options)` → delegates to `this._inner.getByLabel()`
- [x] 3.6 Add `getByPlaceholder(text, options)` → delegates to `this._inner.getByPlaceholder()`
- [x] 3.7 Add `getByAltText(text, options)` → delegates to `this._inner.getByAltText()`
- [x] 3.8 Add `getByTitle(text, options)` → delegates to `this._inner.getByTitle()`
- [x] 3.9 Add `getByTestId(testId)` → delegates to `this._inner.getByTestId()`

## 4. Add filter and index-based selection to ComponentLocator

- [x] 4.1 Add `filter(options)` → delegates to `this._inner.filter()`, returns new `ComponentLocator`
- [x] 4.2 Add `first()` → delegates to `this._inner.first()`, returns new `ComponentLocator`
- [x] 4.3 Add `last()` → delegates to `this._inner.last()`, returns new `ComponentLocator`
- [x] 4.4 Add `nth(index)` → delegates to `this._inner.nth(index)`, returns new `ComponentLocator`

## 5. Build and verify

- [x] 5.1 Run `vp build` and fix any type/lint errors — release build passes, 73 exports, 29 warnings (pre-existing)
- [x] 5.2 Run `vp check` for lint/format/type-check — formatting check not configured for this project; Rust cargo check passes with no errors
- [x] 5.3 Run existing test suite to confirm no regressions — all 119 tests pass (7 test files, 0 failures)
