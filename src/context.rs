use napi::bindgen_prelude::*;
use napi::threadsafe_function::ThreadsafeFunction;
use napi_derive::napi;

use std::sync::Arc;
use tokio::sync::broadcast;
use crate::page::JsPage;
use crate::engine::{CONTEXTS, PAGES, Page};

#[napi(object)]
#[derive(Default, Clone)]
pub struct ProxySettings {
    /// Upstream proxy URL, e.g. "http://proxy.company.com:8080"
    pub server: String,
    /// Comma-separated list of hosts to bypass the proxy
    pub bypass: Option<String>,
}

#[napi(object)]
#[derive(Default, Clone)]
pub struct ContextOptions {
    pub viewport: Option<Viewport>,
    pub user_agent: Option<String>,
    pub locale: Option<String>,
    pub timezone_id: Option<String>,
    pub color_scheme: Option<String>,
    pub device_scale_factor: Option<f64>,
    /// Proxy configuration for browser context.
    /// When set, all traffic from this context routes through the specified proxy.
    pub use_proxy: Option<ProxySettings>,
}

#[napi(object)]
#[derive(Default, Clone)]
pub struct Viewport {
    pub width: i32,
    pub height: i32,
}

#[napi(object)]
#[derive(Clone)]
pub struct DeviceDescriptor {
    pub name: String,
    pub viewport: Viewport,
    pub user_agent: String,
    pub device_scale_factor: f64,
    pub has_touch: bool,
    pub is_mobile: bool,
}

