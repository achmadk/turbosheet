## ADDED Requirements

### Requirement: Polling Assertions

The system SHALL automatically poll and retry assertions (e.g., `toBeVisible()`, `toHaveText()`) until they pass or the timeout is exceeded.

#### Scenario: Element eventually becomes visible

- **WHEN** a test asserts an element should be visible, and it appears after 2 seconds
- **THEN** the assertion passes without failing prematurely

### Requirement: Soft Assertions

The system SHALL support soft assertions that log a failure but allow the test execution to continue, failing the suite only at the end.

#### Scenario: Verifying multiple independent UI states

- **WHEN** a soft assertion fails
- **THEN** the failure is recorded, but the next action in the test continues execution
