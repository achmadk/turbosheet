## ADDED Requirements

### Requirement: Video Recording

The system SHALL support recording browser interactions as video files via CDP `Page.startScreencast`.

#### Scenario: Starting a video recording

- **WHEN** `context.newPage({ recordVideo: { dir: './videos' } })` is called
- **THEN** the system SHALL start capturing frames via CDP `Page.startScreencast`
- **AND** SHALL store captured frames in a buffer until the page is closed

#### Scenario: Video recording with quality options

- **WHEN** `context.newPage({ recordVideo: { dir: './videos', width: 1280, height: 720, quality: 80 } })` is called
- **THEN** the system SHALL capture frames at the specified resolution and JPEG quality
- **AND** SHALL target 30fps capture rate

#### Scenario: Auto-finalize on page close

- **WHEN** a page with active video recording is closed
- **THEN** the system SHALL finalize the video file
- **AND** SHALL encode all captured frames into a WebM file
- **AND** SHALL save the file to the configured directory with a descriptive filename

#### Scenario: Video file accessibility

- **WHEN** a test completes with video recording
- **THEN** the test report SHALL include a link or path to the video file
- **AND** the video SHALL be playable in standard media players
