## ADDED Requirements

### Requirement: Reactive event dispatching

The system SHALL maintain active subscriptions to CDP events (navigation, binding calls, console) and dispatch them without blocking the event loop.

#### Scenario: Frame navigation event

- **WHEN** a `Page.frameNavigated` event occurs
- **THEN** the dispatcher updates the internal URL cache without requiring a new `page.evaluate()` call

### Requirement: Binding callback processing

The system SHALL route binding callbacks to the pending oneshot sender based on the request UUID.

#### Scenario: Receive UUID callback

- **WHEN** a `Runtime.bindingCalled` event is received matching a pending request UUID
- **THEN** the result data is sent across the oneshot channel to unblock the Rust task
