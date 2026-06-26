## ADDED Requirements

### Requirement: HTTPS Transparent Tunneling

The system SHALL tunnel HTTPS requests (CONNECT) to the target host to support secure connections without dropping them.

#### Scenario: Processing HTTPS CONNECT

- **WHEN** the proxy receives a CONNECT request
- **THEN** it upgrades the connection and maintains the TCP tunnel to the server

### Requirement: Route Continuation and Interception

The system SHALL allow passing requests through unmodified or altering request properties (headers, url, post data) and properly process POST payloads.

#### Scenario: Route Continue with modifications

- **WHEN** `route.continue()` is called with overridden headers
- **THEN** the request is forwarded to the destination with the new headers

#### Scenario: POST payload capture

- **WHEN** a POST request is intercepted
- **THEN** the full request body is read and made available to the route handler
