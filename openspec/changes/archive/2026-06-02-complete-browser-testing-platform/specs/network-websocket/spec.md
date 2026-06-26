# network-websocket

## ADDED Requirements

### Requirement: WebSocket routing

The system SHALL intercept WebSocket connections via route patterns.

#### Scenario: Route WebSocket by URL pattern

- **WHEN** user calls `page.routeWebSocket('**/ws', handler)`
- **THEN** system intercepts WebSocket connections matching the pattern

#### Scenario: WebSocket handler receives messages

- **WHEN** page opens WebSocket and server sends message
- **THEN** handler receives the message via `socket.onMessage()`

#### Scenario: Handler sends message to client

- **WHEN** handler calls `socket.send('response data')`
- **THEN** page WebSocket receives the message

#### Scenario: Handler closes connection

- **WHEN** handler calls `socket.close(1000, 'reason')`
- **THEN** page WebSocket receives close event with code and reason

### Requirement: WebSocket handler lifecycle

The system SHALL provide lifecycle callbacks for WebSocket connections.

#### Scenario: onSocketOpen callback

- **WHEN** WebSocket connection is established
- **THEN** handler's `onSocketOpen(socket)` is called

#### Scenario: onSocketMessage callback

- **WHEN** message is received from client or server
- **THEN** handler's `onSocketMessage(socket, message)` is called

#### Scenario: onSocketClose callback

- **WHEN** WebSocket connection closes
- **THEN** handler's `onSocketClose(socket, code, reason)` is called

### Requirement: WebSocket message types

The system SHALL support both text and binary WebSocket messages.

#### Scenario: Receive text message

- **WHEN** page sends text message via WebSocket
- **THEN** handler receives message with `type: 'text'` and `data: string`

#### Scenario: Receive binary message

- **WHEN** page sends binary message via WebSocket
- **THEN** handler receives message with `type: 'binary'` and `data: Uint8Array`

#### Scenario: Send binary message from handler

- **WHEN** handler calls `socket.send binary data`
- **THEN** page WebSocket receives binary message

### Requirement: Unroute WebSocket

The system SHALL stop intercepting WebSocket connections.

#### Scenario: Unroute specific pattern

- **WHEN** user calls `page.unrouteWebSocket('**/ws')`
- **THEN** system stops intercepting WebSocket connections matching the pattern

#### Scenario: Unroute all WebSockets

- **WHEN** user calls `page.unrouteAllWebSockets()`
- **THEN** system stops all WebSocket interception
