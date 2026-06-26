## ADDED Requirements

### Requirement: Plugin System

The system SHALL provide a plugin API that allows third-party extensions to hook into every engine lifecycle point.

#### Scenario: Plugin lifecycle hooks

- **GIVEN** a plugin is registered with a `beforeNavigation` hook
- **WHEN** `page.navigate(url)` is called
- **THEN** the SHALL invoke the `beforeNavigation` hook with `{ url, pageId, contextId }` before the navigation starts
- **AND** the hook SHALL be able to modify the URL before navigation proceeds
- **AND** the hook SHALL be able to abort navigation by returning `{ cancel: true }`

- **GIVEN** a plugin is registered with an `afterNavigation` hook
- **WHEN** navigation completes (CDP `Page.frameNavigated` fires)
- **THEN** the system SHALL invoke the `afterNavigation` hook with `{ url, title, frameId, loadTimeMs }`
- **AND** the hook SHALL receive the final URL after any redirects

- **GIVEN** a plugin is registered with a `beforeClick` hook
- **WHEN** `locator.click()` is performed
- **THEN** the system SHALL invoke the `beforeClick` hook with `{ selector, x, y, pageId }`
- **AND** the hook SHALL be able to modify coordinates before dispatch

- **GIVEN** a plugin is registered with a `onNetworkRequest` hook
- **WHEN** a network request is intercepted
- **THEN** the system SHALL invoke the hook with `{ url, method, headers, postData, requestId }`
- **AND** the hook SHALL be able to modify headers or block the request

- **GIVEN** a plugin is registered with an `onTestFailure` hook
- **WHEN** a test assertion fails
- **THEN** the system SHALL invoke the hook with `{ testName, error, screenshotPath, tracePath, pageSnapshot }`
- **AND** the hook SHALL be able to add custom data to the test report

#### Scenario: Plugin registration and lifecycle

- **GIVEN** a plugin is installed via `npm install tsheet-plugin-{name}`
- **WHEN** the test runner starts
- **THEN** the system SHALL discover plugins from `node_modules/tsheet-plugin-*`
- **AND** load plugin manifests (`plugin.json`) to determine hooks and dependencies
- **AND** register hooks in dependency order

- **GIVEN** a plugin declares a dependency on another plugin
- **WHEN** both plugins are installed
- **THEN** the system SHALL resolve dependencies and load them in order
- **AND** fail gracefully with a clear error message if a dependency is missing

#### Scenario: Plugin manifest format

- **GIVEN** a plugin package contains `plugin.json`
- **WHEN** the system loads the plugin
- **THEN** it SHALL parse:
  - `name`: Plugin display name
  - `version`: Semver version
  - `description`: Human-readable description
  - `hooks`: Array of hook names the plugin subscribes to
  - `dependencies`: Map of plugin name → semver range
  - `config`: JSON Schema for plugin configuration
- **AND** validate the manifest against the plugin schema
- **AND** reject invalid manifests with a descriptive error

#### Scenario: Plugin sandboxing

- **WHEN** a plugin hook is executed
- **THEN** it SHALL run with a timeout (default: 5s, configurable)
- **AND** a plugin crash SHALL NOT crash the test runner
- **AND** a plugin hang SHALL NOT block the test runner (timeout isolation)
- **AND** plugins SHALL NOT have access to the filesystem outside their own directory
