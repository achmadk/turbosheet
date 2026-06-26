## ADDED Requirements

### Requirement: HAR Recording

The system SHALL support recording HTTP Archive (HAR) files from browser network activity.

#### Scenario: Starting HAR recording

- **WHEN** `context.newPage({ recordHar: { path: './traces/network.har' } })` is called
- **THEN** the system SHALL begin recording network events (requests, responses, timing)
- **AND** SHALL format the output as a valid HAR 1.2 specification file

#### Scenario: HAR event capture

- **WHEN** a page makes network requests during HAR recording
- **THEN** the system SHALL capture: request URL, method, headers, POST data, response status, response headers, timing (DNS, TCP, SSL, TTFB, download), and content type
- **AND** SHALL correlate requests with their responses

#### Scenario: HAR playback

- **WHEN** HAR playback mode is enabled with a previously recorded HAR file
- **THEN** the system SHALL match outgoing requests against recorded entries by URL and method
- **AND** SHALL return the recorded response for matching requests
- **AND** SHALL pass through non-matching requests to the real server

#### Scenario: HAR file structure

- **WHEN** a HAR file is written
- **THEN** SHALL include the `log` object with `version: "1.2"`
- **AND** SHALL include `pages` array with page navigation events
- **AND** SHALL include `entries` array with all captured network requests
