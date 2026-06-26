## 1. Foundation & Module Scaffolding

- [x] 1.1 Add `video` feature flag to `Cargo.toml` (default-off, enabled by `--features video`)
- [x] 1.2 Create `src/video/mod.rs` with top-level module structure and re-exports
- [x] 1.3 Create `src/video/recorder.rs` — `VideoRecorder` struct skeleton (start/stop methods, state machine)
- [x] 1.4 Create `src/video/encoder.rs` — encoder pipeline struct skeleton
- [x] 1.5 Create `src/video/config.rs` — `VideoConfig` struct with frame rate, quality, dimensions, output path, retention policy
- [x] 1.6 Add `mod video` to `src/lib.rs` behind `#[cfg(feature = "video")]`
- [x] 1.7 Register napi exports for video JS API types behind feature flag

## 2. CDP Screencast Integration

- [ ] 2.1 Implement `VideoRecorder.start()` — build `Page.startScreencast` params, send CDP command via `page.execute()`
- [ ] 2.2 Subscribe to `Page.screencastFrame` events — spawn tokio task that reads from `page.event_stream()` and filters for screencast frame events
- [ ] 2.3 Implement frame decoding — extract base64 data from event payload, decode to `Vec<u8>`
- [ ] 2.4 Implement `Page.screencastFrameAck` — send ack with `sessionId` immediately after decoding each frame
- [ ] 2.5 Implement frame queue — `mpsc::UnboundedSender<Vec<u8>>` from event handler to encoder consumer
- [ ] 2.6 Implement `VideoRecorder.stop()` — send `Page.stopScreencast`, flush encoder, cancel event handler task
- [ ] 2.7 Implement error recovery — watchdog timer for frame starvation, handle page closed during recording
- [ ] 2.8 Wire `VideoRecorder` into `ChromiumPageEngine` — store `Option<VideoRecorder>`, pass page ref on construction

## 3. FFmpeg Encoding Pipeline

- [ ] 3.1 Implement FFmpeg binary discovery — check PATH, support configurable path, produce actionable error if not found
- [ ] 3.2 Implement subprocess spawning — `std::process::Command` with `ffmpeg -f image2pipe -framerate <fps> -i - -c:v libvpx -b:v <bitrate> -y <output_path>`
- [ ] 3.3 Implement frame pipe writer — write decoded JPEG bytes to subprocess stdin in order received
- [ ] 3.4 Implement encoder flush — close stdin on stop, wait for subprocess exit, check exit status
- [ ] 3.5 Implement bounded frame buffer — max pending frames (default 300), drop oldest when exceeded, log warning
- [ ] 3.6 Implement config passthrough — forward frame_rate, bitrate, codec, output_path from `VideoConfig` to FFmpeg args
- [ ] 3.7 Implement auto-download of static FFmpeg binary (optional) — extend `binary_manager` or create separate downloader

## 4. Video Storage & Trace Integration

- [ ] 4.1 Implement video output path resolution — create per-test directory (e.g., `test-results/<test-name>/`), ensure parent dirs exist
- [ ] 4.2 Add `ScreencastFrame` variant to `TraceEvent` enum in `src/trace/recorder.rs` — store frame_index, timestamp
- [ ] 4.3 Add `screencast` field to trace metadata — `{ path, frame_count, duration_seconds, fps }` written during trace serialization
- [ ] 4.4 Record `ScreencastFrame` events from `VideoRecorder` — push events to `TraceRecorder` during recording
- [ ] 4.5 Update `TraceViewer` HTML — add `<video>` element that loads from `screencast.path`
- [ ] 4.6 Implement timeline–video seek — when user clicks a trace event, seek video to approximate timestamp
- [ ] 4.7 Implement video retention — clean up video files based on `VideoConfig.retain` policy (on-failure/always/never)

## 5. JS API

- [ ] 5.1 Create `VideoRecorderHandle` napi struct — wrapping `Arc<VideoRecorder>` (or channel-based bridge)
- [ ] 5.2 Implement `JsPage.video()` — return `VideoRecorderHandle` for chromium, `null` for other engines
- [ ] 5.3 Implement `handle.start()` — call through to Rust `VideoRecorder.start()`, no-op if already started
- [ ] 5.4 Implement `handle.stop()` — call through to Rust `VideoRecorder.stop()`, return video path (or null)
- [ ] 5.5 Implement `handle.path()` — return video file path after recording completes, null before
- [ ] 5.6 Implement `JsBrowserContext.video(options)` — configure context-level recording options
- [ ] 5.7 Export types in `index.d.ts` — add `VideoRecorderHandle`, video-related types
- [ ] 5.8 Add JS-side `ComponentLocator.video()` pass-through (delegate to page)

## 6. Test Runner Integration

- [ ] 6.1 Add `use.video` config option to test runner config schema
- [ ] 6.2 Implement `VideoMode` enum — `Off`, `On`, `OnFirstRetry`, `RetainOnFailure`
- [ ] 6.3 Wire automatic recording to test lifecycle — start before test, stop after test, attach to result
- [ ] 6.4 Implement `on-first-retry` mode — skip recording on first run, record on retry
- [ ] 6.5 Attach video path to test result output — available in JSON reports and HTML reporter

## 7. Testing & Verification

- [ ] 7.1 Unit test `VideoConfig` — serialization, defaults, custom overrides
- [ ] 7.2 Unit test bounded frame buffer — overflow behavior, frame dropping, warnings
- [ ] 7.3 Unit test FFmpeg argument construction — verify command-line args match expected format
- [ ] 7.4 Integration test: start screencast, capture N frames, stop, verify video file exists and is playable
- [ ] 7.5 Integration test: video is retained on simulated test failure, deleted on pass
- [ ] 7.6 Integration test: `page.video().start()` → `stop()` returns valid path from JS
- [ ] 7.7 Verify existing 119 tests still pass with `video` feature enabled (no regression)
- [ ] 7.8 Verify release build compiles with `--features video`
