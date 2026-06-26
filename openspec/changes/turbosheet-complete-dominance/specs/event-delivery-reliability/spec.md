## ADDED Requirements

### Requirement: Event Delivery Reliability

The system SHALL deliver all events to subscribers without silent data loss, even under load. Events dropped due to full broadcast channels or busy dispatch queues are a correctness hazard.

#### Scenario: Broadcast channel overflow does not drop events silently

- **GIVEN** `src/network/interceptor.rs` uses `tokio::sync::broadcast` for network event distribution
- **WHEN** the broadcast channel's buffer is full and a new event is sent
- **THEN** the system SHALL NOT silently drop the event (current behavior: `send()` returns `Err(TrySendError::Full(_))` which is ignored)
- **AND** SHALL implement at least one of:
  1. Increase the channel capacity to a safe maximum (e.g., 4096+ instead of current)
  2. Switch to `tokio::sync::mpsc` with an unbounded or large bounded buffer (with backpressure)
  3. Log a warning when events are dropped, including event type, count, and timestamp
  4. Provide a configurable buffer size
- **AND** SHALL expose a metric for dropped event count

#### Scenario: NonBlocking event dispatch does not drop events

- **GIVEN** `src/network/events.rs` uses `NapiSync::NonBlocking` for dispatching network events to JavaScript
- **WHEN** the JavaScript event queue is busy and a new event arrives
- **THEN** the system SHALL NOT silently skip the event dispatch
- **AND** SHALL implement at least one of:
  1. Queue the event for delivery when the JS queue is ready
  2. Use `NapiSync::Async` or thread-safe callback instead of NonBlocking
  3. Buffer events in a Rust-side VecDeque and drain them on each poll
- **AND** SHALL log a warning when events are deferred or dropped, with event type and count

#### Scenario: Network interceptor event loss detection

- **GIVEN** a test that makes many rapid network requests (e.g., 100+ in quick succession)
- **WHEN** the network interceptor processes these requests and responses
- **THEN** the system SHALL NOT miss any request/response event
- **AND** SHALL deliver all events to registered route handlers
- **AND** SHALL verify delivery by tracking sent vs. received event counts in the test output
- **AND** SHALL report any discrepancy as a warning or error

#### Scenario: ChromiumEngine handle_event delivery guarantee

- **GIVEN** `ChromiumEngine::handle_event()` receives CDP events from the WebSocket
- **WHEN** events arrive in rapid succession (e.g., during page load with many network and DOM events)
- **THEN** the engine SHALL NOT silently drop events (current behavior is no-op stub)
- **AND** SHALL route each event to all registered subscribers based on event type
- **AND** SHALL handle backpressure from slow subscribers without dropping events for other subscribers
