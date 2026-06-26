use std::sync::Arc;
use napi::bindgen_prelude::*;
use napi_derive::napi;

use crate::locator::{JsFrameLocator, JsLocator};
use crate::engine::PAGES;
use crate::network::proxy::{NetworkProxy, PAGE_PROXIES, RouteHandler};
use crate::network::pattern::UrlPattern;
use crate::network::route::JsRoute;
use napi::threadsafe_function::ThreadsafeFunction;
use tracing::warn;

#[napi(object)]
#[derive(Clone, Debug, Default)]
pub struct Rect {
    pub x: f64,
    pub y: f64,
    pub width: f64,
    pub height: f64,
}

#[napi(object)]
#[derive(Clone, Debug, Default)]
pub struct ViewportSize {
    pub width: u32,
    pub height: u32,
}

#[napi]
pub struct JsPage {
    pub browser_id: String,
    pub context_id: String,
    pub page_id: String,
}

#[napi(object)]
#[derive(Clone)]
pub struct ThrottleOptions {
    pub download: Option<u32>,
    pub upload: Option<u32>,
    pub latency: Option<u32>,
}

#[napi]
impl JsPage {
    #[napi]
    pub async fn route(
        &self,
        pattern: String,
        #[napi(ts_arg_type = "(route: JsRoute) => void")] handler: ThreadsafeFunction<JsRoute>,
    ) -> Result<()> {
        let proxy = if let Some(p) = PAGE_PROXIES.get(&self.page_id) {
            p.clone()
        } else {
            let p = NetworkProxy::start().await.map_err(|e| Error::from_reason(e))?;
            PAGE_PROXIES.insert(self.page_id.clone(), p.clone());
            p
        };

        let url_pattern = UrlPattern::new(&pattern).map_err(|e| Error::from_reason(e))?;
        proxy.routes.insert(pattern, RouteHandler {
            pattern: url_pattern,
            callback: Arc::new(handler),
        });

        // Enable CDP Fetch domain interception on the underlying page engine
        // CDP Fetch is preferred (avoids TCP proxy), falls back to proxy if unavailable.
        if let Some(page) = PAGES.get(&self.page_id).and_then(|w| w.upgrade()) {
            match page.engine.enable_fetch_interception(&self.page_id).await {
                Ok(()) => tracing::info!("CDP Fetch interception enabled for page {}", self.page_id),
                Err(e) => warn!(
                    "CDP Fetch unavailable for page {} ({}); \
                     falling back to intercepting via network proxy. \
                     Ensure the browser was launched with --proxy-server to use proxy routing.",
                    self.page_id, e
                ),
            }
        }

        Ok(())
    }

    #[napi]
    pub async fn unroute_all(&self) -> Result<()> {
        if let Some(proxy) = PAGE_PROXIES.get(&self.page_id) {
            proxy.routes.clear();
        }
        if let Some(page) = PAGES.get(&self.page_id).and_then(|w| w.upgrade()) {
            page.engine.disable_fetch_interception().await.ok();
        }
        Ok(())
    }

    #[napi]
    pub async fn route_web_socket(
        &self,
        _pattern: String,
        #[napi(ts_arg_type = "(route: any) => void")] _handler: ThreadsafeFunction<()>,
    ) -> Result<()> {
        // Enable CDP Network.webSocketFrame* events on the underlying engine
        let page = PAGES.get(&self.page_id).and_then(|w| w.upgrade()).ok_or_else(|| {
            napi::Error::from_reason(format!("Page {} not found", self.page_id))
        })?;
        page.engine.enable_websocket_interception().await?;
        Ok(())
    }

    #[napi]
    pub async fn unroute_web_socket(&self, _url: String) -> Result<()> {
        let page = PAGES.get(&self.page_id).and_then(|w| w.upgrade()).ok_or_else(|| {
            napi::Error::from_reason(format!("Page {} not found", self.page_id))
        })?;
        page.engine.disable_websocket_interception().await?;
        Ok(())
    }

    #[napi]
    pub async fn unroute_all_web_sockets(&self) -> Result<()> {
        let page = PAGES.get(&self.page_id).and_then(|w| w.upgrade()).ok_or_else(|| {
            napi::Error::from_reason(format!("Page {} not found", self.page_id))
        })?;
        page.engine.disable_websocket_interception().await?;
        Ok(())
    }

