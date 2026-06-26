## ADDED Requirements

### Requirement: Encoded video is saved to test output directory

The video encoder SHALL write the output file to a configurable path within the test results directory.

#### Scenario: Video file exists after recording stops

- **WHEN** recording is stopped after a successful capture-and-encode cycle
- **THEN** a valid video file SHALL exist at the configured output path
- **THEN** the video file SHALL be playable by standard video players and browsers

#### Scenario: Output directory is created if missing

- **WHEN** the configured output directory does not exist
- **THEN** the directory SHALL be created (including parent directories)

### Requirement: Video file is attached to trace metadata

The recorded video SHALL be referenced in the trace output so the TraceViewer can load it.

#### Scenario: Video path is stored in trace metadata

- **WHEN** recording stops and produces a video file
- **THEN** the trace metadata SHALL include a `screencast` field with the relative path to the video file
- **THEN** the trace metadata SHALL include the video duration and frame count

#### Scenario: Trace without video has no screencast field

- **WHEN** recording was not enabled during a test run
- **THEN** the trace metadata SHALL NOT include a `screencast` field
- **THEN** existing traces without video SHALL continue to load without error

### Requirement: Video retention respects test result

The system SHALL support configurable video retention policies based on test outcome.

#### Scenario: Keep video on failure only

- **WHEN** `video.retain` is set to `"on-failure"` (default)
- **WHEN** the test passes
- **THEN** the video file SHALL be deleted after trace serialization
- **WHEN** the test fails
- **THEN** the video file SHALL be preserved

#### Scenario: Always keep video

- **WHEN** `video.retain` is set to `"always"`
- **THEN** video files SHALL be preserved regardless of test outcome

#### Scenario: Never keep video

- **WHEN** `video.retain` is set to `"never"`
- **THEN** video files SHALL be deleted immediately after encoding
