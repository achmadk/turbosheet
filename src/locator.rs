use napi::bindgen_prelude::*;
use napi_derive::napi;

use crate::engine::PAGES;

#[napi(object)]
#[derive(Default, Clone)]
pub struct LocatorActionOptions {
    pub timeout: Option<u32>,
    pub no_wait_after: Option<bool>,
    pub force: Option<bool>,
    pub trial: Option<bool>,
}

#[napi(object)]
#[derive(Default, Clone)]
pub struct JsIgnoreRegion {
    pub x: u32,
    pub y: u32,
    pub width: u32,
    pub height: u32,
    pub reason: Option<String>,
}

#[napi(object)]
#[derive(Default, Clone)]
pub struct JsDiffOptions {
    pub threshold: Option<f64>,
    pub semantic_diff: Option<bool>,
    pub ignore_regions: Option<Vec<JsIgnoreRegion>>,
    pub pixel_threshold: Option<f64>,
}

#[cfg(feature = "visual")]
impl JsDiffOptions {
    pub fn into_diff_options(self) -> crate::visual::compare::DiffOptions {
        crate::visual::compare::DiffOptions {
            threshold: self.threshold.unwrap_or(0.1),
            semantic_diff: self.semantic_diff,
            ignore_regions: self.ignore_regions.map(|rs| {
                rs.into_iter().map(|r| crate::visual::compare::IgnoreRegion {
                    x: r.x,
                    y: r.y,
                    width: r.width,
                    height: r.height,
                    reason: r.reason,
                }).collect()
            }),
            pixel_threshold: self.pixel_threshold,
        }
    }
}

#[napi(object)]
#[derive(Default, Clone)]
pub struct FilterOptions {
    pub has_text: Option<String>,
    pub has_not_text: Option<String>,
}

#[napi(object)]
#[derive(Default, Clone)]
pub struct WaitForOptions {
    pub timeout: Option<u32>,
    pub state: Option<String>,
}

#[napi]
pub struct JsLocator {
    pub selector: String,
    pub page_id: String,
}

#[napi]
impl JsLocator {
    async fn check_strict_mode(&self) -> Result<()> {
        let page = PAGES.get(&self.page_id).and_then(|w| w.upgrade()).ok_or_else(|| {
            napi::Error::from_reason(format!("Page {} not found", self.page_id))
        })?;
        let escaped = serde_json::to_string(&self.selector).unwrap_or_else(|_| format!("\"{}\"", self.selector));
        let global_name = page.engine.global_name();
        let count_js = format!("(window['{}'].querySelectorAll ? window['{}'].querySelectorAll({}).length : document.querySelectorAll({}).length)", global_name, global_name, escaped, escaped);
        let count_res = page.engine.evaluate(&count_js).await.unwrap_or_else(|_| "0".to_string());
        let count: u32 = count_res.parse().unwrap_or(0);
        if count > 1 {
            return Err(napi::Error::from_reason("TurbosheetError::ElementNotUnique"));
        }
        Ok(())
    }

    #[napi]
    pub fn locator(&self, selector: String) -> JsLocator {
        JsLocator {
            selector: format!("{} >> {}", self.selector, selector),
            page_id: self.page_id.clone(),
        }
    }

    /// Create a frame-locator scoped to an iframe matching `frame_selector`.
    /// Example: `page.locator('#parent').frameLocator('iframe').locator('button')`
    #[napi]
    pub fn frame_locator(&self, frame_selector: String) -> JsFrameLocator {
        JsFrameLocator {
            frame_selector,
            selector: String::new(),
            page_id: self.page_id.clone(),
        }
    }

    #[napi]
    pub fn filter(&self, options: FilterOptions) -> JsLocator {
        let mut new_selector = self.selector.clone();
        if let Some(text) = options.has_text {
            new_selector = format!("{} >> text={}", new_selector, text);
        }
        JsLocator {
            selector: new_selector,
            page_id: self.page_id.clone(),
        }
    }

    #[napi]
    pub fn first(&self) -> JsLocator {
        JsLocator {
            selector: format!("{} >> nth=0", self.selector),
            page_id: self.page_id.clone(),
        }
    }

    #[napi]
    pub fn last(&self) -> JsLocator {
        JsLocator {
            selector: format!("{} >> nth=-1", self.selector),
            page_id: self.page_id.clone(),
        }
    }

    #[napi]
    pub fn nth(&self, index: u32) -> JsLocator {
        JsLocator {
            selector: format!("{} >> nth={}", self.selector, index),
            page_id: self.page_id.clone(),
        }
    }

