## ADDED Requirements

### Requirement: Trace Viewer Local UI

The system SHALL provide a local web UI to load and visualize JSON trace files generated during test execution.

#### Scenario: Visualizing a trace file

- **WHEN** the user opens the Trace Viewer and selects a trace file
- **THEN** the viewer displays a scrubbable timeline with DOM snapshots, network events, and console logs

### Requirement: DOM Rehydration

The trace viewer SHALL accurately rehydrate and render the DOM state for a given timestamp using the captured HTML and CSS.

#### Scenario: Inspecting element state at failure

- **WHEN** the user clicks on an action marker in the timeline
- **THEN** the DOM rehydrates in an isolated iframe exactly as it appeared at that moment, without executing scripts
