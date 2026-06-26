## ADDED Requirements

### Requirement: Print test results in list format

The system SHALL output test results as a line-by-line list to stdout, showing one line per test with a status symbol, test name, and duration.

#### Scenario: Passing test output

- **WHEN** a test named "should login to dashboard" passes in 1,234ms
- **THEN** the output SHALL contain `✓ should login to dashboard (1234ms)`

#### Scenario: Failed test output

- **WHEN** a test named "should validate email" fails with an error message
- **THEN** the output SHALL contain `✗ should validate email (567ms)`
- **THEN** the output SHALL include the error message on subsequent indented lines

#### Scenario: Skipped test output

- **WHEN** a test named "slow network test" is skipped
- **THEN** the output SHALL contain `- slow network test`

#### Scenario: Timed out test output

- **WHEN** a test named "hanging query" times out
- **THEN** the output SHALL contain `· hanging query (timeout: 30s)`

### Requirement: Show suite hierarchy with indentation

The system SHALL indent nested tests under their suite name to visually represent the hierarchy.

#### Scenario: Nested suite output formatting

- **WHEN** describe "Login flows" contains test "valid" and test "invalid"
- **THEN** the output SHALL show:
  ```
  Login flows
    ✓ valid (100ms)
    ✓ invalid (200ms)
  ```

### Requirement: Print summary line after all tests

The system SHALL print a summary line showing total counts of passed, failed, skipped, and timed out tests.

#### Scenario: Mixed results summary

- **WHEN** 5 pass, 2 fail, 1 skip, 1 timeout
- **THEN** the output SHALL contain a line like `5 passed, 2 failed, 1 skipped, 1 timed out`

### Requirement: Show error details for failures

The system SHALL display the error message and stack trace for each failed test below its result line, indented for readability.

#### Scenario: Failed test with error details

- **WHEN** a test named "should connect" fails with `Error: Connection refused`
- **THEN** the output SHALL contain error details indented below the test line:
  ```
  ✗ should connect (300ms)
    Error: Connection refused
        at Object.<anonymous> (/tests/connect.ts:15)
  ```

### Requirement: Reporter is selectable via config

The system SHALL use the list reporter when `reporter: "list"` is specified in the config (which is the default).

#### Scenario: Default reporter is list

- **WHEN** no explicit reporter is configured
- **THEN** the list reporter SHALL be used by default
- **THEN** output SHALL follow the list reporter format
