use image::{DynamicImage, ImageBuffer, Rgba, RgbaImage};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[cfg(feature = "visual")]
use image::GenericImageView;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiffOptions {
    pub threshold: f64,
    pub semantic_diff: Option<bool>,
    pub ignore_regions: Option<Vec<IgnoreRegion>>,
    pub pixel_threshold: Option<f64>,
}

impl Default for DiffOptions {
    fn default() -> Self {
        Self {
            threshold: 0.1,
            semantic_diff: Some(false),
            ignore_regions: None,
            pixel_threshold: Some(0.1),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IgnoreRegion {
    pub x: u32,
    pub y: u32,
    pub width: u32,
    pub height: u32,
    pub reason: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComparisonResult {
    pub passed: bool,
    pub diff_percentage: f64,
    pub diff_image: Option<DiffImageData>,
    pub width: u32,
    pub height: u32,
    pub baseline_width: u32,
    pub baseline_height: u32,
    pub message: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiffImageData {
    pub width: u32,
    pub height: u32,
    pub data: Vec<u8>,
}

#[derive(Debug, Clone)]
pub struct PixelDiff {
    width: u32,
    height: u32,
    diff_data: Vec<f64>,
}

impl PixelDiff {
    pub fn new(width: u32, height: u32) -> Self {
        Self {
            width,
            height,
            diff_data: vec![0.0; (width * height) as usize],
        }
    }

    pub fn set_pixel(&mut self, x: u32, y: u32, diff: f64) {
        if x < self.width && y < self.height {
            self.diff_data[(y * self.width + x) as usize] = diff;
        }
    }

    pub fn get_diff_percentage(&self, pixel_threshold: f64) -> f64 {
        let threshold_count = self.diff_data
            .iter()
            .filter(|&&d| d > pixel_threshold)
            .count();
        
        (threshold_count as f64 / self.diff_data.len() as f64) * 100.0
    }

    pub fn to_image(&self, threshold: f64) -> RgbaImage {
        let mut img: RgbaImage = ImageBuffer::new(self.width, self.height);
        
        for y in 0..self.height {
            for x in 0..self.width {
                let diff = self.diff_data[(y * self.width + x) as usize];
                let pixel = if diff > threshold {
                    Rgba([255, 0, 0, 255])
                } else if diff > 0.0 {
                    let gray = ((1.0 - diff) * 200.0) as u8;
                    Rgba([gray, gray, gray, 255])
                } else {
                    Rgba([0, 180, 0, 255])
                };
                img.put_pixel(x, y, pixel);
            }
        }
        
        img
    }

    pub fn masked_diff_percentage(&self, mask: &[bool], pixel_threshold: f64) -> f64 {
        let active_count = mask.iter().filter(|&&m| m).count();
        if active_count == 0 {
            return 0.0;
        }
        
        let threshold_count = self.diff_data
            .iter()
            .zip(mask.iter())
            .filter(|(&diff, &active)| active && diff > pixel_threshold)
            .count();
        
        (threshold_count as f64 / active_count as f64) * 100.0
    }
}

#[derive(Debug, Clone)]
pub struct SemanticDiff {
    pub dynamic_regions: Vec<IgnoreRegion>,
    pub changed_regions: Vec<RegionChange>,
}

#[derive(Debug, Clone)]
pub struct RegionChange {
    pub x: u32,
    pub y: u32,
    pub width: u32,
    pub height: u32,
    pub change_type: ChangeType,
    pub severity: f64,
}

#[derive(Debug, Clone, PartialEq)]
pub enum ChangeType {
    Added,
    Removed,
    Modified,
    IgnoredDynamic,
}

impl SemanticDiff {
    pub fn new() -> Self {
        Self {
            dynamic_regions: Vec::new(),
            changed_regions: Vec::new(),
        }
    }

    pub fn add_dynamic_region(&mut self, region: IgnoreRegion) {
        self.dynamic_regions.push(region);
    }

    pub fn add_change(&mut self, change: RegionChange) {
        self.changed_regions.push(change);
    }

    pub fn is_ignored(&self, x: u32, y: u32) -> bool {
        self.dynamic_regions.iter().any(|r| {
            x >= r.x && x < r.x + r.width && y >= r.y && y < r.y + r.height
        })
    }

    pub fn generate_semantic_diff_image(&self, diff: &PixelDiff, threshold: f64) -> RgbaImage {
        let mut img: RgbaImage = ImageBuffer::new(diff.width, diff.height);
        
        for y in 0..diff.height {
            for x in 0..diff.width {
                let pixel_diff = diff.diff_data[(y * diff.width + x) as usize];
                let is_dynamic = self.is_ignored(x, y);
                
                let pixel = if is_dynamic {
                    Rgba([255, 255, 0, 255])
                } else if pixel_diff > threshold {
                    Rgba([255, 0, 0, 255])
                } else if pixel_diff > 0.0 {
                    let gray = ((1.0 - pixel_diff) * 200.0) as u8;
                    Rgba([gray, gray, gray, 255])
                } else {
                    Rgba([0, 180, 0, 255])
                };
                img.put_pixel(x, y, pixel);
            }
        }
        
        img
    }
}

impl Default for SemanticDiff {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(feature = "visual")]
pub mod simd_compare {
    use super::*;
    use image::GenericImageView;

    pub fn compare_simd(
        current: &DynamicImage,
        baseline: &DynamicImage,
        options: &DiffOptions,
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

        let current_pixels = current.to_rgba8();
        let baseline_pixels = baseline.to_rgba8();
        
        let pixel_threshold = options.pixel_threshold.unwrap_or(0.1);
        
        let mut diff = PixelDiff::new(width, height);
        
        for y in 0..height {
            for x in 0..width {
                let curr_pixel = current_pixels.get_pixel(x, y);
                let base_pixel = baseline_pixels.get_pixel(x, y);
                let diff_val = pixel_diff_scalar(curr_pixel, base_pixel);
                diff.set_pixel(x, y, diff_val);
            }
        }

        let mut mask = vec![true; (width * height) as usize];
        
        if let Some(ref regions) = options.ignore_regions {
            for region in regions {
                for ry in region.y..(region.y + region.height).min(height) {
                    for rx in region.x..(region.x + region.width).min(width) {
                        if let Some(idx) = (ry * width + rx).checked_sub(0) {
                            if (idx as usize) < mask.len() {
                                mask[idx as usize] = false;
                            }
                        }
                    }
                }
            }
        }

        let diff_percentage = if options.semantic_diff.unwrap_or(false) {
            diff.masked_diff_percentage(&mask, pixel_threshold)
        } else {
            diff.get_diff_percentage(pixel_threshold)
        };
        
        let passed = diff_percentage <= (options.threshold * 100.0);
        
        let diff_img = if options.semantic_diff.unwrap_or(false) {
            let mut semantic = SemanticDiff::new();
            if let Some(ref regions) = options.ignore_regions {
                for region in regions {
                    semantic.add_dynamic_region(region.clone());
                }
            }
            Some(DiffImageData {
                width,
                height,
                data: semantic.generate_semantic_diff_image(&diff, options.threshold).into_raw(),
            })
        } else {
            Some(DiffImageData {
                width,
                height,
                data: diff.to_image(options.threshold).into_raw(),
            })
        };
        
        ComparisonResult {
            passed,
            diff_percentage,
            diff_image: diff_img,
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

    #[inline]
    fn pixel_diff_scalar(p1: &Rgba<u8>, p2: &Rgba<u8>) -> f64 {
        let r_diff = (p1[0] as f64 - p2[0] as f64).abs() / 255.0;
        let g_diff = (p1[1] as f64 - p2[1] as f64).abs() / 255.0;
        let b_diff = (p1[2] as f64 - p2[2] as f64).abs() / 255.0;
        let a_diff = (p1[3] as f64 - p2[3] as f64).abs() / 255.0;
        (r_diff + g_diff + b_diff + a_diff) / 4.0
    }
}

#[cfg(feature = "visual")]
pub use simd_compare::compare_simd;
