## MODIFIED Requirements

### Requirement: TraceRecorder records ScreencastFrame events (was: TraceRecorder records action, network, console, and DOM events)

The `TraceRecorder` SHALL support a new `ScreencastFrame` event type alongside existing action, network, console, and DOM event types.

#### Scenario: ScreencastFrame trace event is recorded

- **WHEN** a screencast frame is captured with timestamp `T` at frame index `N`
- **THEN** the `TraceRecorder` SHALL record a `ScreencastFrame` trace event containing the frame index `N`, timestamp `T`, and no image data (video file is stored separately)
- **THEN** the event SHALL be inserted in chronological order with other trace events

#### Scenario: Trace metadata includes video reference

- **WHEN** a video recording has been completed for a test
- **THEN** the trace metadata SHALL include `screencast: { path: "relative/path/video.webm", frame_count: N, duration_seconds: D, fps: F }`

### Requirement: TraceViewer displays video alongside timeline (was: TraceViewer displays action list, network waterfall, console, and accessibility tree)

The TraceViewer SHALL support loading and displaying a screencast video synchronized to the event timeline.

#### Scenario: Trace with video shows video player

- **WHEN** a trace is loaded that has a `screencast` metadata field
- **THEN** the TraceViewer SHALL render a `<video>` element in the viewer
- **THEN** the video player SHALL be positioned alongside the event timeline

#### Scenario: Trace without video renders normally

- **WHEN** a trace is loaded that has no `screencast` metadata field
- **THEN** the TraceViewer SHALL render without any video UI
- **THEN** existing trace viewer behavior SHALL be unchanged

#### Scenario: Video playback can be paused at a timeline event

- **WHEN** a user clicks on an event in the timeline
- **THEN** the video player SHOULD seek to the approximate time of that event (based on event timestamp vs video start time)
