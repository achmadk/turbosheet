## ADDED Requirements

### Requirement: Multi-Page and Popup Support

The system SHALL detect and support interaction with popup windows and multiple pages within a browser context.

#### Scenario: Popup detection

- **WHEN** a user action (e.g., clicking a link with `target="_blank"`) opens a new page
- **THEN** the system SHALL detect the popup via CDP `Target.targetCreated`
- **AND** SHALL add the new page to the browser context's page list

#### Scenario: Waiting for a popup

- **WHEN** `context.waitForEvent('page')` is called before clicking a link that opens a popup
- **THEN** the system SHALL wait up to the configured timeout for a new page to appear
- **AND** SHALL resolve with the new Page object when created

#### Scenario: Page opener tracking

- **WHEN** a popup is created
- **THEN** the popup page SHALL have an `opener()` method returning the page that created it
- **AND** the parent page's `context().pages()` SHALL include the popup

#### Scenario: Popup interaction

- **WHEN** a popup page is obtained via `waitForEvent('page')` or from `context.pages()`
- **THEN** the system SHALL support all locator and page actions on the popup
- **AND** SHALL properly manage the popup's lifecycle (close, navigation, dialog handling)
