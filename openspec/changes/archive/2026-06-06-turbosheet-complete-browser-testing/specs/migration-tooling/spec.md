## ADDED Requirements

### Requirement: Playwright test detection

The system SHALL detect and parse Playwright test files.

#### Scenario: Identify Playwright test files

- **WHEN** `tsheet migrate --from=playwright ./tests/**/*.spec.ts` is called
- **THEN** system finds all `.spec.ts` files
- **AND** identifies Playwright test structure

#### Scenario: Parse test.describe blocks

- **WHEN** parsing `test.describe('suite', () => { ... })`
- **THEN** system extracts suite name and nested structure

#### Scenario: Parse test cases

- **WHEN** parsing `test('should work', async ({ page }) => { ... })`
- **THEN** system extracts test name and function

### Requirement: Playwright to TurboSheet conversion

The system SHALL convert Playwright API calls to TurboSheet equivalents.

#### Scenario: Convert page.goto

- **WHEN** `page.goto('url')` is found
- **THEN** convert to `page.goto('url')` (same API)

#### Scenario: Convert locator.click

- **WHEN** `page.locator('selector').click()` is found
- **THEN** convert to `page.locator('selector').click()` (same API)

#### Scenario: Convert expect assertions

- **WHEN** `expect(locator).toBeVisible()` is found
- **THEN** convert to `expect(locator).toBeVisible()` (same API)

#### Scenario: Convert expect assertions with different naming

- **WHEN** `expect(page).toHaveTitle('Title')` is found
- **THEN** convert to `expect(page).toHaveTitle('Title')` (same API)

### Requirement: Cypress test detection

The system SHALL detect and parse Cypress test files.

#### Scenario: Identify Cypress test files

- **WHEN** `tsheet migrate --from=cypress ./cypress/integration/**/*.cy.ts` is called
- **THEN** system finds all `.cy.ts` files
- **AND** identifies Cypress test structure

#### Scenario: Parse describe blocks

- **WHEN** parsing `describe('suite', () => { ... })` or `context('suite', () => { ... })`
- **THEN** system extracts suite name

#### Scenario: Parse it blocks

- **WHEN** parsing `it('should work', () => { ... })`
- **THEN** system extracts test name and function

### Requirement: Cypress to TurboSheet conversion

The system SHALL convert Cypress API calls to TurboSheet equivalents.

#### Scenario: Convert cy.get to locator

- **WHEN** `cy.get('selector')` is found
- **THEN** convert to `page.locator('selector')`

#### Scenario: Convert cy.contains to filter

- **WHEN** `cy.contains('text')` is found
- **THEN** convert to `page.locator('*').filter({ hasText: 'text' })`

#### Scenario: Convert cy.click to click

- **WHEN** `cy.get('selector').click()` is found
- **THEN** convert to `page.locator('selector').click()`

#### Scenario: Convert cy.type to fill

- **WHEN** `cy.get('input').type('text')` is found
- **THEN** convert to `page.locator('input').fill('text')`

#### Scenario: Convert cy.wait to waitForSelector

- **WHEN** `cy.wait(2000)` is found
- **THEN** convert to `page.waitForTimeout(2000)` or remove if unnecessary

### Requirement: Migration warnings

The system SHALL warn about features that need manual review.

#### Scenario: Unsupported feature warning

- **WHEN** `cy.intercept()` is found
- **THEN** system prints warning about manual network mocking setup
- **AND** adds TODO comment in output

#### Scenario: Potential behavior difference

- **WHEN** `cy.screenshot()` is found
- **THEN** system warns that TurboSheet screenshot API is different
- **AND** suggests manual review

### Requirement: Migration dry run

The system SHALL support dry run mode to preview conversions.

#### Scenario: Dry run without file write

- **WHEN** `tsheet migrate --from=playwright --dry-run ./tests/**/*.spec.ts` is called
- **THEN** system prints converted code to stdout
- **AND** does not write any files

#### Scenario: Statistics in dry run

- **WHEN** dry run completes
- **THEN** system prints summary
- **AND** shows conversion coverage percentage

### Requirement: Migration output

The system SHALL write TurboSheet-formatted test files.

#### Scenario: Write converted file

- **WHEN** migration completes for a file
- **THEN** system writes `*.tsheet.ts` file
- **AND** preserves directory structure

#### Scenario: Import statement handling

- **WHEN** file has `import` statements
- **THEN** system updates imports to TurboSheet
- **AND** removes Playwright-specific imports
