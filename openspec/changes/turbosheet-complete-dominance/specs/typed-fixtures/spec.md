## ADDED Requirements

### Requirement: Typed Fixture System

The system SHALL support Playwright-style typed fixtures with automatic setup, teardown, and dependency injection.

#### Scenario: Defining typed fixtures

- **WHEN** `const test = base.extend<{ auth: AuthPage }>({ auth: async ({ page }, use) => { ... } })` is declared
- **THEN** the system SHALL register `auth` as a fixture that depends on `page`
- **AND** `use(authPage)` SHALL make the fixture value available to tests

#### Scenario: Fixture lifecycle

- **WHEN** a test uses a fixture
- **THEN** the system SHALL create the fixture before the test runs
- **AND** SHALL destroy/cleanup the fixture after the test completes (even if the test fails)
- **AND** SHALL create fixtures in dependency order (leaf fixtures first)

#### Scenario: Fixture scopes

- **WHEN** a fixture is defined with `scope: 'worker'`
- **THEN** the fixture SHALL be created once per worker process and shared across all tests in that worker
- **WHEN** a fixture is defined with `scope: 'test'` (default)
- **THEN** the fixture SHALL be created fresh for each test

#### Scenario: Fixture type inference

- **WHEN** `test.extend<{ auth: AuthPage }>(...)` is used
- **THEN** TypeScript SHALL infer the fixture types
- **AND** `test('my test', async ({ auth, page }) => { ... })` SHALL have autocomplete for both built-in and extended fixtures

#### Scenario: No fixture test

- **WHEN** a test is defined with `test('simple', async () => { ... })` without fixtures
- **THEN** the system SHALL run the test without any fixture setup overhead
- **AND** SHALL NOT create any browser pages or contexts
