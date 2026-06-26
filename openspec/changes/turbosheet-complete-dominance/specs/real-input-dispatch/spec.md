## ADDED Requirements

### Requirement: Real Input Events via CDP

The system SHALL use CDP `Input.dispatch*` commands for all user interactions instead of synthetic JavaScript `dispatchEvent()`.

#### Scenario: Click with trusted event

- **WHEN** `locator.click()` is performed
- **THEN** the system SHALL dispatch using CDP `Input.dispatchMouseEvent` with type `mousePressed` + `mouseReleased`
- **AND** the resulting `click` event in the page SHALL have `isTrusted === true`
- **AND** CSS `:active` pseudo-class SHALL be triggered on the target element

#### Scenario: Hover with trusted event

- **WHEN** `locator.hover()` is performed
- **THEN** the system SHALL dispatch using CDP `Input.dispatchMouseEvent` with type `mouseMoved`
- **AND** CSS `:hover` pseudo-class SHALL be triggered on the target element

#### Scenario: Keyboard input with trusted events

- **WHEN** `page.keyboard.press('Enter')` or `locator.press('Enter')` is performed
- **THEN** the system SHALL dispatch using CDP `Input.dispatchKeyEvent` with `rawKeyDown` + `char` + `keyUp`
- **AND** the resulting `keydown`, `keypress`, and `keyup` events SHALL have `isTrusted === true`
- **WHEN** `locator.pressSequentially('hello')` is performed
- **THEN** the system SHALL dispatch individual key events for each character
- **AND** `beforeinput` events SHALL fire on `contenteditable` elements

#### Scenario: Touch events

- **WHEN** touch emulation is enabled and a tap is performed
- **THEN** the system SHALL dispatch using CDP `Input.dispatchTouchEvent` with `touchStart` + `touchEnd`
- **AND** each touch point SHALL include position coordinates and rotation angle