    #[napi]
    pub async fn count(&self, _options: Option<LocatorActionOptions>) -> Result<u32> {
        let page = PAGES.get(&self.page_id).and_then(|w| w.upgrade()).ok_or_else(|| {
            napi::Error::from_reason(format!("Page {} not found", self.page_id))
        })?;
        
        let escaped = serde_json::to_string(&self.selector).unwrap_or_else(|_| format!("\"{}\"", self.selector));
        let g = page.engine.global_name();
        let count_js = format!("(window.{}.querySelectorAll ? window.{}.querySelectorAll({}).length : document.querySelectorAll({}).length)", g, g, escaped, escaped);
        
        let count_res = page.engine.evaluate(&count_js).await.unwrap_or_else(|_| "0".to_string());
        Ok(count_res.parse().unwrap_or(0))
    }

    async fn wait_for_actionability(&self, options: &Option<LocatorActionOptions>) -> Result<()> {
        self.check_strict_mode().await?;
        let force = options.as_ref().and_then(|o| o.force).unwrap_or(false);
        if force {
            return Ok(());
        }
        
        let timeout_ms = options.as_ref().and_then(|o| o.timeout).unwrap_or(30000);
        
        let page = PAGES.get(&self.page_id).and_then(|w| w.upgrade()).ok_or_else(|| {
            napi::Error::from_reason(format!("Page {} not found", self.page_id))
        })?;
        
        page.engine.invoke_action("waitForActionability", vec![
            serde_json::Value::String(self.selector.clone()),
            serde_json::Value::Number(serde_json::Number::from(timeout_ms))
        ]).await.map_err(|e| napi::Error::from_reason(e.to_string()))?;
        
        Ok(())
    }

    #[napi]
    pub async fn click(&self, options: Option<LocatorActionOptions>) -> Result<()> {
        self.wait_for_actionability(&options).await?;
        let page = PAGES.get(&self.page_id).and_then(|w| w.upgrade()).ok_or_else(|| {
            napi::Error::from_reason(format!("Page {} not found", self.page_id))
        })?;
        page.engine.click(&self.selector).await?;
        Ok(())
    }

    #[napi]
    pub async fn dblclick(&self, options: Option<LocatorActionOptions>) -> Result<()> {
        self.wait_for_actionability(&options).await?;
        let page = PAGES.get(&self.page_id).and_then(|w| w.upgrade()).ok_or_else(|| {
            napi::Error::from_reason(format!("Page {} not found", self.page_id))
        })?;
        page.engine.dblclick(&self.selector).await?;
        Ok(())
    }

    #[napi]
    pub async fn right_click(&self, options: Option<LocatorActionOptions>) -> Result<()> {
        self.wait_for_actionability(&options).await?;
        let page = PAGES.get(&self.page_id).and_then(|w| w.upgrade()).ok_or_else(|| {
            napi::Error::from_reason(format!("Page {} not found", self.page_id))
        })?;
        page.engine.right_click(&self.selector).await?;
        Ok(())
    }

    #[napi]
    pub async fn hover(&self, options: Option<LocatorActionOptions>) -> Result<()> {
        self.wait_for_actionability(&options).await?;
        let page = PAGES.get(&self.page_id).and_then(|w| w.upgrade()).ok_or_else(|| {
            napi::Error::from_reason(format!("Page {} not found", self.page_id))
        })?;
        page.engine.hover(&self.selector).await?;
        Ok(())
    }

    #[napi]
    pub async fn fill(&self, value: String, options: Option<LocatorActionOptions>) -> Result<()> {
        self.wait_for_actionability(&options).await?;
        let page = PAGES.get(&self.page_id).and_then(|w| w.upgrade()).ok_or_else(|| {
            napi::Error::from_reason(format!("Page {} not found", self.page_id))
        })?;
        page.engine.fill(&self.selector, &value).await?;
        Ok(())
    }

    #[napi]
    pub async fn check(&self, options: Option<LocatorActionOptions>) -> Result<()> {
        self.wait_for_actionability(&options).await?;
        let page = PAGES.get(&self.page_id).and_then(|w| w.upgrade()).ok_or_else(|| {
            napi::Error::from_reason(format!("Page {} not found", self.page_id))
        })?;
        page.engine.check(&self.selector).await?;
        Ok(())
    }

