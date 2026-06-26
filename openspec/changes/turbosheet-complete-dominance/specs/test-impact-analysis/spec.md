## ADDED Requirements

### Requirement: Test Impact Analysis

The system SHALL determine which tests to run based on code changes, reducing CI time by running only affected tests.

#### Scenario: Building file→test mapping

- **WHEN** `npx tsheet test --coverage` is run
- **THEN** the system SHALL collect code coverage data (lcov format)
- **AND** SHALL build a mapping: which source files are covered by which tests
- **AND** SHALL save the mapping for future `--impact` runs

#### Scenario: Detecting changed files

- **WHEN** `npx tsheet test --impact` is run
- **THEN** the system SHALL parse `git diff HEAD~1` or compare against a stored baseline
- **AND** SHALL identify all changed source files

#### Scenario: Selecting affected tests

- **WHEN** changed files and file→test mapping are both available
- **THEN** the system SHALL select only tests that cover the changed files
- **AND** SHALL run the selected tests instead of the full suite
- **AND** SHALL output "Running N of M tests (impact analysis)"

#### Scenario: Fallback to full suite

- **WHEN** coverage data is missing or stale (>7 days old)
- **THEN** the system SHALL log a warning
- **AND** SHALL fall back to running the full test suite
- **WHEN** `git diff` fails or there's no git history
- **THEN** the system SHALL fall back to full suite

#### Scenario: Impact report

- **WHEN** `--impact` mode completes
- **THEN** the system SHALL output: number of changed files, number of affected tests, time saved vs full run, and list of skipped tests