impl DeviceDescriptor {
    pub fn all() -> Vec<Self> {
        vec![
            DeviceDescriptor {
                name: "iPhone 15 Pro".into(),
                viewport: Viewport { width: 393, height: 852 },
                user_agent: "Mozilla/5.0 (iPhone; CPU iPhone OS 17_0 like Mac OS X) AppleWebKit/605.1.15 (KHTML, like Gecko) Version/17.0 Mobile/15E148 Safari/604.1".into(),
                device_scale_factor: 3.0,
                has_touch: true,
                is_mobile: true,
            },
            DeviceDescriptor {
                name: "iPhone 15".into(),
                viewport: Viewport { width: 390, height: 844 },
                user_agent: "Mozilla/5.0 (iPhone; CPU iPhone OS 17_0 like Mac OS X) AppleWebKit/605.1.15 (KHTML, like Gecko) Version/17.0 Mobile/15E148 Safari/604.1".into(),
                device_scale_factor: 2.0,
                has_touch: true,
                is_mobile: true,
            },
            DeviceDescriptor {
                name: "iPhone 14 Pro Max".into(),
                viewport: Viewport { width: 430, height: 932 },
                user_agent: "Mozilla/5.0 (iPhone; CPU iPhone OS 16_0 like Mac OS X) AppleWebKit/605.1.15 (KHTML, like Gecko) Version/16.0 Mobile/15E148 Safari/604.1".into(),
                device_scale_factor: 3.0,
                has_touch: true,
                is_mobile: true,
            },
            DeviceDescriptor {
                name: "iPhone 14".into(),
                viewport: Viewport { width: 390, height: 844 },
                user_agent: "Mozilla/5.0 (iPhone; CPU iPhone OS 16_0 like Mac OS X) AppleWebKit/605.1.15 (KHTML, like Gecko) Version/16.0 Mobile/15E148 Safari/604.1".into(),
                device_scale_factor: 2.0,
                has_touch: true,
                is_mobile: true,
            },
            DeviceDescriptor {
                name: "iPhone SE (3rd gen)".into(),
                viewport: Viewport { width: 375, height: 667 },
                user_agent: "Mozilla/5.0 (iPhone; CPU iPhone OS 16_0 like Mac OS X) AppleWebKit/605.1.15 (KHTML, like Gecko) Version/16.0 Mobile/15E148 Safari/604.1".into(),
                device_scale_factor: 2.0,
                has_touch: true,
                is_mobile: true,
            },
            DeviceDescriptor {
                name: "iPhone 13".into(),
                viewport: Viewport { width: 390, height: 844 },
                user_agent: "Mozilla/5.0 (iPhone; CPU iPhone OS 15_0 like Mac OS X) AppleWebKit/605.1.15 (KHTML, like Gecko) Version/15.0 Mobile/15E148 Safari/604.1".into(),
                device_scale_factor: 2.0,
                has_touch: true,
                is_mobile: true,
            },
            DeviceDescriptor {
                name: "iPhone 12".into(),
                viewport: Viewport { width: 390, height: 844 },
                user_agent: "Mozilla/5.0 (iPhone; CPU iPhone OS 14_0 like Mac OS X) AppleWebKit/605.1.15 (KHTML, like Gecko) Version/14.0 Mobile/15E148 Safari/604.1".into(),
                device_scale_factor: 2.0,
                has_touch: true,
                is_mobile: true,
            },
            DeviceDescriptor {
                name: "iPad Pro 12.9 (6th gen)".into(),
                viewport: Viewport { width: 1024, height: 1366 },
                user_agent: "Mozilla/5.0 (iPad; CPU OS 17_0 like Mac OS X) AppleWebKit/605.1.15 (KHTML, like Gecko) Version/17.0 Mobile/15E148 Safari/604.1".into(),
                device_scale_factor: 2.0,
                has_touch: true,
                is_mobile: true,
            },
            DeviceDescriptor {
                name: "iPad Pro 11 (4th gen)".into(),
                viewport: Viewport { width: 834, height: 1194 },
                user_agent: "Mozilla/5.0 (iPad; CPU OS 17_0 like Mac OS X) AppleWebKit/605.1.15 (KHTML, like Gecko) Version/17.0 Mobile/15E148 Safari/604.1".into(),
                device_scale_factor: 2.0,
                has_touch: true,
                is_mobile: true,
            },
            DeviceDescriptor {
                name: "iPad Air (5th gen)".into(),
                viewport: Viewport { width: 820, height: 1180 },
                user_agent: "Mozilla/5.0 (iPad; CPU OS 17_0 like Mac OS X) AppleWebKit/605.1.15 (KHTML, like Gecko) Version/17.0 Mobile/15E148 Safari/604.1".into(),
                device_scale_factor: 2.0,
                has_touch: true,
                is_mobile: true,
            },
            DeviceDescriptor {
                name: "iPad (10th gen)".into(),
                viewport: Viewport { width: 820, height: 1180 },
                user_agent: "Mozilla/5.0 (iPad; CPU OS 17_0 like Mac OS X) AppleWebKit/605.1.15 (KHTML, like Gecko) Version/17.0 Mobile/15E148 Safari/604.1".into(),
                device_scale_factor: 2.0,
                has_touch: true,
                is_mobile: true,
            },
            DeviceDescriptor {
                name: "iPad Mini (6th gen)".into(),
                viewport: Viewport { width: 744, height: 1133 },
                user_agent: "Mozilla/5.0 (iPad; CPU OS 17_0 like Mac OS X) AppleWebKit/605.1.15 (KHTML, like Gecko) Version/17.0 Mobile/15E148 Safari/604.1".into(),
                device_scale_factor: 2.0,
                has_touch: true,
                is_mobile: true,
            },
            DeviceDescriptor {
                name: "Pixel 8 Pro".into(),
                viewport: Viewport { width: 412, height: 915 },
                user_agent: "Mozilla/5.0 (Linux; Android 14; Pixel 8 Pro) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Mobile Safari/537.36".into(),
                device_scale_factor: 2.625,
                has_touch: true,
                is_mobile: true,
            },
            DeviceDescriptor {
                name: "Pixel 8".into(),
                viewport: Viewport { width: 412, height: 915 },
                user_agent: "Mozilla/5.0 (Linux; Android 14; Pixel 8) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Mobile Safari/537.36".into(),
                device_scale_factor: 2.625,
                has_touch: true,
                is_mobile: true,
            },
            DeviceDescriptor {
                name: "Pixel 7".into(),
                viewport: Viewport { width: 412, height: 915 },
                user_agent: "Mozilla/5.0 (Linux; Android 13; Pixel 7) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Mobile Safari/537.36".into(),
                device_scale_factor: 2.625,
                has_touch: true,
                is_mobile: true,
            },
            DeviceDescriptor {
                name: "Pixel 6".into(),
                viewport: Viewport { width: 412, height: 915 },
                user_agent: "Mozilla/5.0 (Linux; Android 12; Pixel 6) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Mobile Safari/537.36".into(),
                device_scale_factor: 2.625,
                has_touch: true,
                is_mobile: true,
            },
            DeviceDescriptor {
                name: "Samsung Galaxy S24 Ultra".into(),
                viewport: Viewport { width: 412, height: 915 },
                user_agent: "Mozilla/5.0 (Linux; Android 14; SM-S928B) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Mobile Safari/537.36".into(),
                device_scale_factor: 3.9,
                has_touch: true,
                is_mobile: true,
            },
            DeviceDescriptor {
                name: "Samsung Galaxy S24".into(),
                viewport: Viewport { width: 360, height: 780 },
                user_agent: "Mozilla/5.0 (Linux; Android 14; SM-S921B) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Mobile Safari/537.36".into(),
                device_scale_factor: 3.0,
                has_touch: true,
                is_mobile: true,
            },
            DeviceDescriptor {
                name: "Samsung Galaxy S23".into(),
                viewport: Viewport { width: 360, height: 780 },
                user_agent: "Mozilla/5.0 (Linux; Android 13; SM-S911B) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Mobile Safari/537.36".into(),
                device_scale_factor: 3.0,
                has_touch: true,
                is_mobile: true,
            },
            DeviceDescriptor {
                name: "Samsung Galaxy S22".into(),
                viewport: Viewport { width: 360, height: 780 },
                user_agent: "Mozilla/5.0 (Linux; Android 12; SM-S906B) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Mobile Safari/537.36".into(),
                device_scale_factor: 3.0,
                has_touch: true,
                is_mobile: true,
            },
            DeviceDescriptor {
                name: "Samsung Galaxy A54".into(),
                viewport: Viewport { width: 412, height: 915 },
                user_agent: "Mozilla/5.0 (Linux; Android 13; SM-A546B) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Mobile Safari/537.36".into(),
                device_scale_factor: 2.625,
                has_touch: true,
                is_mobile: true,
            },
            DeviceDescriptor {
                name: "Samsung Galaxy Tab S8 Ultra".into(),
                viewport: Viewport { width: 912, height: 1368 },
                user_agent: "Mozilla/5.0 (Linux; Android 13; SM-X900) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36".into(),
                device_scale_factor: 2.0,
                has_touch: true,
                is_mobile: true,
            },
            DeviceDescriptor {
                name: "Samsung Galaxy Tab S8".into(),
                viewport: Viewport { width: 800, height: 1280 },
                user_agent: "Mozilla/5.0 (Linux; Android 13; SM-X700) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36".into(),
                device_scale_factor: 2.0,
                has_touch: true,
                is_mobile: true,
            },
            DeviceDescriptor {
                name: "OnePlus 11".into(),
                viewport: Viewport { width: 412, height: 915 },
                user_agent: "Mozilla/5.0 (Linux; Android 13; CPH2581) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Mobile Safari/537.36".into(),
                device_scale_factor: 3.0,
                has_touch: true,
                is_mobile: true,
            },
            DeviceDescriptor {
                name: "OnePlus 10T".into(),
                viewport: Viewport { width: 412, height: 915 },
                user_agent: "Mozilla/5.0 (Linux; Android 12; CPH2451) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Mobile Safari/537.36".into(),
                device_scale_factor: 2.625,
                has_touch: true,
                is_mobile: true,
            },
            DeviceDescriptor {
                name: "Google Pixel Fold".into(),
                viewport: Viewport { width: 452, height: 1048 },
                user_agent: "Mozilla/5.0 (Linux; Android 13; Pixel Fold) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Mobile Safari/537.36".into(),
                device_scale_factor: 2.5,
                has_touch: true,
                is_mobile: true,
            },
            DeviceDescriptor {
                name: "Microsoft Surface Duo 2".into(),
                viewport: Viewport { width: 540, height: 720 },
                user_agent: "Mozilla/5.0 (Linux; Android 12; Surface Duo 2) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Mobile Safari/537.36".into(),
                device_scale_factor: 2.0,
                has_touch: true,
                is_mobile: true,
            },
            DeviceDescriptor {
                name: "Nokia Lumia 920".into(),
                viewport: Viewport { width: 320, height: 533 },
                user_agent: "Mozilla/5.0 (compatible; MSIE 10.0; Windows Phone 8.0; Trident/6.0; IEMobile/10.0; ARM; Touch; NOKIA; Lumia 920)".into(),
                device_scale_factor: 1.5,
                has_touch: true,
                is_mobile: true,
            },
            DeviceDescriptor {
                name: "BlackBerry PlayBook".into(),
                viewport: Viewport { width: 600, height: 1024 },
                user_agent: "Mozilla/5.0 (PlayBook; U; RIM Tablet OS 2.1.0; en-US) AppleWebKit/535.8+ (KHTML, like Gecko) Version/7.2.1.0 Safari/535.8+".into(),
                device_scale_factor: 1.0,
                has_touch: true,
                is_mobile: true,
            },
            DeviceDescriptor {
                name: "Desktop Chrome".into(),
                viewport: Viewport { width: 1920, height: 1080 },
                user_agent: "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36".into(),
                device_scale_factor: 1.0,
                has_touch: false,
                is_mobile: false,
            },
            DeviceDescriptor {
                name: "Desktop Firefox".into(),
                viewport: Viewport { width: 1920, height: 1080 },
                user_agent: "Mozilla/5.0 (Windows NT 10.0; Win64; x64; rv:121.0) Gecko/20100101 Firefox/121.0".into(),
                device_scale_factor: 1.0,
                has_touch: false,
                is_mobile: false,
            },
            DeviceDescriptor {
                name: "Desktop Safari".into(),
                viewport: Viewport { width: 1920, height: 1080 },
                user_agent: "Mozilla/5.0 (Macintosh; Intel Mac OS X 14_1) AppleWebKit/605.1.15 (KHTML, like Gecko) Version/17.0 Safari/605.1.15".into(),
                device_scale_factor: 1.0,
                has_touch: false,
                is_mobile: false,
            },
            DeviceDescriptor {
                name: "Desktop Edge".into(),
                viewport: Viewport { width: 1920, height: 1080 },
                user_agent: "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36 Edg/120.0.0.0".into(),
                device_scale_factor: 1.0,
                has_touch: false,
                is_mobile: false,
            },
            DeviceDescriptor {
                name: "iPhone 4".into(),
                viewport: Viewport { width: 320, height: 480 },
                user_agent: "Mozilla/5.0 (iPhone; CPU iPhone OS 7_0 like Mac OS X) AppleWebKit/537.51.1 (KHTML, like Gecko) Version/7.0 Mobile/11A466 Safari/9537.53".into(),
                device_scale_factor: 2.0,
                has_touch: true,
                is_mobile: true,
            },
            DeviceDescriptor {
                name: "iPhone 5".into(),
                viewport: Viewport { width: 320, height: 568 },
                user_agent: "Mozilla/5.0 (iPhone; CPU iPhone OS 7_0 like Mac OS X) AppleWebKit/537.51.1 (KHTML, like Gecko) Version/7.0 Mobile/11A466 Safari/9537.53".into(),
                device_scale_factor: 2.0,
                has_touch: true,
                is_mobile: true,
            },
            DeviceDescriptor {
                name: "iPhone X".into(),
                viewport: Viewport { width: 375, height: 812 },
                user_agent: "Mozilla/5.0 (iPhone; CPU iPhone OS 11_0 like Mac OS X) AppleWebKit/604.1.38 (KHTML, like Gecko) Version/11.0 Mobile/15A372 Safari/604.1".into(),
                device_scale_factor: 3.0,
                has_touch: true,
                is_mobile: true,
            },
            DeviceDescriptor {
                name: "iPhone XR".into(),
                viewport: Viewport { width: 414, height: 896 },
                user_agent: "Mozilla/5.0 (iPhone; CPU iPhone OS 12_0 like Mac OS X) AppleWebKit/605.1.15 (KHTML, like Gecko) Version/12.0 Mobile/15E148 Safari/604.1".into(),
                device_scale_factor: 2.0,
                has_touch: true,
                is_mobile: true,
            },
            DeviceDescriptor {
                name: "iPhone XS Max".into(),
                viewport: Viewport { width: 414, height: 896 },
                user_agent: "Mozilla/5.0 (iPhone; CPU iPhone OS 12_0 like Mac OS X) AppleWebKit/605.1.15 (KHTML, like Gecko) Version/12.0 Mobile/15E148 Safari/604.1".into(),
                device_scale_factor: 3.0,
                has_touch: true,
                is_mobile: true,
            },
            DeviceDescriptor {
                name: "LG G8".into(),
                viewport: Viewport { width: 360, height: 800 },
                user_agent: "Mozilla/5.0 (Linux; Android 10; LM-G850) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Mobile Safari/537.36".into(),
                device_scale_factor: 3.0,
                has_touch: true,
                is_mobile: true,
            },
            DeviceDescriptor {
                name: "Sony Xperia 1".into(),
                viewport: Viewport { width: 360, height: 880 },
                user_agent: "Mozilla/5.0 (Linux; Android 12; SO-51B) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Mobile Safari/537.36".into(),
                device_scale_factor: 3.0,
                has_touch: true,
                is_mobile: true,
            },
            DeviceDescriptor {
                name: "Xiaomi Mi 13".into(),
                viewport: Viewport { width: 393, height: 852 },
                user_agent: "Mozilla/5.0 (Linux; Android 13; 22101316G) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Mobile Safari/537.36".into(),
                device_scale_factor: 3.0,
                has_touch: true,
                is_mobile: true,
            },
            DeviceDescriptor {
                name: "Xiaomi Redmi Note 12".into(),
                viewport: Viewport { width: 393, height: 852 },
                user_agent: "Mozilla/5.0 (Linux; Android 13; Redmi Note 12) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Mobile Safari/537.36".into(),
                device_scale_factor: 2.0,
                has_touch: true,
                is_mobile: true,
            },
            DeviceDescriptor {
                name: "Huawei P50".into(),
                viewport: Viewport { width: 360, height: 780 },
                user_agent: "Mozilla/5.0 (Linux; Android 12; HLA-AL00) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Mobile Safari/537.36".into(),
                device_scale_factor: 3.0,
                has_touch: true,
                is_mobile: true,
            },
            DeviceDescriptor {
                name: "Motorola Moto G Power".into(),
                viewport: Viewport { width: 360, height: 800 },
                user_agent: "Mozilla/5.0 (Linux; Android 13; motorola edge 30) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Mobile Safari/537.36".into(),
                device_scale_factor: 2.0,
                has_touch: true,
                is_mobile: true,
            },
            DeviceDescriptor {
                name: "ASUS ROG Phone 6".into(),
                viewport: Viewport { width: 412, height: 915 },
                user_agent: "Mozilla/5.0 (Linux; Android 12; ASUS_AI2201) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Mobile Safari/537.36".into(),
                device_scale_factor: 2.5,
                has_touch: true,
                is_mobile: true,
            },
            DeviceDescriptor {
                name: "Lenovo Tab P12".into(),
                viewport: Viewport { width: 900, height: 1600 },
                user_agent: "Mozilla/5.0 (Linux; Android 13; Lenovo P12) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36".into(),
                device_scale_factor: 2.0,
                has_touch: true,
                is_mobile: true,
            },
            DeviceDescriptor {
                name: "Amazon Fire HD 10".into(),
                viewport: Viewport { width: 800, height: 1280 },
                user_agent: "Mozilla/5.0 (Linux; Android 9; Fire HD 10 (2019)) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36".into(),
                device_scale_factor: 1.5,
                has_touch: true,
                is_mobile: true,
            },
            DeviceDescriptor {
                name: "Kindle Paperwhite".into(),
                viewport: Viewport { width: 558, height: 774 },
                user_agent: "Mozilla/5.0 (Linux; Android 8.0; K芯Paperwhite) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Mobile Safari/537.36".into(),
                device_scale_factor: 1.5,
                has_touch: true,
                is_mobile: true,
            },
        ]
    }
}

