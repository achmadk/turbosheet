## ADDED Requirements

### Requirement: API + UI Integration Testing

The system SHALL provide an embedded HTTP client within the test framework for combined API and UI testing.

#### Scenario: Making API requests

- **WHEN** `test.request.get('https://api.example.com/users')` is called
- **THEN** the system SHALL make an HTTP GET request using the embedded client
- **AND** SHALL return the response with status, headers, and body

#### Scenario: API request methods

- **WHEN** `test.request.post(url, { data: body })`, `.put()`, `.patch()`, `.delete()` are called
- **THEN** the system SHALL make the corresponding HTTP request
- **AND** SHALL support JSON, form-encoded, and plain text body types

#### Scenario: Response assertions

- **WHEN** an API response is returned
- **THEN** the system SHALL support: `expect(response.status()).toBe(200)`, `expect(response.json()).toMatchObject(...)`, `expect(response.headers()).toHaveProperty('content-type')`

#### Scenario: Auth state sharing

- **WHEN** an API login request returns authentication cookies/tokens
- **THEN** the system SHALL allow passing the auth state to `context.addCookies()` or `context.storageState()`
- **AND** subsequent UI tests SHALL use the same authenticated session

#### Scenario: Mixed API+UI test flow

- **WHEN** a test requires setting up data via API before testing the UI
- **THEN** the system SHALL support: call API to create data → navigate UI → verify data appears → clean up via API after test
