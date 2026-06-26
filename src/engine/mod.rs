use std::collections::HashMap;
use std::sync::{Arc, Weak};
use dashmap::DashMap;
use lazy_static::lazy_static;
use async_trait::async_trait;
use tokio::sync::RwLock;

#[cfg(feature = "chromium")]
pub mod chromium;
#[cfg(feature = "bidi")]
pub mod bidi;
pub mod firefox;
pub mod webkit;
pub mod webdriver;
pub use crate::events;

use crate::error::TurbosheetError;
use crate::context::ContextOptions;
use crate::page::ScreenshotOptions;

pub struct Browser {
    pub id: String,
    pub engine: Arc<dyn BrowserEngine>,
    pub contexts: RwLock<Vec<Arc<Context>>>,
}

/// Drop safety net — if Browser is dropped without an explicit close(),
/// removes the DashMap entry so the global registry doesn't leak.
/// The `close()` method is the primary lifecycle hook and already removes
/// from BROWSERS before dropping — this impl is a second line of defence.
impl Drop for Browser {
    fn drop(&mut self) {
        BROWSERS.remove(&self.id);
    }
}

pub struct Context {
    pub id: String,
    pub browser: Weak<Browser>,
    pub pages: RwLock<Vec<Arc<Page>>>,
    pub engine: Arc<dyn ContextEngine>,
}

impl Drop for Context {
    fn drop(&mut self) {
        CONTEXTS.remove(&self.id);
    }
}

pub struct Page {
    pub id: String,
    pub context: Weak<Context>,
    pub engine: Arc<dyn PageEngine>,
}

impl Drop for Page {
    fn drop(&mut self) {
        PAGES.remove(&self.id);
    }
}

#[async_trait]
pub trait BrowserEngine: Send + Sync {
    async fn new_context(&self, options: Option<ContextOptions>) -> Result<Arc<dyn ContextEngine>, TurbosheetError>;
    async fn close(&self) -> Result<(), TurbosheetError>;
    fn version(&self) -> String;
}

#[async_trait]
pub trait ContextEngine: Send + Sync {
    async fn new_page(&self) -> Result<Arc<dyn PageEngine>, TurbosheetError>;
    async fn pages(&self) -> Result<Vec<Arc<dyn PageEngine>>, TurbosheetError>;
    async fn get_cookies(&self) -> Result<Vec<crate::context::Cookie>, TurbosheetError>;
    /// Get cookies optionally filtered by URL.
    /// Uses Network.getCookies (CDP) or WebDriver GET /cookie with post-filter.
    async fn cookies(&self, urls: Option<Vec<String>>) -> Result<Vec<crate::context::Cookie>, TurbosheetError>;
    async fn set_cookies(&self, cookies: Vec<crate::context::CookieParam>) -> Result<(), TurbosheetError>;
    /// Add cookies via Network.setCookies (CDP) or WebDriver POST /cookie.
    /// Semantically "add" rather than "set" — does not clear existing cookies.
    async fn add_cookies(&self, cookies: Vec<crate::context::CookieParam>) -> Result<(), TurbosheetError>;
    async fn clear_cookies(&self) -> Result<(), TurbosheetError>;
    async fn close(&self) -> Result<(), TurbosheetError>;
}

#[derive(Default, Clone, Copy)]
pub enum LoadState {
    #[default]
    Load,
    DomContentLoaded,
    NetworkIdle,
}

#[derive(Clone, Copy)]
pub struct RetryConfig {
    pub retries: u32,
    pub delay_ms: u64,
}

impl Default for RetryConfig {
    fn default() -> Self {
        Self { retries: 3, delay_ms: 100 }
    }
}

#[async_trait]
pub trait PageEngine: Send + Sync {
    async fn goto(&self, url: &str, state: LoadState) -> Result<(), TurbosheetError>;
    fn url(&self) -> String;
    async fn screenshot(&self, options: Option<ScreenshotOptions>) -> Result<Vec<u8>, TurbosheetError>;
    async fn close(&self) -> Result<(), TurbosheetError>;
    async fn evaluate(&self, js: &str) -> Result<String, TurbosheetError>;
    async fn content(&self) -> Result<String, TurbosheetError>;
    async fn title(&self) -> Result<String, TurbosheetError>;

