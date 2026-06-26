## ADDED Requirements

### Requirement: Test-Level Fixtures

Fixtures defined with `test.extend()` SHALL be created fresh for each test and disposed after test completes.

#### Scenario: Fixture created per test

- **WHEN** `test.extend({ db: async ({}, use) => { ... } })` is defined
- **AND** two tests use the db fixture
- **THEN** each test SHALL receive its own db instance
- **AND** the db instance SHALL be disposed after each test

#### Scenario: Fixture disposal called

- **WHEN** a test using fixture completes (pass or fail)
- **THEN** the fixture cleanup callback SHALL be called

### Requirement: Worker-Level Fixtures

Fixtures with `scope: 'worker'` option SHALL be created once per worker process and shared across all tests in that worker.

#### Scenario: Worker fixture shared

- **WHEN** `test.extend({ db: async ({}, use) => { ... } }, { scope: 'worker' })` is defined
- **AND** 10 tests use the db fixture in same worker
- **THEN** the db instance SHALL be created once
- **AND** all 10 tests SHALL share the same db instance

#### Scenario: Worker fixture disposed on worker shutdown

- **WHEN** all tests in worker complete
- **THEN** the worker fixture cleanup SHALL be called

### Requirement: Global Fixtures

Fixtures with `scope: 'global'` option SHALL be created once and shared across all workers.

#### Scenario: Global fixture single instance

- **WHEN** `test.extend({ config: async ({}, use) => { ... } }, { scope: 'global' })` is defined
- **THEN** config fixture SHALL be created exactly once
- **AND** SHALL be available to all tests across all workers

### Requirement: Fixture Data Access

Fixtures SHALL be provided to test functions via the fixtures parameter.

#### Scenario: Access fixture in test

- **WHEN** test is defined with `test("name", async ({ db }) => { ... })`
- **AND** db fixture is registered
- **THEN** the test function SHALL receive the resolved db value

### Requirement: Fixture Dependencies

Fixtures with dependencies on other fixtures SHALL be resolved in topological order.

#### Scenario: Dependent fixture resolved after dependency

- **WHEN** fixture A depends on fixture B (fixture A's factory uses `await db`)
- **THEN** fixture B SHALL be created before fixture A

#### Scenario: Circular dependency error

- **WHEN** fixture A depends on B and fixture B depends on A
- **THEN** executor SHALL return error "Circular fixture dependency detected"
