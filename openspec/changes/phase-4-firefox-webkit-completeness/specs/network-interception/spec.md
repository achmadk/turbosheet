## ADDED Requirements

### Requirement: BiDi network observation for Firefox

When BiDi is available (Firefox), the system SHALL use BiDi network events (`network.beforeRequestSent`, `network.responseCompleted`) to implement `wait_for_request` and `wait_for_response` instead of polling.

#### Scenario: Request waiter uses BiDi events

- **WHEN** `wait_for_request(url)` is called in Firefox with the `bidi` feature enabled
- **THEN** the waiter SHALL subscribe to `network.beforeRequestSent` events
- **AND** resolve when an event with URL matching the pattern arrives

#### Scenario: Response waiter uses BiDi events

- **WHEN** `wait_for_response(url)` is called in Firefox with the `bidi` feature enabled
- **THEN** the waiter SHALL subscribe to `network.responseCompleted` events
- **AND** resolve when an event with URL matching the pattern arrives

#### Scenario: No-op when BiDi unavailable

- **WHEN** `wait_for_request(url)` is called in Firefox without the `bidi` feature
- **THEN** it SHALL fall back to polling via `performance.getEntriesByType('resource')`