    #[napi]
    pub async fn uncheck(&self, options: Option<LocatorActionOptions>) -> Result<()> {
        self.wait_for_actionability(&options).await?;
        let page = PAGES.get(&self.page_id).and_then(|w| w.upgrade()).ok_or_else(|| {
            napi::Error::from_reason(format!("Page {} not found", self.page_id))
        })?;
        page.engine.uncheck(&self.selector).await?;
        Ok(())
    }

    #[napi]
    pub async fn select(&self, value: String, options: Option<LocatorActionOptions>) -> Result<()> {
        self.wait_for_actionability(&options).await?;
        let page = PAGES.get(&self.page_id).and_then(|w| w.upgrade()).ok_or_else(|| {
            napi::Error::from_reason(format!("Page {} not found", self.page_id))
        })?;
        page.engine.select(&self.selector, &value).await?;
        Ok(())
    }

    #[napi]
    pub async fn focus(&self, options: Option<LocatorActionOptions>) -> Result<()> {
        self.wait_for_actionability(&options).await?;
        let page = PAGES.get(&self.page_id).and_then(|w| w.upgrade()).ok_or_else(|| {
            napi::Error::from_reason(format!("Page {} not found", self.page_id))
        })?;
        page.engine.focus(&self.selector).await?;
        Ok(())
    }

    #[napi]
    pub async fn blur(&self, options: Option<LocatorActionOptions>) -> Result<()> {
        self.wait_for_actionability(&options).await?;
        let page = PAGES.get(&self.page_id).and_then(|w| w.upgrade()).ok_or_else(|| {
            napi::Error::from_reason(format!("Page {} not found", self.page_id))
        })?;
        page.engine.blur(&self.selector).await?;
        Ok(())
    }

    #[napi]
    pub async fn scroll_into_view(&self, options: Option<LocatorActionOptions>) -> Result<()> {
        self.wait_for_actionability(&options).await?;
        let page = PAGES.get(&self.page_id).and_then(|w| w.upgrade()).ok_or_else(|| {
            napi::Error::from_reason(format!("Page {} not found", self.page_id))
        })?;
        page.engine.scroll_into_view(&self.selector).await?;
        Ok(())
    }

    #[napi]
    pub async fn text_content(&self, _options: Option<LocatorActionOptions>) -> Result<Option<String>> {
        self.check_strict_mode().await?;
        let page = PAGES.get(&self.page_id).and_then(|w| w.upgrade()).ok_or_else(|| {
            napi::Error::from_reason(format!("Page {} not found", self.page_id))
        })?;
        let res = page.engine.text_content(&self.selector).await?;
        Ok(res)
    }

    #[napi]
    pub async fn inner_text(&self, _options: Option<LocatorActionOptions>) -> Result<String> {
        self.check_strict_mode().await?;
        let page = PAGES.get(&self.page_id).and_then(|w| w.upgrade()).ok_or_else(|| {
            napi::Error::from_reason(format!("Page {} not found", self.page_id))
        })?;
        let res = page.engine.inner_text(&self.selector).await?;
        Ok(res)
    }

    #[napi(js_name = "innerHTML")]
    pub async fn inner_html(&self, _options: Option<LocatorActionOptions>) -> Result<String> {
        self.check_strict_mode().await?;
        let page = PAGES.get(&self.page_id).and_then(|w| w.upgrade()).ok_or_else(|| {
            napi::Error::from_reason(format!("Page {} not found", self.page_id))
        })?;
        let res = page.engine.inner_html(&self.selector).await?;
        Ok(res)
    }

    #[napi]
    pub async fn get_attribute(&self, name: String, _options: Option<LocatorActionOptions>) -> Result<Option<String>> {
        self.check_strict_mode().await?;
        let page = PAGES.get(&self.page_id).and_then(|w| w.upgrade()).ok_or_else(|| {
            napi::Error::from_reason(format!("Page {} not found", self.page_id))
        })?;
        let res = page.engine.get_attribute(&self.selector, &name).await?;
        Ok(res)
    }

    #[napi]
    pub async fn is_visible(&self, _options: Option<LocatorActionOptions>) -> Result<bool> {
        self.check_strict_mode().await?;
        let page = PAGES.get(&self.page_id).and_then(|w| w.upgrade()).ok_or_else(|| {
            napi::Error::from_reason(format!("Page {} not found", self.page_id))
        })?;
        let res = page.engine.is_visible(&self.selector).await?;
        Ok(res)
    }

    #[napi]
    pub async fn is_enabled(&self, _options: Option<LocatorActionOptions>) -> Result<bool> {
        self.check_strict_mode().await?;
        let page = PAGES.get(&self.page_id).and_then(|w| w.upgrade()).ok_or_else(|| {
            napi::Error::from_reason(format!("Page {} not found", self.page_id))
        })?;
        let res = page.engine.is_enabled(&self.selector).await?;
        Ok(res)
    }

