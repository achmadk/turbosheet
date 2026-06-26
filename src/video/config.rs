/// Video recording configuration.
///
/// Controls frame rate, quality, dimensions, output path,
/// and retention policy for screencast-based video recording.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct VideoConfig {
    /// JPEG quality for screencast frames (1-100, default 80).
    pub quality: u8,
    /// Target frame rate (default 10).
    pub frame_rate: u8,
    /// Maximum frame width in pixels (default 800).
    pub max_width: u32,
    /// Maximum frame height in pixels (default 600).
    pub max_height: u32,
    /// Video encoding bitrate (e.g. "500k", default "500k").
    pub bitrate: String,
    /// Output codec (default "libvpx" for WebM/VP8).
    pub codec: String,
    /// Directory where video files are written.
    pub output_dir: String,
    /// File name for the video (default "recording.webm").
    pub output_filename: String,
    /// Retention policy.
    pub retain: RetentionPolicy,
    /// Maximum number of pending frames before dropping oldest (default 300).
    pub max_pending_frames: usize,
    /// Maximum recording duration in seconds before the watchdog cancels (default 30).
    pub max_recording_duration_secs: u64,
}

/// When to retain recorded video files.
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum RetentionPolicy {
    /// Keep video only when the test fails.
    OnFailure,
    /// Always keep the video file.
    Always,
    /// Delete video immediately after encoding.
    Never,
}

impl Default for VideoConfig {
    fn default() -> Self {
        Self {
            quality: 80,
            frame_rate: 10,
            max_width: 800,
            max_height: 600,
            bitrate: "500k".to_string(),
            codec: "libvpx".to_string(),
            output_dir: "test-results/videos".to_string(),
            output_filename: "recording.webm".to_string(),
            retain: RetentionPolicy::OnFailure,
            max_pending_frames: 300,
            max_recording_duration_secs: 30,
        }
    }
}

impl VideoConfig {
    /// Returns the full output file path.
    pub fn output_path(&self) -> String {
        format!("{}/{}", self.output_dir, self.output_filename)
    }

    /// Generate a unique output path for a given test run.
    ///
    /// Creates a timestamped filename like `<test-name>-<timestamp>.webm`
    /// so concurrent recordings don't collide. The original `output_filename`
    /// extension is preserved.
    pub fn output_path_for_test(&self, test_name: &str) -> String {
        let stem = std::path::Path::new(&self.output_filename)
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("recording");
        let ext = std::path::Path::new(&self.output_filename)
            .extension()
            .and_then(|s| s.to_str())
            .unwrap_or("webm");
        let epoch_nanos = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_nanos();
        let timestamp = format!("{:x}", epoch_nanos); // hex timestamp = sortable + compact
        // Sanitize test name to a safe filename
        let safe_name: String = test_name
            .chars()
            .map(|c| if c.is_alphanumeric() || c == '-' || c == '_' { c } else { '_' })
            .collect();
        format!("{}/{}-{}-{}.{}", self.output_dir, safe_name, stem, timestamp, ext)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let cfg = VideoConfig::default();
        assert_eq!(cfg.quality, 80);
        assert_eq!(cfg.frame_rate, 10);
        assert_eq!(cfg.max_width, 800);
        assert_eq!(cfg.max_height, 600);
        assert_eq!(cfg.bitrate, "500k");
        assert_eq!(cfg.codec, "libvpx");
        assert_eq!(cfg.retain, RetentionPolicy::OnFailure);
    }

    #[test]
    fn test_output_path() {
        let cfg = VideoConfig::default();
        assert_eq!(cfg.output_path(), "test-results/videos/recording.webm");
    }

    #[test]
    fn test_custom_config() {
        let cfg = VideoConfig {
            quality: 60,
            frame_rate: 15,
            max_width: 1280,
            max_height: 720,
            output_dir: "/tmp/videos".to_string(),
            output_filename: "test.webm".to_string(),
            ..Default::default()
        };
        assert_eq!(cfg.output_path(), "/tmp/videos/test.webm");
        assert_eq!(cfg.frame_rate, 15);
        assert_eq!(cfg.max_width, 1280);
    }
}
