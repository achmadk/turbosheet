## ADDED Requirements

### Requirement: Stub API Completion — Silent No-Op Elimination

The system SHALL implement every public API method that currently accepts calls and returns `Ok(())` without performing any action. Silent no-ops are a correctness hazard because users unknowingly depend on them.

#### Scenario: ChromiumEngine.close() actually terminates browser

- **GIVEN** `ChromiumEngine::close()` is called
- **WHEN** the engine has a running Chromium browser process
- **THEN** the engine SHALL send a `Browser.close` CDP command
- **AND** SHALL send SIGTERM to the child process (with graceful timeout)
- **AND** SHALL send SIGKILL if the process does not exit within 5 seconds
- **AND** SHALL wait for the child process to exit and reap the zombie
- **AND** SHALL mark the engine state as closed
- **AND** SHALL return `TurbosheetError::BrowserDisconnected` on subsequent operations

#### Scenario: ChromiumEngine.handle_event() routes CDP events to subscribers

- **GIVEN** `ChromiumEngine::handle_event()` receives a CDP event from the WebSocket connection
- **WHEN** the event is received
- **THEN** the engine SHALL parse the CDP event method name
- **AND** SHALL dispatch the event to all registered event subscribers by event type
- **AND** SHALL convert CDP event parameters to the internal event model
- **AND** SHALL NOT silently drop the event

#### Scenario: touch_screen dispatches touch events

- **GIVEN** `PageEngine::touch_screen()` is called with a touch action (tap, press, move, release, etc.)
- **WHEN** the engine is running in touch-emulation mode (or real touch device)
- **THEN** the engine SHALL dispatch Touch events (touchstart, touchmove, touchend) to the page
- **AND** SHALL support single-touch and multi-touch (Touches, TargetTouches, ChangedTouches)
- **AND** SHALL return `TurbosheetError::UnsupportedOperation` with a clear message when touch is not available (e.g., headless mode without touch emulation)

#### Scenario: emulate_media overrides CSS media features

- **GIVEN** `PageEngine::emulate_media()` is called with media feature overrides
- **WHEN** the override includes `prefers-color-scheme`
- **THEN** the engine SHALL set the CDP `Emulation.setEmulatedMedia` with the requested scheme
- **WHEN** the override includes `prefers-reduced-motion`
- **THEN** the engine SHALL set the CDP `Emulation.setEmulatedMedia` with the requested motion preference
- **WHEN** the override includes `prefers-contrast`, `forced-colors`, or `prefers-reduced-transparency`
- **THEN** the engine SHALL apply each via the appropriate CDP emulation commands
- **AND** SHALL restore original values when called with no arguments

#### Scenario: storage exposes localStorage/sessionStorage handles

- **GIVEN** `PageEngine::storage()` is called with a storage type (local or session)
- **WHEN** the engine is connected to a page
- **THEN** the engine SHALL return a storage handle that supports: `getItem(key)`, `setItem(key, value)`, `removeItem(key)`, `clear()`, `length()`, and `key(index)`
- **AND** SHALL implement these via CDP `DOMStorage` domain commands
- **AND** SHALL support both `localStorage` and `sessionStorage` based on the request

#### Scenario: websocket exposes WebSocket frame interception

- **GIVEN** `PageEngine::websocket()` is called to intercept WebSocket traffic
- **WHEN** the engine is connected to a page
- **THEN** the engine SHALL enable CDP `Network.webSocketFrameReceived` and `Network.webSocketFrameSent` events
- **AND** SHALL expose a handle that provides: `onFrameReceived(callback)`, `onFrameSent(callback)`, `onError(callback)`, `onClose(callback)`
- **AND** SHALL include frame payload, opcode, and mask in callbacks

#### Scenario: accessibility returns accessibility tree snapshot

- **GIVEN** `PageEngine::accessibility()` is called
- **WHEN** the engine is connected to a page
- **THEN** the engine SHALL request the accessibility tree via CDP `Accessibility.getFullAXTree`
- **AND** SHALL return a structured tree with: role, name, value, description, children, bounds, and properties (checked, selected, disabled, expanded, etc.)
- **AND** SHALL support snapshot at full depth or partial (configurable)

#### Scenario: Route.abort() actually aborts the request

- **GIVEN** a network route is intercepted and `Route::abort()` is called
- **WHEN** the corresponding request has not yet completed
- **THEN** the route SHALL abort the HTTP request with an `Aborted` error reason
- **AND** SHALL send a CDP `Fetch.failRequest` with `ErrorReason::BlockedByClient`
- **AND** SHALL reject the request promise with a network error in the page
- **AND** SHALL call any registered abort callback

#### Scenario: BinaryManager downloads Chromium

- **GIVEN** `BinaryManager::download_chromium()` is called
- **WHEN** no local Chromium installation is found
- **THEN** the system SHALL download the appropriate Chromium revision for the platform (linux, mac, windows)
- **AND** SHALL verify the download checksum
- **AND** SHALL extract the browser binary to the configured cache directory
- **AND** SHALL return the path to the downloaded binary
- **WHEN** Chromium is already downloaded
- **THEN** the system SHALL verify the existing installation is intact
- **AND** SHALL return the existing path without re-downloading
