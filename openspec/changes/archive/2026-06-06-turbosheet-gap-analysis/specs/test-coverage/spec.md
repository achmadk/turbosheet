## ADDED Requirements

### Requirement: CLI command tests

Every `tsheet` CLI command SHALL have integration tests that verify correct behavior end-to-end, including error handling.

#### Scenario: tsheet test with no tests exits gracefully

- **WHEN** `tsheet test` is run in a directory with no test files
- **THEN** it SHALL exit with a clear message: "No tests found" and exit code 0

#### Scenario: tsheet migrate reports correct coverage

- **WHEN** `tsheet migrate from playwright` is run on a known Playwright test file
- **THEN** the migration report SHALL list all converted APIs and any unsupported patterns

#### Scenario: tsheet show-trace renders viewer

- **WHEN** `tsheet show-trace test.trace` is run with a valid trace file
- **THEN** it SHALL launch the embedded HTML trace viewer successfully

### Requirement: Migration tool tests

The migration module SHALL have unit tests and integration tests verifying that each source framework's API calls are correctly transformed.

#### Scenario: Playwright import statement transformation

- **WHEN** the migration tool processes a file with `import { test, expect } from '@playwright/test'`
- **THEN** the output SHALL contain `import { test, expect } from 'turbosheet'`

#### Scenario: Cypress chain unwrapping produces valid TS

- **WHEN** the migration tool processes `cy.get('.btn').click()`
- **THEN** the output SHALL contain `await page.locator('.btn').click()`

### Requirement: Visual comparison tests

The visual comparison module (`src/visual/`) SHALL have tests verifying pixel diff, SSIM comparison, and baseline management.

#### Scenario: Identical images produce zero diff

- **WHEN** `PixelDiff::compare(same_image, same_image)` is called
- **THEN** the diff percentage SHALL be 0.0

#### Scenario: Different images produce non-zero diff

- \*\*WHEN` PixelDiff::compare(image_a, image_b)` is called with different images
- **THEN** the diff percentage SHALL be > 0.0
- **AND** the diff image SHALL be a valid PNG

### Requirement: Trace module tests

The trace recorder, serializer, and viewer SHALL have unit tests.

#### Scenario: Trace round-trip serialize/deserialize

- **WHEN** events are recorded and serialized to bytes, then deserialized
- **THEN** the deserialized events SHALL match the original events

#### Scenario: Trace viewer HTML is valid

- **WHEN** `TraceViewer::generate_html()` is called
- **THEN** the output SHALL be valid HTML with embedded CSS and JS

### Requirement: Swarm module unit tests

The swarm module SHALL have unit tests for the controller, job queue, and state manager.

#### Scenario: Job queue processing order

- **WHEN** jobs are submitted to the GridController
- **THEN** they SHALL be processed in FIFO order
- **AND** high-priority jobs SHALL be processed before normal jobs

### Requirement: Component testing tests

The component mount module SHALL have tests for framework detection and iframe injection.

#### Scenario: Framework auto-detection from package.json

- **WHEN** `FrameworkType::detect()` is called in a project with React in package.json
- **THEN** it SHALL return `FrameworkType::React`

### Requirement: Network interception tests

The network proxy and interceptor modules SHALL have integration tests.

#### Scenario: Route fulfillment returns mock response

- **WHEN** `page.route('**/api/data', handler)` is registered and the page makes a matching request
- **THEN** the request SHALL receive the mock response defined in the handler

### Requirement: TypeScript runtime layer tests

The `runtime/` TypeScript files SHALL have unit tests verifying test runner behavior, worker entry points, and JS runtime isolation.

#### Scenario: Test runner parses test file correctly

- **WHEN** the test runner processes a valid .ts file with `test()` and `describe()` calls
- **THEN** it SHALL extract all test names, describe block hierarchy, and hooks correctly