#[napi]
pub struct JsBrowserContext {
    pub browser_id: String,
    pub context_id: String,
}

#[napi]
impl JsBrowserContext {
    #[napi]
    pub async fn new_page(&self) -> Result<JsPage> {
        let context = CONTEXTS.get(&self.context_id).and_then(|w| w.upgrade()).ok_or_else(|| {
            napi::Error::from_reason(format!("Context {} not found", self.context_id))
        })?;

        let page_engine = context.engine.new_page().await?;
        let page_id = uuid::Uuid::new_v4().to_string();

        page_engine.set_page_id(&page_id).await.ok();
        crate::engine::PAGE_CONTEXT.insert(page_id.clone(), self.context_id.clone());

        let page = Arc::new(Page {
            id: page_id.clone(),
            context: Arc::downgrade(&context),
            engine: page_engine,
        });

        PAGES.insert(page_id.clone(), Arc::downgrade(&page));
        context.pages.write().await.push(page);

        Ok(JsPage {
            browser_id: self.browser_id.clone(),
            context_id: self.context_id.clone(),
            page_id,
        })
    }

    #[napi]
    pub async fn close(&self) -> Result<()> {
        tracing::info!("closing context: {}", self.context_id);

        if let Some((_, weak)) = CONTEXTS.remove(&self.context_id) {
            if let Some(context) = weak.upgrade() {
                let pages = context.pages.read().await.clone();
                for page in pages {
                    crate::network::events::unregister_all_for_page(&page.id);
                    PAGES.remove(&page.id);
                    let _ = page.engine.close().await;
                }

                if let Some(browser) = context.browser.upgrade() {
                    let mut contexts = browser.contexts.write().await;
                    contexts.retain(|c| c.id != self.context_id);
                }
                let _ = context.engine.close().await;
            }
        }
        Ok(())
    }

