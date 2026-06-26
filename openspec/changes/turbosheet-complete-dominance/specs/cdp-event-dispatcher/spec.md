## ADDED Requirements

### Requirement: CDP Event Subscription System

The system SHALL implement a CDP Event Dispatcher that routes incoming browser protocol events to registered handlers through typed channels.

#### Scenario: Frame navigation tracking

- **WHEN** the browser fires `Page.frameNavigated`
- **THEN** the EventDispatcher SHALL update the page URL cache WITH zero CDP round-trips
- **AND** the `page.url()` method SHALL return the cached value from the dispatcher

#### Scenario: Console message capture

- **WHEN** the browser fires `Runtime.consoleAPICalled`
- **THEN** the EventDispatcher SHALL route the event to `page.on('console')` handlers
- **AND** each console message SHALL include type (log/warn/error), text, and optional args

#### Scenario: Dialog event handling

- **WHEN** the browser fires `Page.javascriptDialogOpening`
- **THEN** the EventDispatcher SHALL route to dialog handler
- **AND** `page.on('dialog')` SHALL fire with dialog type, message, and accept/dismiss methods
- **AND** unhandled dialogs SHALL be auto-dismissed after a configurable timeout

#### Scenario: Request/Response interception

- **WHEN** the browser fires `Network.requestWillBeSent` and `Network.responseReceived`
- **THEN** the EventDispatcher SHALL route to `page.on('request')` and `page.on('response')` handlers
- **AND** each request SHALL include url, method, headers, and postData
- **AND** each response SHALL include status, headers, and timing info

#### Scenario: Subscriber cleanup

- **WHEN** a page is closed or a handler is removed
- **THEN** the EventDispatcher SHALL clean up all channels for that subscriber
- **AND** no dead handlers SHALL receive future events
