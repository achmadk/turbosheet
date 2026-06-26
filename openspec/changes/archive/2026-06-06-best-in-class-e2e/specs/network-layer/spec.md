## ADDED Requirements

### Requirement: Native HTTPS Interception

The system SHALL intercept network requests natively using the browser's debugging protocol (CDP Fetch domain for Chromium) to support full HTTPS body inspection.

#### Scenario: Mocking an API response

- **WHEN** `page.route` is called with a mock handler for an HTTPS URL
- **THEN** the system MUST intercept the request and return the mocked response without requiring a proxy MITM certificate

### Requirement: Network Event Subscription

The system SHALL expose events for network lifecycle tracking.

#### Scenario: Subscribing to requests

- **WHEN** a user registers a handler via `page.on('request')`
- **THEN** the handler MUST be invoked for every outbound network request with full request details

### Requirement: Request Waiting

The system SHALL provide mechanisms to wait for specific network traffic.

#### Scenario: Waiting for a response

- **WHEN** `page.waitForResponse('**/api/data')` is called
- **THEN** execution MUST pause until a matching response is received or the timeout expires
