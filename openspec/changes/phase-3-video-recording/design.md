## Context

TurboSheet currently supports screenshot capture (`Page.CaptureScreenshot` via CDP) and trace recording (network events, console messages, DOM snapshots), but has no video recording capability. Playwright's trace viewer includes screencast video synchronized to the event timeline — this is the standard for visual test debugging. Without video, TurboSheet users debugging test failures must rely on static screenshots and text logs.

The chromium engine already uses chromiumoxide with a CDP session managed via `Arc<Mutex<Page>>`. Page operations use `page.execute(params)` for CDP commands and `page.event_stream()` for subscribing to CDP events. The existing trace system (`src/trace/`) uses a global recorder singleton with serialization to a compressed binary format and an HTML viewer.

## Goals / Non-Goals

**Goals:**

- Capture screencast video during test execution via CDP `Page.startScreencast`
- Encode captured frames into a playable video file (WebM/VP8 initially)
- Attach video files to the trace format so the TraceViewer can display them
- Expose a JS API matching Playwright's `page.video()` for manual control
- Support automatic recording per-test in the test runner config
- Feature-gate behind a `video` Cargo feature flag (default-off)

**Non-Goals:**

- Streaming video to a remote viewer in real time
- Multi-page or multi-context video synchronization
- Video trimming/editing (beyond what FFmpeg provides)
- Hardware-accelerated encoding (software-only initially)
- Audio recording

## Decisions

### Decision 1: FFmpeg subprocess vs native Rust binding

**Chosen: Subprocess FFmpeg (`std::process::Command`)**

Alternatives considered:

- **Native Rust binding (`ffmpeg-next` crate)**: Adds a complex native dependency with version compatibility issues across platforms. Increases build time significantly. FFmpeg's C API is notoriously hard to use correctly (memory management, corner cases).
- **Custom WebM muxer in pure Rust**: Most portable but a massive undertaking — implementing VP8/VP9 encoding and WebM container format from scratch is a full project in itself.
- **Subprocess FFmpeg**: Zero build-time dependency. FFmpeg is already commonly installed or can be auto-downloaded (like browsers). The subprocess pipeline (stdin pipe with raw frames) is well-understood and used by many tools. Negligible overhead compared to frame capture time.

**Trade-off**: Requires `ffmpeg` binary to be available at runtime. Mitigation: auto-download static ffmpeg binary via `binary_manager` (same pattern as browser binaries), or document as system dependency.

### Decision 2: Per-page VideoRecorder as a standalone struct

**Chosen: New `VideoRecorder` struct in `src/video/` module, owned by `ChromiumPageEngine`**

Rather than threading video state through the trace system, each `ChromiumPageEngine` instance owns an `Option<VideoRecorder>`. This keeps video lifecycle tied to page lifecycle. The recorder subscribes to CDP `Page.screencastFrame` events via the existing CDP event subscription mechanism and manages an internal frame buffer + FFmpeg subprocess.

**Rationale**: Clean separation of concerns. The trace system records _what happened_ (events); the video system records _what was shown_ (visual). They merge at the output layer (trace viewer loads video file referenced in trace metadata).

### Decision 3: CDP Screencast format — JPEG at 10 fps default

**Chosen: JPEG format, 10 fps, quality 80, MaxDimension 800x600**

CDP Screencast supports JPEG (smaller frames, faster transfer) and PNG (lossless). JPEG is preferred for performance — a 30s test at 10fps with JPEG produces ~2-5 MB of frames before encoding. PNG would be 10-20x larger. Quality 80 gives a good balance between size and clarity for debugging.

**Frame rate choice**: 10 fps is sufficient for debugging test flow (seeing navigation, clicks, form fills). Higher frame rates (30fps) would improve smoothness but multiply frame data 3x. Make frame rate configurable.

### Decision 4: Video file format — WebM in VP8

**Chosen: WebM container with VP8 codec via `ffmpeg -c:v libvpx`**

WebM is royalty-free, widely supported in browsers (trace viewer), and VP8 encoding is fast in software. VP9 would give better compression but is slower. MP4/H.264 requires patent-encumbered codecs — avoid for the trace viewer use case.

### Decision 5: Trace attachment via file path reference

**Chosen: Video file path stored in trace metadata, loaded asynchronously by viewer**

The trace format stores a `screencast_path` metadata field pointing to the video file relative to the trace output directory. The trace viewer loads the video via a `<video>` element when the user opens that trace. This avoids bloating the trace binary with video data.

**Alternative considered**: Inline base64 video data in trace — rejected because trace files would become enormous and slow to load.

