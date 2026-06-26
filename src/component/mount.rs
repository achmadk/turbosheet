use napi::bindgen_prelude::*;
use napi_derive::napi;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use crate::engine::{BROWSERS, PAGES, LoadState};
#[cfg(feature = "chromium")]
use crate::engine::chromium::ChromiumEngine;

use crate::component::pool::ComponentContextPool;

lazy_static::lazy_static! {
    static ref COMPONENT_POOL: ComponentContextPool = ComponentContextPool::new(10);
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum FrameworkType {
    React,
    Vue,
    Svelte,
    Auto,
}

impl Default for FrameworkType {
    fn default() -> Self {
        FrameworkType::Auto
    }
}

impl std::fmt::Display for FrameworkType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            FrameworkType::React => write!(f, "react"),
            FrameworkType::Vue => write!(f, "vue"),
            FrameworkType::Svelte => write!(f, "svelte"),
            FrameworkType::Auto => write!(f, "auto"),
        }
    }
}

#[napi(object)]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComponentMountOptions {
    pub framework: Option<String>,
    pub wrapper: Option<String>,
    pub props: Option<HashMap<String, String>>,
    pub timeout: Option<u32>,
}

impl Default for ComponentMountOptions {
    fn default() -> Self {
        Self {
            framework: None,
            wrapper: None,
            props: None,
            timeout: None,
        }
    }
}

#[napi(object)]
#[derive(Debug, Clone)]
pub struct MountedComponent {
    pub html: String,
    pub framework: String,
    pub component_id: String,
    pub page_id: String,
    pub root_html: String,
}

#[napi]
pub async fn mount_component(
    component: String,
    options: Option<ComponentMountOptions>,
) -> Result<MountedComponent> {
    let opts = options.unwrap_or_default();
    let framework = parse_framework(opts.framework.as_deref());
    let timeout_ms = opts.timeout.unwrap_or(5000) as u64;

    tracing::info!("mounting component with framework: {:?}, timeout: {}ms", framework, timeout_ms);

    let browser_id = "component-browser";
    if !BROWSERS.contains_key(browser_id) {
        let launch_opts = crate::browser::LaunchOptions {
            headless: Some(true),
            args: Some(vec![
                "--no-sandbox".to_string(),
                "--disable-dev-shm-usage".to_string(),
                "--disable-gpu".to_string(),
                format!("--user-data-dir=/tmp/turbosheet-component-{}", uuid::Uuid::new_v4()),
            ]),
            ..Default::default()
        };
        #[cfg(feature = "chromium")]
        {
            let engine = ChromiumEngine::launch(launch_opts).await
                .map_err(|e| napi::Error::from_reason(e.to_string()))?;
            let browser = Arc::new(crate::engine::Browser {
                id: browser_id.to_string(),
                engine: Arc::new(engine),
                contexts: tokio::sync::RwLock::new(Vec::new()),
            });
            BROWSERS.insert(browser_id.to_string(), Arc::clone(&browser));
        }
        #[cfg(not(feature = "chromium"))]
        {
            return Err(napi::Error::from_reason("chromium feature not enabled (compile with --features chromium)".to_string()));
        }
    }

    let framework_str = framework.to_string();
    let page_id_opt = COMPONENT_POOL.acquire(&framework_str).await;
    
    let page_id = if let Some(id) = page_id_opt {
        id
    } else {
        return Err(napi::Error::from_reason("Component pool exhausted".to_string()));
    };

    let page_engine = if let Some(page) = PAGES.get(&page_id).and_then(|w| w.upgrade()) {
        page.engine.clone()
    } else {
        let browser = Arc::clone(&BROWSERS.get(browser_id).unwrap());
        let context_engine = browser.engine.new_context(None).await
            .map_err(|e| napi::Error::from_reason(e.to_string()))?;
        
        let context_id = uuid::Uuid::new_v4().to_string();
        let context = Arc::new(crate::engine::Context {
            id: context_id.clone(),
            browser: Arc::downgrade(&browser),
            pages: tokio::sync::RwLock::new(Vec::new()),
            engine: context_engine.clone(),
        });
        crate::engine::CONTEXTS.insert(context_id.clone(), Arc::downgrade(&context));
        browser.contexts.write().await.push(context.clone());

        let page_engine = context_engine.new_page().await
            .map_err(|e| napi::Error::from_reason(e.to_string()))?;
        
        let page = Arc::new(crate::engine::Page {
            id: page_id.clone(),
            context: Arc::downgrade(&context),
            engine: page_engine.clone(),
        });
        PAGES.insert(page_id.clone(), Arc::downgrade(&page));
        context.pages.write().await.push(page);
        page_engine
    };

    let component_id = uuid::Uuid::new_v4().to_string();
    let html = render_component_html(&component, &framework, &opts);

    // Navigate to a minimal placeholder page first, then use set_content
    // to inject the full HTML. Using set_content avoids CDP session
    // disruption that can happen with evaluate on about:blank, and avoids
    // URI length limits of data: URIs.
    let placeholder = "data:text/html,<html><body></body></html>";
    page_engine.goto(placeholder, LoadState::Load).await
        .map_err(|e| napi::Error::from_reason(format!("Component navigation failed: {}", e)))?;

    page_engine.set_content(&html).await
        .map_err(|e| napi::Error::from_reason(format!("Component set_content failed: {}", e)))?;

    // Wait for framework CDN scripts to load and component to render
    tokio::time::sleep(tokio::time::Duration::from_millis(timeout_ms)).await;
    let root_html = page_engine.content().await.unwrap_or_default();

    Ok(MountedComponent {
        html,
        framework: framework_str,
        component_id,
        page_id: page_id.clone(),
        root_html,
    })
}

