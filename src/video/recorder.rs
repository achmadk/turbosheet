use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use tokio::sync::{Mutex, mpsc};

use crate::error::TurbosheetError;
use crate::video::config::VideoConfig;
use crate::video::encoder::VideoEncoder;

/// Recording state machine.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RecordingState {
    Idle,
    Starting,
    Recording,
    Stopping,
    Stopped,
    Failed,
}

/// High-level VideoRecorder that manages screencast capture and encoding.
///
/// Owned by `ChromiumPageEngine` as `Option<VideoRecorder>`.
pub struct VideoRecorder {
    state: Arc<Mutex<RecordingState>>,
    config: VideoConfig,
    encoder: Arc<VideoEncoder>,
    /// Channel for sending decoded frame data from the CDP event handler
    /// to the encoding pipeline.
    frame_tx: Arc<Mutex<Option<mpsc::UnboundedSender<Vec<u8>>>>>,
    /// Handle to the tokio task processing screencast frames.
    event_task: Arc<Mutex<Option<tokio::task::JoinHandle<()>>>>,
    /// Whether recording has been explicitly cancelled (e.g., page closed).
    cancelled: Arc<AtomicBool>,
    /// Total frames captured in the current recording session.
    frame_count: Arc<std::sync::atomic::AtomicU64>,
    /// Video output path (set after recording stops).
    output_path: Arc<Mutex<Option<String>>>,
}

impl VideoRecorder {
    /// Create a new VideoRecorder with the given config.
    pub fn new(config: VideoConfig) -> Self {
        let encoder = VideoEncoder::new(config.clone());
        Self {
            state: Arc::new(Mutex::new(RecordingState::Idle)),
            config,
            encoder: Arc::new(encoder),
            frame_tx: Arc::new(Mutex::new(None)),
            event_task: Arc::new(Mutex::new(None)),
            cancelled: Arc::new(AtomicBool::new(false)),
            frame_count: Arc::new(std::sync::atomic::AtomicU64::new(0)),
            output_path: Arc::new(Mutex::new(None)),
        }
    }

    /// Returns the current recording state.
    pub async fn state(&self) -> RecordingState {
        *self.state.lock().await
    }

    /// Returns the encoder.
    pub fn encoder(&self) -> &Arc<VideoEncoder> {
        &self.encoder
    }

    /// Returns the video configuration.
    pub fn config(&self) -> &VideoConfig {
        &self.config
    }

    /// Returns the frame count for the current recording session.
    pub fn frame_count(&self) -> u64 {
        self.frame_count.load(std::sync::atomic::Ordering::SeqCst)
    }

    /// Returns the output path (set after recording completes).
    pub async fn output_path(&self) -> Option<String> {
        self.output_path.lock().await.clone()
    }

    /// Start recording.
    ///
    /// This method prepares the encoder and returns a frame sender
    /// that the caller (ChromiumPageEngine) can use to push frames.
    /// The caller is responsible for:
    /// 1. Starting the CDP `Page.startScreencast`
    /// 2. Subscribing to `Page.screencastFrame` events
    /// 3. Decoding frames and pushing them via the returned sender
    pub async fn start(&self) -> Result<mpsc::UnboundedSender<Vec<u8>>, TurbosheetError> {
        let mut state = self.state.lock().await;
        match *state {
            RecordingState::Recording | RecordingState::Starting => {
                return Err(TurbosheetError::Other(
                    "Recording already in progress".to_string(),
                ));
            }
            _ => {}
        }
        *state = RecordingState::Starting;
        drop(state);

        self.encoder.start().await?;

        let (tx, mut rx) = mpsc::unbounded_channel::<Vec<u8>>();
        *self.frame_tx.lock().await = Some(tx.clone());

        let encoder = self.encoder.clone();
        let frame_count = self.frame_count.clone();
        let cancelled = self.cancelled.clone();
        let max_pending = self.config.max_pending_frames;
        let state_mutex = self.state.clone();
        let output_path = self.output_path.clone();
        let output_filename = self.config.output_filename.clone();

        let max_duration = self.config.max_recording_duration_secs;
        let handle = tokio::spawn(async move {
            let watchdog_sleep = tokio::time::sleep(std::time::Duration::from_secs(max_duration));
            tokio::pin!(watchdog_sleep);

            let mut frame_buffer: Vec<Vec<u8>> = Vec::with_capacity(max_pending);

            loop {
                tokio::select! {
                    Some(frame) = rx.recv() => {
                        if cancelled.load(Ordering::SeqCst) {
                            break;
                        }
                        if frame_buffer.len() >= max_pending {
                            frame_buffer.remove(0);
                            tracing::warn!("Video frame buffer full, dropping oldest frame");
                        }
                        frame_buffer.push(frame);
                        frame_count.fetch_add(1, Ordering::SeqCst);

                        while let Some(f) = frame_buffer.first() {
                            if encoder.push_frame(f).await.is_ok() {
                                frame_buffer.remove(0);
                            } else {
                                tracing::error!("Encoder pipe broken, dropping {} frames", frame_buffer.len());
                                frame_buffer.clear();
                                break;
                            }
                        }
                    }
                    () = &mut watchdog_sleep => {
                        tracing::warn!(
                            "Video recording watchdog fired after {}s — stopping",
                            max_duration
                        );
                        break;
                    }
                    else => break,
                }
            }

            for f in frame_buffer.drain(..) {
                let _ = encoder.push_frame(&f).await;
            }

            if let Err(e) = encoder.flush().await {
                tracing::error!("Encoder flush error: {}", e);
            }

            *output_path.lock().await = Some(output_filename);

            let mut s = state_mutex.lock().await;
            *s = RecordingState::Stopped;
        });

        *self.event_task.lock().await = Some(handle);

        let mut state = self.state.lock().await;
        *state = RecordingState::Recording;

        Ok(tx)
    }