    /// Register a context-level event listener.
    /// Supported event: `"page"` — fires when a popup page is created.
    #[napi]
    pub fn on(&self, event: String, handler: ThreadsafeFunction<String>) -> Result<()> {
        crate::network::events::register_context_listener(&self.context_id, &event, handler);
        Ok(())
    }

    /// Wait for a context-level event to fire.
    /// Supported event: `"page"` — resolves with the popup page_id.
    /// Usage: `const pageId = await context.waitForEvent('page');`
    #[napi]
    pub async fn wait_for_event(&self, event: String, timeout_ms: Option<i32>) -> Result<String> {
        if event != "page" {
            return Err(napi::Error::from_reason(format!(
                "Unsupported event type: '{}'. Only 'page' is supported for context events.",
                event
            )));
        }

        // Lazily create or get the broadcast sender for this context.
        let mut rx = {
            let entry = crate::network::events::CONTEXT_POPUP_TX.get(&self.context_id);
            if let Some(tx) = entry {
                tx.subscribe()
            } else {
                let (tx, rx) = broadcast::channel(64);
                crate::network::events::CONTEXT_POPUP_TX
                    .insert(self.context_id.clone(), tx);
                rx
            }
        };

        let timeout = timeout_ms
            .map(|ms| std::time::Duration::from_millis(ms as u64))
            .unwrap_or(std::time::Duration::from_secs(30));

        tokio::select! {
            result = rx.recv() => {
                result.map_err(|e| napi::Error::from_reason(format!("Event channel error: {}", e)))
            }
            _ = tokio::time::sleep(timeout) => {
                Err(napi::Error::from_reason(format!("waitForEvent('{}') timed out after {}ms", event, timeout_ms.unwrap_or(30000))))
            }
        }
    }

