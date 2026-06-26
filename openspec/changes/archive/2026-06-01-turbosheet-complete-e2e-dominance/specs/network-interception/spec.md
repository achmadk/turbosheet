## ADDED Requirements

### Requirement: HTTP request interception

TurboSheet SHALL intercept HTTP/HTTPS requests from the browser and allow modification, fulfillment, blocking, or passthrough.

#### Scenario: Route pattern matching

- **WHEN** user calls `await page.route('**/api/**', handler)`
- **THEN** all requests matching the glob pattern SHALL be routed through the handler

#### Scenario: Fulfill with mock response

- **WHEN** handler calls `route.fulfill({ status: 200, body: mockData })`
- **THEN** browser SHALL receive the mock response without hitting the network

#### Scenario: Continue with modifications

- **WHEN** handler calls `route.continue({ headers: { 'x-custom': 'val' } })`
- **THEN** request SHALL proceed to the network with modified headers

#### Scenario: Abort request

- **WHEN** handler calls `route.abort()`
- **THEN** request SHALL be aborted (useful for blocking analytics, fonts, etc.)

### Requirement: Rust-native pattern matching

URL pattern matching SHALL execute in Rust, not JavaScript.

#### Scenario: Unmatched request bypass

- **WHEN** a request URL does NOT match any registered route pattern
- **THEN** the request SHALL pass through without invoking JavaScript — zero overhead

#### Scenario: High-throughput interception

- **WHEN** 1000+ requests per second are generated
- **THEN** pattern matching SHALL execute in Rust without blocking the JS event loop

### Requirement: Response mocking

TurboSheet SHALL support mocking API responses with JSON, binary, or error responses.

#### Scenario: Mock JSON API

- **WHEN** user defines `await page.route('**/api/users', route => route.fulfill({ body: [{ id: 1 }] }))`
- **THEN** browser SHALL receive the mock JSON array

#### Scenario: Mock network error

- **WHEN** handler calls `route.fulfill({ status: 500, body: 'Server Error' })`
- **THEN** browser SHALL receive a 500 response

### Requirement: WebSocket interception

TurboSheet SHALL intercept WebSocket connections and messages.

#### Scenario: Intercept WebSocket creation

- **WHEN** page creates a WebSocket to `wss://example.com/ws`
- **THEN** TurboSheet SHALL allow user to accept, modify, or reject the connection

#### Scenario: Mock WebSocket messages

- **WHEN** user registers a WS message handler
- **THEN** TurboSheet SHALL intercept outgoing and incoming WebSocket frames

### Requirement: Network throttling

TurboSheet SHALL simulate network conditions (latency, bandwidth, offline).

#### Scenario: Simulate slow 3G

- **WHEN** user sets `page.throttle({ download: 400, upload: 100, latency: 200 })`
- **THEN** network requests SHALL experience the specified latency and bandwidth constraints

#### Scenario: Offline mode

- **WHEN** user calls `page.setOffline(true)`
- **THEN** all network requests SHALL fail with a network error
