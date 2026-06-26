## ADDED Requirements

### Requirement: Unified WebDriver Client

The system SHALL extract the duplicated WebDriver logic from Firefox and WebKit engines into a shared, reusable module.

#### Scenario: Launching a WebDriver browser

- **WHEN** Firefox or WebKit is launched
- **THEN** it MUST use the shared `WebDriverClient` for session management and command execution

### Requirement: Dynamic Port Allocation

The system SHALL assign dynamic, available ports to WebDriver instances instead of hardcoded defaults.

#### Scenario: Running parallel browsers

- **WHEN** multiple Firefox or WebKit instances are launched concurrently
- **THEN** each instance MUST bind to a unique port to avoid conflicts

### Requirement: Firefox BiDi Foundation

The system SHALL transition Firefox support towards WebDriver BiDi to enable event subscriptions and script preloading.

#### Scenario: Subscribing to Firefox console events

- **WHEN** `page.on('console')` is registered in Firefox
- **THEN** the system MUST use a BiDi connection to capture and relay the console output
