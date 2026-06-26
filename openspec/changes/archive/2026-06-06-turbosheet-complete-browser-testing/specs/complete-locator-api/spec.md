## ADDED Requirements

### Requirement: Bounding box retrieval

The system SHALL return the bounding box of an element via real CDP calls.

#### Scenario: Get visible element bounding box

- **WHEN** `page.locator('#box').boundingBox()` is called on visible element
- **THEN** system returns `{ x, y, width, height }` with pixel values
- **AND** coordinates are relative to viewport

#### Scenario: Get bounding box of hidden element

- **WHEN** `page.locator('.hidden').boundingBox()` is called on hidden element
- **THEN** system returns `null`

#### Scenario: Bounding box of nested element

- **WHEN** `page.locator('.nested').boundingBox()` is called
- **THEN** system returns bounding box in page coordinates
- **AND** accounts for any transforms

### Requirement: Drag and drop

The system SHALL perform drag and drop operations via real CDP events.

#### Scenario: Drag by selector to selector

- **WHEN** `page.locator('#source').dragTo(page.locator('#target'))` is called
- **THEN** system dispatches dragstart on source
- **AND** dispatches dragover on target
- **AND** dispatches drop on target
- **AND** dispatches dragend on source

#### Scenario: Drag by coordinates

- **WHEN** `page.locator('#source').dragAndDrop(targetX, targetY)` is called
- **THEN** system calculates target position
- **AND** performs drag sequence to coordinates

### Requirement: Keyboard press

The system SHALL send keyboard press events via CDP.

#### Scenario: Press single key

- **WHEN** `page.locator('input').press('Enter')` is called
- **THEN** system sends keydown and keyup events
- **AND** includes correct key code

#### Scenario: Press key combination

- **WHEN** `page.locator('input').press('Control+a')` is called
- **THEN** system sends keydown for Control
- **AND** sends keydown for 'a'
- **AND** sends keyup for 'a'
- **AND** sends keyup for Control

#### Scenario: Press with modifiers

- **WHEN** `page.locator('input').press('Shift+Home')` is called
- **THEN** system sends events with Shift modifier active

### Requirement: Sequential text input

The system SHALL type text character by character with configurable delay.

#### Scenario: Type text sequentially

- **WHEN** `page.locator('input').pressSequentially('hello')` is called
- **THEN** system types each character in sequence
- **AND** default delay between characters is 0ms

#### Scenario: Type with delay

- **WHEN** `page.locator('input').pressSequentially('hello', { delay: 100 })` is called
- **THEN** system types each character with 100ms delay
- **AND** clears existing input value first

### Requirement: File input handling

The system SHALL handle file input elements to upload files.

#### Scenario: Set single file

- **WHEN** `page.locator('input[type=file]').setInputFiles('/path/to/file.txt')` is called
- **THEN** system dispatches change event with file
- **AND** file is available in input.files

#### Scenario: Set multiple files

- **WHEN** `page.locator('input[type=file]').setInputFiles(['file1.txt', 'file2.txt'])` is called
- **THEN** system sets multiple files on input

#### Scenario: Clear file input

- **WHEN** `page.locator('input[type=file]').setInputFiles([])` is called
- **THEN** system clears file selection

### Requirement: Wait for selector

The system SHALL wait for an element to be added to DOM.

#### Scenario: Wait for selector to appear

- **WHEN** `page.waitForSelector('.dynamic-content')` is called
- **THEN** system waits until element appears in DOM
- **AND** returns the element handle

#### Scenario: Wait with state option

- **WHEN** `page.waitForSelector('.hidden', { state: 'hidden' })` is called
- **THEN** system waits for element to become hidden

#### Scenario: Wait with timeout

- **WHEN** `page.waitForSelector('.never-appears', { timeout: 5000 })` is called
- **THEN** system throws error after timeout
