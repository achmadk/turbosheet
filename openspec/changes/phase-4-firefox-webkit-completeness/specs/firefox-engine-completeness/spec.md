## ADDED Requirements

### Requirement: Firefox url() returns actual URL

`FirefoxPageEngine::url()` SHALL return the current page URL, not the placeholder `"about:blank"`.

#### Scenario: URL cached from goto

- **WHEN** `goto("https://example.com/page")` is called
- **THEN** subsequent calls to `url()` SHALL return `"https://example.com/page"` without an async fetch

#### Scenario: URL refreshes after reload

- **WHEN** `reload()` completes
- **THEN** `url()` SHALL reflect any URL changes (e.g., redirects during reload)

### Requirement: Firefox set_content via WebDriver

`FirefoxPageEngine::set_content(html)` SHALL set the page's HTML content.

#### Scenario: Set content on blank page

- **WHEN** `set_content("<h1>Hello</h1>")` is called
- **THEN** the page SHALL navigate to `about:blank` first
- **AND** then evaluate `document.open(); document.write(html); document.close()`
- **AND** subsequent `content()` SHALL return the set content

### Requirement: Firefox set_input_files via WebDriver upload

`FirefoxPageEngine::set_input_files(selector, files)` SHALL upload files to a file input element.

#### Scenario: Upload single file

- **WHEN** `set_input_files("input[type=file]", ["/tmp/test.pdf"])` is called
- **THEN** the engine SHALL find the element and send file paths via WebDriver POST `/element/{id}/upload`

### Requirement: Firefox wait_for_request via BiDi

`FirefoxPageEngine::wait_for_request(url_pattern)` SHALL resolve when a network request matching the URL is initiated.

#### Scenario: Wait for matching request

- **WHEN** a page initiates a fetch to `https://api.example.com/data`
- **AND** `wait_for_request("api.example.com")` is pending
- **THEN** the waiter SHALL resolve when the `network.beforeRequestSent` BiDi event matches the URL pattern

#### Scenario: Wait for request timeout

- **WHEN** no matching request occurs within 30 seconds
- **THEN** `wait_for_request(url)` SHALL return a timeout error

### Requirement: Firefox wait_for_response via BiDi

`FirefoxPageEngine::wait_for_response(url_pattern)` SHALL resolve when a network response matching the URL is received.

#### Scenario: Wait for matching response

- **WHEN** a page receives a response from `https://api.example.com/data`
- **AND** `wait_for_response("api.example.com")` is pending
- **THEN** the waiter SHALL resolve when the `network.responseCompleted` BiDi event matches the URL pattern

### Requirement: Firefox expose_function via BiDi

`FirefoxPageEngine::expose_function(name, js)` SHALL expose a JavaScript function that can be called from page context.

#### Scenario: Expose function that logs

- **WHEN** `expose_function("myLog", "function(msg) { console.log(msg); }")` is called
- **THEN** calling `window.myLog("hello")` from page context SHALL execute the registered JS function
- **AND** the binding SHALL survive page navigations (via BiDi `script.addPreloadScript`)

### Requirement: Firefox evaluate_handle

`FirefoxPageEngine::evaluate_handle(js)` SHALL evaluate JavaScript and return a handle identifier.

#### Scenario: Evaluate simple expression

- **WHEN** `evaluate_handle("document.title")` is called
- **THEN** it SHALL return a JSON string containing the result value

### Requirement: Firefox register_binding via BiDi

`FirefoxPageEngine::register_binding(name)` SHALL register a function binding accessible from page context.

#### Scenario: Register binding that calls back

- **WHEN** `register_binding("__ts_binding_test")` is called
- **THEN** a function SHALL be available in page context as `window.__ts_binding_test`
- **AND** calling it from page context SHALL invoke the registered binding handler

### Requirement: Firefox ContextEngine::pages() tracks live pages

`FirefoxContextEngine::pages()` SHALL return all live pages created by this context.

#### Scenario: Pages returns created pages

- **WHEN** `new_page()` is called twice
- **THEN** `pages()` SHALL return a Vec with 2 entries

#### Scenario: Closed page removed from pages

- **WHEN** a page is closed
- **THEN** it SHALL be removed from `pages()` result
