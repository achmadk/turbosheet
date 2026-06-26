## ADDED Requirements

### Requirement: Test file discovery

The system SHALL discover test files matching patterns `**/*.tsheet.ts` and `**/*.tsheet.spec.ts` from the test directory.

#### Scenario: Discover tsheet.ts files

- **WHEN** test discovery runs on directory `./tests`
- **THEN** system finds all files matching `**/*.tsheet.ts` pattern
- **AND** system finds all files matching `**/*.tsheet.spec.ts` pattern

#### Scenario: Discover with custom patterns

- **WHEN** test discovery runs with patterns `["**/*.spec.ts", "**/*.test.ts"]`
- **THEN** system finds all files matching each specified pattern

### Requirement: Test file parsing

The system SHALL parse TypeScript/JavaScript test files and extract test structure including describe blocks, test cases, and hooks.

#### Scenario: Parse describe blocks

- **WHEN** parsing a file containing `test.describe('suite', () => { ... })`
- **THEN** system extracts suite name 'suite' and nested tests

#### Scenario: Parse test cases

- **WHEN** parsing a file containing `test('should login', async () => { ... })`
- **THEN** system extracts test name 'should login' and function body

#### Scenario: Parse hooks

- **WHEN** parsing a file containing `test.beforeEach(() => { ... })`
- **THEN** system extracts hook type 'beforeEach' and function body

### Requirement: Isolated-v8 test execution

The system SHALL execute test code within an isolated v8 context with TurboSheet globals injected.

#### Scenario: Execute test with page global

- **WHEN** test code calls `page.goto('https://example.com')`
- **THEN** system navigates to the URL in a real browser
- **AND** returns Promise that resolves when navigation completes

#### Scenario: Execute test with expect API

- **WHEN** test code calls `expect(page.locator('.error')).toBeVisible()`
- **THEN** system checks element visibility via real CDP
- **AND** throws AssertionError if element is not visible

#### Scenario: Test timeout handling

- **WHEN** test execution exceeds configured timeout
- **THEN** system terminates the isolate
- **AND** marks test as Timeout status

### Requirement: Fixture lifecycle

The system SHALL support test fixtures with proper scoping and cleanup.

#### Scenario: Test-level fixture

- **WHEN** fixture is defined with `test('name', async ({ page }) => { ... })`
- **THEN** fixture is created fresh for each test
- **AND** cleaned up after test completes

#### Scenario: Worker-level fixture

- **WHEN** fixture is defined with `test.extend({ pool: async () => { ... } })`
- **THEN** fixture is created once per worker process
- **AND** shared across all tests in the worker

#### Scenario: Global setup/teardown

- **WHEN** config specifies `globalSetup` script
- **THEN** system executes script once before all tests
- **AND** waits for completion before starting test execution

### Requirement: Hook execution

The system SHALL execute beforeAll, afterAll, beforeEach, and afterEach hooks with proper scoping.

#### Scenario: beforeAll hook

- **WHEN** `test.beforeAll()` is defined in a describe block
- **THEN** hook runs once before all tests in that block
- **AND** tests in the block wait for hook completion

#### Scenario: afterEach hook

- **WHEN** `test.afterEach()` is defined
- **THEN** hook runs after each test in the block
- **AND** runs even if test fails

### Requirement: Test result collection

The system SHALL collect and report test results including status, duration, and error messages.

#### Scenario: Collect passed test

- **WHEN** test executes without errors
- **THEN** system records status as Passed
- **AND** records actual duration in milliseconds

#### Scenario: Collect failed test with error

- **WHEN** test throws an error or assertion fails
- **THEN** system records status as Failed
- **AND** captures error message and stack trace

#### Scenario: Collect screenshot on failure

- **WHEN** config has `screenshotOnFailure: true`
- **AND** test fails
- **THEN** system captures screenshot
- **AND** attaches path to test result
