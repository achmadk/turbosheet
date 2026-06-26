## ADDED Requirements

### Requirement: BiDi Stream Pre-Processor

The system SHALL implement a Rust-native WebDriver BiDi event stream pre-processor that filters noise, condenses state, and passes only finalized data across the FFI bridge.

#### Scenario: Event stream ingestion

- **WHEN** the Firefox or WebKit engine establishes a BiDi connection
- **THEN** the pre-processor SHALL ingest the raw BiDi event stream
- **AND** SHALL register interest in specific event types (DOM mutations, navigation, console, network)

#### Scenario: Noise filtering

- **WHEN** high-frequency events (log messages, periodic timestamps, heartbeats) arrive
- **THEN** the pre-processor SHALL filter them out before they reach the FFI bridge
- **AND** SHALL drop at least 80% of raw events before passing condensed state to Node.js

#### Scenario: State condensation

- **WHEN** multiple DOM mutations arrive in quick succession
- **THEN** the pre-processor SHALL aggregate them into a single state delta
- **AND** SHALL only emit the condensed state when the stream stabilizes (no new mutations for 50ms)

#### Scenario: Cross-engine consistency

- **WHEN** the same action is performed on Chromium (CDP events) and Firefox (BiDi events)
- **THEN** the pre-processor SHALL normalize the event shapes to a common format
- **AND** the Node.js consumer SHALL receive identical event structures regardless of engine