    #[napi]
    pub async fn is_disabled(&self, _options: Option<LocatorActionOptions>) -> Result<bool> {
        self.check_strict_mode().await?;
        let page = PAGES.get(&self.page_id).and_then(|w| w.upgrade()).ok_or_else(|| {
            napi::Error::from_reason(format!("Page {} not found", self.page_id))
        })?;
        let res = page.engine.is_disabled(&self.selector).await?;
        Ok(res)
    }

    #[napi]
    pub async fn screenshot(&self, options: Option<LocatorActionOptions>) -> Result<Buffer> {
        self.wait_for_actionability(&options).await?;
        let page = PAGES.get(&self.page_id).and_then(|w| w.upgrade()).ok_or_else(|| {
            napi::Error::from_reason(format!("Page {} not found", self.page_id))
        })?;
        
        let _bbox = page.engine.bounding_box(&self.selector).await?;
        let data = page.engine.screenshot(None).await?;
        
        #[cfg(feature = "visual")]
        {
            if let Some(rect) = bbox {
                if let Ok(mut img) = image::load_from_memory(&data) {
                    let cropped = img.crop(
                        rect.x.max(0.0) as u32,
                        rect.y.max(0.0) as u32,
                        rect.width.max(1.0) as u32,
                        rect.height.max(1.0) as u32,
                    );
                    let mut bytes: Vec<u8> = Vec::new();
                    if cropped.write_to(&mut std::io::Cursor::new(&mut bytes), image::ImageFormat::Png).is_ok() {
                        return Ok(Buffer::from(bytes));
                    }
                }
            }
        }
        
        Ok(Buffer::from(data))
    }

    #[napi]
    pub async fn expect_screenshot(&self, _name: String, _options: Option<JsDiffOptions>) -> Result<bool> {
        self.wait_for_actionability(&None).await?;
        let page = PAGES.get(&self.page_id).and_then(|w| w.upgrade()).ok_or_else(|| {
            napi::Error::from_reason(format!("Page {} not found", self.page_id))
        })?;
        
        let _bbox = page.engine.bounding_box(&self.selector).await?;
        let data = page.engine.screenshot(None).await?;
        let _final_data = data.clone();
        
        #[cfg(feature = "visual")]
        {
            if let Some(rect) = bbox {
                if let Ok(mut img) = image::load_from_memory(&data) {
                    let cropped = img.crop(
                        rect.x.max(0.0) as u32,
                        rect.y.max(0.0) as u32,
                        rect.width.max(1.0) as u32,
                        rect.height.max(1.0) as u32,
                    );
                    let mut bytes: Vec<u8> = Vec::new();
                    if cropped.write_to(&mut std::io::Cursor::new(&mut bytes), image::ImageFormat::Png).is_ok() {
                        final_data = bytes;
                    }
                }
            }
            
            let update_baselines = std::env::var("UPDATE_VISUAL_BASELINES").unwrap_or_default() == "1";
            let manager = crate::visual::snapshot::SnapshotManager::new("test.ts");
            
            if let Ok(img) = image::load_from_memory(&final_data) {
                if update_baselines || !manager.baseline_exists(&name) {
                    let _ = manager.save_baseline(&name, &img, &crate::visual::snapshot::ScreenshotOptions::default());
                    return Ok(true);
                }
                
                if let Some(baseline) = manager.load_baseline(&name) {
                    let diff_options = options.map(|o| o.into_diff_options()).unwrap_or_default();
                    let result = crate::visual::compare::compare_simd(&img, &baseline, &diff_options);
                    if !result.passed {
                        return Err(napi::Error::from_reason(result.message));
                    }
                    return Ok(true);
                }
            }
        }
        
        Ok(true)
    }

    #[napi]
    pub async fn bounding_box(&self, _options: Option<LocatorActionOptions>) -> Result<Option<crate::page::Rect>> {
        self.check_strict_mode().await?;
        let page = PAGES.get(&self.page_id).and_then(|w| w.upgrade()).ok_or_else(|| {
            napi::Error::from_reason(format!("Page {} not found", self.page_id))
        })?;
        let res = page.engine.bounding_box(&self.selector).await?;
        Ok(res)
    }

