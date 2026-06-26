# test-migration

## ADDED Requirements

### Requirement: Playwright test migration

The system SHALL convert Playwright test files to TurboSheet format.

#### Scenario: Migrate basic Playwright test

- **WHEN** user runs `tsheet migrate playwright input.spec.ts`
- **THEN** system generates equivalent TurboSheet test file

#### Scenario: Migrate page.click

- **WHEN** Playwright test contains `await page.click('#button')`
- **THEN** migrated code contains `await page.locator('#button').click()`

#### Scenario: Migrate page.fill

- **WHEN** Playwright test contains `await page.fill('input', 'text')`
- **THEN** migrated code contains `await page.locator('input').fill('text')`

#### Scenario: Migrate expect assertions

- **WHEN** Playwright test contains `await expect(page).toHaveTitle('Title')`
- **THEN** migrated code contains `await expect(page).toHaveTitle('Title')`

### Requirement: Cypress test migration

The system SHALL convert Cypress test files to TurboSheet format.

#### Scenario: Migrate cy.get

- **WHEN** Cypress test contains `cy.get('#button').click()`
- **THEN** migrated code contains `await page.locator('#button').click()`

#### Scenario: Migrate cy.contains

- **WHEN** Cypress test contains `cy.contains('text').click()`
- **THEN** migrated code contains `await page.locator('text').click()`

#### Scenario: Migrate cy.wrap

- **WHEN** Cypress test contains `cy.wrap(element).click()`
- **THEN** migrated code contains `await element.click()`

### Requirement: Puppeteer test migration

The system SHALL convert Puppeteer test files to TurboSheet format.

#### Scenario: Migrate page.goto

- **WHEN** Puppeteer test contains `await page.goto('url')`
- **THEN** migrated code contains `await page.goto('url')`

#### Scenario: Migrate page.click

- **WHEN** Puppeteer test contains `await page.click('#button')`
- **THEN** migrated code contains `await page.locator('#button').click()`

#### Scenario: Migrate page.evaluate

- **WHEN** Puppeteer test contains `await page.evaluate(() => document.title)`
- **THEN** migrated code contains `await page.evaluate('return document.title')`

### Requirement: Migration reporting

The system SHALL generate a report of migration results.

#### Scenario: Generate migration report

- **WHEN** user runs `tsheet migrate playwright ./tests --report`
- **THEN** system generates report showing:
  - Number of files migrated
  - Number of tests migrated
  - Lines of code converted
  - Manual review items needed

#### Scenario: Partial migration with manual items

- **WHEN** migration encounters unsupported patterns
- **THEN** system marks those as manual review items in report

### Requirement: Batch migration

The system SHALL migrate entire directories of test files.

#### Scenario: Migrate directory

- **WHEN** user runs `tsheet migrate playwright ./tests`
- **THEN** system migrates all `.spec.ts` files in directory recursively

### Requirement: Migration CLI

The system SHALL provide a CLI for migration operations.

#### Scenario: Show migration help

- **WHEN** user runs `tsheet migrate --help`
- **THEN** system displays usage information

#### Scenario: Validate source framework

- **WHEN** user runs `tsheet migrate unknown ./tests`
- **THEN** system returns error "Unknown framework 'unknown'. Use: playwright, cypress, puppeteer"