    async fn click(&self, selector: &str) -> Result<(), TurbosheetError>;
    async fn dblclick(&self, selector: &str) -> Result<(), TurbosheetError>;
    async fn right_click(&self, selector: &str) -> Result<(), TurbosheetError>;
    async fn hover(&self, selector: &str) -> Result<(), TurbosheetError>;
    async fn fill(&self, selector: &str, value: &str) -> Result<(), TurbosheetError>;
    async fn check(&self, selector: &str) -> Result<(), TurbosheetError>;
    async fn uncheck(&self, selector: &str) -> Result<(), TurbosheetError>;
    async fn select(&self, selector: &str, value: &str) -> Result<(), TurbosheetError>;
    async fn focus(&self, selector: &str) -> Result<(), TurbosheetError>;
    async fn blur(&self, selector: &str) -> Result<(), TurbosheetError>;
    async fn scroll_into_view(&self, selector: &str) -> Result<(), TurbosheetError>;
    async fn text_content(&self, selector: &str) -> Result<Option<String>, TurbosheetError>;
    async fn inner_text(&self, selector: &str) -> Result<String, TurbosheetError>;
    async fn inner_html(&self, selector: &str) -> Result<String, TurbosheetError>;
    async fn get_attribute(&self, selector: &str, name: &str) -> Result<Option<String>, TurbosheetError>;
    async fn is_visible(&self, selector: &str) -> Result<bool, TurbosheetError>;
    async fn is_enabled(&self, selector: &str) -> Result<bool, TurbosheetError>;
    async fn is_disabled(&self, selector: &str) -> Result<bool, TurbosheetError>;
    async fn locator_screenshot(&self, selector: &str, options: Option<ScreenshotOptions>) -> Result<Vec<u8>, TurbosheetError>;
    
    // New Locator APIs
    async fn bounding_box(&self, selector: &str) -> Result<Option<crate::page::Rect>, TurbosheetError>;
    async fn drag_and_drop(&self, source_selector: &str, target_selector: &str) -> Result<(), TurbosheetError>;
    async fn press(&self, selector: &str, key: &str) -> Result<(), TurbosheetError>;
    async fn press_sequentially(&self, selector: &str, text: &str) -> Result<(), TurbosheetError>;
    async fn set_input_files(&self, selector: &str, files: Vec<String>) -> Result<(), TurbosheetError>;

    // New Page APIs
    async fn set_viewport_size(&self, width: u32, height: u32) -> Result<(), TurbosheetError>;
    async fn viewport_size(&self) -> Result<Option<crate::page::ViewportSize>, TurbosheetError>;
    async fn reload(&self) -> Result<(), TurbosheetError>;
    async fn go_back(&self) -> Result<(), TurbosheetError>;
    async fn go_forward(&self) -> Result<(), TurbosheetError>;
    async fn wait_for_request(&self, url: &str) -> Result<(), TurbosheetError>;
    async fn wait_for_response(&self, url: &str) -> Result<(), TurbosheetError>;
    async fn wait_for_selector(&self, selector: &str) -> Result<(), TurbosheetError>;
    async fn evaluate_handle(&self, js: &str) -> Result<(), TurbosheetError>;
    async fn add_script_tag(&self, content: &str) -> Result<(), TurbosheetError>;
    async fn add_style_tag(&self, content: &str) -> Result<(), TurbosheetError>;
    async fn expose_function(&self, name: &str, js: &str) -> Result<(), TurbosheetError>;
    async fn set_content(&self, html: &str) -> Result<(), TurbosheetError>;
    
    // Core injection API
    async fn invoke_action(&self, action: &str, args: Vec<serde_json::Value>) -> Result<serde_json::Value, TurbosheetError>;

    // ── Touch API ──────────────────────────────────────────────
    /// Tap at viewport coordinates. Default no-op (unimplemented).
    async fn tap(&self, _x: f64, _y: f64) -> Result<(), TurbosheetError> {
        Err(TurbosheetError::Other("tap not implemented".into()))
    }
    /// Swipe from (fx,fy) to (tx,ty) in N steps. Default no-op.
    async fn swipe(&self, _fx: f64, _fy: f64, _tx: f64, _ty: f64, _steps: i32) -> Result<(), TurbosheetError> {
        Err(TurbosheetError::Other("swipe not implemented".into()))
    }
    /// Two-finger pinch at (x,y) with scale factor. Default no-op.
    async fn pinch(&self, _x: f64, _y: f64, _scale: f64) -> Result<(), TurbosheetError> {
        Err(TurbosheetError::Other("pinch not implemented".into()))
    }
    /// Press-and-hold at (x,y) for duration_ms. Default no-op.
    async fn long_press(&self, _x: f64, _y: f64, _duration_ms: i32) -> Result<(), TurbosheetError> {
        Err(TurbosheetError::Other("long_press not implemented".into()))
    }