    #[napi]
    pub async fn drag_and_drop(&self, target_selector: String, options: Option<LocatorActionOptions>) -> Result<()> {
        self.wait_for_actionability(&options).await?;
        let page = PAGES.get(&self.page_id).and_then(|w| w.upgrade()).ok_or_else(|| {
            napi::Error::from_reason(format!("Page {} not found", self.page_id))
        })?;
        page.engine.drag_and_drop(&self.selector, &target_selector).await?;
        Ok(())
    }

    #[napi]
    pub async fn press(&self, key: String, options: Option<LocatorActionOptions>) -> Result<()> {
        self.wait_for_actionability(&options).await?;
        let page = PAGES.get(&self.page_id).and_then(|w| w.upgrade()).ok_or_else(|| {
            napi::Error::from_reason(format!("Page {} not found", self.page_id))
        })?;
        page.engine.press(&self.selector, &key).await?;
        Ok(())
    }

    #[napi]
    pub async fn press_sequentially(&self, text: String, options: Option<LocatorActionOptions>) -> Result<()> {
        self.wait_for_actionability(&options).await?;
        let page = PAGES.get(&self.page_id).and_then(|w| w.upgrade()).ok_or_else(|| {
            napi::Error::from_reason(format!("Page {} not found", self.page_id))
        })?;
        page.engine.press_sequentially(&self.selector, &text).await?;
        Ok(())
    }

    #[napi]
    pub async fn set_input_files(&self, files: Vec<String>, options: Option<LocatorActionOptions>) -> Result<()> {
        self.wait_for_actionability(&options).await?;
        let page = PAGES.get(&self.page_id).and_then(|w| w.upgrade()).ok_or_else(|| {
            napi::Error::from_reason(format!("Page {} not found", self.page_id))
        })?;
        page.engine.set_input_files(&self.selector, files).await?;
        Ok(())
    }

    // ── Get-by helpers ─────────────────────────────────────────────

    #[napi]
    pub fn get_by_role(&self, role: String, name: Option<String>) -> JsLocator {
        let selector = if let Some(n) = name {
            format!("{} >> role={}[name=\"{}\"]", self.selector, role, n)
        } else {
            format!("{} >> role={}", self.selector, role)
        };
        JsLocator { selector, page_id: self.page_id.clone() }
    }

    #[napi]
    pub fn get_by_text(&self, text: String, exact: Option<bool>) -> JsLocator {
        let selector = if exact.unwrap_or(false) {
            format!("{} >> text=\"{}\"", self.selector, text)
        } else {
            format!("{} >> text={}", self.selector, text)
        };
        JsLocator { selector, page_id: self.page_id.clone() }
    }

    #[napi]
    pub fn get_by_label(&self, label: String, exact: Option<bool>) -> JsLocator {
        let selector = if exact.unwrap_or(false) {
            format!("{} >> label=\"{}\"", self.selector, label)
        } else {
            format!("{} >> label={}", self.selector, label)
        };
        JsLocator { selector, page_id: self.page_id.clone() }
    }

    #[napi]
    pub fn get_by_placeholder(&self, placeholder: String, exact: Option<bool>) -> JsLocator {
        let selector = if exact.unwrap_or(false) {
            format!("{} >> placeholder=\"{}\"", self.selector, placeholder)
        } else {
            format!("{} >> placeholder={}", self.selector, placeholder)
        };
        JsLocator { selector, page_id: self.page_id.clone() }
    }

    #[napi]
    pub fn get_by_alt_text(&self, alt_text: String, exact: Option<bool>) -> JsLocator {
        let selector = if exact.unwrap_or(false) {
            format!("{} >> alt=\"{}\"", self.selector, alt_text)
        } else {
            format!("{} >> alt={}", self.selector, alt_text)
        };
        JsLocator { selector, page_id: self.page_id.clone() }
    }

    #[napi]
    pub fn get_by_title(&self, title: String, exact: Option<bool>) -> JsLocator {
        let selector = if exact.unwrap_or(false) {
            format!("{} >> title=\"{}\"", self.selector, title)
        } else {
            format!("{} >> title={}", self.selector, title)
        };
        JsLocator { selector, page_id: self.page_id.clone() }
    }

    #[napi]
    pub fn get_by_test_id(&self, test_id: String) -> JsLocator {
        JsLocator {
            selector: format!("{} >> data-testid={}", self.selector, test_id),
            page_id: self.page_id.clone(),
        }
    }