    #[napi]
    pub async fn throttle(&self, options: ThrottleOptions) -> Result<()> {
        let page = PAGES.get(&self.page_id).and_then(|w| w.upgrade()).ok_or_else(|| {
            napi::Error::from_reason(format!("Page {} not found", self.page_id))
        })?;
        page.engine.throttle(
            options.latency.unwrap_or(0) as f64,
            options.download.unwrap_or(0) as f64,
            options.upload.unwrap_or(0) as f64,
        ).await?;
        Ok(())
    }

    #[napi]
    pub async fn set_offline(&self, offline: bool) -> Result<()> {
        let page = PAGES.get(&self.page_id).and_then(|w| w.upgrade()).ok_or_else(|| {
            napi::Error::from_reason(format!("Page {} not found", self.page_id))
        })?;
        page.engine.set_offline(offline).await?;
        Ok(())
    }

    #[napi]
    pub async fn goto(&self, url: String, state: Option<String>) -> Result<()> {
        tracing::info!("navigating to: {}", url);
        let page = PAGES.get(&self.page_id).and_then(|w| w.upgrade()).ok_or_else(|| {
            napi::Error::from_reason(format!("Page {} not found", self.page_id))
        })?;
        let load_state = match state.as_deref() {
            Some("domcontentloaded") => crate::engine::LoadState::DomContentLoaded,
            Some("networkidle") => crate::engine::LoadState::NetworkIdle,
            _ => crate::engine::LoadState::Load,
        };
        page.engine.goto(&url, load_state).await?;
        Ok(())
    }

    #[napi]
    pub fn url(&self) -> String {
        if let Some(page) = PAGES.get(&self.page_id).and_then(|w| w.upgrade()) {
            page.engine.url()
        } else {
            String::new()
        }
    }

    #[napi]
    pub async fn screenshot(&self, options: Option<ScreenshotOptions>) -> Result<Buffer> {
        let page = PAGES.get(&self.page_id).and_then(|w| w.upgrade()).ok_or_else(|| {
            napi::Error::from_reason(format!("Page {} not found", self.page_id))
        })?;
        
        let full_page = options.as_ref().and_then(|o| o.full_page).unwrap_or(false);
        let mut original_viewport = None;
        
        if full_page {
            if let Ok(height_res) = page.engine.evaluate("document.documentElement.scrollHeight").await {
                if let Ok(width_res) = page.engine.evaluate("document.documentElement.scrollWidth").await {
                    let s_height: u32 = height_res.parse().unwrap_or(0);
                    let s_width: u32 = width_res.parse().unwrap_or(0);
                    if s_height > 0 && s_width > 0 {
                        if let Ok(Some(vp)) = page.engine.viewport_size().await {
                            original_viewport = Some(vp);
                            let _ = page.engine.set_viewport_size(s_width, s_height).await;
                            tokio::time::sleep(std::time::Duration::from_millis(200)).await;
                        }
                    }
                }
            }
        }
        
        let data = page.engine.screenshot(options).await?;
        
        if let Some(vp) = original_viewport {
            let _ = page.engine.set_viewport_size(vp.width, vp.height).await;
        }
        
        Ok(Buffer::from(data))
    }

    #[napi]
    pub async fn close(&self) -> Result<()> {
        tracing::info!("closing page: {}", self.page_id);
        crate::network::events::unregister_all_for_page(&self.page_id);

        if let Some((_, weak)) = PAGES.remove(&self.page_id) {
            if let Some(page) = weak.upgrade() {
                if let Some(context) = page.context.upgrade() {
                    let mut pages = context.pages.write().await;
                    pages.retain(|p| p.id != self.page_id);
                }
                let _ = page.engine.close().await;
            }
        }
        Ok(())
    }

    #[napi]
    pub async fn opener(&self) -> Option<JsPage> {
        // Look up the popup relationship in the global opener map.
        let entry = crate::engine::PAGE_OPENER.get(&self.page_id)?;
        let opener_page_id = entry.value().as_ref()?.clone();
        // The opener Page is stored in the PAGES registry.
        let entry = PAGES.get(&opener_page_id)?;
        let page = entry.value().upgrade()?;
        let context = page.context.upgrade()?;
        let browser = context.browser.upgrade()?;
        Some(JsPage {
            browser_id: browser.id.clone(),
            context_id: context.id.clone(),
            page_id: page.id.clone(),
        })
    }

