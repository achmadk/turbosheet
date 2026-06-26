use crate::browser::LaunchOptions;
use crate::context::ContextOptions;
use crate::engine::{BrowserEngine, ContextEngine, PageEngine};
use crate::error::TurbosheetError;
use crate::page::ScreenshotOptions;
use async_trait::async_trait;
use std::collections::HashMap;
use std::process::Stdio;
use std::sync::Arc;
use tokio::process::{Child, Command};
use crate::engine::webdriver::WebDriverClient;

pub struct WebKitEngine {
    process: Child,
    client: Arc<tokio::sync::Mutex<WebDriverClient>>,
}

impl WebKitEngine {
    pub async fn launch(options: LaunchOptions) -> Result<Self, TurbosheetError> {
        let os = std::env::consts::OS;
        let executable = options.executable_path.unwrap_or_else(|| {
            if os == "macos" {
                "/usr/bin/safaridriver".to_string()
            } else {
                "/usr/bin/WebKitWebDriver".to_string()
            }
        });

        let listener = std::net::TcpListener::bind("127.0.0.1:0").map_err(|e| {
            TurbosheetError::LaunchFailed(format!("Failed to bind port: {}", e))
        })?;
        let port = listener.local_addr().map_err(|e| {
            TurbosheetError::LaunchFailed(format!("Failed to get port: {}", e))
        })?.port();
        drop(listener);
        let mut cmd = Command::new(&executable);
        cmd.arg("--port").arg(port.to_string());
        cmd.stdout(Stdio::piped());
        cmd.stderr(Stdio::piped());

        let process = cmd.spawn().map_err(|e| {
            TurbosheetError::LaunchFailed(format!("Failed to spawn WebKit driver: {}", e))
        })?;

        tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;

        let mut client = WebDriverClient::new(format!("http://127.0.0.1:{}", port));
        let capabilities = serde_json::json!({
            "capabilities": {
                "alwaysMatch": {
                    "browserName": "safari"
                }
            }
        });
        client.create_session(capabilities).await?;

        Ok(Self { process, client: Arc::new(tokio::sync::Mutex::new(client)) })
    }
}

#[async_trait]
impl BrowserEngine for WebKitEngine {

    async fn close(&self) -> Result<(), TurbosheetError> {
        let mut client = self.client.lock().await;
        client.delete_session().await?;
        Ok(())
    }

    fn version(&self) -> String {
        "WebKit (WebDriver)".to_string()
    }

    async fn new_context(
        &self,
        _options: Option<ContextOptions>,
    ) -> Result<Arc<dyn ContextEngine>, TurbosheetError> {
        Ok(Arc::new(WebKitContextEngine { client: self.client.clone(), pages: Arc::new(tokio::sync::RwLock::new(Vec::new())) }))
    }
}

pub struct WebKitContextEngine {
    client: Arc<tokio::sync::Mutex<WebDriverClient>>,
    pages: Arc<tokio::sync::RwLock<Vec<Arc<dyn PageEngine>>>>,
}

#[async_trait]
impl ContextEngine for WebKitContextEngine {
    async fn new_page(&self) -> Result<Arc<dyn PageEngine>, TurbosheetError> {
        let stealth_cfg = crate::injection::stealth::StealthConfig::default();
        let (global_name, binding_name) = stealth_cfg.generate_names();
        let pages = self.pages.clone();
        let engine = Arc::new(WebKitPageEngine { 
            client: self.client.clone(),
            global_name,
            binding_name,
            stealth_config: stealth_cfg.clone(),
            core_injected: std::sync::atomic::AtomicBool::new(false),
            current_url: tokio::sync::RwLock::new("about:blank".to_string()),
        });
        pages.write().await.push(engine.clone());
        // Inject core script on page creation
        let core_script = stealth_cfg.process_script(
            crate::injection::scripts::CORE_SCRIPT,
            &engine.global_name,
            &engine.binding_name,
        );
        engine.inject_core_script(&core_script).await?;
        Ok(engine)
    }

