# real-test-execution

## ADDED Requirements

### Requirement: Test file discovery

The system SHALL discover test files matching configured patterns from the project directory.

#### Scenario: Discover default test files

- **WHEN** user runs `tsheet test` with default configuration
- **THEN** system discovers files matching `**/*.tsheet.ts` and `**/*.tsheet.spec.ts`

#### Scenario: Discover with custom pattern

- **WHEN** user configures `testMatch: ['**/*.test.ts']`
- **THEN** system discovers files matching the custom pattern

### Requirement: Test file parsing

The system SHALL parse test files to extract test cases, describe blocks, and hooks.

#### Scenario: Parse simple test

- **WHEN** system parses a file containing `test('name', async () => { ... })`
- **THEN** system extracts test name, function, and location

#### Scenario: Parse nested describes

- **WHEN** system parses a file with nested `describe()` blocks
- **THEN** system creates hierarchical test structure with correct scoping

#### Scenario: Parse hooks

- **WHEN** system parses a file with `beforeAll`, `afterAll`, `beforeEach`, `afterEach`
- **THEN** system associates hooks with correct describe blocks

### Requirement: Test execution

The system SHALL execute tests and report results with correct status.

#### Scenario: Execute passing test

- **WHEN** test function completes without error
- **THEN** system reports test status as 'passed'

#### Scenario: Execute failing test

- **WHEN** test function throws an error
- **THEN** system reports test status as 'failed' with error message

#### Scenario: Execute skipped test

- **WHEN** test is marked with `test.skip()` or `test.skip()`
- **THEN** system reports test status as 'skipped'

### Requirement: Parallel execution

The system SHALL execute tests in parallel using configured worker count.

#### Scenario: Single worker execution

- **WHEN** user runs with `workers: 1`
- **THEN** system executes tests sequentially

#### Scenario: Multiple worker execution

- **WHEN** user runs with `workers: 4`
- **THEN** system executes tests using 4 parallel workers

### Requirement: Test timeout

The system SHALL enforce test timeouts and mark timed-out tests as failed.

#### Scenario: Test timeout

- **WHEN** test execution exceeds configured timeout (default 30s)
- **THEN** system terminates test and reports status as 'timeout'

### Requirement: Test retry

The system SHALL retry failed tests the configured number of times.

#### Scenario: Retry failed test

- **WHEN** test fails and `retries: 2` is configured
- **THEN** system retries test up to 2 additional times before marking as failed

### Requirement: Worker process isolation

The system SHALL isolate test execution in separate processes to prevent state leakage.

#### Scenario: Process cleanup after test

- **WHEN** test completes (pass or fail)
- **THEN** system cleans up the worker process and its JS runtime

#### Scenario: Signal handling

- **WHEN** user sends SIGINT/Ctrl+C during test execution
- **THEN** system gracefully stops all workers and reports partial results

### Requirement: Global setup and teardown

The system SHALL execute global setup before any tests and global teardown after all tests complete.

#### Scenario: Global setup runs first

- **WHEN** `globalSetup` is configured in tsheet.config.ts
- **THEN** system executes the setup script once before any test files

#### Scenario: Global teardown runs last

- **WHEN** `globalTeardown` is configured in tsheet.config.ts
- **THEN** system executes the teardown script once after all tests complete
