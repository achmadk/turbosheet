use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use image::{DynamicImage, ImageBuffer, Rgba, RgbaImage};
use std::collections::HashMap;

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ScreenshotOptions {
    pub full_page: Option<bool>,
    pub element: Option<String>,
    pub viewport: Option<Viewport>,
    pub r#type: Option<String>,
    pub quality: Option<u8>,
    pub animations: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Viewport {
    pub width: u32,
    pub height: u32,
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ScreenshotType {
    Png,
    Jpeg,
    Webp,
}

impl Default for ScreenshotType {
    fn default() -> Self {
        ScreenshotType::Png
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SnapshotResult {
    pub path: PathBuf,
    pub width: u32,
    pub height: u32,
    pub size_bytes: usize,
    pub was_new: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BaselineMetadata {
    pub width: u32,
    pub height: u32,
    pub screenshot_type: String,
    pub created_at: String,
    pub test_file: String,
    pub threshold: f64,
}

impl BaselineMetadata {
    pub fn from_path(path: &PathBuf) -> Option<Self> {
        let meta_path = path.with_extension("meta.json");
        if meta_path.exists() {
            let content = std::fs::read_to_string(&meta_path).ok()?;
            serde_json::from_str(&content).ok()
        } else {
            None
        }
    }

    pub fn save(&self, path: &PathBuf) -> std::io::Result<()> {
        let meta_path = path.with_extension("meta.json");
        let content = serde_json::to_string_pretty(self).map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))?;
        std::fs::write(meta_path, content)
    }
}

pub struct SnapshotManager {
    base_dir: PathBuf,
    test_file: String,
}

impl SnapshotManager {
    pub fn new(test_file: &str) -> Self {
        let test_path = PathBuf::from(test_file);
        let base_dir = test_path
            .parent()
            .map(|p| p.join(".tsheet-snap"))
            .unwrap_or_else(|| PathBuf::from(".tsheet-snap"));
        
        Self {
            base_dir,
            test_file: test_file.to_string(),
        }
    }

    pub fn get_baseline_path(&self, name: &str) -> PathBuf {
        let stem = PathBuf::from(name)
            .file_stem()
            .map(|s| s.to_string_lossy().to_string())
            .unwrap_or_else(|| name.to_string());
        
        let snap_dir = self.base_dir.join(&stem);
        if !snap_dir.exists() {
            let _ = std::fs::create_dir_all(&snap_dir);
        }
        snap_dir.join("baseline.png")
    }

    pub fn get_diff_path(&self, name: &str) -> PathBuf {
        let stem = PathBuf::from(name)
            .file_stem()
            .map(|s| s.to_string_lossy().to_string())
            .unwrap_or_else(|| name.to_string());
        self.base_dir.join(&stem).join("diff.png")
    }

    pub fn baseline_exists(&self, name: &str) -> bool {
        self.get_baseline_path(name).exists()
    }

    pub fn save_baseline(&self, name: &str, image: &DynamicImage, options: &ScreenshotOptions) -> std::io::Result<SnapshotResult> {
        let baseline_path = self.get_baseline_path(name);
        
        if let Some(parent) = baseline_path.parent() {
            std::fs::create_dir_all(parent)?;
        }

        let width = image.width();
        let height = image.height();
        let screenshot_type = options.r#type.as_deref().unwrap_or("png");

        let ext = match screenshot_type {
            "jpeg" | "jpg" => "jpg",
            "webp" => "webp",
            _ => "png",
        };

        let final_path = baseline_path.with_extension(ext);
        
        match ext {
            "jpg" | "jpeg" => {
                image.save_with_format(&final_path, image::ImageFormat::Jpeg)
                    .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))?;
            },
            "webp" => {
                image.save_with_format(&final_path, image::ImageFormat::WebP)
                    .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))?;
            },
            _ => {
                image.save_with_format(&final_path, image::ImageFormat::Png)
                    .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))?;
            }
        }

        let metadata = BaselineMetadata {
            width,
            height,
            screenshot_type: screenshot_type.to_string(),
            created_at: chrono_now(),
            test_file: self.test_file.clone(),
            threshold: 0.0,
        };
        metadata.save(&final_path)?;

        let size_bytes = std::fs::metadata(&final_path)?.len() as usize;

        Ok(SnapshotResult {
            path: final_path,
            width,
            height,
            size_bytes,
            was_new: true,
        })
    }

    pub fn load_baseline(&self, name: &str) -> Option<DynamicImage> {
        let path = self.get_baseline_path(name);
        if path.exists() {
            image::open(&path).ok()
        } else {
            None
        }
    }

    pub fn update_threshold(&self, name: &str, threshold: f64) -> std::io::Result<()> {
        let baseline_path = self.get_baseline_path(name);
        if let Some(mut metadata) = BaselineMetadata::from_path(&baseline_path) {
            metadata.threshold = threshold;
            metadata.save(&baseline_path)?;
        }
        Ok(())
    }
}

