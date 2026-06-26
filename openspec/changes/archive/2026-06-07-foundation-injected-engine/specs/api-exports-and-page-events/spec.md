# api-exports-and-page-events Specification

## Purpose

Export all missing Node.js API functions from TurboSheet's NAPI entry point (`index.js`) and wire `page.on()` event subscription handlers (`dialog`, `request`, `response`, `console`) to the CDP Event Dispatcher.

## ADDED Requirements

### Requirement: Export test runner functions

The system SHALL export `run_tests`, `test`, `expect`, `describe`, `beforeAll`, `afterAll`, `beforeEach`, `afterEach` from the package entry point (`index.js`) by re-exporting them from the internal test runner module.

#### Scenario: Import test function

- **WHEN** a user writes `import { test, expect } from 'turbosheet'`
- **THEN** the `test` and `expect` exports resolve to the NAPI-bound test runner functions

#### Scenario: Import describe and hooks

- **WHEN** a user writes `import { describe, beforeAll, afterAll, beforeEach, afterEach } from 'turbosheet'`
- **THEN** the hooks are available from the package entry point

### Requirement: Export page event types

The system SHALL export TypeScript type definitions for event callback signatures (`DialogEvent`, `RequestEvent`, `ResponseEvent`, `ConsoleEvent`) from `index.d.ts`.

#### Scenario: Type-safe page.on()

- **WHEN** a user writes `page.on('dialog', (dialog: DialogEvent) => ...)`
- **THEN** TypeScript types are available for all event callback parameters

### Requirement: page.on('dialog') implementation

The system SHALL subscribe to CDP `Page.javascriptDialogOpening` events via the CDP Event Dispatcher and invoke registered dialog callbacks with the dialog message, type, and default value. The callback SHALL have access to `dialog.accept()` and `dialog.dismiss()` methods.

#### Scenario: Handle javascript dialog

- **WHEN** `page.on('dialog', handler)` is registered and an `alert('hello')` is triggered
- **THEN** the handler is invoked with a `DialogEvent` containing `message: 'hello'`, `type: 'alert'`, and `accept()`/`dismiss()` methods

#### Scenario: Accept dialog

- **WHEN** `dialog.accept('my input')` is called from within a dialog handler
- **THEN** the system sends CDP `Page.handleJavaScriptDialog` with `accept: true` and `promptText: 'my input'`

### Requirement: page.on('console') implementation

The system SHALL subscribe to CDP `Runtime.consoleAPICalled` events via the CDP Event Dispatcher and invoke registered console callbacks with the console message type, text, and arguments.

#### Scenario: Capture console.log

- **WHEN** `page.on('console', handler)` is registered and `console.log('hello')` runs in the page
- **THEN** the handler is invoked with a `ConsoleEvent` containing `type: 'log'`, `text: 'hello'`

#### Scenario: Capture console.error

- **WHEN** `page.on('console', handler)` is registered and `console.error('fail')` runs
- **THEN** the handler is invoked with `type: 'error'`, `text: 'fail'`

### Requirement: page.on('request') implementation

The system SHALL subscribe to CDP `Network.requestWillBeSent` events via the CDP Event Dispatcher and invoke registered request callbacks with the request URL, method, headers, and post data.

#### Scenario: Intercept outgoing request

- **WHEN** `page.on('request', handler)` is registered and the page fetches `https://api.example.com/data`
- **THEN** the handler is invoked with a `RequestEvent` containing the URL, method, headers, and optional post body

### Requirement: page.on('response') implementation

The system SHALL subscribe to CDP `Network.responseReceived` events via the CDP Event Dispatcher and invoke registered response callbacks with the response URL, status, and headers.

#### Scenario: Intercept response

- **WHEN** `page.on('response', handler)` is registered and a network response is received
- **THEN** the handler is invoked with a `ResponseEvent` containing URL, status code, and headers

### Requirement: page.removeListener implementation

The system SHALL support removing previously registered event listeners, unsubscribing from the CDP Event Dispatcher.

#### Scenario: Remove dialog listener

- **WHEN** `page.on('dialog', handler)` is followed by `page.removeListener('dialog', handler)`
- **THEN** subsequent dialog events are no longer dispatched to that handler
