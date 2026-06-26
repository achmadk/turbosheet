# chromium-stub-implementation Specification

## Purpose

Wire all 12 ChromiumPageEngine stub and incomplete methods with proper CDP protocol commands, replacing current bare `Ok(())` returns and incomplete implementations.

## ADDED Requirements

### Requirement: evaluate_handle implementation

The system SHALL evaluate JavaScript in the page context and return a JS handle reference via CDP `Runtime.callFunctionOn` with `returnByValue: true`.

#### Scenario: Evaluate JS expression

- **WHEN** `evaluate_handle("document.title")` is called
- **THEN** the system invokes `Runtime.callFunctionOn` with the expression and returns the result as a serialized value

### Requirement: add_script_tag implementation

The system SHALL inject a `<script>` element into the page by evaluating JS that creates and appends a script element with the given content.

#### Scenario: Inject script tag

- **WHEN** `add_script_tag("console.log('hi')")` is called
- **THEN** the system creates a script element via CDP evaluation and appends it to `document.head`

### Requirement: add_style_tag implementation

The system SHALL inject a `<style>` element into the page by evaluating JS that creates and appends a style element with the given CSS content.

#### Scenario: Inject style tag

- **WHEN** `add_style_tag("body { background: red; }")` is called
- **THEN** the system creates a style element via CDP evaluation and appends it to `document.head`

### Requirement: expose_function implementation

The system SHALL register a CDP binding that maps a JavaScript-callable function name to a Rust callback, using `Runtime.addBinding` and an internal dispatch table.

#### Scenario: Expose function to page

- **WHEN** `expose_function("myFunc", "...")` is called
- **THEN** the system registers a `Runtime.addBinding` for the function name and stores the callback in the binding registry

### Requirement: set_viewport_size implementation

The system SHALL set the page viewport dimensions via CDP `Emulation.setDeviceMetricsOverride` with the given width and height.

#### Scenario: Set viewport size

- **WHEN** `set_viewport_size(1920, 1080)` is called
- **THEN** the system sends `Emulation.setDeviceMetricsOverride` with width=1920, height=1080

### Requirement: viewport_size implementation

The system SHALL return the current viewport size by evaluating `({width: window.innerWidth, height: window.innerHeight})` in the page context.

#### Scenario: Get viewport size

- **WHEN** `viewport_size()` is called
- **THEN** the system returns `{width, height}` from the page context

### Requirement: reload implementation

The system SHALL reload the current page via CDP `Page.reload` with optional `ignoreCache`.

#### Scenario: Reload page

- **WHEN** `reload()` is called
- **THEN** the system sends `Page.reload` CDP command
- **WHEN** called with `ignoreCache: true`
- **THEN** the system sends `Page.reload` with `ignoreCache` parameter

### Requirement: go_back implementation

The system SHALL navigate back in history via CDP `Page.navigate` with the URL from `Page.getNavigationHistory` previous entry, or via evaluating `window.history.back()`.

#### Scenario: Navigate back

- **WHEN** `go_back()` is called
- **THEN** the system evaluates `window.history.back()` and waits for `Page.frameNavigated` event

### Requirement: go_forward implementation

The system SHALL navigate forward in history by evaluating `window.history.forward()` and waiting for `Page.frameNavigated` event.

#### Scenario: Navigate forward

- **WHEN** `go_forward()` is called
- **THEN** the system evaluates `window.history.forward()` and waits for `Page.frameNavigated` event

### Requirement: set_input_files implementation

The system SHALL set file input elements by resolving the selector to a file input element and using CDP `DOM.setFileInputFiles`.

#### Scenario: Set input files

- **WHEN** `set_input_files("input[type=file]", ["/path/file.pdf"])` is called
- **THEN** the system resolves the selector to a DOM node, then sends `DOM.setFileInputFiles` with the resolved node ID and file paths

### Requirement: wait_for_request implementation

The system SHALL wait for a network request matching the given URL pattern, using the CDP Event Dispatcher's `Network.requestWillBeSent` events.

#### Scenario: Wait for request

- **WHEN** `wait_for_request("**/api/data")` is called with a glob pattern
- **THEN** the system subscribes to the event dispatcher and resolves when a matching `requestWillBeSent` event arrives, with configurable timeout

### Requirement: wait_for_response implementation

The system SHALL wait for a network response matching the given URL pattern, using the CDP Event Dispatcher's `Network.responseReceived` events.

#### Scenario: Wait for response

- **WHEN** `wait_for_response("**/api/data")` is called with a glob pattern
- **THEN** the system subscribes to the event dispatcher and resolves when a matching `responseReceived` event arrives, with configurable timeout