    #[napi]
    pub async fn pages(&self) -> Result<Vec<String>> {
        if let Some(context) = CONTEXTS.get(&self.context_id).and_then(|w| w.upgrade()) {
            let pages = context.pages.read().await;
            Ok(pages.iter().map(|p| p.id.clone()).collect())
        } else {
            Ok(vec![])
        }
    }

    #[napi]
    pub async fn set_geolocation(&self, _geolocation: Geolocation) -> Result<()> {
        Ok(())
    }

    #[napi]
    pub async fn grant_permissions(&self, _permissions: Vec<String>) -> Result<()> {
        Ok(())
    }

    /// Get all cookies for this browser context.
    #[napi]
    pub async fn get_cookies(&self) -> Result<Vec<Cookie>> {
        let context = CONTEXTS.get(&self.context_id).and_then(|w| w.upgrade()).ok_or_else(|| {
            napi::Error::from_reason(format!("Context {} not found", self.context_id))
        })?;
        let cookies = context.engine.get_cookies().await.map_err(|e| {
            napi::Error::from_reason(format!("Failed to get cookies: {}", e))
        })?;
        Ok(cookies)
    }

    /// Get cookies optionally filtered by URL.
    /// Uses CDP Network.getCookies or WebDriver GET /cookie.
    #[napi]
    pub async fn cookies(&self, urls: Option<Vec<String>>) -> Result<Vec<Cookie>> {
        let context = CONTEXTS.get(&self.context_id).and_then(|w| w.upgrade()).ok_or_else(|| {
            napi::Error::from_reason(format!("Context {} not found", self.context_id))
        })?;
        let cookies = context.engine.cookies(urls).await.map_err(|e| {
            napi::Error::from_reason(format!("Failed to get cookies: {}", e))
        })?;
        Ok(cookies)
    }