    /// Evaluate arbitrary JavaScript in the page, scoped to this locator's element.
    /// The JS receives the matched element as `el` and should return a string.
    #[napi]
    pub async fn evaluate(&self, js: String) -> Result<String> {
        self.check_strict_mode().await?;
        let page = PAGES.get(&self.page_id).and_then(|w| w.upgrade()).ok_or_else(|| {
            napi::Error::from_reason(format!("Page {} not found", self.page_id))
        })?;
        let escaped_sel = serde_json::to_string(&self.selector).unwrap_or_else(|_| format!("\"{}\"", self.selector));
        let wrapped = format!(
            r#"(function() {{ const el = document.querySelector({sel}); if (!el) return ''; return (function() {{ {js} }})(); }})()"#,
            sel = escaped_sel,
            js = js
        );
        page.engine.evaluate(&wrapped).await.map_err(|e| napi::Error::from_reason(e.to_string()))
    }

    /// Wait for the element to reach a specific state.
    /// state: "attached" (default), "detached", "visible", or "hidden"
    #[napi]
    pub async fn wait_for(&self, options: Option<WaitForOptions>) -> Result<()> {
        let page = PAGES.get(&self.page_id).and_then(|w| w.upgrade()).ok_or_else(|| {
            napi::Error::from_reason(format!("Page {} not found", self.page_id))
        })?;

        let timeout_ms = options.as_ref().and_then(|o| o.timeout).unwrap_or(5000) as u64;
        let state = options.as_ref().and_then(|o| o.state.as_deref()).unwrap_or("attached");

        let escaped_sel = serde_json::to_string(&self.selector).unwrap_or_else(|_| format!("\"{}\"", self.selector));

        let condition_js = match state {
            "detached" => format!("var e = document.querySelector({sel}); e === null || e === undefined", sel = escaped_sel),
            "visible" => format!(r#"(function() {{ var e = document.querySelector({sel}); if (!e) return 'false'; var s = window.getComputedStyle(e); return String(s.display !== 'none' && s.visibility !== 'hidden' && s.opacity !== '0'); }})()"#, sel = escaped_sel),
            "hidden" => format!(r#"(function() {{ var e = document.querySelector({sel}); if (!e) return 'true'; var s = window.getComputedStyle(e); return String(s.display === 'none' || s.visibility === 'hidden' || s.opacity === '0'); }})()"#, sel = escaped_sel),
            _ => format!("var e = document.querySelector({sel}); e !== null && e !== undefined", sel = escaped_sel),
        };

        let timeout_dur = std::time::Duration::from_millis(timeout_ms);
        let engine = crate::assertions::engine::AssertionEngine::default();

        engine.poll_with_timeout(timeout_dur, || async {
            match page.engine.evaluate(&condition_js).await {
                Ok(res) => {
                    if res == "true" || res == "1" {
                        Ok(())
                    } else {
                        Err(format!("Element did not reach state '{}' within {}ms", state, timeout_ms))
                    }
                }
                Err(e) => Err(e.to_string()),
            }
        }).await.map_err(|e| napi::Error::from_reason(e.to_string()))
    }
}

// ── Factory helpers (for compat-mode napi constructors) ──────────

#[napi]
pub fn create_locator(page_id: String, selector: String) -> JsLocator {
    JsLocator { selector, page_id }
}

#[napi]
pub fn create_frame_locator(page_id: String, frame_selector: String, selector: Option<String>) -> JsFrameLocator {
    JsFrameLocator {
        frame_selector,
        selector: selector.unwrap_or_default(),
        page_id,
    }
}

// ── Frame Locator ─────────────────────────────────────────────────

/// Helper: build JS that targets an element inside an iframe.
/// Returns `(js_code, needs_coords)` where `needs_coords` indicates
/// whether the JS returns a bounding-box JSON for action dispatch.
fn frame_locator_js(frame_selector: &str, inner_selector: &str, expr: &str) -> String {
    let fs = serde_json::to_string(frame_selector).unwrap_or_else(|_| format!("\"{}\"", frame_selector));
    let is = serde_json::to_string(inner_selector).unwrap_or_else(|_| format!("\"{}\"", inner_selector));
    format!(
        r#"(function() {{
            var _f = document.querySelector({fs});
            if (!_f) {{ return null; }}
            var _d = _f.contentDocument || (_f.contentWindow && _f.contentWindow.document);
            if (!_d) {{ return null; }}
            var _el = _d.querySelector({is});
            if (!_el) {{ return null; }}
            return _el.{expr};
        }})()"#,
        fs = fs, is = is, expr = expr
    )
}

