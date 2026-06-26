## ADDED Requirements

### Requirement: Playwright migration

TurboSheet SHALL provide an automated migration tool from Playwright test suites.

#### Scenario: Migrate Playwright project

- **WHEN** user runs `tsheet migrate from playwright`
- **THEN** TurboSheet SHALL scan for `playwright.config.ts` and convert it to `tsheet.config.ts`

#### Scenario: Convert test files

- **WHEN** user runs `tsheet migrate from playwright --tests ./e2e/`
- **THEN** all `.spec.ts` files SHALL be converted to `.tsheet.ts` equivalents

#### Scenario: API mapping

- **WHEN** converting Playwright tests
- **THEN** `page.locator()` SHALL map to `page.locator()`, `page.waitForSelector()` SHALL map to native await (which auto-waits), `page.$eval()` SHALL map to `page.evaluate()`

#### Scenario: Fixture migration

- **WHEN** Playwright project uses custom fixtures with `test.extend()`
- **THEN** fixtures SHALL be preserved in the TurboSheet format

### Requirement: Cypress migration

TurboSheet SHALL provide an automated migration tool from Cypress test suites.

#### Scenario: Migrate Cypress project

- **WHEN** user runs `tsheet migrate from cypress`
- **THEN** TurboSheet SHALL scan for `cypress.config.ts` and `cypress/e2e/` directory

#### Scenario: Convert Cypress syntax

- **WHEN** converting Cypress tests
- **THEN** `cy.visit()` SHALL map to `page.goto()`, `cy.get()` SHALL map to `page.locator()`, `cy.contains()` SHALL map to `page.getByText()`

#### Scenario: Convert Cypress assertions

- **WHEN** converting Cypress tests
- **THEN** `cy.get('.el').should('be.visible')` SHALL map to `await expect(page.locator('.el')).toBeVisible()`

#### Scenario: cy.intercept migration

- **WHEN** converting Cypress network interception
- **THEN** `cy.intercept('GET', '/api/users', mockData)` SHALL map to `page.route('**/api/users', handler)`

### Requirement: Puppeteer migration

TurboSheet SHALL provide an automated migration tool from Puppeteer scripts.

#### Scenario: Convert Puppeteer script

- **WHEN** user runs `tsheet migrate from puppeteer ./scrape.js`
- **THEN** Puppeteer API calls SHALL be converted to TurboSheet equivalents

#### Scenario: API mapping

- **WHEN** converting Puppeteer
- **THEN** `page.$()` SHALL map to `page.locator()`, `page.$$()` SHALL map to `page.locator().all()`, `page.waitForNavigation()` SHALL be dropped (auto-waiting), `page.waitForSelector()` SHALL be dropped (auto-waiting)

### Requirement: Migration report

The migration tool SHALL produce a report of what was migrated and what needs manual attention.

#### Scenario: Migration summary

- **WHEN** migration completes
- **THEN** a summary SHALL show: files converted, files skipped, manual-todo items, and API coverage percentage