    async fn pages(&self) -> Result<Vec<Arc<dyn PageEngine>>, TurbosheetError> {
        let pages = self.pages.read().await;
        Ok(pages.clone())
    }

    async fn get_cookies(&self) -> Result<Vec<crate::context::Cookie>, TurbosheetError> {
        self.cookies(None).await
    }

    async fn cookies(&self, urls: Option<Vec<String>>) -> Result<Vec<crate::context::Cookie>, TurbosheetError> {
        let client = self.client.lock().await;
        let resp = client.execute_command(reqwest::Method::GET, "/cookie", None).await?;
        let mut cookies: Vec<crate::context::Cookie> = resp
            .get("value")
            .and_then(|v| v.as_array())
            .map(|arr| {
                arr.iter().map(|c| {
                    crate::context::Cookie {
                        name: c.get("name").and_then(|v| v.as_str()).unwrap_or("").to_string(),
                        value: c.get("value").and_then(|v| v.as_str()).unwrap_or("").to_string(),
                        domain: c.get("domain").and_then(|v| v.as_str()).unwrap_or("").to_string(),
                        path: c.get("path").and_then(|v| v.as_str()).unwrap_or("/").to_string(),
                        expires: c.get("expiry").and_then(|v| v.as_f64()).unwrap_or(-1.0),
                        size: 0,
                        http_only: c.get("httpOnly").and_then(|v| v.as_bool()).unwrap_or(false),
                        secure: c.get("secure").and_then(|v| v.as_bool()).unwrap_or(false),
                        session: c.get("session").and_then(|v| v.as_bool()).unwrap_or(true),
                        same_site: c.get("sameSite").and_then(|v| v.as_str()).map(|s| s.to_string()),
                        priority: "Medium".to_string(),
                    }
                }).collect()
            })
            .unwrap_or_default();

        // Post-filter by URL if urls is provided
        if let Some(url_list) = urls {
            cookies.retain(|cookie| {
                url_list.iter().any(|url| cookie_matches_url(cookie, url))
            });
        }

        Ok(cookies)
    }

    async fn set_cookies(&self, cookies: Vec<crate::context::CookieParam>) -> Result<(), TurbosheetError> {
        let client = self.client.lock().await;
        for cookie in cookies {
            let mut body = serde_json::json!({
                "cookie": {
                    "name": cookie.name,
                    "value": cookie.value,
                }
            });
            if let Some(ref domain) = cookie.domain {
                body["cookie"]["domain"] = serde_json::json!(domain);
            }
            if let Some(ref path) = cookie.path {
                body["cookie"]["path"] = serde_json::json!(path);
            }
            if let Some(secure) = cookie.secure {
                body["cookie"]["secure"] = serde_json::json!(secure);
            }
            if let Some(http_only) = cookie.http_only {
                body["cookie"]["httpOnly"] = serde_json::json!(http_only);
            }
            if let Some(ref same_site) = cookie.same_site {
                body["cookie"]["sameSite"] = serde_json::json!(same_site);
            }
            client.execute_command(reqwest::Method::POST, "/cookie", Some(body)).await?;
        }
        Ok(())
    }

    async fn add_cookies(&self, cookies: Vec<crate::context::CookieParam>) -> Result<(), TurbosheetError> {
        // WebDriver add/set are the same — both POST /cookie
        self.set_cookies(cookies).await
    }

    async fn clear_cookies(&self) -> Result<(), TurbosheetError> {
        let client = self.client.lock().await;
        client.execute_command(reqwest::Method::DELETE, "/cookie", None).await?;
        Ok(())
    }

    async fn close(&self) -> Result<(), TurbosheetError> {
        Ok(())
    }
}