fn frame_locator_count_js(frame_selector: &str, inner_selector: &str) -> String {
    let fs = serde_json::to_string(frame_selector).unwrap_or_else(|_| format!("\"{}\"", frame_selector));
    let is = serde_json::to_string(inner_selector).unwrap_or_else(|_| format!("\"{}\"", inner_selector));
    format!(
        r#"(function() {{
            var _f = document.querySelector({fs});
            if (!_f) {{ return '0'; }}
            var _d = _f.contentDocument || (_f.contentWindow && _f.contentWindow.document);
            if (!_d) {{ return '0'; }}
            return String(_d.querySelectorAll({is}).length);
        }})()"#,
        fs = fs, is = is
    )
}

fn frame_locator_check_strict_js(frame_selector: &str, inner_selector: &str) -> String {
    let fs = serde_json::to_string(frame_selector).unwrap_or_else(|_| format!("\"{}\"", frame_selector));
    let is = serde_json::to_string(inner_selector).unwrap_or_else(|_| format!("\"{}\"", inner_selector));
    format!(
        r#"(function() {{
            var _f = document.querySelector({fs});
            if (!_f) {{ return '0'; }}
            var _d = _f.contentDocument || (_f.contentWindow && _f.contentWindow.document);
            if (!_d) {{ return '0'; }}
            return String(_d.querySelectorAll({is}).length);
        }})()"#,
        fs = fs, is = is
    )
}

#[napi]
pub struct JsFrameLocator {
    pub frame_selector: String,
    pub selector: String,
    pub page_id: String,
}

#[napi]
impl JsFrameLocator {
    /// Chain a sub-selector within the frame's document
    #[napi]
    pub fn locator(&self, selector: String) -> JsFrameLocator {
        let new_selector = if self.selector.is_empty() {
            selector
        } else {
            format!("{} >> {}", self.selector, selector)
        };
        JsFrameLocator {
            frame_selector: self.frame_selector.clone(),
            selector: new_selector,
            page_id: self.page_id.clone(),
        }
    }

    async fn eval_in_frame(&self, expr: &str) -> Result<String> {
        let page = PAGES.get(&self.page_id).and_then(|w| w.upgrade()).ok_or_else(|| {
            napi::Error::from_reason(format!("Page {} not found", self.page_id))
        })?;
        let js = frame_locator_js(&self.frame_selector, &self.selector, expr);
        let res = page.engine.evaluate(&js).await.map_err(|e| {
            napi::Error::from_reason(format!("Frame locator error: {}", e))
        })?;
        // The JS returns `null` if frame/element not found
        if res == "null" || res.is_empty() {
            return Err(napi::Error::from_reason("Frame or element not found".to_string()));
        }
        Ok(res)
    }

    async fn check_strict_frame(&self) -> Result<()> {
        let page = PAGES.get(&self.page_id).and_then(|w| w.upgrade()).ok_or_else(|| {
            napi::Error::from_reason(format!("Page {} not found", self.page_id))
        })?;
        let js = frame_locator_check_strict_js(&self.frame_selector, &self.selector);
        let count_res = page.engine.evaluate(&js).await.unwrap_or_else(|_| "0".to_string());
        let count: u32 = count_res.parse().unwrap_or(0);
        if count > 1 {
            return Err(napi::Error::from_reason("TurbosheetError::ElementNotUnique"));
        }
        Ok(())
    }

    #[napi]
    pub async fn inner_text(&self, _options: Option<LocatorActionOptions>) -> Result<String> {
        self.check_strict_frame().await?;
        self.eval_in_frame("innerText").await
    }

    #[napi]
    pub async fn text_content(&self, _options: Option<LocatorActionOptions>) -> Result<Option<String>> {
        self.check_strict_frame().await?;
        match self.eval_in_frame("textContent").await {
            Ok(s) => Ok(Some(s)),
            Err(e) => {
                if e.to_string().contains("not found") {
                    Ok(None)
                } else {
                    Err(e)
                }
            }
        }
    }

    #[napi(js_name = "innerHTML")]
    pub async fn inner_html(&self, _options: Option<LocatorActionOptions>) -> Result<String> {
        self.check_strict_frame().await?;
        self.eval_in_frame("innerHTML").await
    }

