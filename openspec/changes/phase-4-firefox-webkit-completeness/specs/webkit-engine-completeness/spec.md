## ADDED Requirements

### Requirement: WebKit url() returns actual URL

`WebKitPageEngine::url()` SHALL return the current page URL, not the placeholder `"about:blank"`.

#### Scenario: URL cached from goto

- **WHEN** `goto("https://example.com/page")` is called
- **THEN** subsequent calls to `url()` SHALL return `"https://example.com/page"` without an async fetch

#### Scenario: URL refreshes after navigation

- **WHEN** `go_forward()` completes
- **THEN** `url()` SHALL reflect the new URL (re-evaluated via `window.location.href`)

### Requirement: WebKit set_content via WebDriver

`WebKitPageEngine::set_content(html)` SHALL set the page's HTML content.

#### Scenario: Set content on blank page

- **WHEN** `set_content("<h1>Hello</h1>")` is called
- **THEN** the page SHALL navigate to `about:blank` first
- **AND** then evaluate `document.open(); document.write(html); document.close()`
- **AND** subsequent `content()` SHALL return the set content

### Requirement: WebKit set_input_files via WebDriver upload

`WebKitPageEngine::set_input_files(selector, files)` SHALL upload files to a file input element.

#### Scenario: Upload single file

- **WHEN** `set_input_files("input[type=file]", ["/tmp/test.pdf"])` is called
- **THEN** the engine SHALL find the element and send file paths via WebDriver POST `/element/{id}/upload`

### Requirement: WebKit wait_for_request via polling

`WebKitPageEngine::wait_for_request(url_pattern)` SHALL resolve when a network request matching the URL is detected via polling.

#### Scenario: Wait for matching API request

- **WHEN** a page makes an XHR/fetch to `https://api.example.com/data`
- **AND** `wait_for_request("api.example.com")` is pending
- **THEN** the waiter SHALL resolve within 30s by polling `performance.getEntriesByType('resource')`

#### Scenario: Wait for request timeout

- **WHEN** no matching request occurs within 30 seconds
- **THEN** `wait_for_request(url)` SHALL return a timeout error

### Requirement: WebKit wait_for_response via polling

`WebKitPageEngine::wait_for_response(url_pattern)` SHALL resolve when a response matching the URL is detected via polling.

#### Scenario: Wait for matching response

- **WHEN** a page receives a response from `https://api.example.com/data`
- **AND** `wait_for_response("api.example.com")` is pending
- **THEN** the waiter SHALL resolve within 30s by polling `performance.getEntriesByType('resource')`

### Requirement: WebKit expose_function via evaluate

`WebKitPageEngine::expose_function(name, js)` SHALL expose a JavaScript function in page context via evaluate injection.

#### Scenario: Expose function on current page

- **WHEN** `expose_function("myLog", "function(msg) { console.log(msg); }")` is called
- **THEN** calling `window.myLog("hello")` from page context SHALL execute the function

#### Scenario: Function does not survive navigation

- **WHEN** the page navigates after `expose_function` was called
- **THEN** the function SHALL be removed from the new page (documented limitation — re-expose after navigation)

### Requirement: WebKit evaluate_handle

`WebKitPageEngine::evaluate_handle(js)` SHALL evaluate JavaScript and return a handle identifier.

#### Scenario: Evaluate simple expression

- **WHEN** `evaluate_handle("document.title")` is called
- **THEN** it SHALL return a JSON string containing the result value

### Requirement: WebKit register_binding via evaluate

`WebKitPageEngine::register_binding(name)` SHALL register a function binding accessible from page context.

#### Scenario: Register binding on current page

- **WHEN** `register_binding("__ts_binding_test")` is called
- **THEN** a function `window.__ts_binding_test` SHALL be available in page context

### Requirement: WebKit ContextEngine::pages() tracks live pages

`WebKitContextEngine::pages()` SHALL return all live pages created by this context.

#### Scenario: Pages returns created pages

- **WHEN** `new_page()` is called twice
- **THEN** `pages()` SHALL return a Vec with 2 entries

#### Scenario: Closed page removed from pages

- **WHEN** a page is closed
- **THEN** it SHALL be removed from `pages()` result