#[napi]
pub async fn component_click(page_id: String, selector: String) -> Result<()> {
    let page = PAGES.get(&page_id).and_then(|w| w.upgrade()).ok_or_else(|| {
        napi::Error::from_reason(format!("Page {} not found", page_id))
    })?;

    page.engine.click(&selector).await.map_err(|e| napi::Error::from_reason(e.to_string()))
}

#[napi]
pub async fn component_fill(page_id: String, selector: String, value: String) -> Result<()> {
    let page = PAGES.get(&page_id).and_then(|w| w.upgrade()).ok_or_else(|| {
        napi::Error::from_reason(format!("Page {} not found", page_id))
    })?;

    page.engine.fill(&selector, &value).await.map_err(|e| napi::Error::from_reason(e.to_string()))
}

#[napi]
pub async fn component_evaluate(page_id: String, js: String) -> Result<String> {
    let page = PAGES.get(&page_id).and_then(|w| w.upgrade()).ok_or_else(|| {
        napi::Error::from_reason(format!("Page {} not found", page_id))
    })?;

    page.engine.evaluate(&js).await.map_err(|e| napi::Error::from_reason(e.to_string()))
}

#[napi]
pub async fn component_screenshot(page_id: String) -> Result<Buffer> {
    let page = PAGES.get(&page_id).and_then(|w| w.upgrade()).ok_or_else(|| {
        napi::Error::from_reason(format!("Page {} not found", page_id))
    })?;

    let data = page.engine.screenshot(None).await.map_err(|e| napi::Error::from_reason(e.to_string()))?;
    Ok(Buffer::from(data))
}

#[napi]
pub async fn component_close(page_id: String) -> Result<()> {
    if let Some((_, weak)) = PAGES.remove(&page_id) {
        if let Some(page) = weak.upgrade() {
            page.engine.close().await.map_err(|e| napi::Error::from_reason(e.to_string()))?;
        }
    }
    Ok(())
}

#[napi]
pub async fn component_mount_close(page_id: String, framework: String) -> Result<()> {
    COMPONENT_POOL.release(&framework, &page_id).await;
    Ok(())
}

fn parse_framework(s: Option<&str>) -> FrameworkType {
    match s.map(|x| x.to_lowercase()).as_deref() {
        Some("react") => FrameworkType::React,
        Some("vue") => FrameworkType::Vue,
        Some("svelte") => FrameworkType::Svelte,
        _ => FrameworkType::Auto,
    }
}

fn render_component_html(component: &str, framework: &FrameworkType, opts: &ComponentMountOptions) -> String {
    match framework {
        FrameworkType::React => render_react_html(component, opts),
        FrameworkType::Vue => render_vue_html(component, opts),
        FrameworkType::Svelte => render_svelte_html(component, opts),
        FrameworkType::Auto => {
            if component.contains("React") || component.contains("jsx") || component.contains("<") {
                render_react_html(component, opts)
            } else if component.contains("<template") {
                render_vue_html(component, opts)
            } else {
                render_react_html(component, opts)
            }
        }
    }
}

