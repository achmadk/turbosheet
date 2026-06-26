# webkit-automation

## ADDED Requirements

### Requirement: WebKit navigation

The system SHALL support page navigation via `goto()` for WebKit browsers.

#### Scenario: Navigate to URL

- **WHEN** user calls `page.goto('https://example.com')` on WebKit
- **THEN** system navigates WebKit to the specified URL

#### Scenario: Navigation with waitUntil

- **WHEN** user calls `page.goto('https://example.com', { waitUntil: 'networkidle' })` on WebKit
- **THEN** system waits until network is idle before resolving

### Requirement: WebKit locator operations

The system SHALL support locator operations (`click`, `fill`, `textContent`, `isVisible`) for WebKit.

#### Scenario: Click element

- **WHEN** user calls `page.locator('#button').click()` on WebKit
- **THEN** system clicks the element in WebKit

#### Scenario: Fill input

- **WHEN** user calls `page.locator('input').fill('text')` on WebKit
- **THEN** system fills the input field in WebKit

### Requirement: WebKit evaluate

The system SHALL support JavaScript evaluation via `page.evaluate()` for WebKit.

#### Scenario: Execute JavaScript

- **WHEN** user calls `page.evaluate('return document.title')` on WebKit
- **THEN** system returns the page title from WebKit

### Requirement: WebKit screenshot

The system SHALL support page screenshots via `page.screenshot()` for WebKit.

#### Scenario: Capture full page screenshot

- **WHEN** user calls `page.screenshot({ fullPage: true })` on WebKit
- **THEN** system captures and returns a PNG of the entire page

### Requirement: WebKit context management

The system SHALL support `newContext()` and `newPage()` for WebKit.

#### Scenario: Create new context

- **WHEN** user calls `browser.newContext()` on WebKit
- **THEN** system creates a new incognito context

#### Scenario: Create new page in context

- **WHEN** user calls `context.newPage()` on WebKit
- **THEN** system creates a new page/tab in WebKit
