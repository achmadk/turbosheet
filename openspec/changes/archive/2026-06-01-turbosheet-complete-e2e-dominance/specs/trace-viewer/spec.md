## ADDED Requirements

### Requirement: Trace recording

TurboSheet SHALL record test execution traces including actions, network events, console output, and DOM snapshots.

#### Scenario: Record test actions

- **WHEN** a test executes actions (click, type, navigate)
- **THEN** each action SHALL be recorded with timestamp, selector, value, duration, and result

#### Scenario: Record network events

- **WHEN** the browser makes HTTP requests during test execution
- **THEN** each request and response SHALL be recorded with URL, status, timing, headers, and body

#### Scenario: Record console output

- **WHEN** the browser logs console messages (log, warn, error, info)
- **THEN** each message SHALL be captured with timestamp, level, and text

#### Scenario: Record DOM snapshots

- **WHEN** user configures `snapshotInterval: 'each-action'` or `'on-failure'`
- **THEN** TurboSheet SHALL capture DOM snapshots using the AI-native `getAgentSnapshot()` format

### Requirement: Trace file format

Trace files SHALL use CBOR (binary JSON) with zstd compression for efficient storage.

#### Scenario: Trace file size

- **WHEN** a 100-action test produces a trace file
- **THEN** the compressed file SHALL be at most 5MB (target: 2MB)

#### Scenario: Trace file extension

- **WHEN** traces are written to disk
- **THEN** they SHALL use `.tsheet-trace` extension

### Requirement: Trace viewer

TurboSheet SHALL provide a web-based trace viewer for inspecting recorded traces.

#### Scenario: Open trace viewer

- **WHEN** user runs `tsheet show-trace ./path/to/test.tsheet-trace`
- **THEN** TurboSheet SHALL open a local web server with the interactive trace viewer

#### Scenario: Timeline navigation

- **WHEN** user opens a trace in the viewer
- **THEN** actions SHALL be displayed in a chronological timeline with screenshots at key frames

#### Scenario: Step inspection

- **WHEN** user clicks on an action in the timeline
- **THEN** viewer SHALL show the DOM snapshot at that moment, console state, and network activity

### Requirement: Trace viewer embedded in HTML reporter

The HTML reporter SHALL include inline trace viewing capability.

#### Scenario: View trace from HTML report

- **WHEN** user opens the HTML test report
- **THEN** each failed test SHALL have a "View Trace" button that opens the trace viewer inline

### Requirement: Rust-compressed trace storage

Trace compression SHALL use Rust's zstd implementation for real-time compression during recording.

#### Scenario: Real-time compression

- **WHEN** trace data is being recorded during test execution
- **THEN** data SHALL be compressed in Rust before crossing the napi-rs bridge to Node.js
