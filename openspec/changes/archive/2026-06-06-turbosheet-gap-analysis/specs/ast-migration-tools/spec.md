## ADDED Requirements

### Requirement: AST-based Playwright to TurboSheet migration

The `tsheet migrate from playwright` command SHALL parse Playwright test files using SWC, transform the AST to TurboSheet API equivalents, and write valid TurboSheet test files. The existing API mapping table SHALL serve as the transformation rule set.

#### Scenario: Playwright test file conversion

- **WHEN** `tsheet migrate from playwright tests/example.spec.ts` is executed
- **THEN** the output SHALL be a valid TurboSheet test file with:
  - `import { test, expect } from 'turbo-sheet'` replacing `import { test, expect } from '@playwright/test'`
  - `page.goto()` calls preserved (same API)
  - `locator()` calls preserved (same API name)
  - `expect(locator).toBeVisible()` preserved (same API)
  - Playwright-specific `page.waitForLoadState()` → TurboSheet equivalent
  - Playwright-specific `page.waitForNavigation()` → TurboSheet equivalent
  - Playwright-specific `page.pause()` → TurboSheet `page.debug()` equivalent
  - `test.use()` preserved (same API)
  - `test.describe()` preserved (same API)
  - Fixture `test.extend()` calls updated to TurboSheet type

#### Scenario: Migration preserves test structure

- **WHEN** a Playwright test file is migrated
- **THEN** the output SHALL preserve: test names, describe blocks, assertion values, selector strings, URL strings, async/await patterns, hook order (beforeAll, afterAll, beforeEach, afterEach), and fixture definitions

#### Scenario: Migration reports unsupported patterns

- **WHEN** a Playwright test contains APIs without TurboSheet equivalents
- **THEN** the migration report SHALL list each unsupported pattern with file, line number, and a suggested manual migration approach

### Requirement: AST-based Cypress to TurboSheet migration

The `tsheet migrate from cypress` command SHALL parse Cypress test files and transform to TurboSheet API equivalents. This is more complex than Playwright migration because Cypress uses a chained `.then()` command pattern vs TurboSheet's imperative async/await.

#### Scenario: Cypress test file conversion

- **WHEN** `tsheet migrate from cypress tests/e2e/spec.cy.js` is executed
- **THEN** the output SHALL be a valid TurboSheet test file with:
  - `cy.visit(url)` → `page.goto(url)`
  - `cy.get(selector)` → `page.locator(selector)`
  - `cy.get(selector).click()` → `await page.locator(selector).click()`
  - `cy.contains(text)` → `page.getByText(text)`
  - `cy.url().should('include', str)` → `await expect(page).toHaveURL(new RegExp(str))`
  - `cy.contains(text).should('be.visible')` → `await expect(page.getByText(text)).toBeVisible()`
  - Chain unwrapping: `.then(($el) => ...)` → `const el = await page.locator(selector)`

### Requirement: AST-based Puppeteer to TurboSheet migration

The `tsheet migrate from puppeteer` command SHALL parse Puppeteer test files and transform to TurboSheet equivalents.

#### Scenario: Puppeteer test file conversion

- **WHEN** `tsheet migrate from puppeteer tests/puppeteer.js` is executed
- **THEN** the output SHALL be a valid TurboSheet test file with:
  - `puppeteer.launch()` → `turboSheet.launch()`
  - `page.goto(url)` preserved
  - `page.$eval(sel, fn)` → `page.evaluate(fn, page.locator(sel))`
  - `page.$(selector)` → `page.locator(selector)`
  - `page.$eval()` and `page.$$eval()` patterns unwrapped
  - `page.screenshot({path})` → `await page.screenshot({path})`

### Requirement: Batch directory migration

The migrate command SHALL support batch processing of entire directories, preserving the directory structure in the output.

#### Scenario: Directory migration preserves structure

- **WHEN** `tsheet migrate from playwright tests/ --output migrated-tests/` is executed
- **THEN** all .spec.ts files in tests/ SHALL be processed recursively
- **AND** the output directory SHALL mirror the input directory structure
- **AND** files with no transformation needed SHALL be copied as-is
