## ADDED Requirements

### Requirement: Locator Visibility Assertion

The assertion engine SHALL evaluate element visibility by calling the page's `is_visible()` method which executes CDP's `document.querySelector()` and `getComputedStyle()` to determine actual visibility state.

#### Scenario: Visible element passes

- **WHEN** `expect(locator).toBeVisible()` is called on a visible element
- **THEN** the assertion SHALL pass and return Ok(())

#### Scenario: Hidden element fails with descriptive error

- **WHEN** `expect(locator).toBeVisible()` is called on a hidden element
- **THEN** the assertion SHALL fail with error message containing selector and visibility state

#### Scenario: Non-existent element fails

- **WHEN** `expect(locator).toBeVisible()` is called with selector matching no elements
- **THEN** the assertion SHALL fail with ElementNotFound error

### Requirement: Locator Text Assertion

The assertion engine SHALL evaluate element text content by calling the page's `text_content()` method which executes CDP's `innerText()`.

#### Scenario: Exact text match passes

- **WHEN** `expect(locator).toHaveText("exact content")` is called
- **THEN** the assertion SHALL pass when element's innerText exactly equals "exact content"

#### Scenario: Regex text match passes

- **WHEN** `expect(locator).toHaveText(RegExp { pattern: "prefix.*suffix" })` is called
- **THEN** the assertion SHALL pass when element's innerText matches the regex

#### Scenario: Text mismatch fails with diff

- **WHEN** `expect(locator).toHaveText("expected")` is called but actual is "different"
- **THEN** the assertion SHALL fail with error showing expected vs actual text

### Requirement: Page URL Assertion

The assertion engine SHALL evaluate page URL by calling the page's `url()` method.

#### Scenario: Exact URL match passes

- **WHEN** `expect(page).toHaveURL("https://example.com/path")` is called
- **THEN** the assertion SHALL pass when page URL exactly equals the expected URL

#### Scenario: Glob pattern URL match passes

- **WHEN** `expect(page).toHaveURL("**/example.com/**")` is called
- **THEN** the assertion SHALL pass when URL matches the glob pattern

#### Scenario: Regex URL match passes

- **WHEN** `expect(page).toHaveURL(RegExp { pattern: ".*\\/dashboard\\?.*" })` is called
- **THEN** the assertion SHALL pass when URL matches the regex

### Requirement: Exponential Backoff Polling

The assertion engine SHALL poll conditions with exponential backoff, starting at `initial_interval` and doubling up to `max_interval` until `timeout`.

#### Scenario: Condition met on first poll

- **WHEN** condition becomes true immediately
- **THEN** assertion SHALL return Ok(()) without additional polls

#### Scenario: Condition met after retries

- **WHEN** condition is false initially but becomes true after 3 polls
- **THEN** assertion SHALL return Ok(()) after successful poll

#### Scenario: Timeout expires

- **WHEN** condition never becomes true and timeout is reached
- **THEN** assertion SHALL return Err with timeout message and last error

### Requirement: Assertion Timeout Configuration

Each assertion SHALL accept an optional `MatcherConfig` with `timeout` and `polling_interval`.

#### Scenario: Custom timeout used

- **WHEN** `expect(locator).toBeVisible({ timeout: 10000 })` is called
- **THEN** the assertion SHALL poll for up to 10000ms before timeout

#### Scenario: Default timeout when not specified

- **WHEN** `expect(locator).toBeVisible()` is called without config
- **THEN** the assertion SHALL use default timeout of 5000ms
