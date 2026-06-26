## ADDED Requirements

### Requirement: Trace recording

The system SHALL record comprehensive traces during test execution.

#### Scenario: Record action events

- **WHEN** `page.locator('#btn').click()` is called
- **THEN** system records action type, selector, timestamp, duration
- **AND** result (success/error)

#### Scenario: Record DOM snapshots

- **WHEN** action is performed
- **THEN** system captures DOM snapshot at key points
- **AND** stores minimal HTML diff

#### Scenario: Record network events

- **WHEN** network request is made
- **THEN** system records URL, method, status, timing
- **AND** request/response headers

#### Scenario: Record console events

- **WHEN** page logs to console
- **THEN** system captures console level, message, args
- **AND** timestamp

### Requirement: Trace file format

The system SHALL use zstd-compressed CBOR format for trace files.

#### Scenario: Compress trace data

- **WHEN** trace recording completes
- **THEN** system encodes events as CBOR
- **AND** compresses with zstd
- **AND** writes to `.turboTrace` file

#### Scenario: File structure

- **WHEN** trace file is created
- **THEN** file has magic header "TTRC"
- **AND** version number
- **AND** entry count
- **AND** timestamp

#### Scenario: Compression ratio

- **WHEN** trace data is 100MB uncompressed
- **THEN** compressed file is approximately 5-10MB
- **AND** decompression takes < 1 second

### Requirement: Trace viewer web interface

The system SHALL provide a web-based trace viewer.

#### Scenario: Render action timeline

- **WHEN** trace file is loaded in viewer
- **THEN** viewer displays timeline of actions
- **AND** shows duration for each action

#### Scenario: Show DOM at point in time

- **WHEN** user clicks action in timeline
- **THEN** viewer shows DOM snapshot at that moment
- **AND** highlights selected element

#### Scenario: Network waterfall

- **WHEN** trace contains network events
- **THEN** viewer displays waterfall chart
- **AND** shows timing breakdown

#### Scenario: Console log viewer

- **WHEN** trace contains console events
- **THEN** viewer displays console output
- **AND** allows filtering by level

### Requirement: Trace CLI tools

The system SHALL provide CLI commands for trace management.

#### Scenario: View trace summary

- **WHEN** `tsheet trace info test.turboTrace` is called
- **THEN** system prints trace metadata
- **AND** shows entry counts by type

#### Scenario: Export trace as JSON

- **WHEN** `tsheet trace export test.turboTrace --format=json` is called
- **THEN** system exports decompressed trace as JSON

#### Scenario: Merge trace files

- **WHEN** `tsheet trace merge test1.turboTrace test2.turboTrace` is called
- **THEN** system combines traces chronologically

### Requirement: Trace server mode

The system SHALL run a trace server for real-time viewing.

#### Scenario: Start trace server

- **WHEN** `tsheet trace serve --port 8080` is called
- **THEN** system starts HTTP server
- **AND** serves trace viewer UI

#### Scenario: Connect to live test

- **WHEN** test runs with `recordTrace: true`
- **THEN** events stream to trace server in real-time
- **AND** viewer updates live

### Requirement: Trace annotations

The system SHALL support adding annotations to traces.

#### Scenario: Add label to action

- **WHEN** `page.locator('#btn').click({ label: 'Submit form' })` is called
- **THEN** trace contains label for that action

#### Scenario: Attach screenshot to action

- **WHEN** `page.locator('#btn').click({ traceScreenshot: true })` is called
- **THEN** trace contains screenshot at action point
