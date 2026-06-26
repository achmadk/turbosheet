## ADDED Requirements

### Requirement: Expanded Matcher Library

The system SHALL provide a comprehensive set of web-first assertions, including `toHaveCount`, `toBeVisible`, `toHaveClass`, and `toBeChecked`.

#### Scenario: Asserting element count

- **WHEN** `expect(locator).toHaveCount(3)` is evaluated
- **THEN** the system MUST poll the DOM until exactly 3 matching elements are found or the timeout expires

### Requirement: Assertion Negation

The system SHALL support logical negation for all matchers via a `.not` modifier.

#### Scenario: Asserting an element is hidden

- **WHEN** `expect(locator).not.toBeVisible()` is evaluated
- **THEN** the system MUST poll until the element is either removed from the DOM or hidden via CSS

### Requirement: Context-Rich Error Messages

The system SHALL output detailed error messages upon assertion failure, including expected/actual values, the selector used, and custom user messages.

#### Scenario: Matcher failure output

- **WHEN** an assertion fails after the timeout
- **THEN** the error MUST clearly display the locator string, the expected value, the actual value received during the final poll, and any custom message provided