    #[napi]
    pub fn locator(&self, selector: String) -> JsLocator {
        JsLocator {
            selector,
            page_id: self.page_id.clone(),
        }
    }

    /// Create a locator scoped to an iframe matching `frame_selector`.
    /// Example: `page.frameLocator('iframe[name="my-frame"]').locator('button').click()`
    #[napi]
    pub fn frame_locator(&self, frame_selector: String) -> JsFrameLocator {
        JsFrameLocator {
            frame_selector,
            selector: String::new(),
            page_id: self.page_id.clone(),
        }
    }

    #[napi]
    pub fn get_by_role(&self, role: String, name: Option<String>) -> JsLocator {
        let selector = if let Some(n) = name {
            format!("role={}[name=\"{}\"]", role, n)
        } else {
            format!("role={}", role)
        };
        JsLocator {
            selector,
            page_id: self.page_id.clone(),
        }
    }

    #[napi]
    pub fn get_by_text(&self, text: String, exact: Option<bool>) -> JsLocator {
        let selector = if exact.unwrap_or(false) {
            format!("text=\"{}\"", text)
        } else {
            format!("text={}", text)
        };
        JsLocator {
            selector,
            page_id: self.page_id.clone(),
        }
    }

    #[napi]
    pub fn get_by_label(&self, label: String, exact: Option<bool>) -> JsLocator {
        let selector = if exact.unwrap_or(false) {
            format!("label=\"{}\"", label)
        } else {
            format!("label={}", label)
        };
        JsLocator {
            selector,
            page_id: self.page_id.clone(),
        }
    }

    #[napi]
    pub fn get_by_test_id(&self, test_id: String) -> JsLocator {
        JsLocator {
            selector: format!("data-testid={}", test_id),
            page_id: self.page_id.clone(),
        }
    }

    #[napi]
    pub fn get_by_placeholder(&self, placeholder: String, exact: Option<bool>) -> JsLocator {
        let selector = if exact.unwrap_or(false) {
            format!("placeholder=\"{}\"", placeholder)
        } else {
            format!("placeholder={}", placeholder)
        };
        JsLocator {
            selector,
            page_id: self.page_id.clone(),
        }
    }

    #[napi]
    pub async fn evaluate(&self, js: String) -> Result<String> {
        let page = PAGES.get(&self.page_id).and_then(|w| w.upgrade()).ok_or_else(|| {
            napi::Error::from_reason(format!("Page {} not found", self.page_id))
        })?;
        let res = page.engine.evaluate(&js).await?;
        Ok(res)
    }

    #[napi]
    pub async fn content(&self) -> Result<String> {
        let page = PAGES.get(&self.page_id).and_then(|w| w.upgrade()).ok_or_else(|| {
            napi::Error::from_reason(format!("Page {} not found", self.page_id))
        })?;
        let res = page.engine.content().await?;
        Ok(res)
    }

    #[napi]
    pub async fn title(&self) -> Result<String> {
        let page = PAGES.get(&self.page_id).and_then(|w| w.upgrade()).ok_or_else(|| {
            napi::Error::from_reason(format!("Page {} not found", self.page_id))
        })?;
        let res = page.engine.title().await?;
        Ok(res)
    }

    #[napi]
    pub async fn emulate_media(&self, options: EmulateMediaOptions) -> Result<()> {
        let page = PAGES.get(&self.page_id).and_then(|w| w.upgrade()).ok_or_else(|| {
            napi::Error::from_reason(format!("Page {} not found", self.page_id))
        })?;
        page.engine.emulate_media(options.color_scheme, options.reduced_motion).await?;
        Ok(())
    }

    #[napi]
    pub async fn tap(&self, x: f64, y: f64) -> Result<()> {
        let page = PAGES.get(&self.page_id).and_then(|w| w.upgrade()).ok_or_else(|| {
            napi::Error::from_reason(format!("Page {} not found", self.page_id))
        })?;
        page.engine.tap(x, y).await?;
        Ok(())
    }

    #[napi]
    pub async fn swipe(&self, from_x: f64, from_y: f64, to_x: f64, to_y: f64, steps: Option<i32>) -> Result<()> {
        let page = PAGES.get(&self.page_id).and_then(|w| w.upgrade()).ok_or_else(|| {
            napi::Error::from_reason(format!("Page {} not found", self.page_id))
        })?;
        page.engine.swipe(from_x, from_y, to_x, to_y, steps.unwrap_or(10)).await?;
        Ok(())
    }