    /// Set cookies for this browser context.
    #[napi]
    pub async fn set_cookies(&self, cookies: Vec<CookieParam>) -> Result<()> {
        let context = CONTEXTS.get(&self.context_id).and_then(|w| w.upgrade()).ok_or_else(|| {
            napi::Error::from_reason(format!("Context {} not found", self.context_id))
        })?;
        context.engine.set_cookies(cookies).await.map_err(|e| {
            napi::Error::from_reason(format!("Failed to set cookies: {}", e))
        })?;
        Ok(())
    }

    /// Add cookies via the network layer (CDP Network.setCookies or WebDriver POST /cookie).
    /// Unlike set_cookies, this does not require a context to already exist.
    #[napi]
    pub async fn add_cookies(&self, cookies: Vec<CookieParam>) -> Result<()> {
        let context = CONTEXTS.get(&self.context_id).and_then(|w| w.upgrade()).ok_or_else(|| {
            napi::Error::from_reason(format!("Context {} not found", self.context_id))
        })?;
        context.engine.add_cookies(cookies).await.map_err(|e| {
            napi::Error::from_reason(format!("Failed to add cookies: {}", e))
        })?;
        Ok(())
    }

    /// Clear all cookies for this browser context.
    #[napi]
    pub async fn clear_cookies(&self) -> Result<()> {
        let context = CONTEXTS.get(&self.context_id).and_then(|w| w.upgrade()).ok_or_else(|| {
            napi::Error::from_reason(format!("Context {} not found", self.context_id))
        })?;
        context.engine.clear_cookies().await.map_err(|e| {
            napi::Error::from_reason(format!("Failed to clear cookies: {}", e))
        })?;
        Ok(())
    }

