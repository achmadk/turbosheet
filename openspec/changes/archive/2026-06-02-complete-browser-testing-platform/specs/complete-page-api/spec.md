# complete-page-api

## ADDED Requirements

### Requirement: setViewportSize

The system SHALL allow setting the viewport size.

#### Scenario: Set viewport size

- **WHEN** user calls `page.setViewportSize({ width: 1280, height: 720 })`
- **THEN** system resizes the viewport to specified dimensions

### Requirement: reload

The system SHALL support page reload.

#### Scenario: Reload page

- **WHEN** user calls `page.reload()`
- **THEN** system reloads the current page

#### Scenario: Reload with waitUntil

- **WHEN** user calls `page.reload({ waitUntil: 'networkidle' })`
- **THEN** system reloads and waits for network idle

### Requirement: goBack and goForward

The system SHALL support browser history navigation.

#### Scenario: Go back

- **WHEN** user calls `page.goBack()`
- **THEN** system navigates to previous page in history

#### Scenario: Go forward

- **WHEN** user calls `page.goForward()`
- **THEN** system navigates to next page in history

### Requirement: waitForRequest

The system SHALL wait for network requests matching a pattern.

#### Scenario: Wait for request

- **WHEN** user calls `page.waitForRequest('**/api/data')`
- **THEN** system waits until a matching request is made and returns it

#### Scenario: Wait for request with timeout

- **WHEN** user calls `page.waitForRequest('**/api/data', { timeout: 5000 })`
- **THEN** system throws timeout error if no matching request within 5 seconds

### Requirement: waitForResponse

The system SHALL wait for network responses matching a pattern.

#### Scenario: Wait for response

- **WHEN** user calls `page.waitForResponse('**/api/data')`
- **THEN** system waits until a matching response is received and returns it

#### Scenario: Wait for response with timeout

- **WHEN** user calls `page.waitForResponse('**/api/data', { timeout: 5000 })`
- **THEN** system throws timeout error if no matching response within 5 seconds

### Requirement: waitForSelector

The system SHALL wait for an element to appear in the DOM.

#### Scenario: Wait for visible selector

- **WHEN** user calls `page.waitForSelector('#element', { state: 'visible' })`
- **THEN** system waits until element is visible and returns the locator

#### Scenario: Wait for hidden selector

- **WHEN** user calls `page.waitForSelector('#element', { state: 'hidden' })`
- **THEN** system waits until element is removed or hidden

#### Scenario: Wait for detached selector

- **WHEN** user calls `page.waitForSelector('#element', { state: 'detached' })`
- **THEN** system waits until element is removed from DOM

### Requirement: evaluateHandle

The system SHALL evaluate JavaScript and return a handle to the result.

#### Scenario: Get JSHandle

- **WHEN** user calls `page.evaluateHandle('() => document.body')`
- **THEN** system returns a JsHandle to the result

### Requirement: addScriptTag

The system SHALL inject a script tag into the page.

#### Scenario: Add script by URL

- **WHEN** user calls `page.addScriptTag({ url: 'https://example.com/script.js' })`
- **THEN** system injects a script tag with the specified URL

#### Scenario: Add script by content

- **WHEN** user calls `page.addScriptTag({ content: 'window.foo = 1' })`
- **THEN** system injects a script tag with the specified content

### Requirement: addStyleTag

The system SHALL inject a style tag into the page.

#### Scenario: Add style by URL

- **WHEN** user calls `page.addStyleTag({ url: 'https://example.com/style.css' })`
- **THEN** system injects a link tag with the specified URL

#### Scenario: Add style by content

- **WHEN** user calls `page.addStyleTag({ content: 'body { color: red; }' })`
- **THEN** system injects a style tag with the specified content

### Requirement: exposeFunction

The system SHALL expose a function from Node.js to the page's JavaScript context.

#### Scenario: Expose function to page

- **WHEN** user calls `page.exposeFunction('myFunc', (arg) => arg * 2)`
- **THEN** system makes `window.myFunc` available in page JavaScript

#### Scenario: Call exposed function from page

- **WHEN** page calls `await window.myFunc(21)`
- **THEN** system returns `42` from the Node.js function

### Requirement: page event handlers

The system SHALL support event listeners on page objects.

#### Scenario: Listen for console event

- **WHEN** user calls `page.on('console', msg => console.log(msg.text()))`
- **THEN** system logs all console messages from the page

#### Scenario: Remove event listener

- **WHEN** user calls `page.off('console', handler)`
- **THEN** system removes the specified event handler
