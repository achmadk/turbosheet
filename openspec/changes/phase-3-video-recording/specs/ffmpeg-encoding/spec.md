## ADDED Requirements

### Requirement: Encoder pipeline feeds frames to FFmpeg subprocess

The encoder SHALL spawn an `ffmpeg` subprocess and pipe decoded JPEG frames to its stdin for video encoding.

#### Scenario: FFmpeg subprocess is spawned with correct arguments

- **WHEN** the encoder pipeline is initialized
- **THEN** an `ffmpeg` subprocess SHALL be spawned with arguments: `-f image2pipe -framerate <fps> -i - -c:v libvpx -b:v <bitrate> -y <output_path>`
- **THEN** the subprocess stdin SHALL be held open for frame data

#### Scenario: Frames are written to stdin in sequence

- **WHEN** individual decoded JPEG frames are pushed to the encoder
- **THEN** each frame SHALL be written to the FFmpeg subprocess's stdin in the order received
- **THEN** the encoder SHALL not block on frame writes (uses a tokio blocking task or async pipe writer)

#### Scenario: Encoder flushes and waits on stop

- **WHEN** the encoder pipeline is stopped
- **THEN** stdin SHALL be closed
- **THEN** the encoder SHALL wait for the FFmpeg subprocess to exit
- **THEN** the subprocess exit status SHALL be checked and non-zero exit codes SHALL surface as errors

### Requirement: Encoder handles FFmpeg not found

The encoder SHALL detect when the `ffmpeg` binary is not available and surface a clear error.

#### Scenario: FFmpeg not found returns actionable error

- **WHEN** the encoder pipeline is initialized but `ffmpeg` is not installed or not in PATH
- **THEN** a descriptive error SHALL be returned indicating the missing dependency
- **THEN** the error SHALL suggest installation instructions or mention the auto-download mechanism

### Requirement: Encoder supports configurable parameters

The encoder SHALL accept configuration for frame rate, video bitrate, codec, and output path.

#### Scenario: Custom frame rate is passed through

- **WHEN** the encoder is initialized with `frame_rate: 15`
- **THEN** the FFmpeg argument SHALL use `-framerate 15`

#### Scenario: Custom bitrate is passed through

- **WHEN** the encoder is initialized with `bitrate: "1000k"`
- **THEN** the FFmpeg argument SHALL use `-b:v 1000k`

### Requirement: Encoder provides safe frame buffer management

The encoder SHALL implement a bounded frame buffer to prevent unbounded memory growth if encoding cannot keep up with frame capture.

#### Scenario: Frame buffer is bounded

- **WHEN** the number of queued frames exceeds the configured maximum (default 300 frames at 10fps = 30s)
- **THEN** the oldest unencoded frames SHALL be dropped
- **THEN** a warning SHALL be logged indicating frame dropping