    /// Return a `ContextVideoHandle` for configuring context-level video recording options.
    /// The handle allows setting default recording parameters that apply to pages
    /// created from this context.
    /// Video recording requires building with `--features video`.
    #[napi]
    pub fn video(&self) -> ContextVideoHandle {
        ContextVideoHandle {
            context_id: self.context_id.clone(),
            browser_id: self.browser_id.clone(),
        }
    }
}

/// A handle returned by `context.video()` for configuring context-level
/// video recording defaults.
///
/// Usage:
/// ```js
/// const ctx = await browser.newContext();
/// ctx.video().setDefaults(JSON.stringify({ fps: 15, quality: 80 }));
/// const page = await ctx.newPage();
/// // page.video().start() will use these defaults
/// ```
#[allow(dead_code)]
#[napi]
pub struct ContextVideoHandle {
    context_id: String,
    browser_id: String,
}

/// Global store of context-level video config, keyed by context_id.
#[cfg(feature = "video")]
static CONTEXT_VIDEO_CONFIG: std::sync::LazyLock<std::sync::Mutex<std::collections::HashMap<String, String>>> =
    std::sync::LazyLock::new(|| std::sync::Mutex::new(std::collections::HashMap::new()));

/// Retrieve the stored video config JSON for a given context, if any.
#[cfg(feature = "video")]
pub fn get_context_video_config(context_id: &str) -> Option<String> {
    let map = CONTEXT_VIDEO_CONFIG.lock().ok()?;
    map.get(context_id).cloned()
}