pub struct WebKitPageEngine {
    client: Arc<tokio::sync::Mutex<WebDriverClient>>,
    global_name: String,
    binding_name: String,
    stealth_config: crate::injection::stealth::StealthConfig,
    core_injected: std::sync::atomic::AtomicBool,
    current_url: tokio::sync::RwLock<String>,
}

impl WebKitPageEngine {
    async fn find_element(&self, selector: &str) -> Result<String, TurbosheetError> {
        let client = self.client.lock().await;
        // Basic CSS selector mapping
        let body = serde_json::json!({
            "using": "css selector",
            "value": selector
        });
        let res = client.execute_command(reqwest::Method::POST, "/element", Some(body)).await?;
        
        // W3C WebDriver element reference key
        for key in res.as_object().unwrap_or(&serde_json::Map::new()).keys() {
            if key.starts_with("element-6066-11e4-a52e-4f735466cecf") || key.contains("element") {
                return Ok(res[key].as_str().unwrap_or("").to_string());
            }
        }
        Err(TurbosheetError::ElementNotFound {
            message: format!("Element not found: {}", selector),
            selector: Some(selector.to_string()),
        })
    }

    async fn _refresh_url(&self) -> Result<String, TurbosheetError> {
        let client = self.client.lock().await;
        let body = serde_json::json!({
            "script": "window.location.href",
            "args": []
        });
        let res = client.execute_command(reqwest::Method::POST, "/execute/sync", Some(body)).await?;
        Ok(res.as_str().unwrap_or("about:blank").to_string())
    }

    /// Fallback network waiter: poll performance entries when BiDi is unavailable.
    async fn _wait_for_network(&self, kind: &str, url: &str) -> Result<(), TurbosheetError> {
        let escaped_url = serde_json::to_string(&url).unwrap_or_default();
        let js = match kind {
            "request" => format!(
                r#"(() => performance.getEntriesByType('resource')
                    .filter(e => e.name.includes({}))[0]?.name || '')()"#,
                escaped_url
            ),
            _ => format!(
                r#"(() => performance.getEntriesByType('resource')
                    .filter(e => e.name.includes({}) && e.responseEnd > 0)[0]?.responseEnd ? 'found' : '')()"#,
                escaped_url
            ),
        };
        let start = std::time::Instant::now();
        let timeout = std::time::Duration::from_secs(30);
        loop {
            if start.elapsed() > timeout {
                return Err(TurbosheetError::Other(format!(
                    "Timeout waiting for network {}: {}",
                    kind, url
                )));
            }
            let res = self.evaluate(&js).await?;
            if !res.is_empty() && res != "\"\"" && res != "\"\n\"" {
                return Ok(());
            }
            tokio::time::sleep(std::time::Duration::from_millis(200)).await;
        }
    }
}

