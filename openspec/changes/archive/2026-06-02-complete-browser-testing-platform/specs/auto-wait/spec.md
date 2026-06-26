# auto-wait

## ADDED Requirements

### Requirement: Automatic waiting for element visibility

The system SHALL automatically wait for elements to be visible before performing actions.

#### Scenario: Wait for element to be visible

- **WHEN** user calls `page.locator('#button').click()`
- **AND** element is not yet visible
- **THEN** system waits up to timeout for element to become visible before clicking

#### Scenario: Wait timeout results in error

- **WHEN** user calls `page.locator('#button').click()`
- **AND** element does not become visible within timeout
- **THEN** system throws error "Timeout waiting for element #button to be visible"

### Requirement: Automatic waiting for element attachment

The system SHALL automatically wait for elements to be attached to DOM.

#### Scenario: Wait for element in DOM

- **WHEN** user calls `page.locator('.item').first().click()`
- **AND** element is not yet in DOM
- **THEN** system waits for element to be attached before proceeding

### Requirement: Automatic waiting for element to be actionable

The system SHALL wait for elements to be actionable (visible and enabled) before interactions.

#### Scenario: Wait for enabled input

- **WHEN** user calls `page.locator('input').fill('text')`
- **AND** input is disabled
- **THEN** system waits for input to become enabled

#### Scenario: Wait for stable element

- **WHEN** user calls `page.locator('.animating').click()`
- **AND** element is animating
- **THEN** system waits for element to stop animating

### Requirement: Automatic waiting for navigation

The system SHALL automatically wait for navigation to complete after actions.

#### Scenario: Wait after click navigation

- **WHEN** user calls `page.locator('a').click()`
- **AND** click triggers navigation
- **THEN** system waits for navigation to complete

#### Scenario: Wait after form submit

- **WHEN** user calls `page.locator('form').evaluate(f => f.submit())`
- **AND** submit triggers navigation
- **THEN** system waits for navigation to complete

### Requirement: Configurable auto-wait timeout

The system SHALL allow configuring the timeout for automatic waits.

#### Scenario: Custom wait timeout

- **WHEN** config sets `timeout: 60000`
- **AND** user calls `page.locator('#button').click()`
- **THEN** system waits up to 60 seconds for element

### Requirement: Disable auto-wait for specific actions

The system SHALL allow disabling automatic waiting for specific actions.

#### Scenario: NoWaitFor action

- **WHEN** user calls `page.locator('#button').click({ noWaitAfter: true })`
- **THEN** system performs click immediately without waiting

### Requirement: Wait for network idle

The system SHALL optionally wait for network to be idle before considering action complete.

#### Scenario: Wait for idle after navigation

- **WHEN** user calls `page.goto('https://example.com', { waitUntil: 'networkidle' })`
- **THEN** system waits for no network connections for 500ms

#### Scenario: Do not wait for idle

- **WHEN** user calls `page.goto('https://example.com', { waitUntil: 'commit' })`
- **THEN** system resolves as soon as navigation commits
