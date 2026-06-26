# complete-locator-api

## ADDED Requirements

### Requirement: scrollIntoView

The system SHALL scroll an element into view before performing actions.

#### Scenario: Scroll element into view

- **WHEN** user calls `locator.scrollIntoView()`
- **THEN** system scrolls the element into the viewport

### Requirement: focus and blur

The system SHALL support focusing and blurring elements.

#### Scenario: Focus element

- **WHEN** user calls `locator.focus()`
- **THEN** system focuses the element

#### Scenario: Blur element

- **WHEN** user calls `locator.blur()`
- **THEN** system removes focus from the element

### Requirement: boundingBox

The system SHALL return the bounding box of an element.

#### Scenario: Get visible element bounding box

- **WHEN** user calls `locator.boundingBox()` on a visible element
- **THEN** system returns `{ x, y, width, height }` with pixel values

#### Scenario: Get hidden element bounding box

- **WHEN** user calls `locator.boundingBox()` on a hidden element
- **THEN** system returns `null`

### Requirement: locator screenshot

The system SHALL capture a screenshot of a specific element.

#### Scenario: Capture element screenshot

- **WHEN** user calls `locator.screenshot()`
- **THEN** system captures and returns PNG image of the element

### Requirement: dragAndDrop

The system SHALL support drag and drop operations.

#### Scenario: Drag and drop element

- **WHEN** user calls `sourceLocator.dragAndDrop(targetLocator)`
- **THEN** system performs drag from source to target element

### Requirement: hover

The system SHALL hover over an element.

#### Scenario: Hover over element

- **WHEN** user calls `locator.hover()`
- **THEN** system moves mouse cursor over the element

### Requirement: press and pressSequentially

The system SHALL simulate keyboard key presses.

#### Scenario: Press single key

- **WHEN** user calls `locator.press('Enter')`
- **THEN** system presses the Enter key on the element

#### Scenario: Press key sequence

- **WHEN** user calls `locator.pressSequentially('hello')`
- **THEN** system types 'hello' into the element

### Requirement: setInputFiles

The system SHALL support file upload via input elements.

#### Scenario: Upload single file

- **WHEN** user calls `locator.setInputFiles('/path/to/file.txt')`
- **THEN** system sets the file as the input value

#### Scenario: Upload multiple files

- **WHEN** user calls `locator.setInputFiles(['/path/a.txt', '/path/b.txt'])`
- **THEN** system sets both files as input values

#### Scenario: Clear file input

- **WHEN** user calls `locator.setInputFiles([])`
- **THEN** system clears the file input