### Decision 6: Frame ack via dedicated event handler task

**Chosen: Spawn a tokio task that listens on `page.event_stream()` for `Page.screencastFrame`, decodes and queues each frame, then immediately sends `Page.screencastFrameAck`**

CDP requires `Page.screencastFrameAck` to be sent for every frame received — otherwise Chrome stops sending frames. The ack must happen asynchronously and not block the encoding pipeline. The event handler appends frame data to a `mpsc::UnboundedSender<Vec<u8>>` that the encoder task consumes.

## Risks / Trade-offs

| Risk                                        | Mitigation                                                                                                                         |
| ------------------------------------------- | ---------------------------------------------------------------------------------------------------------------------------------- |
| FFmpeg binary not found at runtime          | Auto-download static ffmpeg binary via `binary_manager::BrowserBinaryManager` pattern; document as known dependency                |
| Screencast frame rate drops under load      | Make frame rate and quality configurable; CDP's `maxInterval` can throttle if encoding lags                                        |
| Large video files consume disk space        | Implement configurable retention policy (delete on pass, keep on failure); warn when disk usage exceeds threshold                  |
| Video encoding is CPU-intensive             | Run encoding in a separate tokio blocking task; don't block the main event loop; consider frame dropping if buffer grows too large |
| CDP screencast won't work in headless mode  | Chrome 112+ supports screencast in headless mode; verify and document minimum Chrome version                                       |
| `Page.screencastFrame` events stop arriving | Implement watchdog timer — if no frame received for 5s, restart screencast or surface error                                        |
| Video feature increases binary size         | Feature-gate behind `video` Cargo feature flag; video module excluded from default build                                           |

## Migration Plan

1. **Phase 3a — Core CDP integration (Week 1)**
   - Add `video` feature flag to `Cargo.toml`
   - Create `src/video/mod.rs` with `VideoRecorder` struct
   - Implement CDP screencast subscribe/ack flow in `ChromiumPageEngine`
   - Test with a simple frame dump (save frames as JPEG files for inspection)

2. **Phase 3b — FFmpeg encoding pipeline (Week 2)**
   - Implement FFmpeg subprocess encoder in `src/video/encoder.rs`
   - Pipe decoded frames to FFmpeg stdin
   - Handle encoder lifecycle (start, feed, stop, wait)
   - Test with known frame sequences

3. **Phase 3c — Video storage + trace integration (Week 2)**
   - Implement video file output to `test-results/<test>/video.webm`
   - Extend `TraceEvent` with `ScreencastFrame` event variant
   - Modify `TraceRecorder` to reference video file in trace metadata
   - Update TraceViewer HTML to load and display video

4. **Phase 3d — JS API + test runner integration (Week 3)**
   - Add `page.video()` JS API surface
   - Add `context.video()` for context-level recording
   - Add `use.video` config option in test runner
   - Wire automatic recording lifecycle to test hooks
   - Integration tests

## Open Questions

- Should video recording be enabled by default in headed mode (like Playwright)? Recommended: opt-in via `use.video` config.
- What video retention strategy? Standard: keep video on failure only, delete on pass. Configurable.
- Should we support MP4/H.264 for non-trace-viewer use cases (CI artifact upload?)? Defer to future.

## Implementation Notes

### CDP API Surface Used

```
Page.startScreencast
  Parameters: { format: "jpeg", quality: 80, maxWidth: 800, maxHeight: 600, everyNthFrame: 1 }
  Returns: ScreencastStarted

Page.screencastFrame  (event)
  Parameters: { data: base64, metadata: { pageScaleFactor, offsetTop, deviceWidth, deviceHeight, scrollOffsetX, scrollOffsetY, timestamp } }

Page.screencastFrameAck
  Parameters: { sessionId: number }

Page.stopScreencast
```

### FFmpeg Command Structure

```bash
# Receive raw JPEG frames via stdin pipe, convert to video
ffmpeg -f image2pipe -framerate 10 -i - -c:v libvpx -b:v 500k -y output.webm
```

### Frame Flow Architecture

```
CDP Page.screencastFrame
       │
       ▼
  event_stream listener task
       │
       ▼
  decode base64 → Vec<u8> (JPEG bytes)
       │
       ▼
  send ack (Page.screencastFrameAck)
       │
       ▼
  mpsc::UnboundedSender ──► mpsc::UnboundedReceiver
                                  │
                                  ▼
                         FFmpeg subprocess (stdin)
                                  │
                                  ▼
                            output.webm
```