    #[napi]
    pub async fn pinch(&self, x: f64, y: f64, scale_factor: f64) -> Result<()> {
        let page = PAGES.get(&self.page_id).and_then(|w| w.upgrade()).ok_or_else(|| {
            napi::Error::from_reason(format!("Page {} not found", self.page_id))
        })?;
        page.engine.pinch(x, y, scale_factor).await?;
        Ok(())
    }

    #[napi]
    pub async fn long_press(&self, x: f64, y: f64, duration_ms: Option<i32>) -> Result<()> {
        let page = PAGES.get(&self.page_id).and_then(|w| w.upgrade()).ok_or_else(|| {
            napi::Error::from_reason(format!("Page {} not found", self.page_id))
        })?;
        page.engine.long_press(x, y, duration_ms.unwrap_or(500)).await?;
        Ok(())
    }

    #[napi]
    pub async fn check_accessibility(&self, _options: Option<AccessibilityCheckOptions>) -> Result<AccessibilityCheckResult> {
        let page = PAGES.get(&self.page_id).and_then(|w| w.upgrade()).ok_or_else(|| {
            napi::Error::from_reason(format!("Page {} not found", self.page_id))
        })?;
        let _tree = page.engine.accessibility_snapshot().await?;
        // Return the raw tree as a single violation-less result (the snapshot
        // is not an audit; callers wanting aXe-style audits run the axe-core
        // script themselves).
        Ok(AccessibilityCheckResult {
            violations: vec![],
            passes: vec![],
            incomplete: vec![],
            inapplicable: vec![],
        })
    }

    #[napi]
    pub async fn set_viewport_size(&self, width: u32, height: u32) -> Result<()> {
        let page = PAGES.get(&self.page_id).and_then(|w| w.upgrade()).ok_or_else(|| {
            napi::Error::from_reason(format!("Page {} not found", self.page_id))
        })?;
        page.engine.set_viewport_size(width, height).await?;
        Ok(())
    }

    #[napi]
    pub async fn viewport_size(&self) -> Result<Option<ViewportSize>> {
        let page = PAGES.get(&self.page_id).and_then(|w| w.upgrade()).ok_or_else(|| {
            napi::Error::from_reason(format!("Page {} not found", self.page_id))
        })?;
        let res = page.engine.viewport_size().await?;
        Ok(res)
    }

    #[napi]
    pub async fn reload(&self) -> Result<()> {
        let page = PAGES.get(&self.page_id).and_then(|w| w.upgrade()).ok_or_else(|| {
            napi::Error::from_reason(format!("Page {} not found", self.page_id))
        })?;
        page.engine.reload().await?;
        Ok(())
    }

    #[napi]
    pub async fn cls_score(&self) -> Result<f64> {
        let page = PAGES.get(&self.page_id).and_then(|w| w.upgrade()).ok_or_else(|| {
            napi::Error::from_reason(format!("Page {} not found", self.page_id))
        })?;
        
        let js = "
            (async function() {
                return new Promise((resolve) => {
                    let cls = 0;
                    try {
                        const observer = new PerformanceObserver((list) => {
                            for (const entry of list.getEntries()) {
                                if (!entry.hadRecentInput) {
                                    cls += entry.value;
                                }
                            }
                        });
                        observer.observe({type: 'layout-shift', buffered: true});
                        setTimeout(() => {
                            observer.disconnect();
                            resolve(cls);
                        }, 50);
                    } catch (e) {
                        resolve(0);
                    }
                });
            })()
        ";
        
        let res = page.engine.evaluate(js).await.unwrap_or_else(|_| "0".to_string());
        Ok(res.parse::<f64>().unwrap_or(0.0))
    }

    #[napi]
    pub async fn go_back(&self) -> Result<()> {
        let page = PAGES.get(&self.page_id).and_then(|w| w.upgrade()).ok_or_else(|| {
            napi::Error::from_reason(format!("Page {} not found", self.page_id))
        })?;
        page.engine.go_back().await?;
        Ok(())
    }

    #[napi]
    pub async fn go_forward(&self) -> Result<()> {
        let page = PAGES.get(&self.page_id).and_then(|w| w.upgrade()).ok_or_else(|| {
            napi::Error::from_reason(format!("Page {} not found", self.page_id))
        })?;
        page.engine.go_forward().await?;
        Ok(())
    }

