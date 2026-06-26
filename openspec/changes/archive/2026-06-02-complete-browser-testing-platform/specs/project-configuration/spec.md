# project-configuration

## ADDED Requirements

### Requirement: tsheet.config.ts loading

The system SHALL load configuration from `tsheet.config.ts` in the project root.

#### Scenario: Load default config file

- **WHEN** `tsheet.config.ts` exists in project root
- **THEN** system loads and parses configuration

#### Scenario: Load .tsheetrc config file

- **WHEN** `.tsheetrc` exists in project root
- **THEN** system loads and parses configuration

#### Scenario: Load from package.json

- **WHEN** `package.json` contains `tsheet` field with configuration
- **THEN** system loads configuration from that field

#### Scenario: No config file

- **WHEN** no configuration file exists
- **THEN** system uses default configuration

### Requirement: Test directory configuration

The system SHALL configure the test file search directory.

#### Scenario: Custom test directory

- **WHEN** config sets `testDir: './e2e-tests'`
- **THEN** system discovers tests only in that directory

### Requirement: Test match patterns

The system SHALL configure test file matching patterns.

#### Scenario: Custom test match

- **WHEN** config sets `testMatch: ['**/*.spec.ts', '**/*.test.ts']`
- **THEN** system discovers files matching both patterns

### Requirement: Timeout configuration

The system SHALL configure default test timeout.

#### Scenario: Custom timeout

- **WHEN** config sets `timeout: 60000` (60 seconds)
- **THEN** tests timeout after 60 seconds instead of default 30

### Requirement: Retry configuration

The system SHALL configure test retry behavior.

#### Scenario: Enable retries

- **WHEN** config sets `retries: 2`
- **THEN** failed tests are retried up to 2 times

### Requirement: Worker count configuration

The system SHALL configure parallel worker count.

#### Scenario: Serial execution

- **WHEN** config sets `workers: 1`
- **THEN** tests execute serially

#### Scenario: Parallel execution

- **WHEN** config sets `workers: 4`
- **THEN** tests execute with up to 4 parallel workers

### Requirement: Reporter configuration

The system SHALL configure test reporter.

#### Scenario: Single reporter

- **WHEN** config sets `reporter: 'list'`
- **THEN** system uses list reporter

#### Scenario: Multiple reporters

- **WHEN** config sets `reporter: ['list', ['html', { outputFolder: './report' }]]`
- **THEN** system uses both reporters

### Requirement: Global setup and teardown

The system SHALL execute global setup/teardown scripts.

#### Scenario: Global setup

- **WHEN** config sets `globalSetup: './scripts/setup.ts'`
- **THEN** system executes the script before any tests

#### Scenario: Global teardown

- **WHEN** config sets `globalTeardown: './scripts/teardown.ts'`
- **THEN** system executes the script after all tests complete

### Requirement: Projects configuration

The system SHALL support multiple project configurations (like Playwright projects).

#### Scenario: Multiple browser projects

- **WHEN** config has projects targeting chromium and firefox
- **THEN** tests run against both browsers

#### Scenario: Multiple viewport projects

- **WHEN** config has projects with different viewports
- **THEN** tests run at each viewport size
