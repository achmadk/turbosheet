use std::process::{Child, Command, Stdio};
use std::sync::Arc;
use tokio::sync::Mutex;

use crate::error::TurbosheetError;
use crate::video::config::VideoConfig;

/// Encoder state.
#[derive(Debug, PartialEq, Eq)]
pub enum EncoderState {
    Idle,
    Running,
    Flushing,
    Finished,
    Failed(String),
}

/// FFmpeg subprocess-based video encoder.
///
/// Spawns an `ffmpeg` process and pipes decoded JPEG frames to its stdin.
pub struct VideoEncoder {
    state: Arc<Mutex<EncoderState>>,
    config: VideoConfig,
    /// The FFmpeg subprocess handle (stdin writer + child process).
    process: Arc<Mutex<Option<EncoderProcess>>>,
}

struct EncoderProcess {
    child: Child,
}

impl VideoEncoder {
    /// Create a new encoder with the given config.
    pub fn new(config: VideoConfig) -> Self {
        Self {
            state: Arc::new(Mutex::new(EncoderState::Idle)),
            config,
            process: Arc::new(Mutex::new(None)),
        }
    }

    /// Returns a reference to the shared state mutex.
    pub fn state(&self) -> &Arc<Mutex<EncoderState>> {
        &self.state
    }

    /// Spawn the FFmpeg subprocess and prepare to receive frames.
    ///
    /// Returns an error if `ffmpeg` is not found in PATH or if spawning fails.
    pub async fn start(&self) -> Result<(), TurbosheetError> {
        let mut state = self.state.lock().await;
        if *state != EncoderState::Idle {
            return Err(TurbosheetError::Other(
                "Encoder already started".to_string(),
            ));
        }

        let output_path = self.config.output_path();
        let frame_rate = self.config.frame_rate;
        let bitrate = &self.config.bitrate;
        let codec = &self.config.codec;

        let ffmpeg_path = which_ffmpeg().ok_or_else(|| {
            TurbosheetError::Other(
                "ffmpeg not found. Install ffmpeg or place it in PATH. \
                 See https://ffmpeg.org/download.html"
                    .to_string(),
            )
        })?;

        if let Some(parent) = std::path::Path::new(&output_path).parent() {
            std::fs::create_dir_all(parent).map_err(|e| {
                TurbosheetError::Other(format!("Failed to create output directory: {}", e))
            })?;
        }

        let mut args: Vec<String> = Vec::new();

        // Input: JPEG frames from pipe
        args.extend_from_slice(&[
            "-f".into(), "image2pipe".into(),
            "-framerate".into(), frame_rate.to_string(),
            "-i".into(), "-".into(),
        ]);

        // Encoder and quality
        args.extend_from_slice(&[
            "-c:v".into(), codec.clone(),
            "-b:v".into(), bitrate.clone(),
        ]);

        // Force YUV 4:2:0 planar pixel format (required by most players for WebM/MP4)
        args.extend_from_slice(&["-pix_fmt".into(), "yuv420p".into()]);

        // If dimensions are configured, add a scale filter so the output
        // resolution is deterministic regardless of the input JPEG size.
        let max_width = self.config.max_width;
        let max_height = self.config.max_height;
        if max_width > 0 && max_height > 0 {
            args.extend_from_slice(&[
                "-vf".into(), format!("scale={}:{}:force_original_aspect_ratio=decrease,pad={}:{}:(ow-iw)/2:(oh-ih)/2:color=black", max_width, max_height, max_width, max_height),
            ]);
        }

        // Codec-specific presets
        if codec == "libvpx" || codec == "libvpx-vp9" {
            // VP8/VP9 realtime preset for low-latency encoding during tests
            args.extend_from_slice(&["-deadline".into(), "realtime".into(), "-cpu-used".into(), "4".into()]);
        } else if codec == "libx264" || codec == "libx265" {
            // H.264/H.265 ultrafast preset for test recording
            args.extend_from_slice(&["-preset".into(), "ultrafast".into(), "-tune".into(), "zerolatency".into()]);
        }

        // Variable frame rate (account for dropped/sporadic screencast frames)
        args.extend_from_slice(&["-vsync".into(), "vfr".into()]);

        // Overwrite output without asking
        args.push("-y".into());

        // Output path
        args.push(output_path);

        let child = Command::new(ffmpeg_path)
            .args(&args)
            .stdin(Stdio::piped())
            .stdout(Stdio::null())
            .stderr(Stdio::inherit())
            .spawn()
            .map_err(|e| {
                TurbosheetError::Other(format!("Failed to spawn ffmpeg: {}", e))
            })?;

        *self.process.lock().await = Some(EncoderProcess { child });
        *state = EncoderState::Running;

        Ok(())
    }

