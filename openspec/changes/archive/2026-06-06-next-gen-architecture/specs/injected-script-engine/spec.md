## ADDED Requirements

### Requirement: Shadow DOM Piercing

The injected script engine SHALL support piercing Shadow DOM boundaries automatically when evaluating selectors like `css=` or `text=`.

#### Scenario: Element inside Shadow DOM

- **WHEN** a user queries for an element that resides inside an open Shadow DOM
- **THEN** the engine resolves the selector successfully without requiring explicit shadow piercing operators

### Requirement: Auto-Scrolling for Actionability

The injected script engine SHALL automatically scroll target elements into the visible viewport before performing actionability checks or dispatching events.

#### Scenario: Clicking an off-screen element

- **WHEN** the user attempts to click an element that is currently scrolled out of view
- **THEN** the engine scrolls the element into view before performing the `elementFromPoint` hit-test

### Requirement: Deep Actionability Checks

The injected script engine SHALL verify that an element is stably attached to the DOM and is receiving pointer events before interacting with it.

#### Scenario: Interacting with pointer-events: none

- **WHEN** an element is visually present but styled with `pointer-events: none`
- **THEN** the actionability check fails and waits for the element to become interactive
