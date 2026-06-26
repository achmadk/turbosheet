## ADDED Requirements

### Requirement: Interactive test recording

TurboSheet SHALL record user browser interactions and generate executable test files.

#### Scenario: Start recording

- **WHEN** user runs `tsheet codegen --url https://example.com`
- **THEN** TurboSheet SHALL open a browser window with a recording overlay

#### Scenario: Record click actions

- **WHEN** user clicks on elements in the recording browser
- **THEN** TurboSheet SHALL record the click with a robust selector (preferring `getByRole`, `getByText`, `getByTestId`)

#### Scenario: Record form input

- **WHEN** user types into form fields
- **THEN** TurboSheet SHALL record the `fill()` action with the input value

#### Scenario: Generate test file

- **WHEN** user stops recording
- **THEN** TurboSheet SHALL generate a `.tsheet.ts` file with recorded actions as assertions between steps

### Requirement: Smart selector generation

The codegen SHALL prioritize accessible selectors over fragile CSS selectors.

#### Scenario: getByRole priority

- **WHEN** user clicks a `<button role="tab" name="Settings">`
- **THEN** TurboSheet SHALL generate `page.getByRole('tab', { name: 'Settings' })` instead of a CSS selector

#### Scenario: getByTestId fallback

- **WHEN** element has a `data-testid` attribute but no accessible role
- **THEN** TurboSheet SHALL generate `page.getByTestId('submit-button')`

### Requirement: Assertion suggestions

The codegen SHALL suggest assertions based on page state after each action.

#### Scenario: Assertion after navigation

- **WHEN** user navigates to a new page
- **THEN** codegen SHALL suggest `expect(page).toHaveURL('...')` and `expect(page.locator('h1')).toHaveText('...')`

#### Scenario: Assertion after form submit

- **WHEN** user submits a form and a success toast appears
- **THEN** codegen SHALL suggest `expect(page.getByText('Success')).toBeVisible()`
