## Why

Video recording of test runs is the single largest gap between TurboSheet and Playwright. Playwright already records traces with screencast video, making failures debuggable at a glance. Without video, TurboSheet users must reproduce failures blind — no visual timeline, no way to see what the browser showed before an assertion failed. Adding CDP-based screencast recording closes this gap and brings TurboSheet to feature parity with Playwright's tracing experience.

## What Changes

- **CDP Screencast integration**: Subscribe to `Page.screencastFrame` CDP events via `Page.startScreencast`, decode JPEG/PNG frame data, and route frames through an encoding pipeline
- **FFmpeg encoding pipeline**: Encode raw screencast frames into a video file (WebM/MP4) using either a native FFmpeg binding or subprocess-based encoding with configurable quality, resolution, and frame rate
- **Video storage & trace attachment**: Store encoded videos in a per-test output directory and attach them to the existing trace format so the trace viewer can replay them alongside network/console events
- **Recorder JS API**: Expose `page.video().start()` / `page.video().stop()` / `context.video()` on the JS side, matching Playwright's `page.video()` API surface
- **Automatic recording in test runner**: Option to auto-record video for each test (configurable via `use.video` in test config), with cleanup logic for passed tests

## Capabilities

### New Capabilities

- `screencast-capture`: CDP `Page.startScreencast` / `Page.screencastFrame` integration — frame subscription, quality/format configuration, frame routing to encoder
- `ffmpeg-encoding`: FFmpeg-based video encoding pipeline — takes raw frames, produces a playable video file with configurable codec, quality, and frame rate
- `video-storage`: Storage of encoded video files and attachment to the trace format so the TraceViewer can display videos alongside existing event data
- `recorder-api`: JS API (`page.video()`, `context.video()`) for controlling recording lifecycle — start, stop, path, and metadata

### Modified Capabilities

- `trace-recorder`: Extend the existing trace system (in `src/trace/`) to include screencast frame metadata as a new event type, enabling timeline-synchronized playback in the trace viewer

## Impact

- **`src/engine/chromium.rs`**: Add CDP `Page.startScreencast` / `Page.screencastFrame` handling — new method on `ChromiumPageEngine`, frame event subscription via existing CDP event stream pattern
- **`Cargo.toml`**: New feature flag `video` (default-off), optional FFmpeg binding crate (e.g., `ffmpeg-next` or custom subprocess wrapper)
- **`src/video/`**: New module — encoder pipeline, frame buffer, output management
- **`src/trace/`**: Extend `TraceEvent` to include `ScreencastFrame` variant; modify `TraceRecorder` and viewer to handle video attachment
- **`index.js`**: New `VideoRecorder` JS class exposing Playwright-compatible API
- **Build time**: FFmpeg binding may add compile-time dependency; alternatively use subprocess approach with `ffmpeg` binary (no build-time dep)
- **Disk usage**: Video files for test runs can be large (1-5 MB per 30s test at moderate quality); storage management/culling strategy needed
