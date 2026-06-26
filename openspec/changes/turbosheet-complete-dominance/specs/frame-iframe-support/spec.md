## ADDED Requirements

### Requirement: Frame and IFrame Support

The system SHALL support inspecting and interacting with iframes and nested browsing contexts.

#### Scenario: Listing frames on a page

- **WHEN** `page.frames()` is called
- **THEN** the system SHALL return a list of all frames on the page, including the main frame
- **AND** each frame SHALL include its name, URL, and execution context ID

#### Scenario: Frame access by name

- **WHEN** `page.frame('sidebar')` is called
- **THEN** the system SHALL return the Frame object with that name
- **AND** SHALL return null if no frame with that name exists

#### Scenario: Frame locator

- **WHEN** `page.frameLocator('iframe[src="/app"]')` is called
- **THEN** the system SHALL return a `FrameLocator` object scoped to matching iframes
- **AND** subsequent locator actions SHALL execute within that frame's context

#### Scenario: Cross-origin iframe interaction

- **WHEN** interacting with an element inside a cross-origin iframe
- **THEN** the system SHALL switch to the iframe's execution context
- **AND** SHALL execute all CDP commands within that context
- **AND** SHALL correctly handle cross-origin DOM access restrictions

#### Scenario: Nested iframe support

- **WHEN** a page contains iframes within iframes
- **THEN** the system SHALL track the nesting hierarchy
- **AND** `frameLocator` SHALL support chaining for nested frames: `page.frameLocator('#outer').frameLocator('#inner')`
