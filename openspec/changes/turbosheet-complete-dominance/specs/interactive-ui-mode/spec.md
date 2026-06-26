## ADDED Requirements

### Requirement: Interactive Test Runner UI

The system SHALL provide an optional interactive `--ui` mode that launches a graphical test runner interface.

#### Scenario: Launching the UI

- **WHEN** `npx tsheet test --ui` is executed
- **THEN** the system SHALL start the test runner in UI mode
- **AND** SHALL open an Electron/React application window
- **AND** the CLI SHALL remain usable without the UI (UI is optional)

#### Scenario: Test list and status

- **WHEN** the UI is running and tests are being executed
- **THEN** the UI SHALL display a list of all test files
- **AND** SHALL show real-time status (pending/running/passed/failed/skipped) for each test
- **AND** SHALL show test duration and timing information

#### Scenario: Live log streaming

- **WHEN** a test is running in the UI
- **THEN** the UI SHALL display real-time console output from the test
- **AND** SHALL support ANSI color formatting
- **AND** SHALL allow filtering logs by level (info/warn/error)

#### Scenario: Interactive re-run

- **WHEN** the user clicks "re-run" on a failed test in the UI
- **THEN** the system SHALL re-execute only that test
- **AND** SHALL update the UI with new results in real-time
- **WHEN** the user clicks "re-run failed"
- **THEN** the system SHALL re-execute all failed tests

#### Scenario: Screenshot preview

- **WHEN** a test captures a screenshot or fails with a screenshot
- **THEN** the UI SHALL display the screenshot inline
- **AND** SHALL support before/after comparison for visual regression tests
