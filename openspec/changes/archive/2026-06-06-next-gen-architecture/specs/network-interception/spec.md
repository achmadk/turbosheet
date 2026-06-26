## ADDED Requirements

### Requirement: Ergonomic Declarative Mocking

The network interception system SHALL provide a high-level, declarative API for mocking network requests without requiring users to write raw proxy handlers.

#### Scenario: Mocking an API response

- **WHEN** the user calls `page.route("**/api/data", { body: "mocked" })`
- **THEN** any network request matching the pattern is intercepted and immediately returned the mocked body

### Requirement: HAR Recording and Playback

The network interception system SHALL support recording all network traffic to a standard HAR file format, and replaying requests from a HAR file.

#### Scenario: Replaying a HAR file

- **WHEN** the network proxy is configured with a previously recorded HAR file
- **THEN** incoming requests that match entries in the HAR file are intercepted and served the recorded response locally