fn render_react_html(component: &str, opts: &ComponentMountOptions) -> String {
    let escaped = component.replace('`', "\\`").replace("$", "\\$");
    let wrapper = opts.wrapper.as_ref().map(|w| w.as_str()).unwrap_or("div");

    let mut html = String::new();
    html.push_str("<!DOCTYPE html>\n");
    html.push_str("<html>\n<head>\n");
    html.push_str("  <meta charset=\"UTF-8\">\n");
    html.push_str("  <script src=\"https://unpkg.com/react@18/umd/react.production.min.js\" crossorigin></script>\n");
    html.push_str("  <script src=\"https://unpkg.com/react-dom@18/umd/react-dom.production.min.js\" crossorigin></script>\n");
    html.push_str("  <script src=\"https://unpkg.com/@babel/standalone/babel.min.js\"></script>\n");
    html.push_str("  <style>\n");
    html.push_str("    * { box-sizing: border-box; }\n");
    html.push_str("    body { margin: 0; font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', sans-serif; }\n");
    html.push_str("  </style>\n");
    html.push_str("</head>\n<body>\n");
    html.push_str("  <div id=\"root\"></div>\n");
    html.push_str("  <script type=\"text/babel\" data-presets=\"react\">\n");

    let script = format!(
        "const Wrapper = (props) => <{wrapper} {{...props}} />;\n\
         const Component = () => {{\n\
           return <Wrapper>{{{escaped}}}</Wrapper>;\n\
         }};\n\
         try {{\n\
           const root = ReactDOM.createRoot(document.getElementById('root'));\n\
           root.render(<Component />);\n\
         }} catch (e) {{\n\
           document.getElementById('root').innerHTML = '<div style=\"color: #dc2626; padding: 20px; background: #fef2f2; border: 1px solid #fecaca; border-radius: 8px; margin: 20px;\"><strong>Error:</strong> ' + e.message + '</div>';\n\
         }}",
        wrapper = wrapper,
        escaped = escaped
    );
    html.push_str(&script);
    html.push_str("\n  </script>\n");
    html.push_str("</body>\n</html>");

    html
}

fn render_vue_html(component: &str, _opts: &ComponentMountOptions) -> String {
    let escaped = component.replace('`', "\\`").replace("$", "\\$");

    let mut html = String::new();
    html.push_str("<!DOCTYPE html>\n");
    html.push_str("<html>\n<head>\n");
    html.push_str("  <meta charset=\"UTF-8\">\n");
    html.push_str("  <script src=\"https://unpkg.com/vue@3/dist/vue.global.prod.js\"></script>\n");
    html.push_str("  <style>\n");
    html.push_str("    * { box-sizing: border-box; }\n");
    html.push_str("    body { margin: 0; font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', sans-serif; }\n");
    html.push_str("  </style>\n");
    html.push_str("</head>\n<body>\n");
    html.push_str("  <div id=\"app\"></div>\n");
    html.push_str("  <script>\n");
    html.push_str("    const app = Vue.createApp({\n");
    html.push_str("      template: `\n");
    html.push_str(&escaped);
    html.push_str("`\n");
    html.push_str("    });\n");
    html.push_str("    app.mount('#app');\n");
    html.push_str("  </script>\n");
    html.push_str("</body>\n</html>");

    html
}

fn render_svelte_html(component: &str, _opts: &ComponentMountOptions) -> String {
    let escaped = component.replace('`', "\\`").replace("$", "\\$");

    let mut html = String::new();
    html.push_str("<!DOCTYPE html>\n");
    html.push_str("<html>\n<head>\n");
    html.push_str("  <meta charset=\"UTF-8\">\n");
    html.push_str("  <script src=\"https://unpkg.com/svelte@3.59.2/compiler.js\"></script>\n");
    html.push_str("  <style>\n");
    html.push_str("    * { box-sizing: border-box; }\n");
    html.push_str("    body { margin: 0; font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', sans-serif; }\n");
    html.push_str("  </style>\n");
    html.push_str("</head>\n<body>\n");
    html.push_str("  <div id=\"root\"></div>\n");
    html.push_str("  <script type=\"module\">\n");
    
    let script = format!(
        "try {{\n\
           const {{ js }} = svelte.compile(`{escaped}`, {{ format: 'esm' }});\n\
           const code = js.code.replace(/svelte\\/internal/g, 'https://unpkg.com/svelte@3.59.2/internal/index.mjs');\n\
           const blob = new Blob([code], {{ type: 'text/javascript' }});\n\
           const url = URL.createObjectURL(blob);\n\
           import(url).then(mod => {{\n\
             window.__svelte_app = new mod.default({{ target: document.getElementById('root') }});\n\
           }});\n\
         }} catch (e) {{\n\
           document.getElementById('root').innerHTML = '<div style=\"color: #dc2626; padding: 20px; background: #fef2f2; border: 1px solid #fecaca; border-radius: 8px; margin: 20px;\"><strong>Svelte Error:</strong> ' + e.message + '</div>';\n\
         }}",
        escaped = escaped
    );
    html.push_str(&script);
    html.push_str("\n  </script>\n");
    html.push_str("</body>\n</html>");

    html
}

