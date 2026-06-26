## ADDED Requirements

### Requirement: DOM State Diffing on Failure

The system SHALL capture and diff the DOM state when a test assertion fails, showing exactly what changed.

#### Scenario: Capture DOM snapshot on assertion failure

- **GIVEN** a test assertion fails (e.g., `expect(locator).toHaveText('Submit')`)
- **WHEN** the assertion fails because the element has different text
- **THEN** the system SHALL capture the current DOM state of the relevant element and its subtree
- **AND** capture the expected DOM state (from the last known good state or stored snapshot)
- **AND** produce a diff showing what changed

#### Scenario: DOM diff output format

- **WHEN** a DOM diff is produced
- **THEN** it SHALL show:
  - Lines prefixed with `+` for nodes/content present only in the current state
  - Lines prefixed with `-` for nodes/content present only in the expected state
  - Attributes that changed (e.g., `class: "active" → "inactive"`)
  - Text content that changed (e.g., `"Submit" → "Submitting..."`)
- **AND** colorize the output in terminal (green for additions, red for removals, yellow for changes)
- **AND** limit output to the subtree of the relevant element (not entire page)

#### Scenario: Deep structural comparison

- **GIVEN** a complex DOM subtree
- **WHEN** a diff is requested
- **THEN** the system SHALL perform a tree-aware comparison:
  - Match nodes by a combination of selector path, index among siblings, and key attributes
  - Detect node insertion: show full subtree of inserted node
  - Detect node removal: show full subtree of removed node
  - Detect node reordering: show before/after positions
  - Detect text-only changes: show old text → new text
  - Detect attribute-only changes: show changed attributes only
- **AND** the comparison SHALL be computed in the Rust engine (not JS) for performance

#### Scenario: Snapshot-based DOM diffing

- **GIVEN** a test uses `expect(page).toMatchDOMSnapshot(name)`
- **WHEN** the DOM differs from the stored snapshot
- **THEN** the system SHALL produce a structured diff at the DOM node level
- **AND** include the diff in the test failure output
- **AND** save the diff as an artifact (JSON format) for CI debugging
- **AND** the diff artifact SHALL include: snapshot path, current DOM path, node-level changes array

#### Scenario: Selective diff scoping

- **GIVEN** a large page with a specific region of interest
- **WHEN** `expect(locator).toMatchDOMSnapshot(name)` is called on a specific locator
- **THEN** the diff SHALL be scoped to that locator's subtree only
- **AND** exclude changes outside the scoped region from the diff output
