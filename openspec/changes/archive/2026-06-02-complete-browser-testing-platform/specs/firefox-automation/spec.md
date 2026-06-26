# firefox-automation

## ADDED Requirements

### Requirement: Firefox navigation

The system SHALL support page navigation via `goto()` for Firefox browsers.

#### Scenario: Navigate to URL

- **WHEN** user calls `page.goto('https://example.com')` on Firefox
- **THEN** system navigates Firefox to the specified URL

#### Scenario: Navigation with waitUntil

- **WHEN** user calls `page.goto('https://example.com', { waitUntil: 'networkidle' })` on Firefox
- **THEN** system waits until network is idle before resolving

### Requirement: Firefox locator operations

The system SHALL support locator operations (`click`, `fill`, `textContent`, `isVisible`) for Firefox.

#### Scenario: Click element

- **WHEN** user calls `page.locator('#button').click()` on Firefox
- **THEN** system clicks the element in Firefox

#### Scenario: Fill input

- **WHEN** user calls `page.locator('input').fill('text')` on Firefox
- **THEN** system fills the input field in Firefox

### Requirement: Firefox evaluate

The system SHALL support JavaScript evaluation via `page.evaluate()` for Firefox.

#### Scenario: Execute JavaScript

- **WHEN** user calls `page.evaluate('return document.title')` on Firefox
- **THEN** system returns the page title from Firefox

### Requirement: Firefox screenshot

The system SHALL support page screenshots via `page.screenshot()` for Firefox.

#### Scenario: Capture full page screenshot

- **WHEN** user calls `page.screenshot({ fullPage: true })` on Firefox
- **THEN** system captures and returns a PNG of the entire page

### Requirement: Firefox context management

The system SHALL support `newContext()` and `newPage()` for Firefox.

#### Scenario: Create new context

- **WHEN** user calls `browser.newContext()` on Firefox
- **THEN** system creates a new incognito context

#### Scenario: Create new page in context

- **WHEN** user calls `context.newPage()` on Firefox
- **THEN** system creates a new page/tab in Firefox