#[async_trait]
impl PageEngine for WebKitPageEngine {
    async fn invoke_action(&self, action: &str, args: Vec<serde_json::Value>) -> Result<serde_json::Value, TurbosheetError> {
        let mut js_args = String::new();
        for arg in args {
            js_args.push_str(&arg.to_string());
            js_args.push_str(", ");
        }
        if js_args.len() >= 2 {
            js_args.truncate(js_args.len() - 2);
        }
        
        let script = self.stealth_config.process_script(
            crate::injection::scripts::ACTIONS_SCRIPT,
            &self.global_name,
            &self.binding_name,
        );
        let accesses_global = self.stealth_config.global_expr(&self.global_name);
        let accesses_content = format!("{}.content", accesses_global);

        let js = format!("
            let callback = arguments[arguments.length - 1];
            (async function() {{
                if (!{} || !{}) {{
                    {}
                }}
                try {{
                    let res = await {}.{}({});
                    callback(JSON.stringify({{ok: true, data: res}}));
                }} catch (e) {{
                    callback(JSON.stringify({{ok: false, error: e.message}}));
                }}
            }})()
        ", accesses_global, accesses_content, script, accesses_global, action, js_args);
        
        let client = self.client.lock().await;
        let body = serde_json::json!({
            "script": js,
            "args": []
        });
        let res = client.execute_command(reqwest::Method::POST, "/execute/async", Some(body)).await?;
        
        if res.get("ok").and_then(|v| v.as_bool()) == Some(true) {
            Ok(res.get("data").cloned().unwrap_or(serde_json::Value::Null))
        } else {
            let err = res.get("error").and_then(|v| v.as_str()).unwrap_or("Unknown error");
            Err(TurbosheetError::Other(err.to_string()))
        }
    }
    
    async fn goto(&self, url: &str, _state: crate::engine::LoadState) -> Result<(), TurbosheetError> {
        let client = self.client.lock().await;
        client.execute_command(reqwest::Method::POST, "/url", Some(serde_json::json!({"url": url}))).await?;
        *self.current_url.write().await = url.to_string();
        Ok(())
    }
    
    fn url(&self) -> String {
        self.current_url.try_read().map(|u| u.clone()).unwrap_or_else(|_| "about:blank".to_string())
    }
    
    async fn screenshot(&self, _options: Option<ScreenshotOptions>) -> Result<Vec<u8>, TurbosheetError> {
        let client = self.client.lock().await;
        let res = client.execute_command(reqwest::Method::GET, "/screenshot", None).await?;
        let base64_str = res.as_str().unwrap_or("");
        use base64::{Engine as _, engine::general_purpose::STANDARD};
        STANDARD.decode(base64_str).map_err(|e| TurbosheetError::Other(format!("Screenshot decode error: {}", e)))
    }
    
    async fn close(&self) -> Result<(), TurbosheetError> {
        let client = self.client.lock().await;
        let _ = client.execute_command(reqwest::Method::DELETE, "/window", None).await;
        Ok(())
    }
    
    async fn evaluate(&self, js: &str) -> Result<String, TurbosheetError> {
        let client = self.client.lock().await;
        let body = serde_json::json!({
            "script": js,
            "args": []
        });
        let res = client.execute_command(reqwest::Method::POST, "/execute/sync", Some(body)).await?;
        Ok(res.to_string())
    }
    
    async fn content(&self) -> Result<String, TurbosheetError> {
        let res = self.invoke_action("content", vec![]).await?;
        Ok(res.as_str().unwrap_or_default().to_string())
    }
    
    async fn title(&self) -> Result<String, TurbosheetError> {
        let res = self.invoke_action("title", vec![]).await?;
        Ok(res.as_str().unwrap_or_default().to_string())
    }
    
    async fn click(&self, selector: &str) -> Result<(), TurbosheetError> {
        let el_id = self.find_element(selector).await?;
        let client = self.client.lock().await;
        client.execute_command(reqwest::Method::POST, &format!("/element/{}/click", el_id), Some(serde_json::json!({}))).await?;
        Ok(())
    }
    
    async fn dblclick(&self, selector: &str) -> Result<(), TurbosheetError> {
        self.invoke_action("dblclick", vec![serde_json::Value::String(selector.to_string())]).await?;
        Ok(())
    }
    
    async fn right_click(&self, selector: &str) -> Result<(), TurbosheetError> {
        self.invoke_action("rightClick", vec![serde_json::Value::String(selector.to_string())]).await?;
        Ok(())
    }
    
    async fn hover(&self, selector: &str) -> Result<(), TurbosheetError> {
        self.invoke_action("hover", vec![serde_json::Value::String(selector.to_string())]).await?;
        Ok(())
    }
    
    async fn fill(&self, selector: &str, value: &str) -> Result<(), TurbosheetError> {
        let el_id = self.find_element(selector).await?;
        let client = self.client.lock().await;
        client.execute_command(reqwest::Method::POST, &format!("/element/{}/clear", el_id), Some(serde_json::json!({}))).await?;
        client.execute_command(reqwest::Method::POST, &format!("/element/{}/value", el_id), Some(serde_json::json!({"text": value}))).await?;
        Ok(())
    }
    
    async fn check(&self, selector: &str) -> Result<(), TurbosheetError> {
        self.invoke_action("check", vec![serde_json::Value::String(selector.to_string())]).await?;
        Ok(())
    }
    
    async fn uncheck(&self, selector: &str) -> Result<(), TurbosheetError> {
        self.invoke_action("uncheck", vec![serde_json::Value::String(selector.to_string())]).await?;
        Ok(())
    }
    
    async fn select(&self, selector: &str, value: &str) -> Result<(), TurbosheetError> {
        self.invoke_action("select", vec![serde_json::Value::String(selector.to_string()), serde_json::Value::String(value.to_string())]).await?;
        Ok(())
    }
    
    async fn focus(&self, selector: &str) -> Result<(), TurbosheetError> {
        self.invoke_action("focus", vec![serde_json::Value::String(selector.to_string())]).await?;
        Ok(())
    }
    
    async fn blur(&self, selector: &str) -> Result<(), TurbosheetError> {
        self.invoke_action("blur", vec![serde_json::Value::String(selector.to_string())]).await?;
        Ok(())
    }
    
    async fn scroll_into_view(&self, selector: &str) -> Result<(), TurbosheetError> {
        self.invoke_action("scrollIntoView", vec![serde_json::Value::String(selector.to_string())]).await?;
        Ok(())
    }
    
    async fn text_content(&self, selector: &str) -> Result<Option<String>, TurbosheetError> {
        let el_id = self.find_element(selector).await?;
        let client = self.client.lock().await;
        let res = client.execute_command(reqwest::Method::GET, &format!("/element/{}/text", el_id), None).await?;
        Ok(res.as_str().map(|s| s.to_string()))
    }
    
    async fn inner_text(&self, selector: &str) -> Result<String, TurbosheetError> {
        self.text_content(selector).await.map(|o| o.unwrap_or_default())
    }
    
    async fn inner_html(&self, selector: &str) -> Result<String, TurbosheetError> {
        let res = self.invoke_action("innerHTML", vec![serde_json::Value::String(selector.to_string())]).await?;
        Ok(res.as_str().unwrap_or("").to_string())
    }
    
    async fn get_attribute(&self, selector: &str, name: &str) -> Result<Option<String>, TurbosheetError> {
        let el_id = self.find_element(selector).await?;
        let client = self.client.lock().await;
        let res = client.execute_command(reqwest::Method::GET, &format!("/element/{}/attribute/{}", el_id, name), None).await?;
        if res.is_null() {
            Ok(None)
        } else {
            Ok(res.as_str().map(|s| s.to_string()))
        }
    }
    
    async fn is_visible(&self, selector: &str) -> Result<bool, TurbosheetError> {
        let res = self.invoke_action("isVisible", vec![serde_json::Value::String(selector.to_string())]).await?;
        Ok(res.as_bool().unwrap_or(false))
    }
    
    async fn is_enabled(&self, selector: &str) -> Result<bool, TurbosheetError> {
        let res = self.invoke_action("isEnabled", vec![serde_json::Value::String(selector.to_string())]).await?;
        Ok(res.as_bool().unwrap_or(false))
    }
    
    async fn is_disabled(&self, selector: &str) -> Result<bool, TurbosheetError> {
        let res = self.invoke_action("isDisabled", vec![serde_json::Value::String(selector.to_string())]).await?;
        Ok(res.as_bool().unwrap_or(false))
    }
    
    async fn locator_screenshot(&self, selector: &str, _options: Option<ScreenshotOptions>) -> Result<Vec<u8>, TurbosheetError> {
        let el_id = self.find_element(selector).await?;
        let client = self.client.lock().await;
        let res = client.execute_command(reqwest::Method::GET, &format!("/element/{}/screenshot", el_id), None).await?;
        let base64_str = res.as_str().unwrap_or("");
        use base64::{Engine as _, engine::general_purpose::STANDARD};
        STANDARD.decode(base64_str).map_err(|e| TurbosheetError::Other(format!("Screenshot decode error: {}", e)))
    }
    
    async fn bounding_box(&self, selector: &str) -> Result<Option<crate::page::Rect>, TurbosheetError> {
        let el_id = self.find_element(selector).await?;
        let client = self.client.lock().await;
        let res = client.execute_command(reqwest::Method::GET, &format!("/element/{}/rect", el_id), None).await?;
        if let Some(rect) = res.as_object() {
            Ok(Some(crate::page::Rect {
                x: rect.get("x").and_then(|v| v.as_f64()).unwrap_or(0.0),
                y: rect.get("y").and_then(|v| v.as_f64()).unwrap_or(0.0),
                width: rect.get("width").and_then(|v| v.as_f64()).unwrap_or(0.0),
                height: rect.get("height").and_then(|v| v.as_f64()).unwrap_or(0.0),
            }))
        } else {
            Ok(None)
        }
    }
    
    async fn drag_and_drop(&self, source: &str, target: &str) -> Result<(), TurbosheetError> {
        self.invoke_action("dragAndDrop", vec![
            serde_json::Value::String(source.to_string()),
            serde_json::Value::String(target.to_string()),
        ]).await?;
        Ok(())
    }
    
    async fn press(&self, selector: &str, key: &str) -> Result<(), TurbosheetError> {
        let el_id = self.find_element(selector).await?;
        let client = self.client.lock().await;
        client.execute_command(reqwest::Method::POST, &format!("/element/{}/value", el_id), Some(serde_json::json!({"text": key}))).await?;
        Ok(())
    }
    
    async fn press_sequentially(&self, selector: &str, text: &str) -> Result<(), TurbosheetError> {
        let el_id = self.find_element(selector).await?;
        let client = self.client.lock().await;
        client.execute_command(reqwest::Method::POST, &format!("/element/{}/value", el_id), Some(serde_json::json!({"text": text}))).await?;
        Ok(())
    }
    
    async fn set_input_files(&self, selector: &str, files: Vec<String>) -> Result<(), TurbosheetError> {
        let escaped_sel = serde_json::to_string(&selector).unwrap_or_else(|_| format!("\"{}\"", selector));
        let files_json = serde_json::to_string(&files).unwrap_or_default();
        let js = format!(
            r#"(function() {{
                const el = document.querySelector({});
                if (!el) throw new Error('Element not found');
                const dt = new DataTransfer();
                const filePaths = {};
                for (const path of filePaths) {{
                    const name = path.split('/').pop() || 'file';
                    const file = new File([new Blob([''])], name, {{ type: 'application/octet-stream' }});
                    dt.items.add(file);
                }}
                el.files = dt.files;
                el.dispatchEvent(new Event('change', {{ bubbles: true }}));
                return true;
            }})()"#,
            escaped_sel, files_json
        );
        let _ = self.evaluate(&js).await?;
        Ok(())
    }
    
    async fn set_viewport_size(&self, width: u32, height: u32) -> Result<(), TurbosheetError> {
        let client = self.client.lock().await;
        client.execute_command(reqwest::Method::POST, "/window/rect", Some(serde_json::json!({"width": width, "height": height}))).await?;
        Ok(())
    }
    
    async fn viewport_size(&self) -> Result<Option<crate::page::ViewportSize>, TurbosheetError> {
        let client = self.client.lock().await;
        let res = client.execute_command(reqwest::Method::GET, "/window/rect", None).await?;
        if let Some(rect) = res.as_object() {
            Ok(Some(crate::page::ViewportSize {
                width: rect.get("width").and_then(|v| v.as_u64()).unwrap_or(0) as u32,
                height: rect.get("height").and_then(|v| v.as_u64()).unwrap_or(0) as u32,
            }))
        } else {
            Ok(None)
        }
    }
    
    async fn reload(&self) -> Result<(), TurbosheetError> {
        let client = self.client.lock().await;
        client.execute_command(reqwest::Method::POST, "/refresh", Some(serde_json::json!({}))).await?;
        if let Ok(url) = self._refresh_url().await {
            *self.current_url.write().await = url;
        }
        Ok(())
    }
    
    async fn go_back(&self) -> Result<(), TurbosheetError> {
        let client = self.client.lock().await;
        client.execute_command(reqwest::Method::POST, "/back", Some(serde_json::json!({}))).await?;
        if let Ok(url) = self._refresh_url().await {
            *self.current_url.write().await = url;
        }
        Ok(())
    }
    
    async fn go_forward(&self) -> Result<(), TurbosheetError> {
        let client = self.client.lock().await;
        client.execute_command(reqwest::Method::POST, "/forward", Some(serde_json::json!({}))).await?;
        if let Ok(url) = self._refresh_url().await {
            *self.current_url.write().await = url;
        }
        Ok(())
    }
    
    async fn wait_for_request(&self, url: &str) -> Result<(), TurbosheetError> {
        self._wait_for_network("request", url).await
    }
    async fn wait_for_response(&self, url: &str) -> Result<(), TurbosheetError> {
        self._wait_for_network("response", url).await
    }
    
    async fn wait_for_selector(&self, selector: &str) -> Result<(), TurbosheetError> {
        let start = std::time::Instant::now();
        let timeout = std::time::Duration::from_secs(30);
        loop {
            if start.elapsed() > timeout {
                return Err(TurbosheetError::Other(format!("Timeout waiting for {}", selector)));
            }
            if self.find_element(selector).await.is_ok() {
                return Ok(());
            }
            tokio::time::sleep(std::time::Duration::from_millis(100)).await;
        }
    }
    
    async fn evaluate_handle(&self, js: &str) -> Result<(), TurbosheetError> {
        self.evaluate(js).await?;
        Ok(())
    }
    async fn add_script_tag(&self, content: &str) -> Result<(), TurbosheetError> {
        self.invoke_action("addScriptTag", vec![serde_json::Value::String(content.to_string())]).await?;
        Ok(())
    }
    async fn add_style_tag(&self, content: &str) -> Result<(), TurbosheetError> {
        self.invoke_action("addStyleTag", vec![serde_json::Value::String(content.to_string())]).await?;
        Ok(())
    }
    async fn expose_function(&self, name: &str, js_body: &str) -> Result<(), TurbosheetError> {
        let escaped_body = serde_json::to_string(js_body).unwrap_or_default();
        let js = format!(r#"window['{}'] = function(...args) {{ return eval({}) }}"#, name, escaped_body);
        self.evaluate(&js).await?;
        Ok(())
    }
    async fn set_content(&self, html: &str) -> Result<(), TurbosheetError> {
        use crate::engine::LoadState;
        self.goto("about:blank", LoadState::DomContentLoaded).await?;
        let escaped = serde_json::to_string(&html).unwrap_or_default();
        self.evaluate(&format!("document.documentElement.innerHTML = {}", escaped)).await?;
        Ok(())
    }
    async fn inject_core_script(&self, script: &str) -> Result<(), TurbosheetError> {
        if self.core_injected.load(std::sync::atomic::Ordering::SeqCst) {
            return Ok(());
        }
        self.evaluate(script).await?;
        self.core_injected.store(true, std::sync::atomic::Ordering::SeqCst);
        Ok(())
    }
    async fn register_binding(&self, name: &str) -> Result<(), TurbosheetError> {
        let js = format!(
            r#"window['{}'] = function(...args) {{
                return navigator['{global}'].callBinding('{binding}', ...args)
            }}"#,
            name,
            global = self.global_name,
            binding = self.binding_name,
        );
        self.evaluate(&js).await?;
        Ok(())
    }
    fn global_name(&self) -> &str { &self.global_name }

    async fn get_local_storage(&self) -> Result<HashMap<String, String>, TurbosheetError> {
        let js = r#"(function() {
            const items = {};
            for (let i = 0; i < window.localStorage.length; i++) {
                const key = window.localStorage.key(i);
                items[key] = window.localStorage.getItem(key);
            }
            return JSON.stringify(items);
        })()"#;
        let result = self.evaluate(js).await?;
        // WebDriver response: {"value": <stringified JSON>}
        let parsed: serde_json::Value = serde_json::from_str(&result).unwrap_or_default();
        let inner = parsed.get("value").and_then(|v| v.as_str()).unwrap_or("{}");
        let map: HashMap<String, String> = serde_json::from_str(inner).unwrap_or_default();
        Ok(map)
    }

    async fn set_local_storage(&self, items: HashMap<String, String>) -> Result<(), TurbosheetError> {
        for (key, value) in items {
            let js = format!(
                "window.localStorage.setItem({}, {})",
                serde_json::to_string(&key).unwrap_or_else(|_| format!("\"{}\"", key)),
                serde_json::to_string(&value).unwrap_or_else(|_| format!("\"{}\"", value)),
            );
            self.evaluate(&js).await?;
        }
        Ok(())
    }

    async fn clear_local_storage(&self) -> Result<(), TurbosheetError> {
        self.evaluate("window.localStorage.clear()").await?;
        Ok(())
    }

    async fn get_session_storage(&self) -> Result<HashMap<String, String>, TurbosheetError> {
        let js = r#"(function() {
            const items = {};
            for (let i = 0; i < window.sessionStorage.length; i++) {
                const key = window.sessionStorage.key(i);
                items[key] = window.sessionStorage.getItem(key);
            }
            return JSON.stringify(items);
        })()"#;
        let result = self.evaluate(js).await?;
        let parsed: serde_json::Value = serde_json::from_str(&result).unwrap_or_default();
        let inner = parsed.get("value").and_then(|v| v.as_str()).unwrap_or("{}");
        let map: HashMap<String, String> = serde_json::from_str(inner).unwrap_or_default();
        Ok(map)
    }

    async fn set_session_storage(&self, items: HashMap<String, String>) -> Result<(), TurbosheetError> {
        for (key, value) in items {
            let js = format!(
                "window.sessionStorage.setItem({}, {})",
                serde_json::to_string(&key).unwrap_or_else(|_| format!("\"{}\"", key)),
                serde_json::to_string(&value).unwrap_or_else(|_| format!("\"{}\"", value)),
            );
            self.evaluate(&js).await?;
        }
        Ok(())
    }

    async fn clear_session_storage(&self) -> Result<(), TurbosheetError> {
        self.evaluate("window.sessionStorage.clear()").await?;
        Ok(())
    }
}

/// Check whether a cookie matches a URL for cookie filtering.
/// A cookie matches if the URL's host matches the cookie domain and
/// the URL's path starts with the cookie path.
fn cookie_matches_url(cookie: &crate::context::Cookie, url_str: &str) -> bool {
    // Simple host extraction: get the hostname part of the URL
    let url = url_str.trim_start_matches("http://")
                     .trim_start_matches("https://");
    let host = url.split('/').next().unwrap_or(url);
    let path = if let Some(pos) = url.find('/') {
        &url[pos..]
    } else {
        "/"
    };
    let path = if path.is_empty() { "/" } else { path };

    // Check domain: cookie.domain matches host or host ends with ".cookie.domain"
    let cookie_domain = cookie.domain.trim_start_matches('.');
    let domain_match = host == cookie_domain
        || host.ends_with(&format!(".{}", cookie_domain))
        || cookie_domain.is_empty();

    // Check path: cookie.path is a prefix of the URL path
    let path_match = path.starts_with(&cookie.path);

    domain_match && path_match
}
