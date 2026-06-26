## ADDED Requirements

### Requirement: Puppeteer Adapter Correctness

The Puppeteer migration adapter SHALL produce correct output for all common Puppeteer APIs, with precise string replacements that do not corrupt unrelated code.

#### Scenario: Puppeteer emulateNetworkConditions has a mapping

- **GIVEN** `migrate_puppeteer()` returns a list of API mappings
- **WHEN** a user runs migration
- **THEN** `page.emulateNetworkConditions()` SHALL appear in the API mappings
- **AND** it SHALL be marked appropriately (automated or manual)

#### Scenario: Puppeteer setExtraHTTPHeaders has a mapping

- **GIVEN** `migrate_puppeteer()` returns a list of API mappings
- **WHEN** a user runs migration
- **THEN** `page.setExtraHTTPHeaders()` SHALL appear in the API mappings

#### Scenario: Puppeteer screencast has a mapping

- **GIVEN** `migrate_puppeteer()` returns a list of API mappings
- **WHEN** a user runs migration
- **THEN** `page.screencast()` SHALL appear in the API mappings

#### Scenario: screenshot type replacement is scoped to screenshot calls

- **GIVEN** `migrate_puppeteer_file()` processes a file with `type: 'png'` or `type: 'jpeg'`
- **WHEN** these strings appear outside `screenshot()` calls (e.g., in type annotations, variable values)
- **THEN** the replacement SHALL NOT corrupt unrelated code
- **AND** SHALL only comment out these properties when they appear inside `screenshot()` configuration

#### Scenario: viewport property replacements are scoped to setViewport calls

- **GIVEN** `migrate_puppeteer_file()` processes a file with `width:`, `height:`, or `deviceScaleFactor:` properties
- **WHEN** these strings appear outside `setViewport()` calls
- **THEN** the replacement SHALL NOT corrupt unrelated code (common variable names like `width` and `height` appear everywhere)
- **AND** SHALL only comment out these properties when they appear inside `setViewport()` configuration

#### Scenario: page.emulate() automated flag is accurate

- **GIVEN** `migrate_puppeteer()` returns an API mapping for `page.emulate()`
- **WHEN** the mapping is marked `automated: true`
- **THEN** `migrate_puppeteer_file()` SHALL actually perform a string replacement for this API
- **AND** SHALL NOT claim automation without a corresponding replacement

### Requirement: Cypress Adapter Correctness

The Cypress migration adapter SHALL produce syntactically valid output and convert common Cypress patterns correctly.

#### Scenario: cy.get() preserves quote style

- **GIVEN** `migrate_cypress_file()` processes a file with `cy.get()`
- **WHEN** the selector uses single quotes: `cy.get('.my-class')`
- **THEN** the output SHALL be valid JavaScript with matching quotes: `await page.locator('.my-class')`
- **WHEN** the selector uses double quotes: `cy.get(".my-class")`
- **THEN** the output SHALL be valid JavaScript with matching quotes: `await page.locator(".my-class")`
- **AND** the adapter SHALL NOT produce mismatched quote pairs (e.g., `await page.locator(".my-class')`)

#### Scenario: cy.contains() preserves quote style

- **GIVEN** `migrate_cypress_file()` processes a file with `cy.contains()`
- **WHEN** the text uses single quotes
- **THEN** the output SHALL have matching quotes
- **AND** SHALL NOT produce quote mismatches

#### Scenario: cy.wait() is converted to page.waitForTimeout()

- **GIVEN** `migrate_cypress_file()` processes a file with `cy.wait(milliseconds)`
- **WHEN** the wait has a numeric argument
- **THEN** the output SHALL convert to `await page.waitForTimeout(milliseconds)`
- **WHEN** the wait has a selector argument (implicit waiting)
- **THEN** the output SHALL add a comment explaining auto-waiting

#### Scenario: cy.intercept callback conversion is syntactically valid

- **GIVEN** `migrate_cypress_file()` processes a file with `cy.intercept()` using callback pattern
- **WHEN** the replacement converts `}, (req, res) => {` to the route pattern
- **THEN** the output SHALL be syntactically valid JavaScript
- **AND** SHALL NOT produce mismatched braces or broken function signatures

### Requirement: Playwright Adapter Correctness

The Playwright migration adapter SHALL correctly convert Playwright APIs to TurboSheet equivalents.

#### Scenario: $$eval conversion preserves array semantics

- **GIVEN** `migrate_playwright_file()` processes a file with `page.$$eval()`
- **WHEN** converting to `page.evaluate()`
- **THEN** the output SHALL note that `$$eval` returns an array and `evaluate` expects a different signature
- **AND** SHALL NOT silently drop the array dimension

#### Scenario: page.addInitScript has a mapping

- **GIVEN** `migrate_playwright()` returns API mappings
- **WHEN** a user runs migration
- **THEN** `page.addInitScript()` SHALL appear in the mappings

#### Scenario: page.route has correct automated flag

- **GIVEN** `migrate_playwright()` returns API mappings
- **WHEN** checking the `page.route()` mapping
- **THEN** it SHALL be marked `automated: true` (as TurboSheet supports route interception)
