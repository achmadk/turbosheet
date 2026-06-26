## ADDED Requirements

### Requirement: Code Generation from Browser Interactions

The system SHALL support recording user interactions and generating test code in multiple framework syntaxes.

#### Scenario: Starting code recording

- **WHEN** `npx tsheet codegen` is run
- **THEN** the system SHALL open a browser window in headed mode
- **AND** SHALL start recording all user interactions (clicks, navigation, form fills, assertions)
- **AND** SHALL display the generated code in real-time as interactions happen

#### Scenario: Interaction capture

- **WHEN** a user clicks an element during code recording
- **THEN** the system SHALL generate the appropriate locator (`page.getByText()`, `page.getByRole()`, etc.)
- **AND** SHALL add a `click()` action to the generated code
- **WHEN** a user types into an input field
- **THEN** the system SHALL generate a `fill()` or `pressSequentially()` action

#### Scenario: Multi-framework code generation

- **WHEN** the user selects a target framework (TurboSheet, Playwright, or Cypress)
- **THEN** the system SHALL generate code in the selected framework's API syntax
- **AND** SHALL use the migration engine in reverse for Playwright/Cypress output

#### Scenario: Assertion recording

- **WHEN** a user right-clicks an element and selects "assert visibility" from the context menu
- **THEN** the system SHALL add `expect(locator).toBeVisible()` to the generated code
- **AND** SHALL support other assertion types (text content, attribute value, count, enabled state)

#### Scenario: Code export

- **WHEN** the user finishes recording
- **THEN** the system SHALL allow saving the generated code to a file
- **AND** SHALL copy the generated code to clipboard
