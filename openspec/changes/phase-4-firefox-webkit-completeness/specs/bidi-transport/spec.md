## ADDED Requirements

### Requirement: BiDi WebSocket connection management

The system SHALL provide a `BidiClient` that manages a WebSocket connection to a BiDi-enabled WebDriver endpoint.

#### Scenario: Successful WebSocket handshake

- **WHEN** `BidiClient::connect(url)` is called with a valid WebDriver BiDi WebSocket URL
- **THEN** the client SHALL establish a WebSocket connection and return a `BidiClient` instance

#### Scenario: Connection failure with retry

- **WHEN** the WebSocket handshake fails (e.g., WebDriver not ready)
- **THEN** the client SHALL retry up to 3 times with exponential backoff (200ms, 500ms, 1s)

#### Scenario: Ping/pong keepalive

- **WHEN** no messages are sent for 30 seconds
- **THEN** the transport SHALL send a WebSocket ping frame and expect a pong within 10 seconds

### Requirement: BiDi command execution

The `BidiClient` SHALL support sending BiDi commands and receiving responses asynchronously.

#### Scenario: Send command and receive response

- **WHEN** `client.send_command(method, params)` is called
- **THEN** the client SHALL send a JSON message with `id`, `method`, and `params` fields over the WebSocket
- **AND** resolve with the response `{id, result}` when the server responds

#### Scenario: Command timeout

- **WHEN** no response is received within 30 seconds
- **THEN** `send_command` SHALL return a timeout error and cancel the pending command

### Requirement: BiDi event subscription

The `BidiClient` SHALL support subscribing to BiDi events via `session.subscribe` and routing events to typed listeners.

#### Scenario: Subscribe to network events

- **WHEN** `client.subscribe(["network.beforeRequestSent", "network.responseCompleted"])` is called
- **THEN** the client SHALL send a `session.subscribe` command with the specified event names
- **AND** route received events to registered listeners

#### Scenario: Event stream access

- **WHEN** a subscribed BiDi event arrives on the WebSocket
- **THEN** the client SHALL deserialize the event and push it to the appropriate listener channel

### Requirement: BiDi session management

The `BidiClient` SHALL maintain a BiDi session tied to a WebDriver session.

#### Scenario: Create session from WebDriver session

- **WHEN** `BidiClient::from_webdriver_session(webdriver_url, session_id)` is called
- **THEN** the client SHALL extract the BiDi WebSocket URL from the WebDriver session capabilities
- **AND** connect to the BiDi WebSocket endpoint

#### Scenario: Session teardown

- **WHEN** `client.close()` is called
- **THEN** the WebSocket SHALL be gracefully closed
- **AND** all pending commands SHALL be cancelled with an error
