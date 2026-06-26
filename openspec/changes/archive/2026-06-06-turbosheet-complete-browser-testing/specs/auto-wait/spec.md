## ADDED Requirements

### Requirement: Visibility wait condition

The system SHALL wait for an element to be visible before performing actions.

#### Scenario: Wait for visible element

- **WHEN** `page.locator('.loading').click()` is called on a hidden element
- **THEN** system polls `isVisible()` until element becomes visible
- **OR** times out after configured timeout

#### Scenario: Visibility check criteria

- **WHEN** checking if element is visible
- **THEN** element exists in DOM
- **AND** `display !== 'none'`
- **AND** `visibility !== 'hidden'`
- **AND** `opacity !== '0'`
- **AND** element has non-zero bounding box

### Requirement: Stability wait condition

The system SHALL wait for an element to stop moving before performing actions.

#### Scenario: Wait for stable element

- **WHEN** `page.locator('.animating').click()` is called on an animating element
- **THEN** system uses MutationObserver to detect DOM changes
- **AND** waits for 2 consecutive stable checks (100ms apart)

#### Scenario: Stable means no mutation

- **WHEN** element has no DOM mutations for 200ms
- **THEN** element is considered stable

### Requirement: Actionability wait condition

The system SHALL wait for an element to be actionable (visible, enabled, stable) before clicking.

#### Scenario: Wait for actionable element

- **WHEN** `page.locator('button').click()` is called
- **THEN** system waits for element to be visible
- **AND** waits for element to be enabled
- **AND** waits for element to be stable
- **AND** waits for element not to be covered

#### Scenario: Enabled check

- **WHEN** checking if element is enabled
- **THEN** element does not have `disabled` attribute
- **AND** element does not have `aria-disabled="true"`

#### Scenario: Not covered check

- **WHEN** clicking element that is covered by another element
- **THEN** system detects covering element via `document.elementFromPoint`
- **AND** waits for covering element to be removed or moved

### Requirement: Custom wait function

The system SHALL support user-defined wait conditions via `waitForFunction`.

#### Scenario: Wait for custom function

- **WHEN** `page.waitForFunction('() => document.querySelectorAll(".item").length > 5')` is called
- **THEN** system polls function until it returns truthy
- **OR** times out

#### Scenario: Wait with arguments

- **WHEN** `page.waitForFunction('(n) => n > 5', 10)` is called
- **THEN** system passes arguments to the function
- **AND** polls with those arguments

### Requirement: Exponential backoff

The system SHALL use exponential backoff with jitter for wait condition polling.

#### Scenario: Backoff strategy

- **WHEN** polling for a condition
- **THEN** initial interval is 100ms
- **AND** max interval is 5000ms
- **AND** multiplier is 1.5x
- **AND** jitter is ±10%

#### Scenario: Backoff with timeout

- **WHEN** condition is not met and timeout is reached
- **THEN** system throws error with clear message
- **AND** includes last evaluated value in error

### Requirement: Auto-wait configuration

The system SHALL allow configuring auto-wait behavior per action or globally.

#### Scenario: Per-action timeout

- **WHEN** `locator.click({ timeout: 10000 })` is called
- **THEN** system uses 10 second timeout for this action only

#### Scenario: Global default timeout

- **WHEN** `browser.launch({ timeout: 30000 })` is called
- **THEN** all actions use 30 second default timeout

#### Scenario: Disable auto-wait

- **WHEN** `locator.click({ force: true })` is called
- **THEN** system performs action without waiting
- **AND** bypasses actionability checks
