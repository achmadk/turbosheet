use napi::bindgen_prelude::*;
use napi_derive::napi;
use std::sync::Arc;

use crate::context::JsBrowserContext;
use crate::error::TurbosheetError;
use crate::engine::{BROWSERS, CONTEXTS, PAGES, Browser, Context};
#[cfg(feature = "chromium")]
use crate::engine::chromium::ChromiumEngine;

#[napi(object)]
#[derive(Default, Clone)]
pub struct LaunchOptions {
    pub r#type: Option<String>,
    pub browser: Option<String>,
    pub headless: Option<bool>,
    pub args: Option<Vec<String>>,
    pub executable_path: Option<String>,
    pub timeout: Option<u32>,
    pub firefox_user_prefs: Option<std::collections::HashMap<String, String>>,
    pub chromium_args: Option<Vec<String>>,
}

#[napi]
pub struct JsBrowser {
    pub id: String,
}

#[napi]
impl JsBrowser {
    pub(crate) async fn launch(options: LaunchOptions) -> Result<Self> {
        let browser_type = options.browser.as_deref().or(options.r#type.as_deref()).unwrap_or("chromium");
        let timeout_ms = options.timeout.unwrap_or(30_000) as u64;

        tracing::info!("launching browser: {browser_type} (timeout: {timeout_ms}ms)");

        let id = uuid::Uuid::new_v4().to_string();

        if browser_type == "chromium" {
            #[cfg(feature = "chromium")]
            {
                let engine = ChromiumEngine::launch(options).await?;
                let browser = Arc::new(Browser { id: id.clone(), engine: Arc::new(engine), contexts: tokio::sync::RwLock::new(Vec::new()) });
                BROWSERS.insert(id.clone(), Arc::clone(&browser));
            }
            #[cfg(not(feature = "chromium"))]
            {
                return Err(TurbosheetError::LaunchFailed(
                    "chromium feature not enabled (compile with --features chromium)".into(),
                ).into());
            }
        } else if browser_type == "firefox" {
            let engine = crate::engine::firefox::FirefoxEngine::launch(options).await?;
            let browser = Arc::new(Browser { id: id.clone(), engine: Arc::new(engine), contexts: tokio::sync::RwLock::new(Vec::new()) });
            BROWSERS.insert(id.clone(), Arc::clone(&browser));
        } else if browser_type == "webkit" {
            let engine = crate::engine::webkit::WebKitEngine::launch(options).await?;
            let browser = Arc::new(Browser { id: id.clone(), engine: Arc::new(engine), contexts: tokio::sync::RwLock::new(Vec::new()) });
            BROWSERS.insert(id.clone(), Arc::clone(&browser));
        } else {
            return Err(TurbosheetError::LaunchFailed(
                format!("Unsupported browser type: {}", browser_type),
            )
            .into());
        }

        Ok(JsBrowser { id })
    }

    #[napi]
    pub async fn new_context(&self, options: Option<crate::context::ContextOptions>) -> Result<JsBrowserContext> {
        let browser_guard = BROWSERS.get(&self.id).ok_or_else(|| {
            napi::Error::from_reason(format!("Browser {} not found", self.id))
        })?;
        let browser = Arc::clone(&browser_guard);
        drop(browser_guard);

        let context_engine = browser.engine.new_context(options).await?;
        let context_id = uuid::Uuid::new_v4().to_string();
        
        let context = Arc::new(Context {
            id: context_id.clone(),
            browser: Arc::downgrade(&browser),
            pages: tokio::sync::RwLock::new(Vec::new()),
            engine: context_engine,
        });

        CONTEXTS.insert(context_id.clone(), Arc::downgrade(&context));
        browser.contexts.write().await.push(context);

        Ok(JsBrowserContext {
            browser_id: self.id.clone(),
            context_id,
        })
    }

    #[napi]
    pub async fn close(&self) -> Result<()> {
        tracing::info!("closing browser: {}", self.id);

        if let Some((_, browser)) = BROWSERS.remove(&self.id) {
            let contexts = browser.contexts.read().await.clone();
            for ctx in contexts {
                let pages = ctx.pages.read().await.clone();
                for page in pages {
                    crate::network::events::unregister_all_for_page(&page.id);
                    PAGES.remove(&page.id);
                    let _ = page.engine.close().await;
                }
                CONTEXTS.remove(&ctx.id);
                let _ = ctx.engine.close().await;
            }
            let _ = browser.engine.close().await;
        }
        Ok(())
    }

    #[napi]
    pub fn version(&self) -> String {
        if let Some(browser_guard) = BROWSERS.get(&self.id) {
            browser_guard.engine.version()
        } else {
            env!("CARGO_PKG_VERSION").to_string()
        }
    }

    #[napi]
    pub async fn contexts(&self) -> Result<Vec<String>> {
        if let Some(browser_guard) = BROWSERS.get(&self.id) {
            let contexts = browser_guard.contexts.read().await;
            Ok(contexts.iter().map(|c| c.id.clone()).collect())
        } else {
            Ok(vec![])
        }
    }
}
