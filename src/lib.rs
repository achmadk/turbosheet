use napi::bindgen_prelude::*;
use napi_derive::napi;

mod browser;
mod context;
mod page;
mod locator;
mod error;
mod network;
mod assertions;
mod binary_manager;
pub mod engine;
pub mod events;
pub mod injection;
pub mod test_runner;
pub mod reporters;
pub mod component;
pub mod plugin;
pub mod migrate;
#[cfg(feature = "traces")]
pub mod trace;
#[cfg(feature = "visual")]
pub mod visual;
#[cfg(all(target_arch = "wasm32", feature = "edge"))]
pub mod edge;
#[cfg(feature = "swarm")]
pub mod swarm;
#[cfg(feature = "video")]
pub mod video;

pub use error::TurbosheetError;
pub use assertions::matchers::{expect, expect_poll};

#[cfg(feature = "traces")]
use trace::{TraceRecorder, serialize_trace, deserialize_trace, trace_to_json_string};

/// TurboSheet native module entry point.
#[napi]
pub fn version() -> String {
    format!("0.1.0 (turbosheet-rust-core)")
}

/// Install a browser binary.
#[napi]
pub async fn install(browser_type: String) -> Result<String> {
    let path = binary_manager::BrowserBinaryManager::install(&browser_type)
        .await
        .map_err(|e| napi::Error::from_reason(e.to_string()))?;
    Ok(path.to_string_lossy().to_string())
}

/// Launch a browser instance.
#[napi]
pub async fn launch(options: Option<browser::LaunchOptions>) -> Result<browser::JsBrowser> {
    let opts = options.unwrap_or_default();
    browser::JsBrowser::launch(opts).await
}

/// Get all device descriptors.
#[napi]
pub fn devices() -> Vec<context::DeviceDescriptor> {
    context::DeviceDescriptor::all()
}

#[cfg(feature = "traces")]
#[napi]
pub fn trace_start_recording() {
    let _recorder = TraceRecorder::new(None);
    // Note: In production, we'd use a proper singleton or context-based approach
    // This is a simplified version for demonstration
}

#[cfg(feature = "traces")]
#[napi]
pub fn trace_clear() {
    futures::executor::block_on(async {
        trace::GLOBAL_RECORDER.clear().await;
    });
}

#[cfg(feature = "traces")]
#[napi]
pub fn trace_stop_and_serialize(test_name: String) -> Result<Buffer> {
    let events = futures::executor::block_on(async {
        trace::GLOBAL_RECORDER.get_events().await
    });
    
    let serialized = serialize_trace(&events, &test_name, None)
        .map_err(|e| napi::Error::from_reason(e.to_string()))?;
    
    Ok(Buffer::from(serialized))
}

#[cfg(feature = "traces")]
#[napi]
pub fn trace_load_and_deserialize(data: Buffer) -> Result<String> {
    let (events, metadata) = deserialize_trace(&data)
        .map_err(|e| napi::Error::from_reason(e.to_string()))?;
    
    // Return events and metadata as a JSON string since napi-rs doesn't handle nested structs well
    let result = serde_json::json!({
        "events": events,
        "metadata": metadata
    });
    
    serde_json::to_string(&result)
        .map_err(|e| napi::Error::from_reason(e.to_string()))
}

#[cfg(feature = "traces")]
#[napi]
pub fn trace_generate_viewer_html() -> String {
    trace::viewer::TraceViewer::generate_html()
}

#[cfg(feature = "traces")]
#[napi]
pub fn trace_write_viewer_to_file(path: String) -> Result<()> {
    trace::viewer::TraceViewer::write_to_file(&path)
        .map_err(|e| napi::Error::from_reason(e.to_string()))
}

#[cfg(feature = "traces")]
#[napi]
pub fn trace_events_to_json() -> Result<String> {
    let events = futures::executor::block_on(async {
        trace::GLOBAL_RECORDER.get_events().await
    });
    
    trace_to_json_string(&events)
        .map_err(|e| napi::Error::from_reason(e.to_string()))
}

#[cfg(not(feature = "traces"))]
#[napi]
pub fn trace_start_recording() {
    panic!("Trace feature not enabled. Build with --features traces");
}

#[cfg(not(feature = "traces"))]
#[napi]
pub fn trace_clear() {
    // no-op when traces feature is disabled
}

#[cfg(not(feature = "traces"))]
#[napi]
pub fn trace_stop_and_serialize(_test_name: String) -> Result<Buffer> {
    panic!("Trace feature not enabled. Build with --features traces");
}

