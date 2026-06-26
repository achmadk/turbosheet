## ADDED Requirements

### Requirement: Test filtering with --grep

The test runner SHALL support filtering tests by name pattern using the `--grep` flag, matching against test names and describe block names.

#### Scenario: Grep filter runs matching tests

- **WHEN** `tsheet test --grep "login"` is executed
- **THEN** only tests whose name or containing describe block name matches the pattern "login" SHALL be executed

#### Scenario: Grep invert with --grep-invert

- **WHEN** `tsheet test --grep "slow" --grep-invert` is executed
- **THEN** tests matching "slow" SHALL be excluded

### Requirement: Project references for monorepo support

The test runner SHALL support `projects` configuration that defines multiple test projects within a monorepo, each with independent test directories, browsers, and viewport settings.

#### Scenario: Multiple projects run in parallel

- **WHEN** tsheet.config.ts defines two projects with different testDirs
- **THEN** both projects' tests SHALL be discovered and executed
- **AND** each project SHALL use its own configured browser and viewport

#### Scenario: Project-level configuration inheritance

- **WHEN** a project defines `testDir`, `browser`, and `viewport`
- **THEN** those settings SHALL override the global config for that project's tests only

### Requirement: Global setup and teardown

The test runner SHALL support `globalSetup` and `globalTeardown` configuration files that execute once per test run, before and after all tests.

#### Scenario: Global setup runs before any test

- **WHEN** tsheet.config.ts specifies `globalSetup: './setup.ts'`
- **THEN** the setup file SHALL be executed before any test file
- **AND** data returned by the setup function SHALL be available to all tests via a global fixture

#### Scenario: Global teardown runs after all tests

- **WHEN** tsheet.config.ts specifies `globalTeardown: './teardown.ts'`
- **THEN** the teardown file SHALL be executed after all tests complete, regardless of pass/fail status

### Requirement: Per-test timeout configuration

The test runner SHALL support configurable timeouts at the global, project, and individual test level.

#### Scenario: Timeout configuration hierarchy

- **WHEN** no timeout is specified
- **THEN** the default timeout of 30 seconds SHALL apply
- **WHEN** config sets `timeout: 60000`
- **THEN** all tests SHALL use 60s timeout
- **WHEN** `test.setTimeout(120000)` is called in a test
- **THEN** that specific test SHALL use 120s timeout

### Requirement: Reporter configuration

The test runner SHALL support multiple simultaneous reporters configured via config or CLI.

#### Scenario: Multiple reporters configured

- **WHEN** config specifies `reporter: [['html', { output: './report' }], ['json'], ['dot']]`
- **THEN** HTML, JSON, and dot reporters SHALL all produce output simultaneously

### Requirement: Full CLI parity

The `tsheet test` command SHALL support all CLI flags: `--reporter`, `--shard`, `--retries`, `--workers`, `--timeout`, `--grep`, `--grep-invert`, `--update-screenshots`, `--project`, `--list`, `--pass-with-no-tests`, `--forbid-only`.

#### Scenario: CLI flags are all functional

- **WHEN** each CLI flag from the required list is passed to `tsheet test`
- **THEN** it SHALL produce the expected behavior without errors