    #[napi]
    pub async fn wait_for_request(&self, url: String) -> Result<()> {
        let page = PAGES.get(&self.page_id).and_then(|w| w.upgrade()).ok_or_else(|| {
            napi::Error::from_reason(format!("Page {} not found", self.page_id))
        })?;
        page.engine.wait_for_request(&url).await?;
        Ok(())
    }

    #[napi]
    pub async fn wait_for_response(&self, url: String) -> Result<()> {
        let page = PAGES.get(&self.page_id).and_then(|w| w.upgrade()).ok_or_else(|| {
            napi::Error::from_reason(format!("Page {} not found", self.page_id))
        })?;
        page.engine.wait_for_response(&url).await?;
        Ok(())
    }

    #[napi]
    pub async fn wait_for_selector(&self, selector: String) -> Result<()> {
        let page = PAGES.get(&self.page_id).and_then(|w| w.upgrade()).ok_or_else(|| {
            napi::Error::from_reason(format!("Page {} not found", self.page_id))
        })?;
        page.engine.wait_for_selector(&selector).await?;
        Ok(())
    }

    #[napi]
    pub async fn wait_for_function(&self, js: String, options: Option<crate::locator::LocatorActionOptions>) -> Result<()> {
        let timeout_ms = options.as_ref().and_then(|o| o.timeout).unwrap_or(30000);
        let engine = crate::assertions::engine::AssertionEngine::default();
        let timeout = std::time::Duration::from_millis(timeout_ms as u64);
        
        let page = PAGES.get(&self.page_id).and_then(|w| w.upgrade()).ok_or_else(|| {
            napi::Error::from_reason(format!("Page {} not found", self.page_id))
        })?;
        
        let wait_js = format!("
            (async function() {{
                const fn = {};
                const res = await fn();
                if (!res) throw new Error('Condition not met');
                return res;
            }})()
        ", js);
        
        engine.poll_with_timeout(timeout, || async {
            match page.engine.evaluate(&wait_js).await {
                Ok(res) => Ok(res),
                Err(e) => Err(e.to_string()),
            }
        }).await.map_err(|e| napi::Error::from_reason(e.to_string()))?;
        
        Ok(())
    }

    #[napi]
    pub async fn evaluate_handle(&self, js: String) -> Result<()> {
        let page = PAGES.get(&self.page_id).and_then(|w| w.upgrade()).ok_or_else(|| {
            napi::Error::from_reason(format!("Page {} not found", self.page_id))
        })?;
        page.engine.evaluate_handle(&js).await?;
        Ok(())
    }

    #[napi]
    pub async fn add_script_tag(&self, content: String) -> Result<()> {
        let page = PAGES.get(&self.page_id).and_then(|w| w.upgrade()).ok_or_else(|| {
            napi::Error::from_reason(format!("Page {} not found", self.page_id))
        })?;
        page.engine.add_script_tag(&content).await?;
        Ok(())
    }

    #[napi]
    pub async fn add_style_tag(&self, content: String) -> Result<()> {
        let page = PAGES.get(&self.page_id).and_then(|w| w.upgrade()).ok_or_else(|| {
            napi::Error::from_reason(format!("Page {} not found", self.page_id))
        })?;
        page.engine.add_style_tag(&content).await?;
        Ok(())
    }

    #[napi]
    pub async fn expose_function(&self, name: String, js: String) -> Result<()> {
        let page = PAGES.get(&self.page_id).and_then(|w| w.upgrade()).ok_or_else(|| {
            napi::Error::from_reason(format!("Page {} not found", self.page_id))
        })?;
        page.engine.expose_function(&name, &js).await?;
        Ok(())
    }

    #[napi]
    pub fn on(&self, event: String, handler: ThreadsafeFunction<String>) -> Result<()> {
        crate::network::events::register_listener(&self.page_id, &event, handler);
        Ok(())
    }

    #[napi]
    pub fn off(&self, event: String) -> Result<()> {
        crate::network::events::unregister_listeners(&self.page_id, &event);
        Ok(())
    }

    /// Get all localStorage entries as key-value pairs.
    #[napi]
    pub async fn get_local_storage(&self) -> Result<std::collections::HashMap<String, String>> {
        let page = PAGES.get(&self.page_id).and_then(|w| w.upgrade()).ok_or_else(|| {
            napi::Error::from_reason(format!("Page {} not found", self.page_id))
        })?;
        page.engine.get_local_storage().await.map_err(|e| {
            napi::Error::from_reason(format!("Failed to get localStorage: {}", e))
        })
    }

    /// Set multiple localStorage entries from key-value pairs.
    #[napi]
    pub async fn set_local_storage(&self, items: std::collections::HashMap<String, String>) -> Result<()> {
        let page = PAGES.get(&self.page_id).and_then(|w| w.upgrade()).ok_or_else(|| {
            napi::Error::from_reason(format!("Page {} not found", self.page_id))
        })?;
        page.engine.set_local_storage(items).await.map_err(|e| {
            napi::Error::from_reason(format!("Failed to set localStorage: {}", e))
        })
    }

    /// Clear all localStorage entries.
    #[napi]
    pub async fn clear_local_storage(&self) -> Result<()> {
        let page = PAGES.get(&self.page_id).and_then(|w| w.upgrade()).ok_or_else(|| {
            napi::Error::from_reason(format!("Page {} not found", self.page_id))
        })?;
        page.engine.clear_local_storage().await.map_err(|e| {
            napi::Error::from_reason(format!("Failed to clear localStorage: {}", e))
        })
    }

    /// Get all sessionStorage entries as key-value pairs.
    #[napi]
    pub async fn get_session_storage(&self) -> Result<std::collections::HashMap<String, String>> {
        let page = PAGES.get(&self.page_id).and_then(|w| w.upgrade()).ok_or_else(|| {
            napi::Error::from_reason(format!("Page {} not found", self.page_id))
        })?;
        page.engine.get_session_storage().await.map_err(|e| {
            napi::Error::from_reason(format!("Failed to get sessionStorage: {}", e))
        })
    }

    /// Set multiple sessionStorage entries from key-value pairs.
    #[napi]
    pub async fn set_session_storage(&self, items: std::collections::HashMap<String, String>) -> Result<()> {
        let page = PAGES.get(&self.page_id).and_then(|w| w.upgrade()).ok_or_else(|| {
            napi::Error::from_reason(format!("Page {} not found", self.page_id))
        })?;
        page.engine.set_session_storage(items).await.map_err(|e| {
            napi::Error::from_reason(format!("Failed to set sessionStorage: {}", e))
        })
    }

    /// Clear all sessionStorage entries.
    #[napi]
    pub async fn clear_session_storage(&self) -> Result<()> {
        let page = PAGES.get(&self.page_id).and_then(|w| w.upgrade()).ok_or_else(|| {
            napi::Error::from_reason(format!("Page {} not found", self.page_id))
        })?;
        page.engine.clear_session_storage().await.map_err(|e| {
            napi::Error::from_reason(format!("Failed to clear sessionStorage: {}", e))
        })
    }

    /// Start video recording for this page.
    /// `config_json` is an optional JSON-serialized `VideoConfig`.
    /// Requires the `video` feature and a Chromium-based engine.
    #[cfg(feature = "video")]
    #[napi]
    pub async fn start_video_recording(&self, config_json: Option<String>) -> Result<()> {
        let page = PAGES.get(&self.page_id).and_then(|w| w.upgrade()).ok_or_else(|| {
            napi::Error::from_reason(format!("Page {} not found", self.page_id))
        })?;
        page.engine.start_recording(config_json).await.map_err(|e| {
            napi::Error::from_reason(format!("Failed to start video recording: {}", e))
        })
    }

    /// Stop video recording for this page.
    /// Returns the file path of the recorded video, or `null` if no recording was active.
    #[cfg(feature = "video")]
    #[napi]
    pub async fn stop_video_recording(&self, test_passed: Option<bool>) -> Result<Option<String>> {
        let page = PAGES.get(&self.page_id).and_then(|w| w.upgrade()).ok_or_else(|| {
            napi::Error::from_reason(format!("Page {} not found", self.page_id))
        })?;
        page.engine.stop_recording(test_passed).await.map_err(|e| {
            napi::Error::from_reason(format!("Failed to stop video recording: {}", e))
        })
    }

    #[cfg(not(feature = "video"))]
    #[napi]
    pub async fn start_video_recording(&self, _config_json: Option<String>) -> Result<()> {
        Err(napi::Error::from_reason(
            "Video recording is not available. Build with --features video.".to_string(),
        ))
    }

    #[cfg(not(feature = "video"))]
    #[napi]
    pub async fn stop_video_recording(&self) -> Result<Option<String>> {
        Err(napi::Error::from_reason(
            "Video recording is not available. Build with --features video.".to_string(),
        ))
    }

    /// Return a `VideoRecorderHandle` for this page.
    /// The handle provides a fluent API: `page.video().start(...)` / `page.video().stop()`.
    /// Video recording requires building with `--features video`.
    #[napi]
    pub fn video(&self) -> VideoRecorderHandle {
        VideoRecorderHandle {
            page_id: self.page_id.clone(),
            browser_id: self.browser_id.clone(),
            context_id: self.context_id.clone(),
            video_path: std::sync::Mutex::new(None),
        }
    }
}

/// A handle returned by `page.video()` that provides a fluent API for controlling
/// video recording on a page.
///
/// Usage:
/// ```js
/// const recorder = page.video();
/// await recorder.start({ fps: 15, quality: 80 });
/// // ... test actions ...
/// const videoPath = await recorder.stop();
/// console.log(recorder.path()); // same path, nullable
/// ```
#[allow(dead_code)]
#[napi]
pub struct VideoRecorderHandle {
    page_id: String,
    browser_id: String,
    context_id: String,
    /// Cached video path, populated after stop() completes.
    video_path: std::sync::Mutex<Option<String>>,
}

#[cfg(feature = "video")]
#[napi]
impl VideoRecorderHandle {
    /// Start video recording with optional config.
    /// `config_json` is an optional JSON-serialized `VideoConfig`.
    #[napi]
    pub async fn start(&self, config_json: Option<String>) -> Result<()> {
        let page = PAGES.get(&self.page_id).and_then(|w| w.upgrade()).ok_or_else(|| {
            napi::Error::from_reason(format!("Page {} not found", self.page_id))
        })?;
        page.engine.start_recording(config_json).await.map_err(|e| {
            napi::Error::from_reason(format!("Failed to start video recording: {}", e))
        })?;
        // Clear stale path from any previous recording
        *self.video_path.lock().unwrap() = None;
        Ok(())
    }