    #[napi]
    pub async fn is_visible(&self, _options: Option<LocatorActionOptions>) -> Result<bool> {
        self.check_strict_frame().await?;
        let page = PAGES.get(&self.page_id).and_then(|w| w.upgrade()).ok_or_else(|| {
            napi::Error::from_reason(format!("Page {} not found", self.page_id))
        })?;
        let fs = serde_json::to_string(&self.frame_selector).unwrap_or_default();
        let is = serde_json::to_string(&self.selector).unwrap_or_default();
        let js = format!(
            r#"(function() {{
                var _f = document.querySelector({fs});
                if (!_f) {{ return 'false'; }}
                var _d = _f.contentDocument || (_f.contentWindow && _f.contentWindow.document);
                if (!_d) {{ return 'false'; }}
                var _el = _d.querySelector({is});
                if (!_el) {{ return 'false'; }}
                var s = window.getComputedStyle(_el);
                return String(s.display !== 'none' && s.visibility !== 'hidden' && _el.offsetParent !== null);
            }})()"#,
            fs = fs, is = is
        );
        let res = page.engine.evaluate(&js).await.unwrap_or_else(|_| "false".to_string());
        Ok(res == "true")
    }

    #[napi]
    pub async fn is_enabled(&self, _options: Option<LocatorActionOptions>) -> Result<bool> {
        self.check_strict_frame().await?;
        // Elements without a `disabled` property (div, span, etc.) are inherently enabled.
        // `disabled !== true` returns true for both `false` and `undefined`.
        let res = self.eval_in_frame("disabled !== true").await.unwrap_or_else(|_| "true".to_string());
        Ok(res == "true")
    }

    #[napi]
    pub async fn is_disabled(&self, _options: Option<LocatorActionOptions>) -> Result<bool> {
        self.check_strict_frame().await?;
        let res = self.eval_in_frame("disabled === true").await.unwrap_or_else(|_| "false".to_string());
        Ok(res == "true")
    }

    #[napi]
    pub async fn count(&self, _options: Option<LocatorActionOptions>) -> Result<u32> {
        let page = PAGES.get(&self.page_id).and_then(|w| w.upgrade()).ok_or_else(|| {
            napi::Error::from_reason(format!("Page {} not found", self.page_id))
        })?;
        let js = frame_locator_count_js(&self.frame_selector, &self.selector);
        let res = page.engine.evaluate(&js).await.unwrap_or_else(|_| "0".to_string());
        Ok(res.parse().unwrap_or(0))
    }

    #[napi]
    pub async fn get_attribute(&self, name: String, _options: Option<LocatorActionOptions>) -> Result<Option<String>> {
        self.check_strict_frame().await?;
        let inner = self.selector.clone();
        let js = format!(
            r#"(function() {{
                var _f = document.querySelector({fs});
                if (!_f) {{ return null; }}
                var _d = _f.contentDocument || (_f.contentWindow && _f.contentWindow.document);
                if (!_d) {{ return null; }}
                var _el = _d.querySelector({is});
                if (!_el) {{ return null; }}
                var _v = _el.getAttribute({attr});
                return _v === null ? null : _v;
            }})()"#,
            fs = serde_json::to_string(&self.frame_selector).unwrap_or_default(),
            is = serde_json::to_string(&inner).unwrap_or_default(),
            attr = serde_json::to_string(&name).unwrap_or_default(),
        );
        let page = PAGES.get(&self.page_id).and_then(|w| w.upgrade()).ok_or_else(|| {
            napi::Error::from_reason(format!("Page {} not found", self.page_id))
        })?;
        let res = page.engine.evaluate(&js).await.map_err(|e| {
            napi::Error::from_reason(format!("Frame get_attribute error: {}", e))
        })?;
        if res == "null" || res.is_empty() {
            Ok(None)
        } else {
            Ok(Some(res))
        }
    }

    #[napi]
    pub fn first(&self) -> JsFrameLocator {
        JsFrameLocator {
            frame_selector: self.frame_selector.clone(),
            selector: format!("{} >> nth=0", self.selector),
            page_id: self.page_id.clone(),
        }
    }

    #[napi]
    pub fn last(&self) -> JsFrameLocator {
        JsFrameLocator {
            frame_selector: self.frame_selector.clone(),
            selector: format!("{} >> nth=-1", self.selector),
            page_id: self.page_id.clone(),
        }
    }

    #[napi]
    pub fn nth(&self, index: u32) -> JsFrameLocator {
        JsFrameLocator {
            frame_selector: self.frame_selector.clone(),
            selector: format!("{} >> nth={}", self.selector, index),
            page_id: self.page_id.clone(),
        }
    }

    #[napi]
    pub fn filter(&self, options: FilterOptions) -> JsFrameLocator {
        let mut new_selector = self.selector.clone();
        if let Some(text) = options.has_text {
            new_selector = format!("{} >> text={}", new_selector, text);
        }
        JsFrameLocator {
            frame_selector: self.frame_selector.clone(),
            selector: new_selector,
            page_id: self.page_id.clone(),
        }
    }
}
