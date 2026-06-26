# real-input-dispatch Specification

## Purpose

Replace synthetic `dispatchEvent(new MouseEvent(...))` / `dispatchEvent(new KeyboardEvent(...))` calls with CDP `Input.dispatchMouseEvent`, `Input.dispatchKeyEvent`, and `Input.dispatchTouchEvent` to produce trusted browser events (`isTrusted=true`).

## ADDED Requirements

### Requirement: CDP mouse dispatch for click actions

The system SHALL dispatch all click-related actions (click, dblclick, right_click) via CDP `Input.dispatchMouseEvent` with `type: 'mousePressed'` and `type: 'mouseReleased'` sequences, using the element's bounding box coordinates computed by the injected script (or CDP `DOM.getBoxModel`).

#### Scenario: Click element

- **WHEN** `click(".button")` is called and the element is found
- **THEN** the system sends `Input.dispatchMouseEvent` with `type: 'mousePressed'`, `button: 'left'`, `clickCount: 1`, `x`/`y` set to element center coordinates, followed by `type: 'mouseReleased'`

#### Scenario: Double-click element

- **WHEN** `dblclick(".button")` is called
- **THEN** the system sends two press-release sequences with `clickCount: 2`

#### Scenario: Right-click element

- **WHEN** `right_click(".button")` is called
- **THEN** the system sends press-release with `button: 'right'`

### Requirement: CDP mouse dispatch for hover

The system SHALL move the mouse cursor to the element's position via CDP `Input.dispatchMouseEvent` with `type: 'mouseMoved'` using the element's bounding box coordinates.

#### Scenario: Hover element

- **WHEN** `hover(".menu")` is called
- **THEN** the system sends `Input.dispatchMouseEvent` with `type: 'mouseMoved'` to the element's center coordinates

### Requirement: CDP mouse dispatch for checkbox/radio

The system SHALL dispatch a click press-release sequence via CDP `Input.dispatchMouseEvent` for check and uncheck actions, targeting the checkbox/radio element's bounding box.

#### Scenario: Check checkbox

- **WHEN** `check("#agree")` is called
- **THEN** the system dispatches `Input.dispatchMouseEvent` press-release at the input element's coordinates

#### Scenario: Uncheck checkbox

- **WHEN** `uncheck("#agree")` is called
- **THEN** the system dispatches `Input.dispatchMouseEvent` press-release at the input element's coordinates

### Requirement: CDP key dispatch for keyboard actions

The system SHALL dispatch keyboard input via CDP `Input.dispatchKeyEvent` with `type: 'keyDown'` and `type: 'keyUp'` sequences for press actions, and `Input.insertText` for text input.

#### Scenario: Press key

- **WHEN** `press("#field", "Enter")` is called
- **THEN** the system sends `Input.dispatchKeyEvent` with `type: 'keyDown'`, `key: 'Enter'`, followed by `type: 'keyUp'`

#### Scenario: Type text sequentially

- **WHEN** `press_sequentially("#field", "hello")` is called
- **THEN** the system sends `Input.dispatchKeyEvent` for each character with `type: 'keyDown'`/`'keyUp'`, or uses `Input.insertText` for the full string

#### Scenario: Type special keys

- **WHEN** `press("#field", "Control+a")` is called
- **THEN** the system dispatches `keyDown` for Control, `keyDown` for 'a', `keyUp` for 'a', `keyUp` for Control

### Requirement: CDP touch dispatch for touch actions

The system SHALL dispatch touch events via CDP `Input.dispatchTouchEvent` with `type: 'touchStart'`, `'touchMove'`, `'touchEnd'` for drag and drop operations.

#### Scenario: Drag and drop

- **WHEN** `drag_and_drop("#source", "#target")` is called
- **THEN** the system dispatches `Input.dispatchTouchEvent` with `touchStart` at source, `touchMove` along path to target, `touchEnd` at target

### Requirement: Coordinate resolution via injected script

The system SHALL use the injected script engine (or CDP `DOM.getBoxModel` as fallback) to resolve element selectors to bounding box coordinates before dispatching CDP Input events.

#### Scenario: Resolve coordinates before click

- **WHEN** an action requires coordinates
- **THEN** the system first calls the injected script's `querySelector` + `getBoundingClientRect` (or CDP `DOM.getBoxModel`), then passes the coordinates to the Input dispatch command

### Requirement: Auto-wait before dispatch

The system SHALL perform actionability checks (visible, stable, enabled) via the injected script before dispatching CDP Input events, retrying with exponential backoff.

#### Scenario: Wait and click

- **WHEN** `click(".late-loading")` is called on a not-yet-visible element
- **THEN** the system polls actionability via injected script with exponential backoff (50ms, 100ms, 200ms, 400ms) up to configured timeout, then dispatches `Input.dispatchMouseEvent`
