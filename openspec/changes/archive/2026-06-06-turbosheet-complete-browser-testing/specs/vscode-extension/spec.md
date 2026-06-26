## ADDED Requirements

### Requirement: Test Explorer

The VS Code extension SHALL show discovered tests in Test Explorer panel.

#### Scenario: Show test tree

- **WHEN** workspace contains `*.tsheet.ts` files
- **THEN** Test Explorer shows test hierarchy
- **AND** suites and tests are displayed

#### Scenario: Refresh tests

- **WHEN** `tsheet.refreshTests` command is run
- **THEN** Test Explorer updates with new/removed tests

#### Scenario: Test status icons

- **WHEN** tests pass or fail
- **THEN** Test Explorer shows colored icons
- **AND** shows duration badges

### Requirement: Run/Debug test from editor

The VS Code extension SHALL provide run and debug buttons.

#### Scenario: Run single test

- **WHEN** user clicks "Run" button above test
- **THEN** `tsheet test --filter="test name"` executes
- **AND** output appears in terminal

#### Scenario: Debug test

- **WHEN** user clicks "Debug" button above test
- **THEN** `tsheet test --debug --filter="test name"` executes
- **AND** VS Code debugger attaches

#### Scenario: Run all tests in file

- **WHEN** user clicks "Run" in file decoration
- **THEN** all tests in file execute

### Requirement: Inline decorations

The VS Code extension SHALL show pass/fail decorations in editor.

#### Scenario: Show pass decoration

- **WHEN** test passes
- **THEN** green checkmark appears in gutter
- **AND** test name is highlighted green

#### Scenario: Show fail decoration

- **WHEN** test fails
- **THEN** red X appears in gutter
- **AND** error message appears on hover

#### Scenario: Show running indicator

- **WHEN** test is executing
- **THEN** spinner appears in gutter

### Requirement: Trace viewer integration

The VS Code extension SHALL embed TurboTrace viewer.

#### Scenario: Open trace from test result

- **WHEN** user clicks trace link in test output
- **THEN** trace viewer opens in VS Code
- **AND** shows timeline for that test

#### Scenario: Navigate from trace to source

- **WHEN** user clicks action in trace viewer
- **THEN** editor opens at corresponding test code

### Requirement: IntelliSense for TurboSheet API

The VS Code extension SHALL provide code completion for TurboSheet API.

#### Scenario: Complete page methods

- **WHEN** user types `page.`
- **THEN** VS Code shows completion list
- **AND** includes `goto`, `locator`, `screenshot`, etc.

#### Scenario: Complete locator methods

- **WHEN** user types `page.locator('...').`
- **THEN** VS Code shows `click`, `fill`, `hover`, etc.

#### Scenario: Complete expect matchers

- **WHEN** user types `expect(locator).`
- **THEN** VS Code shows `toBeVisible`, `toHaveText`, etc.

### Requirement: Configuration UI

The VS Code extension SHALL provide settings UI.

#### Scenario: Configure browser path

- **WHEN** user opens TurboSheet settings
- **THEN** UI allows setting browser executable paths

#### Scenario: Configure default timeout

- **WHEN** user sets default timeout in settings
- **THEN** all tests use that timeout unless overridden