    /// Write a decoded JPEG frame to the FFmpeg subprocess stdin.
    pub async fn push_frame(&self, frame_data: &[u8]) -> Result<(), TurbosheetError> {
        let state = self.state.lock().await;
        if *state != EncoderState::Running {
            return Err(TurbosheetError::Other(
                "Encoder not running".to_string(),
            ));
        }
        drop(state);

        let mut proc_guard = self.process.lock().await;
        if let Some(ref mut proc) = *proc_guard {
            use std::io::Write;
            if let Some(ref mut stdin) = proc.child.stdin {
                stdin.write_all(frame_data).map_err(|e| {
                    TurbosheetError::Other(format!("Failed to write frame to ffmpeg: {}", e))
                })?;
            }
        }
        Ok(())
    }

    /// Flush the encoder: close stdin and wait for the subprocess to exit.
    pub async fn flush(&self) -> Result<(), TurbosheetError> {
        let mut state = self.state.lock().await;
        if *state != EncoderState::Running {
            return Ok(());
        }
        *state = EncoderState::Flushing;
        drop(state);

        let mut proc_guard = self.process.lock().await;
        if let Some(ref mut proc) = proc_guard.take() {
            drop(proc.child.stdin.take());
            let exit_status = proc.child.wait().map_err(|e| {
                TurbosheetError::Other(format!("Failed to wait for ffmpeg: {}", e))
            })?;
            if !exit_status.success() {
                return Err(TurbosheetError::Other(format!(
                    "ffmpeg exited with code: {:?}",
                    exit_status.code()
                )));
            }
        }

        let mut state = self.state.lock().await;
        *state = EncoderState::Finished;
        Ok(())
    }

    /// Cancel the encoding process (e.g., if page is closed mid-recording).
    pub async fn cancel(&self) {
        let mut proc_guard = self.process.lock().await;
        if let Some(ref mut proc) = proc_guard.take() {
            let _ = proc.child.kill();
            let _ = proc.child.wait();
        }
        let mut state = self.state.lock().await;
        *state = EncoderState::Finished;
    }
}

fn which_ffmpeg() -> Option<String> {
    if let Ok(path) = std::env::var("PATH") {
        for dir in std::env::split_paths(&path) {
            let candidate = dir.join("ffmpeg");
            if candidate.is_file() {
                return Some(candidate.to_string_lossy().to_string());
            }
            #[cfg(target_os = "windows")]
            {
                let candidate_exe = dir.join("ffmpeg.exe");
                if candidate_exe.is_file() {
                    return Some(candidate_exe.to_string_lossy().to_string());
                }
            }
        }
    }
    for candidate in &[
        "/usr/bin/ffmpeg",
        "/usr/local/bin/ffmpeg",
        "/opt/homebrew/bin/ffmpeg",
    ] {
        if std::path::Path::new(candidate).is_file() {
            return Some(candidate.to_string());
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::video::config::VideoConfig;

    #[test]
    fn test_encoder_initial_state() {
        let encoder = VideoEncoder::new(VideoConfig::default());
        let state = encoder.state().blocking_lock();
        assert_eq!(*state, EncoderState::Idle);
    }

    #[tokio::test]
    async fn test_start_without_ffmpeg_returns_error() {
        let old_path = std::env::var("PATH").ok();
        std::env::set_var("PATH", "");
        let encoder = VideoEncoder::new(VideoConfig::default());
        let result = encoder.start().await;
        assert!(result.is_err());
        if let Some(path) = old_path {
            std::env::set_var("PATH", path);
        }
    }
}
