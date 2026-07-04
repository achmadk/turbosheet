## ADDED Requirements

### Requirement: TestRunner triggers video recording around test execution

When `video_on_failure` is enabled in the test config, the system SHALL start video recording before executing a test file and stop recording after execution completes or fails.

#### Scenario: Video recording starts before test

- **WHEN** `TestExecutor::execute_tests()` spawns a worker for a test file
- **AND** `config.video_on_failure` is `true`
- **THEN** `ChromiumPageEngine::start_recording()` SHALL be called before the test executes

#### Scenario: Video recording stops after test

- **WHEN** a test file execution completes
- **AND** recording was started
- **THEN** `ChromiumPageEngine::stop_recording()` SHALL be called with the test outcome

#### Scenario: Video path appears in test result

- **WHEN** a test completes with video recording enabled
- **THEN** the video file path SHALL be returned via `stop_recording()`
- **AND** `TestResult.video_paths` SHALL contain the path

#### Scenario: Retention policy deletes on pass

- **WHEN** a test passes
- **AND** retention policy is `OnFailure`
- **THEN** the recorded video file SHALL be deleted

#### Scenario: No video when feature disabled

- **WHEN** the `video` Cargo feature is not enabled
- **THEN** `start_recording()` SHALL return an error
- **AND** execution SHALL continue without recording

### Requirement: Video encoding produces valid output

The `VideoEncoder` SHALL produce a playable video file when frames are pushed via FFmpeg subprocess.

#### Scenario: Encoder produces output file

- **WHEN** `VideoEncoder::start()` is called
- **AND** JPEG frames are pushed via `push_frame()`
- **AND** `flush()` is called
- **THEN** a video file SHALL exist at the configured output path

#### Scenario: Missing ffmpeg returns error

- **WHEN** `VideoEncoder::start()` is called
- **AND** `ffmpeg` is not found in PATH
- **THEN** the method SHALL return a `TurbosheetError` with a clear message about missing ffmpeg

### Requirement: CDP screencast is managed correctly

The system SHALL start CDP `Page.startScreencast` when recording begins and stop it when recording ends.

#### Scenario: Screencast starts on record

- **WHEN** `start_recording()` succeeds
- **THEN** a `Page.startScreencast` command SHALL be sent via CDP with JPEG format

#### Scenario: Screencast stops on record end

- **WHEN** `stop_recording()` is called
- **THEN** a `Page.stopScreencast` command SHALL be sent via CDP
