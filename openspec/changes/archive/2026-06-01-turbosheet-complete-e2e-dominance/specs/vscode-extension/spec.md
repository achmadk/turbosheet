## ADDED Requirements

### Requirement: Test Explorer integration

The VS Code extension SHALL display all discovered tests in VS Code's Test Explorer sidebar.

#### Scenario: Test discovery

- **WHEN** user opens a project with `.tsheet.ts` files
- **THEN** the extension SHALL discover all tests and display them in the Test Explorer tree

#### Scenario: Run individual test

- **WHEN** user clicks the "Run" icon next to a specific test
- **THEN** that single test SHALL execute and show pass/fail status inline

#### Scenario: Run entire file

- **WHEN** user clicks "Run All Tests" in the Test Explorer header
- **THEN** all tests in all discovered files SHALL execute

### Requirement: Debug mode

The extension SHALL support step-through debugging of tests.

#### Scenario: Debug test

- **WHEN** user clicks "Debug" on a test
- **THEN** VS Code SHALL start a debug session with breakpoints and source map support

#### Scenario: Breakpoint in test

- **WHEN** user sets a breakpoint in a `.tsheet.ts` file and runs in debug mode
- **THEN** execution SHALL pause at the breakpoint

### Requirement: Inline pass/fail decorations

The extension SHALL show test results inline in the source code.

#### Scenario: Pass decoration

- **WHEN** a test passes
- **THEN** a green checkmark SHALL appear next to the test name in the source

#### Scenario: Fail decoration

- **WHEN** a test fails
- **THEN** a red X SHALL appear with the error message inline

### Requirement: Trace viewer integration

The extension SHALL open TurboTrace files in a custom editor panel.

#### Scenario: Open trace from test failure

- **WHEN** user clicks "View Trace" on a failed test
- **THEN** the TurboTrace viewer SHALL open in a VS Code webview panel

#### Scenario: Trace viewer features

- **WHEN** trace viewer is open
- **THEN** user SHALL see action timeline, DOM snapshots, network log, and console output
