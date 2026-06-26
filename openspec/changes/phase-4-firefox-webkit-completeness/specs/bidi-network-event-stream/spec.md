## ADDED Requirements

### Requirement: Network event subscription

The BiDi client SHALL subscribe to `network.beforeRequestSent`, `network.responseCompleted`, and `network.fetchError` events.

#### Scenario: Subscribe after session creation

- **WHEN** a BidiClient is connected to a BiDi session
- **THEN** it SHALL subscribe to network events via `session.subscribe` command

#### Scenario: Handle network.beforeRequestSent

- **WHEN** the browser initiates any network request
- **THEN** a `network.beforeRequestSent` event SHALL be received with `request.requestId`, `request.url`, `request.method`, and `request.headers`

#### Scenario: Handle network.responseCompleted

- **WHEN** a network response completes
- **THEN** a `network.responseCompleted` event SHALL be received with `request.requestId`, `response.url`, `response.status`, and `response.headers`

#### Scenario: Handle network.fetchError

- **WHEN** a network request fails
- **THEN** a `network.fetchError` event SHALL be received with `request.requestId`, `errorText`

### Requirement: URL-pattern waiters using oneshot channels

The system SHALL provide `UrlWaiter` that resolves a oneshot when a BiDi event URL matches a pattern.

#### Scenario: Request waiter resolves on URL match

- **WHEN** `UrlWaiter::new("api.example.com", request_kind)` is registered
- **AND** a `network.beforeRequestSent` event arrives with URL containing `"api.example.com"`
- **THEN** the waiter SHALL resolve with the request metadata

#### Scenario: Multiple concurrent waiters

- **WHEN** 3 `wait_for_request` calls are pending with different patterns
- **AND** one matching event arrives
- **THEN** only the matching waiter SHALL resolve; others remain pending

#### Scenario: Waiter cleanup on timeout

- **WHEN** a waiter times out after 30 seconds
- **THEN** the waiter SHALL be removed from the active waiters list
- **AND** the corresponding future SHALL resolve with a timeout error