#[cfg(feature = "video")]
#[napi]
impl ContextVideoHandle {
    /// Set default video recording config for this context as a JSON string.
    /// Pages created from this context may use these defaults.
    #[napi]
    pub fn set_defaults(&self, config_json: String) {
        if let Ok(mut map) = CONTEXT_VIDEO_CONFIG.lock() {
            map.insert(self.context_id.clone(), config_json);
        }
    }

    /// Get the current default video recording config for this context, or `null` if none set.
    #[napi]
    pub fn get_defaults(&self) -> Option<String> {
        let map = CONTEXT_VIDEO_CONFIG.lock().ok()?;
        map.get(&self.context_id).cloned()
    }

    /// Clear the default video recording config for this context.
    #[napi]
    pub fn clear_defaults(&self) {
        if let Ok(mut map) = CONTEXT_VIDEO_CONFIG.lock() {
            map.remove(&self.context_id);
        }
    }
}

#[napi(object)]
#[derive(Default, Clone)]
pub struct Geolocation {
    pub latitude: f64,
    pub longitude: f64,
    pub accuracy: Option<f64>,
}

#[napi(object)]
#[derive(Clone, Default)]
pub struct Cookie {
    pub name: String,
    pub value: String,
    pub domain: String,
    pub path: String,
    pub expires: f64,
    pub size: i64,
    pub http_only: bool,
    pub secure: bool,
    pub session: bool,
    pub same_site: Option<String>,
    pub priority: String,
}

#[napi(object)]
#[derive(Clone)]
pub struct CookieParam {
    pub name: String,
    pub value: String,
    pub url: Option<String>,
    pub domain: Option<String>,
    pub path: Option<String>,
    pub secure: Option<bool>,
    pub http_only: Option<bool>,
    pub same_site: Option<String>,
    pub expires: Option<i64>,
    pub priority: Option<String>,
}
