## ADDED Requirements

### Requirement: AI Test Generation from Recordings

The system SHALL record user interactions during a manual QA session and generate test code from the recording.

#### Scenario: Record user session

- **GIVEN** `tsheet codegen --record --output tests/generated.spec.ts` is invoked
- **WHEN** a browser window opens
- **THEN** the system SHALL record all user interactions: clicks, keyboard input, navigation, scroll, hover
- **AND** record DOM snapshots at each interaction point (selector, text content, element state)
- **AND** record network requests and responses
- **AND** record screenshots at configurable intervals

#### Scenario: Interaction capture fidelity

- **WHEN** a user interacts with the browser
- **THEN** the system SHALL capture:
  - Each click: selector, coordinates, target element text, target element tag+attributes
  - Each keyboard input: target selector, characters typed, final value
  - Each navigation: from URL, to URL, timing
  - Each form submission: form selector, field values before submission
  - Hover events: selector, coordinates (for hover-triggered UIs)
  - Drag-and-drop: source selector, target selector
- **AND** deduplicate rapid interactions (e.g., mousemove events are grouped)

#### Scenario: Generate TurboSheet test code

- **WHEN** recording stops and `--output` is specified
- **THEN** the system SHALL generate idiomatic TurboSheet test code:
  - Navigate to the starting URL
  - Use locator-based selectors (preferring `getByRole`, `getByText`, `getByTestId` in that order)
  - Add `expect()` assertions for visible state changes
  - Include `await page.waitForLoadState()` after navigations
  - Group related interactions into logical test steps
  - Add comments for unclear interactions (user hesitation, multiple rapid clicks)
- **AND** the generated code SHALL be syntactically valid TypeScript
- **AND** follow TurboSheet's recommended test structure

#### Scenario: Generate Playwright-compatible code

- **GIVEN** `--format playwright` is specified
- **WHEN** test code is generated
- **THEN** the output SHALL use Playwright's API syntax:
  - `page.locator()` instead of TurboSheet's locator API
  - `page.getByRole()`, `page.getByText()` instead of TurboSheet equivalents
  - `expect(page).toHaveURL()` instead of TurboSheet assertions
- **AND** the generated code SHALL be valid Playwright test code

#### Scenario: Generate Cypress-compatible code

- **GIVEN** `--format cypress` is specified
- **WHEN** test code is generated
- **THEN** the output SHALL use Cypress's chaining API:
  - `cy.get()` instead of locator API
  - `cy.contains()` for text-based selection
  - `cy.url().should('contain')` for URL assertions
- **AND** the generated code SHALL be valid Cypress test code

#### Scenario: AI-powered test improvement

- **GIVEN** a recorded session with interaction data and DOM snapshots
- **WHEN** the test is generated
- **THEN** the system SHALL use the DOM snapshot data to:
  - Select the most robust selector strategy (prefer `getByRole` over CSS selectors)
  - Add reasonable wait/retry logic based on observed timing between interactions
  - Generate assertions based on observed state after each action
  - Identify form field types and generate appropriate fill/select interactions
  - Detect and exclude non-testable interactions (cursor movements between fields)
- **AND** the AI model SHALL run locally (no external API calls) using heuristic-based analysis

#### Scenario: Recording report

- **WHEN** a recording session ends
- **THEN** the system SHALL produce a recording report with:
  - Total interactions recorded
  - Test steps identified
  - Selectors used and their strategy (role, text, testid, css)
  - Generated test file path
  - Coverage estimate (% of recorded interactions converted to test code)
