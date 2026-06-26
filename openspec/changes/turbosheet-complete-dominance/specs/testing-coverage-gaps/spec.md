## ADDED Requirements

### Requirement: Migration Adapter Unit Tests

The migration adapters for Puppeteer, Cypress, and Playwright SHALL have comprehensive unit test coverage for all string-replacement-based file transformations.

#### Scenario: Puppeteer adapter file conversion tests

- **GIVEN** the Puppeteer migration adapter in `src/migrate/puppeteer.rs`
- **WHEN** tests are executed
- **THEN** the test suite SHALL cover:
  - `migrate_puppeteer_file()` replaces imports correctly (require, import from puppeteer)
  - `migrate_puppeteer_file()` replaces `page.waitForSelector()` with auto-wait comment
  - `migrate_puppeteer_file()` replaces `page.waitForNavigation()` with auto-wait comment
  - `migrate_puppeteer_file()` replaces `page.type()` with `page.fill()`
  - `migrate_puppeteer_file()` does NOT replace `width:` / `height:` outside setViewport context
  - `migrate_puppeteer_file()` does NOT replace `type: 'png'` / `type: 'jpeg'` outside screenshot context
  - `migrate_puppeteer()` returns the expected set of API mappings
  - `migrate_puppeteer()` calculates coverage correctly
  - Full conversion of a realistic Puppeteer test script produces valid output
  - The `page.emulate()` mapping accuracy (`automated` flag matches actual replacement)

#### Scenario: Cypress adapter file conversion tests

- **GIVEN** the Cypress migration adapter in `src/migrate/cypress.rs`
- **WHEN** tests are executed
- **THEN** the test suite SHALL cover:
  - `migrate_cypress_file()` replaces imports correctly
  - `migrate_cypress_file()` produces syntactically valid output for `cy.get()` with single quotes
  - `migrate_cypress_file()` produces syntactically valid output for `cy.get()` with double quotes
  - `migrate_cypress_file()` produces syntactically valid output for `cy.contains()` with both quote styles
  - `migrate_cypress_file()` converts `cy.wait(2000)` to `page.waitForTimeout(2000)`
  - `migrate_cypress_file()` converts `cy.clear()` to `.fill('')`
  - `migrate_cypress_file()` replaces all `cy.should()` variants correctly
  - `migrate_cypress_file()` replaces all interceptor patterns correctly
  - Full conversion of a realistic Cypress test script produces valid output
  - `migrate_cypress_config()` converts config keys correctly

#### Scenario: Playwright adapter file conversion tests

- **GIVEN** the Playwright migration adapter in `src/migrate/playwright.rs`
- **WHEN** tests are executed
- **THEN** the test suite SHALL cover:
  - `migrate_playwright_file()` replaces imports correctly
  - `migrate_playwright_file()` replaces `page.waitForSelector()` correctly
  - `migrate_playwright_file()` replaces `page.$eval()` correctly
  - `migrate_playwright_file()` replaces `page.$$eval()` with an array-aware wrapper or comment
  - Full conversion of a realistic Playwright test script produces valid output

### Requirement: Error Handling Unit Tests

The error handling foundation SHALL have tests proving all conversion paths work correctly.

#### Scenario: TurbosheetError conversion tests

- **GIVEN** the error types in `src/error.rs`
- **WHEN** tested
- **THEN** the tests SHALL verify:
  - `From<serde_json::Error>` produces the correct variant with message
  - `From<url::ParseError>` produces the correct variant with details
  - `From<reqwest::Error>` produces the correct variant
  - The error round-trips through N-API correctly (Rust error → JsError → caught in JS)

#### Scenario: Panic hook tests

- **GIVEN** the panic hook is installed
- **WHEN** a Rust panic occurs in a N-API function
- **THEN** the test SHALL verify:
  - The Node process does NOT crash
  - A JavaScript error is thrown with the panic message
  - The error includes file and line information
  - The error is an instance of the expected error class

### Requirement: Event Delivery Tests

The event delivery system SHALL have tests for reliability under load.

#### Scenario: Broadcast channel overflow test

- **GIVEN** the network event interceptor
- **WHEN** events are produced faster than consumers can process them
- **THEN** the test SHALL verify:
  - No events are silently dropped (or if they are, a warning is logged)
  - The system degrades gracefully without panicking
  - The consumer eventually catches up when load subsides

#### Scenario: NonBlocking event dispatch test

- **GIVEN** the N-API event dispatch system
- **WHEN** events arrive while the JavaScript event loop is busy
- **THEN** the test SHALL verify:
  - Events are queued and eventually delivered
  - No events are lost during the busy period
  - The event order is preserved (or explicitly noted if not)

### Requirement: Close and Cleanup Tests

The system SHALL have tests verifying proper resource cleanup.

#### Scenario: Browser close actually kills process

- **GIVEN** a browser instance is launched
- **WHEN** `browser.close()` is called
- **THEN** the test SHALL verify:
  - The browser process is no longer running (check via OS process list or PID)
  - The CDP WebSocket connection is closed
  - No orphan processes remain after test completion
- **AND** SHALL work on Linux, macOS, and Windows

#### Scenario: Mount timeout test

- **GIVEN** a component mount is attempted on a page that does not render
- **WHEN** the mount operation is initiated
- **THEN** the test SHALL verify:
  - The mount fails within the configured timeout
  - A `TimeoutError` is returned (not an indefinite hang)
  - The error includes diagnostic information about page state