    /// Stop recording.
    ///
    /// Closes the frame channel and waits for the encoder to flush.
    /// If `test_passed` is `Some`, enforces the retention policy:
    /// - `Always` → keep
    /// - `OnFailure` → delete if test passed
    /// - `Never` → always delete
    pub async fn stop(&self, test_passed: Option<bool>) -> Result<Option<String>, TurbosheetError> {
        let mut state = self.state.lock().await;
        if *state != RecordingState::Recording && *state != RecordingState::Starting {
            return Ok(None);
        }
        *state = RecordingState::Stopping;
        drop(state);

        if let Some(tx) = self.frame_tx.lock().await.take() {
            drop(tx);
        }

        if let Some(handle) = self.event_task.lock().await.take() {
            let _ = handle.await;
        }

        let path = self.output_path.lock().await.clone();

        if let Some(passed) = test_passed {
            self.enforce_retention(passed).await;
        }

        Ok(path)
    }

    /// Cancel recording (e.g., page was closed mid-recording).
    pub async fn cancel(&self) {
        self.cancelled.store(true, Ordering::SeqCst);
        if let Some(tx) = self.frame_tx.lock().await.take() {
            drop(tx);
        }
        if let Some(handle) = self.event_task.lock().await.take() {
            let _ = handle.await;
        }
        self.encoder.cancel().await;
        let mut state = self.state.lock().await;
        *state = RecordingState::Stopped;
    }

    /// Whether the recording was cancelled.
    pub fn is_cancelled(&self) -> bool {
        self.cancelled.load(Ordering::SeqCst)
    }

    /// Delete the recorded video file (if one exists).
    pub async fn delete_output(&self) {
        if let Some(path) = self.output_path.lock().await.take() {
            let _ = std::fs::remove_file(&path);
            tracing::debug!("Deleted video recording: {}", path);
        }
    }

    /// Enforce the retention policy for the recording.
    ///
    /// * `RetentionPolicy::Always` — keep the file.
    /// * `RetentionPolicy::OnFailure` — delete if the test passed.
    /// * `RetentionPolicy::Never` — always delete.
    pub async fn enforce_retention(&self, test_passed: bool) {
        let retain = self.config.retain;
        match retain {
            crate::video::config::RetentionPolicy::Always => {
                // keep
            }
            crate::video::config::RetentionPolicy::OnFailure if test_passed => {
                self.delete_output().await;
            }
            crate::video::config::RetentionPolicy::Never => {
                self.delete_output().await;
            }
            _ => {
                // OnFailure + test_failed → keep
            }
        }
    }

    /// Configure the output path before starting recording.
    ///
    /// Useful when the caller (e.g. test runner) wants to control the
    /// exact path for a specific test. Must be called before `start()`.
    pub async fn set_output_path(&self, path: String) {
        *self.output_path.lock().await = Some(path);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_initial_state_idle() {
        let recorder = VideoRecorder::new(VideoConfig::default());
        assert_eq!(recorder.state().await, RecordingState::Idle);
    }

    #[tokio::test]
    async fn test_start_without_ffmpeg_returns_error() {
        let old_path = std::env::var("PATH").ok();
        std::env::set_var("PATH", "");
        let recorder = VideoRecorder::new(VideoConfig::default());
        let result = recorder.start().await;
        assert!(result.is_err());
        if let Some(path) = old_path {
            std::env::set_var("PATH", path);
        }
    }

    #[tokio::test]
    async fn test_stop_when_idle_is_noop() {
        let recorder = VideoRecorder::new(VideoConfig::default());
        let result = recorder.stop().await;
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), None);
    }
}
