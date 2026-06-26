## ADDED Requirements

### Requirement: Injected Core Script Injection

The system SHALL pre-inject a lightweight `injected-core.js` script (≤2KB minified) into every page at document start via `Page.addScriptToEvaluateOnNewDocument` (CDP), `script.addPreloadScript` (BiDi), and framework equivalents (WebKit).

#### Scenario: Core script auto-injection on page creation

- **WHEN** a new page is created or navigated
- **THEN** the system SHALL inject `injected-core.js` before any page JavaScript executes
- **AND** the script SHALL establish the `__turbosheet` binding bridge to Rust
- **AND** the script SHALL register a MutationObserver for stability detection
- **AND** the total injected script SHALL be under 2KB minified

#### Scenario: Core script per-frame injection

- **WHEN** a page contains iframes or the browser fires `Page.frameAttached`
- **THEN** the system SHALL inject `injected-core.js` into each new frame
- **AND** each frame SHALL have its own binding bridge with a unique binding name

### Requirement: Injected Actions On-Demand Loading

The system SHALL load `injected-actions.js` (≤15KB minified) only when a user action is first invoked, caching it in the page context.

#### Scenario: Lazy action script loading

- **WHEN** a user invokes `.click()`, `.fill()`, or any action for the first time on a page
- **THEN** the system SHALL evaluate `injected-actions.js` into the page via `Runtime.evaluate()`
- **AND** SHALL cache the loaded functions for subsequent calls
- **AND** SHALL NOT re-inject on subsequent actions

#### Scenario: Actionability checks via injected script

- **WHEN** a user action is performed
- **THEN** the injected script SHALL check visibility, stability, enabled state, and `pointer-events: none`
- **AND** SHALL use MutationObserver-based wait instead of fixed-interval polling
- **AND** SHALL report results back to Rust via the binding bridge

### Requirement: Binding Bridge Communication

The system SHALL implement a request-response correlation bridge using UUIDs and `DashMap<UUID, oneshot::Sender>`.

#### Scenario: Rust-initiated action via binding

- **WHEN** Rust invokes an action on the page
- **THEN** the system SHALL generate a UUID, send it to the injected script via `Runtime.evaluate`
- **AND** SHALL wait on a `oneshot::Receiver` for the response
- **AND** the injected script SHALL report the result via `Runtime.addBinding` with the matching UUID

#### Scenario: Navigation-aware message handling

- **WHEN** a page navigates while a binding request is pending
- **THEN** the system SHALL re-queue the pending request
- **AND** SHALL re-attempt after the new page finishes loading
- **AND** SHALL fail the request if it exceeds the configured timeout
