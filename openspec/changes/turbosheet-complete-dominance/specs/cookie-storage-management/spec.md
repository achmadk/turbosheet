## ADDED Requirements

### Requirement: Cookie Management

The system SHALL support full cookie CRUD operations via CDP `Network.setCookies`, `Network.getCookies`, and `Network.deleteCookies`.

#### Scenario: Adding cookies

- **WHEN** `context.addCookies([{ name: 'session', value: 'abc', domain: '.example.com' }])` is called
- **THEN** the system SHALL execute CDP `Network.setCookies` with the provided cookies
- **AND** subsequent page navigations SHALL include the set cookies

#### Scenario: Reading cookies

- **WHEN** `context.cookies(['https://example.com'])` is called
- **THEN** the system SHALL execute CDP `Network.getCookies` with the provided URLs
- **AND** SHALL return an array of cookie objects with name, value, domain, path, expires, httpOnly, secure, sameSite

#### Scenario: Clearing cookies

- **WHEN** `context.clearCookies()` is called
- **THEN** the system SHALL execute CDP `Network.deleteCookies` for cookies matching provided URLs
- **AND** SHALL clear all cookies if no URLs are provided

### Requirement: Storage State Serialization

The system SHALL support serializing and restoring cookies, localStorage, and sessionStorage for authentication state reuse.

#### Scenario: Storage state export

- **WHEN** `context.storageState({ path: 'auth.json' })` is called
- **THEN** the system SHALL serialize cookies + localStorage + sessionStorage to the specified file
- **AND** SHALL return the state as an object if no path is provided

#### Scenario: Storage state import

- **WHEN** `browser.newContext({ storageState: 'auth.json' })` is called
- **THEN** the system SHALL restore cookies, localStorage, and sessionStorage from the file
- **AND** new pages in the context SHALL have the restored auth state