    // ── Storage API ──────────────────────────────────────────
    /// Retrieve all localStorage entries as a JSON string.
    async fn get_local_storage(&self) -> Result<HashMap<String, String>, TurbosheetError> {
        Ok(HashMap::new())
    }
    /// Set multiple localStorage entries at once (from key-value pairs).
    async fn set_local_storage(&self, _items: HashMap<String, String>) -> Result<(), TurbosheetError> {
        Ok(())
    }
    /// Clear all localStorage entries.
    async fn clear_local_storage(&self) -> Result<(), TurbosheetError> {
        Ok(())
    }
    /// Retrieve all sessionStorage entries as a JSON string.
    async fn get_session_storage(&self) -> Result<HashMap<String, String>, TurbosheetError> {
        Ok(HashMap::new())
    }
    /// Set multiple sessionStorage entries at once (from key-value pairs).
    async fn set_session_storage(&self, _items: HashMap<String, String>) -> Result<(), TurbosheetError> {
        Ok(())
    }
    /// Clear all sessionStorage entries.
    async fn clear_session_storage(&self) -> Result<(), TurbosheetError> {
        Ok(())
    }

    /// Set the well-known page identifier used to dispatch N-API events
    /// (`page.on('request'|'response')`) from the background event-handler task.
    async fn set_page_id(&self, _id: &str) -> Result<(), TurbosheetError> {
        Ok(())
    }

    // ── Media Emulation ──────────────────────────────────────────
    /// Override CSS media features (prefers-color-scheme, prefers-reduced-motion, etc.).
    async fn emulate_media(&self, _color_scheme: Option<String>, _reduced_motion: Option<String>) -> Result<(), TurbosheetError> {
        Ok(())
    }

    // ── Network Condition Overrides ──────────────────────────────
    /// Toggle offline mode for the page.
    async fn set_offline(&self, _offline: bool) -> Result<(), TurbosheetError> {
        Ok(())
    }
    /// Throttle network with latency (ms), download (bps), upload (bps).
    async fn throttle(&self, _latency: f64, _download: f64, _upload: f64) -> Result<(), TurbosheetError> {
        Ok(())
    }

    // ── WebSocket Frame Interception ──────────────────────────
    /// Enable interception of WebSocket frames (Network.webSocketFrame* events).
    async fn enable_websocket_interception(&self) -> Result<(), TurbosheetError> {
        Err(TurbosheetError::Other("WebSocket interception not supported on this engine".into()))
    }
    /// Disable WebSocket frame interception.
    async fn disable_websocket_interception(&self) -> Result<(), TurbosheetError> {
        Ok(())
    }

    // ── Accessibility Tree ────────────────────────────────────
    /// Return the full accessibility tree snapshot via CDP Accessibility.getFullAXTree
    /// or engine-specific equivalent.
    async fn accessibility_snapshot(&self) -> Result<serde_json::Value, TurbosheetError> {
        Err(TurbosheetError::Other("Accessibility snapshot not supported on this engine".into()))
    }
    
    // Injected Script Engine APIs
    async fn inject_core_script(&self, script: &str) -> Result<(), TurbosheetError>;
    async fn register_binding(&self, name: &str) -> Result<(), TurbosheetError>;
    fn global_name(&self) -> &str;

    // CDP Fetch domain interception (no-op for non-Chromium engines)
    async fn enable_fetch_interception(&self, _page_id: &str) -> Result<(), TurbosheetError> {
        Ok(())
    }
    async fn disable_fetch_interception(&self) -> Result<(), TurbosheetError> {
        Ok(())
    }

    // ── Video Recording ───────────────────────────────────────────
    /// Start recording the page viewport via CDP screencast.
    /// `config_json` is optional JSON-serialized `VideoConfig`.
    /// Default: fails with "not supported" for non-Chromium engines.
    async fn start_recording(&self, _config_json: Option<String>) -> Result<(), TurbosheetError> {
        Err(TurbosheetError::Other("Video recording not supported on this engine".into()))
    }

