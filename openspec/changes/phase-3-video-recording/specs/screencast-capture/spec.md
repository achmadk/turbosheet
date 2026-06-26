## ADDED Requirements

### Requirement: VideoRecorder starts CDP screencast on a page

`VideoRecorder` SHALL subscribe to CDP `Page.screencastFrame` events and initiate `Page.startScreencast` when recording begins.

#### Scenario: Start screencast sends CDP command

- **WHEN** `VideoRecorder.start()` is called on an active page
- **THEN** `Page.startScreencast` SHALL be called with JPEG format, configurable quality (default 80), configurable max dimensions (default 800x600), and `everyNthFrame: 1`
- **THEN** The CDP event subscription for `Page.screencastFrame` SHALL be active

#### Scenario: Start screencast returns error on closed page

- **WHEN** `VideoRecorder.start()` is called on a closed page
- **THEN** it SHALL return an error indicating the page is no longer available

### Requirement: VideoRecorder processes screencast frames

`VideoRecorder` SHALL listen for `Page.screencastFrame` CDP events and route each received frame through the encoding pipeline.

#### Scenario: Frame received is decoded and sent to encoder

- **WHEN** a `Page.screencastFrame` event is received with base64-encoded JPEG data
- **THEN** the base64 data SHALL be decoded into raw bytes
- **THEN** the decoded frame bytes SHALL be pushed to the FFmpeg encoder pipeline
- **THEN** `Page.screencastFrameAck` SHALL be immediately sent with the frame's `sessionId`

#### Scenario: Frame received with empty data is skipped

- **WHEN** a `Page.screencastFrame` event is received with empty or null data
- **THEN** the frame SHALL be skipped (no error, no encoding)
- **THEN** `Page.screencastFrameAck` SHALL still be sent

### Requirement: VideoRecorder stops screencast

`VideoRecorder` SHALL send `Page.stopScreencast` when recording is stopped.

#### Scenario: Stop sends CDP command and finalizes encoder

- **WHEN** `VideoRecorder.stop()` is called
- **THEN** `Page.stopScreencast` SHALL be called via CDP
- **THEN** the encoding pipeline SHALL be flushed and finalized
- **THEN** the encoded video file path SHALL be returned

#### Scenario: Stop on inactive recording is no-op

- **WHEN** `VideoRecorder.stop()` is called when recording was never started or already stopped
- **THEN** it SHALL return without error and without sending any CDP command
