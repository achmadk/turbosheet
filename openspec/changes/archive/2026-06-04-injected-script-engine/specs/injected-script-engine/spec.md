## ADDED Requirements

### Requirement: Pre-inject core script

The system SHALL inject a lightweight core script into all new documents prior to execution of page scripts.

#### Scenario: Script injection on new page

- **WHEN** a new page or frame is created
- **THEN** the system uses `Page.addScriptToEvaluateOnNewDocument` (or equivalent) to inject `injected-core.js`

### Requirement: Establish Rust-JS Binding

The system SHALL establish a two-way communication bridge via CDP bindings to transfer action results.

#### Scenario: JS binding callback

- **WHEN** an interaction completes in the injected script
- **THEN** it sends a JSON payload with a UUID to the registered binding (e.g., `__tsReport`)
