## ADDED Requirements

### Requirement: Firefox and WebKit engine parity with Chromium

The FirefoxPageEngine and WebKitPageEngine SHALL implement all 57 methods of the PageEngine trait without no-op returns. Each method SHALL use the most appropriate implementation strategy: native WebDriver protocol endpoints where available, JavaScript injection via invoke_action() where WebDriver lacks support, and polling loops where needed.

#### Scenario: All PageEngine methods return real results on Firefox

- **WHEN** all 57 PageEngine trait methods are called on Firefox with valid selectors and inputs
- **THEN** every method SHALL return a meaningful result or a descriptive error — no method SHALL silently return Ok(())

#### Scenario: All PageEngine methods return real results on WebKit

- **WHEN** all 57 PageEngine trait methods are called on WebKit with valid selectors and inputs
- **THEN** every method SHALL return a meaningful result or a descriptive error — no method SHALL silently return Ok(())

### Requirement: No-op method elimination

The following no-op methods SHALL be implemented on both engines: `set_input_files`, `wait_for_request`, `wait_for_response`, `evaluate_handle`, `expose_function`, `set_content`, `inject_core_script`, `register_binding`, `set_viewport` (already done in Firefox, verify WebKit), `reload` (verify WebKit), `go_back`, `go_forward`, `url()` (SHALL return actual URL, not hardcoded "about:blank"), `pages()` (SHALL return actual open pages, not empty vec).

#### Scenario: set_input_files uploads files via WebDriver

- **WHEN** set_input_files is called with a file path on Firefox or WebKit
- **THEN** the file SHALL be uploaded to the browser using WebDriver element send-keys with the local file path, or JavaScript File constructor as fallback

#### Scenario: wait_for_request/wait_for_response implement polling

- **WHEN** wait_for_request or wait_for_response is called with a URL pattern
- **THEN** the engine SHALL poll network event logs for matching request/response within a configurable timeout, returning Ok(()) on match or Err on timeout

#### Scenario: url() returns current page URL

- **WHEN** url() is called after navigation on Firefox or WebKit
- **THEN** it SHALL return the actual current page URL, not "about:blank"

#### Scenario: pages() returns open pages

- **WHEN** pages() is called on a context with multiple open pages
- **THEN** it SHALL return all active PageEngine instances

### Requirement: JS-eval-fallback method quality parity

Methods using `invoke_action()` (JS injection) SHALL produce equivalent results to their Chromium CDP counterparts. The injected scripts already exist in `injected-actions.js` — the gap is that Firefox/WebKit engines don't call them for all methods. Critical methods: `dblclick`, `right_click`, `hover` (currently uses raw evaluate()), `check`, `uncheck`, `select`, `focus`, `blur`, `scroll_into_view`, `is_visible`, `is_enabled`, `is_disabled`, `drag_and_drop` (currently uses evaluate() with synthetic events — should use invoke_action).

#### Scenario: hover dispatches proper mouse events via injected actions

- **WHEN** hover() is called on Firefox or WebKit
- **THEN** it SHALL use invoke_action("hover", [selector]) instead of raw evaluate() with MouseEvent constructor

#### Scenario: drag_and_drop uses invoke_action

- **WHEN** drag_and_drop() is called on Firefox or WebKit
- **THEN** it SHALL use invoke_action("dragAndDrop", [source, target]) for consistency with Chromium's implementation

### Requirement: Injected script engine support

The register_binding, inject_core_script, expose_function, set_content methods SHALL work on all three engines. Chromium uses CDP `Runtime.addBinding` — Firefox/WebKit SHALL implement equivalent functionality via injected script message passing.

#### Scenario: register_binding enables Rust-to-JS callbacks on Firefox

- **WHEN** register_binding() is called on Firefox
- **THEN** subsequent invoke_action() calls SHALL be able to send messages back to Rust via the binding mechanism (currently Chromium-only)