#[cfg(not(feature = "traces"))]
#[napi]
pub fn trace_load_and_deserialize(_data: Buffer) -> Result<napi::JsObject> {
    panic!("Trace feature not enabled. Build with --features traces");
}

#[cfg(not(feature = "traces"))]
#[napi]
pub fn trace_generate_viewer_html() -> String {
    panic!("Trace feature not enabled. Build with --features traces");
}

#[cfg(not(feature = "traces"))]
#[napi]
pub fn trace_write_viewer_to_file(_path: String) -> Result<()> {
    panic!("Trace feature not enabled. Build with --features traces");
}

#[cfg(feature = "visual")]
#[napi]
pub fn visual_compare_screenshots(
    current: Buffer,
    baseline: Buffer,
    threshold: Option<f64>,
) -> Result<String> {
    use image::ImageBuffer;
    use visual::compare::{compare_simd, DiffOptions};
    
    let current_img = image::load_from_memory(&current)
        .map_err(|e| napi::Error::from_reason(format!("Failed to load current image: {}", e)))?;
    let baseline_img = image::load_from_memory(&baseline)
        .map_err(|e| napi::Error::from_reason(format!("Failed to load baseline image: {}", e)))?;
    
    let options = DiffOptions {
        threshold: threshold.unwrap_or(0.1),
        semantic_diff: Some(false),
        ignore_regions: None,
        pixel_threshold: Some(0.1),
    };
    
    let result = compare_simd(&current_img, &baseline_img, &options);
    
    serde_json::to_string(&result)
        .map_err(|e| napi::Error::from_reason(e.to_string()))
}

#[cfg(feature = "visual")]
#[napi]
pub fn visual_save_snapshot(
    image_data: Buffer,
    path: String,
    options: Option<String>,
) -> Result<()> {
    use std::path::Path;
    
    let img = image::load_from_memory(&image_data)
        .map_err(|e| napi::Error::from_reason(format!("Failed to load image: {}", e)))?;
    
    if let Some(parent) = Path::new(&path).parent() {
        std::fs::create_dir_all(parent)
            .map_err(|e| napi::Error::from_reason(format!("Failed to create directory: {}", e)))?;
    }
    
    img.save(&path)
        .map_err(|e| napi::Error::from_reason(format!("Failed to save image: {}", e)))?;
    
    Ok(())
}

#[cfg(feature = "visual")]
#[napi]
pub fn visual_generate_diff_image(
    current: Buffer,
    baseline: Buffer,
    threshold: Option<f64>,
) -> Result<Buffer> {
    use image::ImageBuffer;
    use visual::snapshot::visual_utils::{create_diff_image, pixel_diff};
    
    let current_img = image::load_from_memory(&current)
        .map_err(|e| napi::Error::from_reason(format!("Failed to load current image: {}", e)))?;
    let baseline_img = image::load_from_memory(&baseline)
        .map_err(|e| napi::Error::from_reason(format!("Failed to load baseline image: {}", e)))?;
    
    let thresh = threshold.unwrap_or(0.1) / 255.0;
    
    let (diff_img, _diff_pct) = create_diff_image(&current_img, &baseline_img, thresh);
    
    let mut png_data = Vec::new();
    diff_img.write_to(&mut std::io::Cursor::new(&mut png_data), image::ImageFormat::Png)
        .map_err(|e| napi::Error::from_reason(format!("Failed to encode diff image: {}", e)))?;
    
    Ok(Buffer::from(png_data))
}

#[cfg(not(feature = "visual"))]
#[napi]
pub fn visual_compare_screenshots(
    _current: Buffer,
    _baseline: Buffer,
    _threshold: Option<f64>,
) -> Result<String> {
    panic!("Visual feature not enabled. Build with --features visual");
}

#[cfg(not(feature = "visual"))]
#[napi]
pub fn visual_save_snapshot(
    _image_data: Buffer,
    _path: String,
    _options: Option<String>,
) -> Result<()> {
    panic!("Visual feature not enabled. Build with --features visual");
}

#[cfg(not(feature = "visual"))]
#[napi]
pub fn visual_generate_diff_image(
    _current: Buffer,
    _baseline: Buffer,
    _threshold: Option<f64>,
) -> Result<Buffer> {
    panic!("Visual feature not enabled. Build with --features visual");
}

#[cfg(feature = "video")]
#[napi]
pub fn video_is_available() -> bool {
    true
}

#[cfg(not(feature = "video"))]
#[napi]
pub fn video_is_available() -> bool {
    false
}