#[napi]
pub fn detect_framework_from_package_json(package_json: String) -> String {
    if package_json.contains("\"react\"") {
        "react".to_string()
    } else if package_json.contains("\"vue\"") {
        "vue".to_string()
    } else if package_json.contains("\"svelte\"") {
        "svelte".to_string()
    } else {
        "unknown".to_string()
    }
}

/// Detect framework from test file imports by scanning for import statements.
/// Supports: `from 'react'`, `from "react"`, `from 'vue'`, `from "vue"`, etc.
#[napi]
pub fn detect_framework_from_source(source: String) -> String {
    if source.contains("from 'react'") || source.contains("from \"react\"")
        || source.contains("require('react')") || source.contains("require(\"react\")")
        || source.contains("from 'react-dom'") || source.contains("from \"react-dom\"")
    {
        "react".to_string()
    } else if source.contains("from 'vue'") || source.contains("from \"vue\"")
        || source.contains("require('vue')") || source.contains("require(\"vue\")")
    {
        "vue".to_string()
    } else if source.contains("from 'svelte'") || source.contains("from \"svelte\"")
        || source.contains("require('svelte')") || source.contains("require(\"svelte\")")
    {
        "svelte".to_string()
    } else {
        "unknown".to_string()
    }
}

/// Get component's rendered HTML content after mount
#[napi]
pub async fn component_get_content(page_id: String) -> Result<String> {
    let page = PAGES.get(&page_id).and_then(|w| w.upgrade()).ok_or_else(|| {
        napi::Error::from_reason(format!("Page {} not found", page_id))
    })?;
    page.engine.content().await.map_err(|e| napi::Error::from_reason(e.to_string()))
}

/// Get computed styles for a selector within a component
#[napi]
pub async fn component_get_computed_style(page_id: String, selector: String, property: String) -> Result<String> {
    let page = PAGES.get(&page_id).and_then(|w| w.upgrade()).ok_or_else(|| {
        napi::Error::from_reason(format!("Page {} not found", page_id))
    })?;

    let escaped_sel = serde_json::to_string(&selector).unwrap_or_else(|_| format!("\"{}\"", selector));
    let escaped_prop = serde_json::to_string(&property).unwrap_or_else(|_| format!("\"{}\"", property));
    let js = format!(
        r#"(() => {{
            const el = document.querySelector({});
            if (!el) return '';
            return window.getComputedStyle(el).getPropertyValue({});
        }})()"#,
        escaped_sel, escaped_prop
    );
    page.engine.evaluate(&js).await.map_err(|e| napi::Error::from_reason(e.to_string()))
}

/// Wait for a condition within a component
#[napi]
pub async fn component_wait_for(
    page_id: String,
    js_condition: String,
    timeout_ms: Option<u32>,
) -> Result<()> {
    let page = PAGES.get(&page_id).and_then(|w| w.upgrade()).ok_or_else(|| {
        napi::Error::from_reason(format!("Page {} not found", page_id))
    })?;

    let timeout = timeout_ms.unwrap_or(5000) as u64;
    let engine = crate::assertions::engine::AssertionEngine::default();
    let timeout_dur = std::time::Duration::from_millis(timeout);

    engine.poll_with_timeout(timeout_dur, || async {
        match page.engine.evaluate(&js_condition).await {
            Ok(res) => {
                if res == "true" || res == "1" {
                    Ok(())
                } else {
                    Err("Condition not met".to_string())
                }
            }
            Err(e) => Err(e.to_string()),
        }
    }).await.map_err(|e| napi::Error::from_reason(e.to_string()))
}