fn chrono_now() -> String {
    use std::time::{SystemTime, UNIX_EPOCH};
    let duration = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default();
    let secs = duration.as_secs();
    let millis = duration.subsec_millis();
    format!("{}.{:03}Z", secs, millis)
}

#[cfg(feature = "visual")]
pub mod visual_utils {
    use super::*;
    use image::GenericImageView;

    pub fn create_diff_image(
        current: &DynamicImage,
        baseline: &DynamicImage,
        threshold: f64,
    ) -> (RgbaImage, f64) {
        let (width, height) = current.dimensions();
        let baseline_resized = if baseline.dimensions() != (width, height) {
            baseline.resize_exact(width, height, image::imageops::FilterType::Lanczos3)
        } else {
            baseline.clone()
        };

        let mut diff_img: RgbaImage = ImageBuffer::new(width, height);
        let mut diff_count = 0u64;
        let total_pixels = (width * height) as f64;

        for (x, y, pixel) in current.to_rgba8().enumerate_pixels() {
            let baseline_pixel = baseline_resized.get_pixel(x, y);
            let diff = pixel_diff(&pixel, &baseline_pixel);
            
            if diff > threshold {
                diff_img.put_pixel(x, y, Rgba([255, 0, 0, 255]));
                diff_count += 1;
            } else if diff > 0.0 {
                let gray = ((1.0 - diff) * 128.0) as u8;
                diff_img.put_pixel(x, y, Rgba([gray, gray, gray, 255]));
            } else {
                diff_img.put_pixel(x, y, Rgba([0, 255, 0, 255]));
            }
        }

        let diff_percentage = (diff_count as f64 / total_pixels) * 100.0;
        (diff_img, diff_percentage)
    }

    pub fn pixel_diff(p1: &Rgba<u8>, p2: &Rgba<u8>) -> f64 {
        let r_diff = (p1[0] as f64 - p2[0] as f64).abs() / 255.0;
        let g_diff = (p1[1] as f64 - p2[1] as f64).abs() / 255.0;
        let b_diff = (p1[2] as f64 - p2[2] as f64).abs() / 255.0;
        let a_diff = (p1[3] as f64 - p2[3] as f64).abs() / 255.0;
        
        (r_diff + g_diff + b_diff + a_diff) / 4.0
    }

    pub fn compare_images(
        current: &DynamicImage,
        baseline: &DynamicImage,
        threshold: f64,
    ) -> ComparisonResult {
        let (width, height) = current.dimensions();
        let baseline_dims = baseline.dimensions();
        
        if width != baseline_dims.0 || height != baseline_dims.1 {
            return ComparisonResult {
                passed: false,
                diff_percentage: 100.0,
                diff_image: None,
                width,
                height,
                baseline_width: baseline_dims.0,
                baseline_height: baseline_dims.1,
                message: format!("Size mismatch: {}x{} vs {}x{}", width, height, baseline_dims.0, baseline_dims.1),
            };
        }

        let (diff_img, diff_percentage) = create_diff_image(current, baseline, threshold);
        
        let passed = diff_percentage <= (threshold * 100.0);
        
        ComparisonResult {
            passed,
            diff_percentage,
            diff_image: Some(crate::visual::compare::DiffImageData {
                width: diff_img.width(),
                height: diff_img.height(),
                data: diff_img.into_raw(),
            }),
            width,
            height,
            baseline_width: baseline_dims.0,
            baseline_height: baseline_dims.1,
            message: if passed {
                format!("Screenshots match ({}% different)", diff_percentage)
            } else {
                format!("Screenshots differ by {}%", diff_percentage)
            },
        }
    }
}

use crate::visual::compare::ComparisonResult;