    /// Stop recording and finalize the video file.
    /// Returns the output file path on success.
    /// `test_passed` is `Some` when the caller knows the test outcome,
    /// enabling retention-policy enforcement (e.g. delete on pass).
    async fn stop_recording(&self, _test_passed: Option<bool>) -> Result<Option<String>, TurbosheetError> {
        Err(TurbosheetError::Other("Video recording not supported on this engine".into()))
    }
}

lazy_static! {
    /// Lookup registry — stores Arc so the browser stays alive as long as
    /// any reference (JsBrowser, tests) points at it.  Explicit close()
    /// removes the entry; Drop on Browser provides a second-line cleanup.
    pub static ref BROWSERS: DashMap<String, Arc<Browser>> = DashMap::new();
    /// Lookup registry — stores Weak; owned by Browser::contexts Vec.
    pub static ref CONTEXTS: DashMap<String, Weak<Context>> = DashMap::new();
    /// Lookup registry — stores Weak; owned by Context::pages Vec.
    pub static ref PAGES: DashMap<String, Weak<Page>> = DashMap::new();
    /// Opener relationships: child_page_id → Some(opener_page_id).
    /// Populated by ChromiumContextEngine when a popup is detected.
    pub static ref PAGE_OPENER: DashMap<String, Option<String>> = DashMap::new();
    /// Page-to-context mapping: page_id → context_id.
    /// Populated when a page is created so popup events can be dispatched
    /// to the correct context-level event listeners.
    pub static ref PAGE_CONTEXT: DashMap<String, String> = DashMap::new();
}