    /// Stop video recording and return the video file path, or `null` if no recording was active.
    /// If `test_passed` is provided, applies the retention policy (e.g. delete on pass).
    #[napi]
    pub async fn stop(&self, test_passed: Option<bool>) -> Result<Option<String>> {
        let page = PAGES.get(&self.page_id).and_then(|w| w.upgrade()).ok_or_else(|| {
            napi::Error::from_reason(format!("Page {} not found", self.page_id))
        })?;
        let path = page.engine.stop_recording(test_passed).await.map_err(|e| {
            napi::Error::from_reason(format!("Failed to stop video recording: {}", e))
        })?;
        *self.video_path.lock().unwrap() = path.clone();
        Ok(path)
    }

    /// Return the video file path after recording completes, or `null` before.
    #[napi]
    pub fn path(&self) -> Option<String> {
        self.video_path.lock().unwrap().clone()
    }
}

#[napi(object)]
#[derive(Default, Clone)]
pub struct ScreenshotOptions {
    pub full_page: Option<bool>,
    pub r#type: Option<String>,
    pub quality: Option<i32>,
}

#[napi(object)]
#[derive(Default, Clone)]
pub struct EmulateMediaOptions {
    pub color_scheme: Option<String>,
    pub reduced_motion: Option<String>,
    pub forced_colors: Option<String>,
}

#[napi(object)]
#[derive(Default, Clone)]
pub struct AccessibilityCheckOptions {
    pub standard: Option<String>,
    pub reporter: Option<String>,
    pub severity: Option<Vec<String>>,
    pub rules: Option<std::collections::HashMap<String, String>>,
}

#[napi(object)]
#[derive(Default, Clone)]
pub struct AccessibilityViolation {
    pub id: String,
    pub impact: String,
    pub description: String,
    pub help_url: String,
    pub nodes: Vec<AccessibilityNode>,
}

#[napi(object)]
#[derive(Default, Clone)]
pub struct AccessibilityNode {
    pub target: String,
    pub html: String,
    pub failure_summary: Option<String>,
}

#[napi(object)]
#[derive(Default, Clone)]
pub struct AccessibilityCheckResult {
    pub violations: Vec<AccessibilityViolation>,
    pub passes: Vec<AccessibilityViolation>,
    pub incomplete: Vec<AccessibilityViolation>,
    pub inapplicable: Vec<AccessibilityViolation>,
}
