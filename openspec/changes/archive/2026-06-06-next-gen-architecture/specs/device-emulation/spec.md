## ADDED Requirements

### Requirement: Pre-configured Device Descriptors

The system SHALL provide a set of common device profiles (e.g., iPhone 13, Pixel 5) that automatically configure viewport, user-agent, and touch support.

#### Scenario: Emulating a mobile device

- **WHEN** a user instantiates a page with the "iPhone 13" descriptor
- **THEN** the browser context automatically applies mobile viewport dimensions, scaling, and user-agent string

### Requirement: Browser Context Isolation

The system SHALL ensure that each new page or context operates in strict isolation, lacking shared cookies, local storage, or caches with other contexts.

#### Scenario: Launching independent contexts

- **WHEN** multiple tests run in parallel
- **THEN** each test uses an isolated context so that login states do not bleed between tests