// ── Tests ─────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    /// Minimal browser engine for ownership-model tests (close is the
    /// only operation the cascade path exercises).
    struct TestEngine;
    #[async_trait]
    impl BrowserEngine for TestEngine {
        async fn new_context(
            &self,
            _: Option<ContextOptions>,
        ) -> Result<Arc<dyn ContextEngine>, TurbosheetError> {
            unimplemented!()
        }
        async fn close(&self) -> Result<(), TurbosheetError> {
            Ok(())
        }
        fn version(&self) -> String {
            "test".into()
        }
    }

    /// Minimal context engine for ownership-model tests.
    struct TestContextEngine;
    #[async_trait]
    impl ContextEngine for TestContextEngine {
        async fn new_page(&self) -> Result<Arc<dyn PageEngine>, TurbosheetError> {
            unimplemented!()
        }
        async fn pages(&self) -> Result<Vec<Arc<dyn PageEngine>>, TurbosheetError> {
            Ok(vec![])
        }
        async fn get_cookies(&self) -> Result<Vec<crate::context::Cookie>, TurbosheetError> {
            Ok(vec![])
        }
        async fn cookies(&self, _urls: Option<Vec<String>>) -> Result<Vec<crate::context::Cookie>, TurbosheetError> {
            Ok(vec![])
        }
        async fn set_cookies(&self, _cookies: Vec<crate::context::CookieParam>) -> Result<(), TurbosheetError> {
            Ok(())
        }
        async fn add_cookies(&self, _cookies: Vec<crate::context::CookieParam>) -> Result<(), TurbosheetError> {
            Ok(())
        }
        async fn clear_cookies(&self) -> Result<(), TurbosheetError> {
            Ok(())
        }
        async fn close(&self) -> Result<(), TurbosheetError> {
            Ok(())
        }
    }

    /// Minimal page engine for ownership-model tests.
    /// Most methods panic — only `close()` is exercised by the cascade path.
    struct TestPageEngine;
    #[async_trait]
    impl PageEngine for TestPageEngine {
        async fn goto(&self, _: &str, _: LoadState) -> Result<(), TurbosheetError> { unimplemented!() }
        fn url(&self) -> String { String::new() }
        async fn screenshot(&self, _: Option<ScreenshotOptions>) -> Result<Vec<u8>, TurbosheetError> { unimplemented!() }
        async fn close(&self) -> Result<(), TurbosheetError> { Ok(()) }
        async fn evaluate(&self, _: &str) -> Result<String, TurbosheetError> { unimplemented!() }
        async fn content(&self) -> Result<String, TurbosheetError> { unimplemented!() }
        async fn title(&self) -> Result<String, TurbosheetError> { unimplemented!() }
        async fn click(&self, _: &str) -> Result<(), TurbosheetError> { unimplemented!() }
        async fn dblclick(&self, _: &str) -> Result<(), TurbosheetError> { unimplemented!() }
        async fn right_click(&self, _: &str) -> Result<(), TurbosheetError> { unimplemented!() }
        async fn hover(&self, _: &str) -> Result<(), TurbosheetError> { unimplemented!() }
        async fn fill(&self, _: &str, _: &str) -> Result<(), TurbosheetError> { unimplemented!() }
        async fn check(&self, _: &str) -> Result<(), TurbosheetError> { unimplemented!() }
        async fn uncheck(&self, _: &str) -> Result<(), TurbosheetError> { unimplemented!() }
        async fn select(&self, _: &str, _: &str) -> Result<(), TurbosheetError> { unimplemented!() }
        async fn focus(&self, _: &str) -> Result<(), TurbosheetError> { unimplemented!() }
        async fn blur(&self, _: &str) -> Result<(), TurbosheetError> { unimplemented!() }
        async fn scroll_into_view(&self, _: &str) -> Result<(), TurbosheetError> { unimplemented!() }
        async fn text_content(&self, _: &str) -> Result<Option<String>, TurbosheetError> { unimplemented!() }
        async fn inner_text(&self, _: &str) -> Result<String, TurbosheetError> { unimplemented!() }
        async fn inner_html(&self, _: &str) -> Result<String, TurbosheetError> { unimplemented!() }
        async fn get_attribute(&self, _: &str, _: &str) -> Result<Option<String>, TurbosheetError> { unimplemented!() }
        async fn is_visible(&self, _: &str) -> Result<bool, TurbosheetError> { unimplemented!() }
        async fn is_enabled(&self, _: &str) -> Result<bool, TurbosheetError> { unimplemented!() }
        async fn is_disabled(&self, _: &str) -> Result<bool, TurbosheetError> { unimplemented!() }
        async fn locator_screenshot(&self, _: &str, _: Option<ScreenshotOptions>) -> Result<Vec<u8>, TurbosheetError> { unimplemented!() }
        async fn bounding_box(&self, _: &str) -> Result<Option<crate::page::Rect>, TurbosheetError> { unimplemented!() }
        async fn drag_and_drop(&self, _: &str, _: &str) -> Result<(), TurbosheetError> { unimplemented!() }
        async fn press(&self, _: &str, _: &str) -> Result<(), TurbosheetError> { unimplemented!() }
        async fn press_sequentially(&self, _: &str, _: &str) -> Result<(), TurbosheetError> { unimplemented!() }
        async fn set_input_files(&self, _: &str, _: Vec<String>) -> Result<(), TurbosheetError> { unimplemented!() }
        async fn set_viewport_size(&self, _: u32, _: u32) -> Result<(), TurbosheetError> { unimplemented!() }
        async fn viewport_size(&self) -> Result<Option<crate::page::ViewportSize>, TurbosheetError> { unimplemented!() }
        async fn reload(&self) -> Result<(), TurbosheetError> { unimplemented!() }
        async fn go_back(&self) -> Result<(), TurbosheetError> { unimplemented!() }
        async fn go_forward(&self) -> Result<(), TurbosheetError> { unimplemented!() }
        async fn wait_for_request(&self, _: &str) -> Result<(), TurbosheetError> { unimplemented!() }
        async fn wait_for_response(&self, _: &str) -> Result<(), TurbosheetError> { unimplemented!() }
        async fn wait_for_selector(&self, _: &str) -> Result<(), TurbosheetError> { unimplemented!() }
        async fn evaluate_handle(&self, _: &str) -> Result<(), TurbosheetError> { unimplemented!() }
        async fn add_script_tag(&self, _: &str) -> Result<(), TurbosheetError> { unimplemented!() }
        async fn add_style_tag(&self, _: &str) -> Result<(), TurbosheetError> { unimplemented!() }
        async fn expose_function(&self, _: &str, _: &str) -> Result<(), TurbosheetError> { unimplemented!() }
        async fn set_content(&self, _: &str) -> Result<(), TurbosheetError> { unimplemented!() }
        async fn get_local_storage(&self) -> Result<HashMap<String, String>, TurbosheetError> { Ok(HashMap::new()) }
        async fn set_local_storage(&self, _: HashMap<String, String>) -> Result<(), TurbosheetError> { Ok(()) }
        async fn clear_local_storage(&self) -> Result<(), TurbosheetError> { Ok(()) }
        async fn get_session_storage(&self) -> Result<HashMap<String, String>, TurbosheetError> { Ok(HashMap::new()) }
        async fn set_session_storage(&self, _: HashMap<String, String>) -> Result<(), TurbosheetError> { Ok(()) }
        async fn clear_session_storage(&self) -> Result<(), TurbosheetError> { Ok(()) }
        async fn invoke_action(&self, _: &str, _: Vec<serde_json::Value>) -> Result<serde_json::Value, TurbosheetError> { unimplemented!() }
        async fn inject_core_script(&self, _: &str) -> Result<(), TurbosheetError> { unimplemented!() }
        async fn register_binding(&self, _: &str) -> Result<(), TurbosheetError> { unimplemented!() }
        fn global_name(&self) -> &str { "__ts_g_test" }
    }

    /// BROWSERS stores Arc<Browser>, so the entry persists even after
    /// the local Arc goes out of scope.  Explicit remove() is required.
    #[tokio::test]
    async fn browser_persists_via_dashmap_arc() {
        let id = "persist-test-br".to_string();
        let browser = Arc::new(Browser {
            id: id.clone(),
            engine: Arc::new(TestEngine),
            contexts: RwLock::new(Vec::new()),
        });
        BROWSERS.insert(id.clone(), Arc::clone(&browser));
        assert!(BROWSERS.contains_key(&id));

        // Drop local Arc — DashMap still holds the entry
        drop(browser);
        assert!(BROWSERS.contains_key(&id), "DashMap Arc keeps Browser alive");

        // Explicit remove is needed
        BROWSERS.remove(&id);
        assert!(!BROWSERS.contains_key(&id), "Removed from DashMap");
    }

    /// CONTEXTS and PAGES still use Weak<…> storage.
    /// When the hierarchy's strong refs are released the DashMap entries
    /// become dangling and are cleaned up by Drop.
    #[tokio::test]
    async fn weak_dashmap_is_drop_safe() {
        let ctx_id = "drop-test-ctx".to_string();
        let pg_id = "drop-test-pg".to_string();

        // Set up Browser → Context → Page hierarchy
        let browser = Arc::new(Browser {
            id: "drop-test-parent".to_string(),
            engine: Arc::new(TestEngine),
            contexts: RwLock::new(Vec::new()),
        });

        {
            // Context holds a Weak<Browser> so it doesn't prevent drop
            let context = Arc::new(Context {
                id: ctx_id.clone(),
                browser: Arc::downgrade(&browser),
                pages: RwLock::new(Vec::new()),
                engine: Arc::new(TestContextEngine),
            });
            CONTEXTS.insert(ctx_id.clone(), Arc::downgrade(&context));
            browser.contexts.write().await.push(context.clone());

            let page = Arc::new(Page {
                id: pg_id.clone(),
                context: Arc::downgrade(&context),
                engine: Arc::new(TestPageEngine),
            });
            PAGES.insert(pg_id.clone(), Arc::downgrade(&page));
            context.pages.write().await.push(page);

            // All three are findable via DashMap (BROWSERS no longer Weak)
            assert!(CONTEXTS.get(&ctx_id).and_then(|w| w.upgrade()).is_some());
            assert!(PAGES.get(&pg_id).and_then(|w| w.upgrade()).is_some());
        }
        // context and page Arcs dropped — hierarchy's strong refs still
        // held by `browser` (for context) and context.pages (for page).
        // Context.pages still holds Arc<Page> so page is alive.
        assert!(
            CONTEXTS.get(&ctx_id).and_then(|w| w.upgrade()).is_some(),
            "Context still alive via browser.contexts"
        );
        assert!(
            PAGES.get(&pg_id).and_then(|w| w.upgrade()).is_some(),
            "Page still alive via context.pages"
        );

        // Cleanup
        CONTEXTS.remove(&ctx_id);
        PAGES.remove(&pg_id);
    }

    /// Hierarchy: Browser.contexts tracks owned Contexts;
    /// Context.pages tracks owned Pages.
    #[tokio::test]
    async fn hierarchy_is_maintained() {
        let browser = Arc::new(Browser {
            id: "hierarchy-br".to_string(),
            engine: Arc::new(TestEngine),
            contexts: RwLock::new(Vec::new()),
        });

        let ctx1 = Arc::new(Context {
            id: "hierarchy-ctx-1".to_string(),
            browser: Arc::downgrade(&browser),
            pages: RwLock::new(Vec::new()),
            engine: Arc::new(TestContextEngine),
        });
        let ctx2 = Arc::new(Context {
            id: "hierarchy-ctx-2".to_string(),
            browser: Arc::downgrade(&browser),
            pages: RwLock::new(Vec::new()),
            engine: Arc::new(TestContextEngine),
        });
        browser.contexts.write().await.push(ctx1.clone());
        browser.contexts.write().await.push(ctx2.clone());

        assert_eq!(browser.contexts.read().await.len(), 2);

        let pg1 = Arc::new(Page {
            id: "hierarchy-pg-1".to_string(),
            context: Arc::downgrade(&ctx1),
            engine: Arc::new(TestPageEngine),
        });
        let pg2 = Arc::new(Page {
            id: "hierarchy-pg-2".to_string(),
            context: Arc::downgrade(&ctx1),
            engine: Arc::new(TestPageEngine),
        });
        ctx1.pages.write().await.push(pg1.clone());
        ctx1.pages.write().await.push(pg2);

        assert_eq!(ctx1.pages.read().await.len(), 2);
        assert_eq!(ctx2.pages.read().await.len(), 0);

        // Upward traversal via Weak
        let ctx_from_weak = pg1.context.upgrade().unwrap();
        assert_eq!(ctx_from_weak.id, "hierarchy-ctx-1");
        let br_from_weak = ctx1.browser.upgrade().unwrap();
        assert_eq!(br_from_weak.id, "hierarchy-br");
    }

    /// Cascade close: closing a context removes its pages from PAGES
    /// lookup; closing a browser removes contexts from CONTEXTS lookup.
    /// BROWSERS now stores Arc so remove returns a live Arc directly.
    #[tokio::test]
    async fn cascade_close_cleans_dashmaps() {
        let br_id = "cascade-br".to_string();
        let ctx_id = "cascade-ctx".to_string();
        let pg_id = "cascade-pg".to_string();

        let browser = Arc::new(Browser {
            id: br_id.clone(),
            engine: Arc::new(TestEngine),
            contexts: RwLock::new(Vec::new()),
        });
        BROWSERS.insert(br_id.clone(), Arc::clone(&browser));

        let context = Arc::new(Context {
            id: ctx_id.clone(),
            browser: Arc::downgrade(&browser),
            pages: RwLock::new(Vec::new()),
            engine: Arc::new(TestContextEngine),
        });
        CONTEXTS.insert(ctx_id.clone(), Arc::downgrade(&context));
        browser.contexts.write().await.push(context.clone());

        let page = Arc::new(Page {
            id: pg_id.clone(),
            context: Arc::downgrade(&context),
            engine: Arc::new(TestPageEngine),
        });
        PAGES.insert(pg_id.clone(), Arc::downgrade(&page));
        context.pages.write().await.push(page);

        // All findable
        assert!(BROWSERS.contains_key(&br_id));
        assert!(CONTEXTS.get(&ctx_id).and_then(|w| w.upgrade()).is_some());
        assert!(PAGES.get(&pg_id).and_then(|w| w.upgrade()).is_some());

        // Simulate page close: remove from PAGES + remove from parent
        if let Some((_, weak)) = PAGES.remove(&pg_id) {
            if let Some(pg) = weak.upgrade() {
                if let Some(ctx) = pg.context.upgrade() {
                    ctx.pages.write().await.retain(|p| p.id != pg_id);
                }
            }
        }
        assert!(PAGES.get(&pg_id).and_then(|w| w.upgrade()).is_none());
        assert_eq!(context.pages.read().await.len(), 0);

        // Simulate context close: remove from CONTEXTS + remove from parent
        if let Some((_, weak)) = CONTEXTS.remove(&ctx_id) {
            if let Some(ctx) = weak.upgrade() {
                if let Some(br) = ctx.browser.upgrade() {
                    br.contexts.write().await.retain(|c| c.id != ctx_id);
                }
            }
        }
        assert!(CONTEXTS.get(&ctx_id).and_then(|w| w.upgrade()).is_none());
        assert_eq!(browser.contexts.read().await.len(), 0);

        // Simulate browser close: remove from BROWSERS directly
        BROWSERS.remove(&br_id);
        assert!(!BROWSERS.contains_key(&br_id));
    }
}
