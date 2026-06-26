## ADDED Requirements

### Requirement: Smart Selectors Support

The system SHALL support Playwright-style smart selector prefixes, including `text=`, `role=`, and `data-testid=`.

#### Scenario: Querying by text

- **WHEN** a locator is created with `text=Submit`
- **THEN** the selector engine MUST locate elements containing the exact or substring text "Submit"

#### Scenario: Querying by ARIA role

- **WHEN** a locator is created with `role=button[name="Submit"]`
- **THEN** the selector engine MUST locate elements matching the ARIA role and accessible name

### Requirement: Locator Composition and Filtering

The system SHALL support chaining locators and applying filters to narrow down selections.

#### Scenario: Filtering locators

- **WHEN** `.filter({ hasText: 'Premium' })` is called on a locator
- **THEN** the resulting locator MUST only match elements that contain the specified text

### Requirement: Strict Mode

The system SHALL enforce strict mode for all locator actions, failing if multiple elements match the selector.

#### Scenario: Ambiguous click

- **WHEN** a `.click()` action is performed on a locator resolving to 2 or more elements
- **THEN** the system MUST throw a `TurbosheetError::ElementNotUnique` exception
