## ADDED Requirements

### Requirement: Interactive debug mode

TurboSheet SHALL provide an interactive debug runner that shows a command log with step-through capabilities.

#### Scenario: Start debug mode

- **WHEN** user runs `tsheet debug --url https://app.com`
- **THEN** TurboSheet SHALL open a browser and a debug panel showing the command log

#### Scenario: Command log display

- **WHEN** actions are executed in the browser
- **THEN** each action SHALL appear in the command log with status (pending, passed, failed)

#### Scenario: Step-through execution

- **WHEN** user clicks "Step" in the debug panel
- **THEN** the next action SHALL execute and pause, showing the DOM snapshot after execution

### Requirement: DOM snapshot on pause

After each step, TurboSheet SHALL capture a DOM snapshot using the AI-native format.

#### Scenario: Snapshot display

- **WHEN** execution pauses after a step
- **THEN** the DOM snapshot SHALL be displayed in the debug panel as a visual tree (not raw HTML)

#### Scenario: Snapshot exploration

- **WHEN** user clicks on an element in the snapshot tree
- **THEN** the element's computed properties (visibility, text, attributes) SHALL be displayed

### Requirement: Live browser context

The debug runner SHALL maintain a live browser connection for interactive exploration.

#### Scenario: Execute ad-hoc command

- **WHEN** user types `page.evaluate(() => document.title)` in the debug console
- **THEN** the command SHALL execute in the live browser and return the result

#### Scenario: Modify and continue

- **WHEN** user edits a selector in the command log and re-runs it
- **THEN** the modified command SHALL execute in the same browser session
