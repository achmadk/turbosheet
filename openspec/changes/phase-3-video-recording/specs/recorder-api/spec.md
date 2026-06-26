## ADDED Requirements

### Requirement: Page exposes video() handle

`JsPage` SHALL expose a `video()` method that returns a `VideoRecorderHandle` for controlling recording.

#### Scenario: video() returns handle on supported engine

- **WHEN** `page.video()` is called on a chromium-engine page
- **THEN** it SHALL return a `VideoRecorderHandle` object (or null if video feature is not enabled)

#### Scenario: video() returns null on unsupported engine

- **WHEN** `page.video()` is called on a firefox or webkit page
- **THEN** it SHALL return `null`

### Requirement: VideoRecorderHandle controls recording lifecycle

`VideoRecorderHandle` SHALL expose `start()` and `stop()` methods matching Playwright's `page.video()` API conventions.

#### Scenario: start() begins recording

- **WHEN** `handle.start()` is called
- **THEN** recording SHALL begin on the associated page
- **THEN** calling `start()` again while already recording SHALL be a no-op (not an error)

#### Scenario: stop() stops recording and returns path

- **WHEN** `handle.stop()` is called while recording
- **THEN** recording SHALL stop
- **THEN** it SHALL return the absolute path to the recorded video file (or null if no frames were captured)

#### Scenario: stop() on inactive recording returns null

- **WHEN** `handle.stop()` is called on a handle where `start()` was never called or already stopped
- **THEN** it SHALL return `null` without error

#### Scenario: path() returns recorded video path

- **WHEN** recording has stopped and a video file exists
- **THEN** `handle.path()` SHALL return the absolute path to the video file
- **WHEN** called before recording stops or when no recording was made
- **THEN** `handle.path()` SHALL return `null`

### Requirement: BrowserContext exposes video() for context-level recording

`JsBrowserContext` SHALL expose a `video()` method that returns a handle for setting up video recording options.

#### Scenario: context.video() configures recording options

- **WHEN** `context.video(options)` is called with `{ dir: "./videos" }`
- **THEN** subsequent pages created in this context SHALL automatically record video to the specified directory

### Requirement: Test runner supports use.video config

The test runner config SHALL support a `use.video` option for automatic per-test video recording.

#### Scenario: use.video enables auto-recording

- **WHEN** test config has `use.video: { mode: 'on-first-retry', dir: './test-results/videos' }`
- **THEN** video SHALL be recorded on the first retry of a failed test

#### Scenario: use.video mode 'on' records all tests

- **WHEN** test config has `use.video: { mode: 'on' }`
- **THEN** video SHALL be recorded for every test

#### Scenario: use.video attaches video to test result

- **WHEN** a test has an associated video file
- **THEN** the test result SHALL include a `video` field with the relative path to the video file
