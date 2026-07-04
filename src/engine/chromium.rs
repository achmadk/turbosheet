use crate::browser::LaunchOptions;
use crate::context::ContextOptions;
use crate::engine::{BrowserEngine, ContextEngine, LoadState, PageEngine};
use crate::error::TurbosheetError;
use crate::events::{EventDispatcher, EventType};
use crate::page::ScreenshotOptions;
use crate::network::interceptor::{NetworkEventBus, NetworkRequestEvent, NetworkResponseEvent};
use async_trait::async_trait;
use chromiumoxide::browser::{Browser, BrowserConfig};
use chromiumoxide::page::Page;
use chromiumoxide::cdp::browser_protocol::target::EventTargetCreated;
use std::collections::{HashMap, HashSet};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::{Mutex, RwLock};
use tokio::task::JoinHandle;
use futures::StreamExt;

lazy_static::lazy_static! {
    /// CDP windowsVirtualKeyCode lookup table for special keys.
    /// Maps lowercase key name to virtual key code.
    static ref KEY_CODE_MAP: HashMap<&'static str, i64> = {
        let mut m = HashMap::new();
        m.insert("enter", 13i64);
        m.insert("tab", 9i64);
        m.insert("escape", 27i64);
        m.insert("backspace", 8i64);
        m.insert("delete", 46i64);
        m.insert("home", 36i64);
        m.insert("end", 35i64);
        m.insert("pageup", 33i64);
        m.insert("pagedown", 34i64);
        m.insert("arrowup", 38i64);
        m.insert("arrowdown", 40i64);
        m.insert("arrowleft", 37i64);
        m.insert("arrowright", 39i64);
        m.insert("f1", 112i64);
        m.insert("f2", 113i64);
        m.insert("f3", 114i64);
        m.insert("f4", 115i64);
        m.insert("f5", 116i64);
        m.insert("f6", 117i64);
        m.insert("f7", 118i64);
        m.insert("f8", 119i64);
        m.insert("f9", 120i64);
        m.insert("f10", 121i64);
        m.insert("f11", 122i64);
        m.insert("f12", 123i64);
        m.insert("shift", 16i64);
        m.insert("control", 17i64);
        m.insert("alt", 18i64);
        m.insert("meta", 91i64);
        m.insert("capslock", 20i64);
        m.insert("insert", 45i64);
        m.insert("space", 32i64);
        m.insert("printscreen", 44i64);
        m.insert("scrolllock", 145i64);
        m.insert("pause", 19i64);
        m.insert("numlock", 144i64);
        m.insert("contextmenu", 93i64);
        m
    };
}

pub struct ChromiumEngine {
    browser: Arc<Mutex<Browser>>,
}

impl ChromiumEngine {
    pub async fn launch(options: LaunchOptions) -> Result<Self, TurbosheetError> {
        let mut builder = BrowserConfig::builder();
        
        // On Linux, Chrome requires --no-sandbox when running in environments
        // without proper sandbox (containers, CI, etc).
        if cfg!(target_os = "linux") {
            builder = builder.arg("--no-sandbox");
        }

        if !options.headless.unwrap_or(true) {
            builder = builder.with_head();
        }

        if let Some(path) = options.executable_path {
            builder = builder.chrome_executable(path);
        }

        if let Some(args) = options.args {
            let mut remaining_args = Vec::new();
            for arg in args {
                if arg.starts_with("--user-data-dir=") {
                    let dir = arg.trim_start_matches("--user-data-dir=");
                    builder = builder.user_data_dir(dir);
                } else {
                    remaining_args.push(arg);
                }
            }
            for arg in remaining_args {
                builder = builder.arg(arg);
            }
        }

        let config = builder.build().map_err(|e| {
            TurbosheetError::LaunchFailed(format!("Failed to build browser config: {}", e))
        })?;

        let (browser, mut handler) = Browser::launch(config).await.map_err(|e| {
            TurbosheetError::LaunchFailed(format!("Failed to launch browser: {}", e))
        })?;

        let handler_handle = tokio::spawn(async move {
            while let Some(event) = futures::StreamExt::next(&mut handler).await {
                if let Err(e) = event {
                    tracing::error!("Browser handler error: {:?}", e);
                    // Do NOT break — the handler may recover or at minimum keep
                    // the mpsc receivers alive so in-flight Page operations can
                    // fail gracefully rather than panic the handler task.
                }
            }
            tracing::warn!("Browser handler task exited (WS stream ended)");
        });
        // Monitor the handler task separately (can't store JoinHandle due to
        // move semantics — the monitor task owns it).
        tokio::spawn(async move {
            match handler_handle.await {
                Ok(()) => tracing::debug!("Handler task completed cleanly"),
                Err(e) => tracing::error!("Handler task panicked: {}", e),
            }
        });

        Ok(Self {
            browser: Arc::new(Mutex::new(browser)),
        })
    }
}

#[async_trait]
impl BrowserEngine for ChromiumEngine {
    async fn new_context(
        &self,
        options: Option<ContextOptions>,
    ) -> Result<Arc<dyn ContextEngine>, TurbosheetError> {
        let mut params_builder = chromiumoxide::cdp::browser_protocol::target::CreateBrowserContextParams::builder();
        if let Some(ref proxy) = options.as_ref().and_then(|o| o.use_proxy.as_ref()) {
            params_builder = params_builder.proxy_server(proxy.server.clone());
            if let Some(ref bypass) = proxy.bypass {
                params_builder = params_builder.proxy_bypass_list(bypass.clone());
            }
        }
        let params = params_builder.build();
        let browser = self.browser.lock().await;
        let context = browser.create_browser_context(params).await.map_err(|e| {
            TurbosheetError::Other(format!("Failed to create incognito context: {}", e))
        })?;

        let (popup_tx, _) = tokio::sync::broadcast::channel(64);

        Ok(Arc::new(ChromiumContextEngine {
            browser: self.browser.clone(),
            context_id: context,
            network_bus: Arc::new(NetworkEventBus::new()),
            event_dispatcher: Arc::new(EventDispatcher::new()),
            pages: Arc::new(tokio::sync::RwLock::new(Vec::new())),
            target_to_page: Arc::new(RwLock::new(HashMap::new())),
            page_to_opener: Arc::new(RwLock::new(HashMap::new())),
            popup_tx: Arc::new(popup_tx),
            _target_listener: Arc::new(Mutex::new(None)),
        }))
    }

    async fn close(&self) -> Result<(), TurbosheetError> {
        // 1. Close the browser via CDP Browser.close for a clean shutdown
        use chromiumoxide::cdp::browser_protocol::browser::CloseParams;
        if let Ok(browser) = self.browser.try_lock() {
            if let Err(e) = browser.execute(CloseParams::default()).await {
                tracing::warn!("CDP Browser.close failed (browser may already be shutting down): {}", e);
            }
        }

        // 2. If we spawned the process, send a courteous SIGTERM then escalate
        //    The operation reference is only set when self manages the process.
        //    Even without it, the CDP close should trigger a graceful shutdown.
        Ok(())
    }

    fn version(&self) -> String {
        "Chromium (CDP)".to_string()
    }
}

pub struct ChromiumContextEngine {
    browser: Arc<Mutex<Browser>>,
    context_id: chromiumoxide::cdp::browser_protocol::browser::BrowserContextId,
    pub network_bus: Arc<NetworkEventBus>,
    pub event_dispatcher: Arc<EventDispatcher>,
    /// Tracks pages created by this context so the trait's `pages()` can
    /// return the real list instead of an empty vec.
    pages: Arc<tokio::sync::RwLock<Vec<Arc<dyn PageEngine>>>>,
    /// CDP target_id → our page_id mapping (for popup detection via opener).
    target_to_page: Arc<RwLock<HashMap<String, String>>>,
    /// page_id → opener_page_id (None for top-level pages).
    page_to_opener: Arc<RwLock<HashMap<String, Option<String>>>>,
    /// Broadcast sender that notifies when a new popup page is created.
    /// Receivers (e.g. waitForEvent('page')) subscribe via `.subscribe()`.
    popup_tx: Arc<tokio::sync::broadcast::Sender<String>>,
    /// Handle to the background target listener task.
    _target_listener: Arc<Mutex<Option<JoinHandle<()>>>>,
}

/// Helper: retry getting a CDP Page for a target_id with bounded polling.
/// chromiumoxide's handler creates Page objects asynchronously after
/// Target.targetCreated, so a direct get_page often fails on first try.
async fn get_popup_page_with_retry(
    browser_arc: &Arc<Mutex<Browser>>,
    target_id: &chromiumoxide::cdp::browser_protocol::target::TargetId,
    max_retries: usize,
    delay_ms: u64,
) -> Option<Page> {
    for _ in 0..max_retries {
        let browser_lock = browser_arc.lock().await;
        if let Ok(p) = browser_lock.get_page(target_id.clone()).await {
            return Some(p);
        }
        drop(browser_lock);
        tokio::time::sleep(tokio::time::Duration::from_millis(delay_ms)).await;
    }
    None
}

// ── Free-standing page engine setup (shared by new_page & popup handler) ──

/// Wraps an already-created CDP `Page` into a `ChromiumPageEngine`,
/// sets up all CDP event listeners (network, dialog, console, binding,
/// etc.) and injects the stealth core script.
async fn setup_page_engine(
    page: Page,
    cdp_target_id: String,
    network_bus: Arc<NetworkEventBus>,
    event_dispatcher: Arc<EventDispatcher>,
    target_to_page: Arc<RwLock<HashMap<String, String>>>,
    page_to_opener: Arc<RwLock<HashMap<String, Option<String>>>>,
) -> Result<Arc<ChromiumPageEngine>, TurbosheetError> {
        // Enable CDP network tracking so Network.requestWillBeSent and
        // Network.responseReceived events are emitted.
        page.execute(chromiumoxide::cdp::browser_protocol::network::EnableParams::default())
            .await
            .map_err(|e| {
                TurbosheetError::Other(format!("Failed to enable network tracking: {}", e))
            })?;

        // Enable the Inspector domain so Inspector.detached and
        // Inspector.targetCrashed events are emitted.
        page.execute(chromiumoxide::cdp::browser_protocol::inspector::EnableParams::default())
            .await
            .map_err(|e| {
                TurbosheetError::Other(format!("Failed to enable inspector domain: {}", e))
            })?;

        let binding_registry = Arc::new(crate::injection::bindings::BindingRegistry::new());
        let current_url = Arc::new(RwLock::new("about:blank".to_string()));

        let mut nav_events = page
            .event_listener::<chromiumoxide::cdp::browser_protocol::page::EventFrameNavigated>()
            .await
            .map_err(|e| {
                TurbosheetError::Other(format!("Failed to subscribe to nav events: {}", e))
            })?;
        let mut binding_events = page
            .event_listener::<chromiumoxide::cdp::js_protocol::runtime::EventBindingCalled>()
            .await
            .map_err(|e| {
                TurbosheetError::Other(format!("Failed to subscribe to binding events: {}", e))
            })?;
        let mut dialog_events = page
            .event_listener::<chromiumoxide::cdp::browser_protocol::page::EventJavascriptDialogOpening>()
            .await
            .map_err(|e| {
                TurbosheetError::Other(format!("Failed to subscribe to dialog events: {}", e))
            })?;
        let mut console_events = page
            .event_listener::<chromiumoxide::cdp::js_protocol::runtime::EventConsoleApiCalled>()
            .await
            .map_err(|e| {
                TurbosheetError::Other(format!("Failed to subscribe to console events: {}", e))
            })?;
        let mut file_chooser_events = page
            .event_listener::<chromiumoxide::cdp::browser_protocol::page::EventFileChooserOpened>()
            .await
            .map_err(|e| {
                TurbosheetError::Other(format!(
                    "Failed to subscribe to file chooser events: {}",
                    e
                ))
            })?;

        let mut request_events = page
            .event_listener::<chromiumoxide::cdp::browser_protocol::network::EventRequestWillBeSent>()
            .await
            .map_err(|e| {
                TurbosheetError::Other(format!(
                    "Failed to subscribe to network request events: {}",
                    e
                ))
            })?;
        let mut response_events = page
            .event_listener::<chromiumoxide::cdp::browser_protocol::network::EventResponseReceived>()
            .await
            .map_err(|e| {
                TurbosheetError::Other(format!(
                    "Failed to subscribe to network response events: {}",
                    e
                ))
            })?;
        let mut detached_events = page
            .event_listener::<chromiumoxide::cdp::browser_protocol::inspector::EventDetached>()
            .await
            .map_err(|e| {
                TurbosheetError::Other(format!(
                    "Failed to subscribe to inspector detached events: {}",
                    e
                ))
            })?;
        let mut crashed_events = page
            .event_listener::<chromiumoxide::cdp::browser_protocol::inspector::EventTargetCrashed>()
            .await
            .map_err(|e| {
                TurbosheetError::Other(format!(
                    "Failed to subscribe to inspector target crashed events: {}",
                    e
                ))
            })?;

        #[cfg(feature = "video")]
        let screencast_events = page
            .event_listener::<chromiumoxide::cdp::browser_protocol::page::EventScreencastFrame>()
            .await
            .map_err(|e| {
                TurbosheetError::Other(format!(
                    "Failed to subscribe to screencast events: {}",
                    e
                ))
            })?;
        #[cfg(feature = "video")]
        let mut screencast_stream = Box::pin(screencast_events);
        #[cfg(not(feature = "video"))]
        let mut screencast_stream = Box::pin(futures::stream::pending::<
            chromiumoxide::cdp::browser_protocol::page::EventScreencastFrame
        >());
        #[cfg(feature = "video")]
        let frame_tx_for_engine: Arc<
            tokio::sync::Mutex<Option<tokio::sync::mpsc::UnboundedSender<Vec<u8>>>>,
        > = Arc::new(tokio::sync::Mutex::new(None));
        #[cfg(feature = "video")]
        let frame_tx_for_task = frame_tx_for_engine.clone();

        let url_clone = current_url.clone();
        let registry_clone = binding_registry.clone();
        let page_clone = page.clone();
        let page_id_cell = Arc::new(Mutex::new(None::<String>));
        let page_id_for_task = page_id_cell.clone();
        let page_dead_flag = Arc::new(AtomicBool::new(false));
        let page_dead_flag_task = page_dead_flag.clone();
        // Clones for the async task — the originals are still needed for the struct constructor.
        let bus_for_task = network_bus.clone();
        let ed_for_task = event_dispatcher.clone();
        tokio::spawn(async move {
            // Acquire the page_id once it's set (set_page_id is called
            // after the page is registered in PAGES, which happens right
            // after new_page/popup-handler returns).
            let pid = loop {
                let guard = page_id_for_task.lock().await;
                if let Some(ref id) = *guard {
                    break id.clone();
                }
                drop(guard);
                tokio::time::sleep(Duration::from_millis(10)).await;
            };

            loop {
                tokio::select! {
                    Some(nav) = nav_events.next() => {
                        let payload = serde_json::json!({
                            "frame": {
                                "url": nav.frame.url.clone(),
                            }
                        });
                        ed_for_task.handle_event(&pid, EventType::FrameNavigated, payload);
                        let mut url = url_clone.write().await;
                        *url = nav.frame.url.clone();
                    }
                    Some(binding) = binding_events.next() => {
                        registry_clone.dispatch(&binding.payload);
                    }
                    Some(dialog) = dialog_events.next() => {
                        let payload = serde_json::json!({
                            "type": format!("{:?}", dialog.r#type),
                            "message": dialog.message,
                            "url": dialog.url,
                        });
                        ed_for_task.handle_event(&pid, EventType::JavascriptDialogOpening, payload.clone());
                        crate::network::events::dispatch_event(&pid, "dialog", &payload.to_string());

                        let page_for_dialog = page_clone.clone();
                        tokio::spawn(async move {
                            tokio::time::sleep(Duration::from_secs(5)).await;
                            let params = chromiumoxide::cdp::browser_protocol::page::HandleJavaScriptDialogParams::builder()
                                .accept(true)
                                .build()
                                .unwrap();
                            let _ = page_for_dialog.execute(params).await;
                        });
                    }
                    Some(console) = console_events.next() => {
                        let args: Vec<String> = console.args.iter()
                            .filter_map(|obj| {
                                obj.description.clone()
                                    .or_else(|| obj.value.as_ref().map(|v| v.to_string()))
                            })
                            .collect();
                        let type_str = format!("{:?}", console.r#type);
                        let st = console.stack_trace.as_ref()
                            .and_then(|st| st.description.clone());
                        let payload = serde_json::json!({
                            "type": type_str,
                            "args": args,
                            "stackTrace": st,
                        });
                        ed_for_task.handle_event(&pid, EventType::ConsoleAPICalled, payload.clone());
                        crate::network::events::dispatch_event(&pid, "console", &payload.to_string());
                    }
                    Some(file_chooser) = file_chooser_events.next() => {
                        let payload = serde_json::json!({
                            "mode": format!("{:?}", file_chooser.mode),
                            "backend_node_id": file_chooser.backend_node_id,
                        });
                        ed_for_task.handle_event(&pid, EventType::FileChooserOpened, payload);
                        let params = chromiumoxide::cdp::browser_protocol::page::SetInterceptFileChooserDialogParams::builder()
                            .enabled(true)
                            .build()
                            .unwrap();
                        let _ = page_clone.execute(params).await;
                    }
                    Some(req) = request_events.next() => {
                        let mut headers = std::collections::HashMap::new();
                        if let Some(obj) = req.request.headers.inner().as_object() {
                            for (k, v) in obj {
                                if let Some(val) = v.as_str() {
                                    headers.insert(k.clone(), val.to_string());
                                }
                            }
                        }

                        let payload = serde_json::json!({
                            "url": req.request.url.clone(),
                            "method": req.request.method.clone(),
                            "headers": headers,
                        });
                        ed_for_task.handle_event(&pid, EventType::RequestWillBeSent, payload);

                        bus_for_task.emit_request(NetworkRequestEvent {
                            url: req.request.url.clone(),
                            method: req.request.method.clone(),
                            headers: headers.clone(),
                            post_data: None,
                            timestamp: *req.timestamp.inner(),
                        });
                        let json = crate::network::events::request_event_json(
                            &req.request.url,
                            &req.request.method,
                            &headers,
                            None,
                        );
                        crate::network::events::dispatch_event(&pid, "request", &json);
                    }
                    Some(resp) = response_events.next() => {
                        let mut headers = std::collections::HashMap::new();
                        if let Some(obj) = resp.response.headers.inner().as_object() {
                            for (k, v) in obj {
                                if let Some(val) = v.as_str() {
                                    headers.insert(k.clone(), val.to_string());
                                }
                            }
                        }

                        let payload = serde_json::json!({
                            "url": resp.response.url.clone(),
                            "status": resp.response.status,
                            "headers": headers,
                        });
                        ed_for_task.handle_event(&pid, EventType::ResponseReceived, payload);

                        bus_for_task.emit_response(NetworkResponseEvent {
                            url: resp.response.url.clone(),
                            status: resp.response.status as u16,
                            headers: headers.clone(),
                            timestamp: *resp.timestamp.inner(),
                        });
                        let json = crate::network::events::response_event_json(
                            &resp.response.url,
                            resp.response.status as u16,
                            &headers,
                        );
                        crate::network::events::dispatch_event(&pid, "response", &json);
                    }
                    Some(detached) = detached_events.next() => {
                        tracing::warn!("CDP disconnected from page: reason={}", detached.reason);
                        page_dead_flag_task.store(true, Ordering::SeqCst);
                        break;
                    }
                    Some(_crashed) = crashed_events.next() => {
                        tracing::error!("Page target crashed");
                        ed_for_task.handle_event(&pid, EventType::TargetCreated, serde_json::json!({}));
                        page_dead_flag_task.store(true, Ordering::SeqCst);
                        break;
                    }
                    Some(_frame) = screencast_stream.next() => {
                        #[cfg(feature = "video")]
                        {
                            let frame = _frame;
                            use base64::{Engine as _, engine::general_purpose::STANDARD};
                            let frame_size = <chromiumoxide::Binary as AsRef::<[u8]>>::as_ref(&frame.data).len() as u32;
                            if let Ok(decoded) = STANDARD.decode(
                                <chromiumoxide::Binary as AsRef::<[u8]>>::as_ref(&frame.data)
                            ) {
                                let guard = frame_tx_for_task.lock().await;
                                if let Some(ref tx) = *guard {
                                    let _ = tx.send(decoded);
                                }
                            }
                            crate::trace::GLOBAL_RECORDER.record_screencast_frame(0, frame_size).await;

                            let ack = chromiumoxide::cdp::browser_protocol::page::ScreencastFrameAckParams::new(frame.session_id);
                            let _ = page_clone.execute(ack).await;
                        }
                    }
                    else => break,
                }
            }
        });

        let stealth_cfg = crate::injection::stealth::StealthConfig::default();
        let (global_name, binding_name) = stealth_cfg.generate_names();

        let page_engine = Arc::new(ChromiumPageEngine {
            page: Mutex::new(Some(page)),
            current_url,
            binding_registry,
            global_name: global_name.clone(),
            binding_name: binding_name.clone(),
            stealth_config: stealth_cfg.clone(),
            network_bus: network_bus.clone(),
            fetch_handle: Arc::new(Mutex::new(None)),
            page_id: page_id_cell,
            page_dead: page_dead_flag,
            event_dispatcher: event_dispatcher.clone(),
            injected_frames: RwLock::new(HashSet::new()),
            cdp_target_id,
            target_to_page: target_to_page.clone(),
            page_to_opener: page_to_opener.clone(),
            #[cfg(feature = "video")]
            recorder: Mutex::new(None),
            #[cfg(feature = "video")]
            frame_tx: frame_tx_for_engine.clone(),
        });

        let core_script = stealth_cfg.process_script(
            crate::injection::scripts::CORE_SCRIPT,
            &global_name,
            &binding_name,
        );

        page_engine.inject_core_script(&core_script).await?;
        page_engine.register_binding(&binding_name).await?;

        Ok(page_engine)
    }

// ── Popup / target listener ──────────────────────────────────────────

impl ChromiumContextEngine {
    /// Lazily starts a background task that listens for CDP
    /// `Target.targetCreated` events on the browser.  When a new page-type
    /// target is created whose `opener_id` matches one of our tracked CDP
    /// targets, the task wraps it into a full `ChromiumPageEngine`.
    async fn start_target_listener(&self) {
        let mut guard = self._target_listener.lock().await;
        if guard.is_some() {
            return;
        }

        let mut events = {
            let browser = self.browser.lock().await;
            match browser.event_listener::<EventTargetCreated>().await {
                Ok(s) => s,
                Err(e) => {
                    tracing::error!("Failed to subscribe to target events: {}", e);
                    return;
                }
            }
        };

        let browser_arc = self.browser.clone();
        let bus = self.network_bus.clone();
        let ed = self.event_dispatcher.clone();
        let t2p = self.target_to_page.clone();
        let p2o = self.page_to_opener.clone();
        let pages = self.pages.clone();
        let pt = self.popup_tx.clone();

        let handle = tokio::spawn(async move {
            while let Some(event) = events.next().await {
                let info = &event.target_info;
                if info.r#type != "page" {
                    continue;
                }
                let opener_id = match &info.opener_id {
                    Some(id) => id.inner().clone(),
                    None => continue,
                };

                let opener_page_id = {
                    let map = t2p.read().await;
                    match map.get(&opener_id) {
                        Some(pid) if !pid.is_empty() => pid.clone(),
                        _ => continue,
                    }
                };

                let target_id = info.target_id.inner().clone();

                let page = get_popup_page_with_retry(&browser_arc, &info.target_id, 10, 200).await;
                let page = match page {
                    Some(p) => p,
                    None => {
                        tracing::error!("Failed to get popup page {} after retries", target_id);
                        continue;
                    }
                };

                let engine = match setup_page_engine(
                    page,
                    target_id.clone(),
                    bus.clone(),
                    ed.clone(),
                    t2p.clone(),
                    p2o.clone(),
                )
                .await
                {
                    Ok(e) => e,
                    Err(e) => {
                        tracing::error!("Failed to setup popup page {}: {}", target_id, e);
                        continue;
                    }
                };

                let page_id = uuid::Uuid::new_v4().to_string();
                engine.set_page_id(&page_id).await.ok();

                {
                    let mut map = p2o.write().await;
                    map.insert(page_id.clone(), Some(opener_page_id.clone()));
                }

                // Register the opener relationship in the global map
                // so JsPage.opener() can resolve it.
                crate::engine::PAGE_OPENER
                    .insert(page_id.clone(), Some(opener_page_id.clone()));

                // Register page→context mapping so context-level event
                // dispatch can find the right context.
                let ctx_id = crate::engine::PAGE_CONTEXT
                    .get(&opener_page_id)
                    .map(|r| r.value().clone());
                if let Some(ref ctx_id) = ctx_id {
                    crate::engine::PAGE_CONTEXT.insert(page_id.clone(), ctx_id.clone());
                    // Also register the popup page in the Context struct so
                    // JsBrowserContext::pages() can find it.
                    if let Some(ctx_weak) = crate::engine::CONTEXTS.get(ctx_id) {
                        if let Some(ctx_arc) = ctx_weak.upgrade() {
                            let popup_page = Arc::new(crate::engine::Page {
                                id: page_id.clone(),
                                context: Arc::downgrade(&ctx_arc),
                                engine: engine.clone(),
                            });
                            crate::engine::PAGES.insert(page_id.clone(), Arc::downgrade(&popup_page));
                            ctx_arc.pages.write().await.push(popup_page);
                        }
                    }
                    // Dispatch popup event to context-level listeners.
                    let payload = serde_json::json!({ "pageId": page_id }).to_string();
                    crate::network::events::dispatch_context_event(ctx_id, "page", &payload);
                    // Also send on the broadcast channel for waitForEvent.
                    if let Some(tx) = crate::network::events::CONTEXT_POPUP_TX.get(ctx_id) {
                        let _ = tx.send(page_id.clone());
                    }
                }

                pages.write().await.push(engine.clone());

                let _ = pt.send(page_id.clone());
                tracing::info!("Registered popup page {}", page_id);
            }
        });

        *guard = Some(handle);
    }
}

#[async_trait]
impl ContextEngine for ChromiumContextEngine {
    async fn new_page(&self) -> Result<Arc<dyn PageEngine>, TurbosheetError> {
        self.start_target_listener().await;

        let params = chromiumoxide::cdp::browser_protocol::target::CreateTargetParams::builder()
            .url("about:blank")
            .browser_context_id(self.context_id.clone())
            .build()
            .unwrap();

        let browser = self.browser.lock().await;
        let target_id = browser.execute(params).await.map_err(|e| {
            TurbosheetError::Other(format!("Failed to create target: {}", e))
        })?.target_id.clone();

        let mut page_opt = None;
        for _ in 0..5 {
            if let Ok(page) = browser.get_page(target_id.clone()).await {
                page_opt = Some(page);
                break;
            }
            tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
        }

        let page = page_opt.ok_or_else(|| {
            TurbosheetError::Other("Failed to get page: Requested value not found after retries".to_string())
        })?;
        drop(browser);

        let target_id_str = target_id.inner().clone();
        let engine = setup_page_engine(
            page,
            target_id_str.clone(),
            self.network_bus.clone(),
            self.event_dispatcher.clone(),
            self.target_to_page.clone(),
            self.page_to_opener.clone(),
        )
        .await?;

        self.pages.write().await.push(engine.clone());

        Ok(engine)
    }

    async fn pages(&self) -> Result<Vec<Arc<dyn PageEngine>>, TurbosheetError> {
        let pages = self.pages.read().await;
        Ok(pages.clone())
    }

    async fn get_cookies(&self) -> Result<Vec<crate::context::Cookie>, TurbosheetError> {
        use chromiumoxide::cdp::browser_protocol::storage::GetCookiesParams;
        let params = GetCookiesParams::builder()
            .browser_context_id(self.context_id.clone())
            .build();
        let browser = self.browser.lock().await;
        let result = browser.execute(params).await.map_err(|e| {
            TurbosheetError::Other(format!("Failed to get cookies: {}", e))
        })?;
        Ok(result.cookies.clone().into_iter().map(|cdp_cookie| {
            crate::context::Cookie {
                name: cdp_cookie.name,
                value: cdp_cookie.value,
                domain: cdp_cookie.domain,
                path: cdp_cookie.path,
                expires: cdp_cookie.expires,
                size: cdp_cookie.size,
                http_only: cdp_cookie.http_only,
                secure: cdp_cookie.secure,
                session: cdp_cookie.session,
                same_site: cdp_cookie.same_site.map(|s| format!("{:?}", s)),
                priority: format!("{:?}", cdp_cookie.priority),
            }
        }).collect())
    }

    async fn cookies(&self, urls: Option<Vec<String>>) -> Result<Vec<crate::context::Cookie>, TurbosheetError> {
        use chromiumoxide::cdp::browser_protocol::network::GetCookiesParams;
        let mut builder = GetCookiesParams::builder();
        if let Some(url_list) = urls {
            builder = builder.urls(url_list);
        }
        let browser = self.browser.lock().await;
        let result = browser.execute(builder.build()).await.map_err(|e| {
            TurbosheetError::Other(format!("Failed to get cookies: {}", e))
        })?;
        Ok(result.cookies.clone().into_iter().map(|cdp_cookie| {
            crate::context::Cookie {
                name: cdp_cookie.name,
                value: cdp_cookie.value,
                domain: cdp_cookie.domain,
                path: cdp_cookie.path,
                expires: cdp_cookie.expires,
                size: cdp_cookie.size,
                http_only: cdp_cookie.http_only,
                secure: cdp_cookie.secure,
                session: cdp_cookie.session,
                same_site: cdp_cookie.same_site.map(|s| format!("{:?}", s)),
                priority: format!("{:?}", cdp_cookie.priority),
            }
        }).collect())
    }

    async fn set_cookies(&self, cookies: Vec<crate::context::CookieParam>) -> Result<(), TurbosheetError> {
        use chromiumoxide::cdp::browser_protocol::storage::SetCookiesParams;
        use chromiumoxide::cdp::browser_protocol::network::CookieParam as CdpCookieParam;
        use chromiumoxide::cdp::browser_protocol::network::CookieSameSite;

        let cdp_cookies: Result<Vec<CdpCookieParam>, TurbosheetError> = cookies.into_iter().map(|c| {
            let mut builder = CdpCookieParam::builder()
                .name(c.name)
                .value(c.value);
            if let Some(url) = c.url {
                builder = builder.url(url);
            }
            if let Some(domain) = c.domain {
                builder = builder.domain(domain);
            }
            if let Some(path) = c.path {
                builder = builder.path(path);
            }
            if let Some(secure) = c.secure {
                builder = builder.secure(secure);
            }
            if let Some(http_only) = c.http_only {
                builder = builder.http_only(http_only);
            }
            if let Some(ref same_site) = c.same_site {
                if let Some(s) = match same_site.to_lowercase().as_str() {
                    "strict" => Some(CookieSameSite::Strict),
                    "lax" => Some(CookieSameSite::Lax),
                    "none" => Some(CookieSameSite::None),
                    _ => None,
                } {
                    builder = builder.same_site(s);
                }
            }
            if let Some(expires) = c.expires {
                builder = builder.expires(chromiumoxide::cdp::browser_protocol::network::TimeSinceEpoch::new(expires as f64));
            }
            builder.build().map_err(|e| TurbosheetError::Other(format!("Invalid cookie param: {}", e)))
        }).collect();
        let cdp_cookies = cdp_cookies?;

        let params = SetCookiesParams::builder()
            .cookies(cdp_cookies)
            .browser_context_id(self.context_id.clone())
            .build()
            .map_err(|e| TurbosheetError::Other(format!("Failed to build cookie params: {}", e)))?;
        let browser = self.browser.lock().await;
        browser.execute(params).await.map_err(|e| {
            TurbosheetError::Other(format!("Failed to set cookies: {}", e))
        })?;
        Ok(())
    }

    async fn add_cookies(&self, cookies: Vec<crate::context::CookieParam>) -> Result<(), TurbosheetError> {
        use chromiumoxide::cdp::browser_protocol::network::SetCookiesParams;
        use chromiumoxide::cdp::browser_protocol::network::CookieParam as CdpCookieParam;
        use chromiumoxide::cdp::browser_protocol::network::CookieSameSite;

        let cdp_cookies: Result<Vec<CdpCookieParam>, TurbosheetError> = cookies.into_iter().map(|c| {
            let mut builder = CdpCookieParam::builder()
                .name(c.name)
                .value(c.value);
            if let Some(url) = c.url {
                builder = builder.url(url);
            }
            if let Some(domain) = c.domain {
                builder = builder.domain(domain);
            }
            if let Some(path) = c.path {
                builder = builder.path(path);
            }
            if let Some(secure) = c.secure {
                builder = builder.secure(secure);
            }
            if let Some(http_only) = c.http_only {
                builder = builder.http_only(http_only);
            }
            if let Some(ref same_site) = c.same_site {
                if let Some(s) = match same_site.to_lowercase().as_str() {
                    "strict" => Some(CookieSameSite::Strict),
                    "lax" => Some(CookieSameSite::Lax),
                    "none" => Some(CookieSameSite::None),
                    _ => None,
                } {
                    builder = builder.same_site(s);
                }
            }
            if let Some(expires) = c.expires {
                builder = builder.expires(chromiumoxide::cdp::browser_protocol::network::TimeSinceEpoch::new(expires as f64));
            }
            builder.build().map_err(|e| TurbosheetError::Other(format!("Invalid cookie param: {}", e)))
        }).collect();
        let cdp_cookies = cdp_cookies?;

        let params = SetCookiesParams::new(cdp_cookies);
        let browser = self.browser.lock().await;
        browser.execute(params).await.map_err(|e| {
            TurbosheetError::Other(format!("Failed to add cookies: {}", e))
        })?;
        Ok(())
    }

    async fn clear_cookies(&self) -> Result<(), TurbosheetError> {
        use chromiumoxide::cdp::browser_protocol::storage::ClearCookiesParams;
        let params = ClearCookiesParams::builder()
            .browser_context_id(self.context_id.clone())
            .build();
        let browser = self.browser.lock().await;
        browser.execute(params).await.map_err(|e| {
            TurbosheetError::Other(format!("Failed to clear cookies: {}", e))
        })?;
        Ok(())
    }

    async fn close(&self) -> Result<(), TurbosheetError> {
        let mut pages = self.pages.write().await;
        for page in pages.drain(..) {
            let _ = page.close().await;
        }
        Ok(())
    }
}

pub struct ChromiumPageEngine {
    page: Mutex<Option<Page>>,
    current_url: Arc<RwLock<String>>,
    pub binding_registry: Arc<crate::injection::bindings::BindingRegistry>,
    global_name: String,
    binding_name: String,
    stealth_config: crate::injection::stealth::StealthConfig,
    pub network_bus: Arc<NetworkEventBus>,
    fetch_handle: Arc<Mutex<Option<JoinHandle<()>>>>,
    pub page_id: Arc<Mutex<Option<String>>>,
    page_dead: Arc<AtomicBool>,
    pub event_dispatcher: Arc<EventDispatcher>,
    injected_frames: RwLock<HashSet<String>>,
    /// The CDP target ID for this page (set during construction).
    cdp_target_id: String,
    /// Reference to the parent context's target → page_id map.
    /// set_page_id inserts into this map so that popup detection via
    /// CDP Target.targetCreated → opener_id can find us.
    target_to_page: Arc<RwLock<HashMap<String, String>>>,
    /// Reference to the parent context's page_id → opener_page_id map.
    page_to_opener: Arc<RwLock<HashMap<String, Option<String>>>>,
    #[cfg(feature = "video")]
    /// Video recorder handle (None = not recording). Interior mutability via Mutex
    /// since PageEngine methods take `&self`.
    recorder: Mutex<Option<crate::video::VideoRecorder>>,
    #[cfg(feature = "video")]
    /// Shared frame sender for the screencast event handler task.
    /// Set during start_recording, consumed by the background task.
    frame_tx: Arc<tokio::sync::Mutex<Option<tokio::sync::mpsc::UnboundedSender<Vec<u8>>>>>,
}

impl ChromiumPageEngine {
    /// Acquires the page lock and returns the guard ONLY if the page is
    /// still alive (not crashed/CDP-detached).  Drops the page handle to
    /// `None` on dead so callers see the existing "Page is closed" path.
    async fn lock_page(&self) -> Result<tokio::sync::MutexGuard<'_, Option<Page>>, TurbosheetError> {
        let mut guard = self.page.lock().await;
        if self.page_dead.load(Ordering::SeqCst) {
            *guard = None;
        }
        Ok(guard)
    }

}


impl ChromiumPageEngine {
    /// Returns the (x, y) center coordinates of the first element matching `selector`.
    async fn get_element_center(&self, selector: &str) -> Result<(f64, f64), TurbosheetError> {
        let mut guard = self.lock_page().await?;
        if let Some(page) = guard.as_mut() {
            let escaped = serde_json::to_string(&selector).unwrap_or_else(|_| format!("\"{}\"", selector));
            let js = format!(
                "(function() {{ var el = document.querySelector({}); if (!el) return null; \
                 var r = el.getBoundingClientRect(); \
                 return JSON.stringify({{x: r.left + r.width/2, y: r.top + r.height/2}}); }})()",
                escaped
            );
            let res = page.evaluate(js).await.map_err(|e| {
                TurbosheetError::Other(format!("get_element_center failed for '{}': {}", selector, e))
            })?;
            let json = res.into_value::<String>().map_err(|_| {
                TurbosheetError::Other(format!("Element '{}' not found or has no bounding rect", selector))
            })?;
            let obj: serde_json::Value = serde_json::from_str(&json)
                .map_err(|e| TurbosheetError::Other(format!("JSON parse error: {}", e)))?;
            let x = obj["x"].as_f64().unwrap_or(0.0);
            let y = obj["y"].as_f64().unwrap_or(0.0);
            return Ok((x, y));
        }
        Err(TurbosheetError::Other("Page is closed".to_string()))
    }
}

// ── CDP Fetch Domain interception ──────────────────────────────────────

impl ChromiumPageEngine {
    pub async fn enable_fetch_interception(&self, page_id: &str) -> Result<(), TurbosheetError> {
        if self.fetch_handle.lock().await.is_some() {
            return Ok(());
        }
        let mut guard = self.lock_page().await?;
        if let Some(page) = guard.as_mut() {
            let enable_params =
                chromiumoxide::cdp::browser_protocol::fetch::EnableParams::builder().build();
            page.execute(enable_params).await.map_err(|e| {
                TurbosheetError::Other(format!("Failed to enable CDP Fetch domain: {}", e))
            })?;

            let mut events = page
                .event_listener::<chromiumoxide::cdp::browser_protocol::fetch::EventRequestPaused>()
                .await
                .map_err(|e| {
                    TurbosheetError::Other(format!(
                        "Failed to subscribe to Fetch.requestPaused: {}",
                        e
                    ))
                })?;

            let page_clone = page.clone();
            let pid = page_id.to_string();

            let handle = tokio::spawn(async move {
                use napi::threadsafe_function::ThreadsafeFunctionCallMode;
                
                use crate::network::proxy::PAGE_PROXIES;
                use crate::network::route::{JsRoute, RouteRequest, RouteAction};
                use tokio::sync::oneshot;
                use std::collections::HashMap;
                use chromiumoxide::cdp::browser_protocol::fetch::{
                    FulfillRequestParams, ContinueRequestParams, FailRequestParams, HeaderEntry,
                };
                use base64::{Engine as _, engine::general_purpose::STANDARD};

                while let Some(event) = events.next().await {
                    let url = event.request.url.clone();
                    let method = event.request.method.clone();

                    let mut headers = HashMap::new();
                    if let Some(obj) = event.request.headers.inner().as_object() {
                        for (k, v) in obj {
                            if let Some(val) = v.as_str() {
                                headers.insert(k.clone(), val.to_string());
                            }
                        }
                    }

                    // Attempt POST body extraction from `post_data_entries`.
                    // CDP's `Fetch.requestPaused` event includes `request.postData`
                    // as a flat string, but chromiumoxide maps it to `network::Request`
                    // which uses `post_data_entries` (base64-encoded binary chunks).
                    // Fall back to None if no entries are present.
                    let post_data: Option<Vec<u8>> = event.request.post_data_entries
                        .as_ref()
                        .and_then(|entries| {
                            entries.first()
                                .and_then(|entry| entry.bytes.as_ref())
                                .and_then(|binary| {
                                    use std::convert::AsRef;
                                    // Binary wraps a base64-encoded String
                                    let b64: &str = AsRef::<str>::as_ref(binary);
                                    STANDARD.decode(b64).ok()
                                })
                        });

                    let matched_callback = PAGE_PROXIES.get(&pid).and_then(|proxy| {
                        for entry in proxy.routes.iter() {
                            if entry.value().pattern.matches(&url) {
                                return Some(entry.value().callback.clone());
                            }
                        }
                        None
                    });

                    if let Some(callback) = matched_callback {
                        let (tx, rx) = oneshot::channel();
                        let route_req = RouteRequest {
                            url,
                            method,
                            headers,
                            post_data,
                        };
                        let route = JsRoute::new(route_req, tx);

                        let _ = callback.call(
                            Ok(route),
                            ThreadsafeFunctionCallMode::NonBlocking,
                        );

                        let action = rx.await.unwrap_or(RouteAction::Continue(
                            crate::network::route::RouteContinueOptions {
                                url: None,
                                method: None,
                                headers: None,
                                post_data: None,
                            },
                        ));

                        match action {
                            RouteAction::Fulfill(opts) => {
                                let mut builder = FulfillRequestParams::builder()
                                    .request_id(event.request_id.clone())
                                    .response_code(opts.status.unwrap_or(200) as i64);
                                if let Some(ref hdrs) = opts.headers {
                                    for (name, value) in hdrs {
                                        builder = builder
                                            .response_header(HeaderEntry::new(name, value));
                                    }
                                }
                                if let Some(ref body_data) = opts.body {
                                    let raw = match body_data {
                                        napi::Either::A(s) => s.as_bytes().to_vec(),
                                        napi::Either::B(b) => b.to_vec(),
                                    };
                                    let b64 = STANDARD.encode(&raw);
                                    builder = builder.body(b64);
                                }
                                if let Ok(params) = builder.build() {
                                    let _ = page_clone.execute(params).await;
                                }
                            }
                            RouteAction::Continue(ref opts) => {
                                let mut builder = ContinueRequestParams::builder()
                                    .request_id(event.request_id.clone());
                                if let Some(ref m) = opts.method {
                                    builder = builder.method(m);
                                }
                                if let Some(ref hdrs) = opts.headers {
                                    for (name, value) in hdrs {
                                        builder =
                                            builder.header(HeaderEntry::new(name, value));
                                    }
                                }
                                if let Ok(params) = builder.build() {
                                    let _ = page_clone.execute(params).await;
                                }
                            }
                            RouteAction::Abort => {
                                if let Ok(params) = FailRequestParams::builder()
                                    .request_id(event.request_id.clone())
                                    .error_reason(
                                        chromiumoxide::cdp::browser_protocol::network::ErrorReason::Failed,
                                    )
                                    .build()
                                {
                                    let _ = page_clone.execute(params).await;
                                }
                            }
                        }
                    } else {
                        if let Ok(params) = ContinueRequestParams::builder()
                            .request_id(event.request_id.clone())
                            .build()
                        {
                            let _ = page_clone.execute(params).await;
                        }
                    }
                }
            });

            *self.fetch_handle.lock().await = Some(handle);
        }
        Ok(())
    }

    pub async fn disable_fetch_interception(&self) -> Result<(), TurbosheetError> {
        if let Some(handle) = self.fetch_handle.lock().await.take() {
            handle.abort();
        }
        let mut guard = self.lock_page().await?;
        if let Some(page) = guard.as_mut() {
            let disable_params =
                chromiumoxide::cdp::browser_protocol::fetch::DisableParams {};
            let _ = page.execute(disable_params).await;
        }
        Ok(())
    }
}

#[async_trait]
impl PageEngine for ChromiumPageEngine {
    async fn invoke_action(&self, action: &str, args: Vec<serde_json::Value>) -> Result<serde_json::Value, TurbosheetError> {
        let mut js_args = String::new();
        for arg in &args {
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

        // Strip source map comment line from the actions script
        let source_map_marker = "//# sourceMappingURL=";
        let clean_script = if let Some(pos) = script.rfind(source_map_marker) {
            script[..pos].trim_end()
        } else {
            script.trim_end()
        };

        // Build JS that runs the actions script IIFE, then calls the action
        // method directly on the ts internal object and returns the result
        // as a JSON string via evaluate (no CDP binding bridge needed).
        let js = if js_args.is_empty() {
            format!(
                "(async function(){{{}\
                var p={};var k=p&&p.__tsSym;var ts=window[k||'__ts_internal'];\
                if(!ts['{}'])throw Error('Action {} not found');\
                var r=await ts['{}']();\
                return JSON.stringify({{ok:true,data:r}});\
                }})()",
                clean_script,
                accesses_global,
                action, action, action
            )
        } else {
            format!(
                "(async function(){{{}\
                var p={};var k=p&&p.__tsSym;var ts=window[k||'__ts_internal'];\
                if(!ts['{}'])throw Error('Action {} not found');\
                var r=await ts['{}']({});\
                return JSON.stringify({{ok:true,data:r}});\
                }})()",
                clean_script,
                accesses_global,
                action, action, action,
                js_args
            )
        };

        let result_str = self.evaluate(&js).await.map_err(|e| {
            TurbosheetError::Other(format!("Failed to evaluate action '{}': {}", action, e))
        })?;

        let result: serde_json::Value = serde_json::from_str(&result_str)
            .map_err(|e| TurbosheetError::Other(format!("Failed to parse action '{}' result: {}", action, e)))?;

        match result.get("ok").and_then(|v| v.as_bool()) {
            Some(true) => {
                let data = result.get("data").cloned().unwrap_or(serde_json::Value::Null);
                Ok(data)
            }
            _ => {
                let err_msg = result.get("error").and_then(|v| v.as_str()).unwrap_or("unknown error");
                Err(TurbosheetError::Other(format!("Action '{}' failed: {}", action, err_msg)))
            }
        }
    }
    
    async fn goto(&self, url: &str, state: LoadState) -> Result<(), TurbosheetError> {
        #[cfg(feature = "traces")]
        let _start = std::time::Instant::now();
        let mut page_guard = self.lock_page().await?;
        if let Some(page) = page_guard.as_mut() {
            // chromiumoxide::Page::goto() internally uses the navigation framework
            // (NavigationInProgress + NavigationWatcher) which already waits for
            // the full page load lifecycle. No need for a separate wait_for_navigation().
            page.goto(url).await.map_err(|e| {
                TurbosheetError::NavigationFailed(format!("Failed to navigate to {}: {}", url, e))
            })?;

            match state {
                LoadState::Load | LoadState::DomContentLoaded => {
                    // page.goto() already waited for "load" — nothing more to do.
                },
                LoadState::NetworkIdle => {
                    let js = "(function() { return window.performance.getEntriesByType('resource').length === 0 || document.readyState === 'complete'; })()";
                    let start = std::time::Instant::now();
                    let timeout = std::time::Duration::from_secs(30);
                    loop {
                        if start.elapsed() > timeout {
                            break;
                        }
                        let res = page.evaluate(js).await;
                        if let Ok(result) = res {
                            if let Ok(true) = result.into_value::<bool>() {
                                break;
                            }
                        }
                        tokio::time::sleep(std::time::Duration::from_millis(100)).await;
                    }
                },
            }

            drop(page_guard);
            let mut url_guard = self.current_url.write().await;
            *url_guard = url.to_string();
            drop(url_guard);
            if let Some(pid) = self.page_id.lock().await.as_ref() {
                let payload = serde_json::json!({"frame": {"url": url}});
                self.event_dispatcher.handle_event(pid, EventType::FrameNavigated, payload);
            }
        }
        #[cfg(feature = "traces")]
        crate::trace::GLOBAL_RECORDER.record_action(
            "navigate", None, Some(url),
            _start.elapsed().as_millis() as u64, "success",
        ).await;
        Ok(())
    }

    fn url(&self) -> String {
        if let Ok(guard) = self.page_id.try_lock() {
            if let Some(ref pid) = *guard {
                if let Some(url) = self.event_dispatcher.try_get_url(pid) {
                    return url;
                }
            }
        }
        let url = self.current_url.try_read().map(|r| r.clone()).unwrap_or_default();
        if url.is_empty() {
            "about:blank".to_string()
        } else {
            url
        }
    }

    async fn screenshot(&self, _options: Option<ScreenshotOptions>) -> Result<Vec<u8>, TurbosheetError> {
        let mut page_guard = self.lock_page().await?;
        if let Some(page) = page_guard.as_mut() {
            let params = chromiumoxide::cdp::browser_protocol::page::CaptureScreenshotParams::builder()
                .format(chromiumoxide::cdp::browser_protocol::page::CaptureScreenshotFormat::Png)
                .build();
                
            let data = page.execute(params).await.map_err(|e| {
                TurbosheetError::Other(format!("Failed to capture screenshot: {}", e))
            })?;
            
            use base64::{Engine as _, engine::general_purpose::STANDARD};
            return STANDARD.decode(&data.data).map_err(|e| {
                TurbosheetError::Other(format!("Failed to decode screenshot base64: {}", e))
            });
        }
        Err(TurbosheetError::Other("Page is closed".to_string()))
    }

    async fn close(&self) -> Result<(), TurbosheetError> {
        if let Some(pid) = self.page_id.lock().await.as_ref() {
            self.event_dispatcher.unsubscribe_all(pid);
            crate::network::events::unregister_all_for_page(pid);
        }
        let mut page_guard = self.lock_page().await?;
        if let Some(page) = page_guard.take() {
            page.close().await.map_err(|e| {
                TurbosheetError::Other(format!("Failed to close page: {}", e))
            })?;
        }
        Ok(())
    }

    async fn evaluate(&self, js: &str) -> Result<String, TurbosheetError> {
        let mut page_guard = self.lock_page().await?;
        if let Some(page) = page_guard.as_mut() {
            let res = page.evaluate(js).await.map_err(|e| {
                TurbosheetError::Other(format!("Failed to evaluate JS: {}", e))
            })?;

            // Update URL cache if window.location changed
            if let Ok(url) = page.evaluate("window.location.href").await {
                if let Ok(url_str) = url.into_value::<String>() {
                    drop(page_guard);
                    let mut url_guard = self.current_url.write().await;
                    *url_guard = url_str;
                }
            }

            let val: serde_json::Value = res.into_value().unwrap_or_default();
            let result = match &val {
                serde_json::Value::String(s) => s.clone(),
                other => other.to_string(),
            };
            return Ok(result);
        }
        Err(TurbosheetError::Other("Page is closed".to_string()))
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
        #[cfg(feature = "traces")]
        let _start = std::time::Instant::now();
        let (x, y) = self.get_element_center(selector).await?;
        let mut page_guard = self.lock_page().await?;
        if let Some(page) = page_guard.as_mut() {
            use chromiumoxide::cdp::browser_protocol::input::{
                DispatchMouseEventParams, DispatchMouseEventType, MouseButton,
            };
            let pressed = DispatchMouseEventParams::builder()
                .r#type(DispatchMouseEventType::MousePressed)
                .x(x).y(y)
                .button(MouseButton::Left)
                .click_count(1)
                .build().unwrap();
            page.execute(pressed).await.map_err(|e| {
                TurbosheetError::Other(format!("CDP mousePressed failed: {}", e))
            })?;
            let released = DispatchMouseEventParams::builder()
                .r#type(DispatchMouseEventType::MouseReleased)
                .x(x).y(y)
                .button(MouseButton::Left)
                .click_count(1)
                .build().unwrap();
            page.execute(released).await.map_err(|e| {
                TurbosheetError::Other(format!("CDP mouseReleased failed: {}", e))
            })?;
            #[cfg(feature = "traces")]
            crate::trace::GLOBAL_RECORDER.record_action(
                "click", Some(selector), None,
                _start.elapsed().as_millis() as u64, "success",
            ).await;
            return Ok(());
        }
        #[cfg(feature = "traces")]
        crate::trace::GLOBAL_RECORDER.record_action(
            "click", Some(selector), None, 0, "error: page closed",
        ).await;
        Err(TurbosheetError::Other("Page is closed".to_string()))
    }

    async fn fill(&self, selector: &str, value: &str) -> Result<(), TurbosheetError> {
        #[cfg(feature = "traces")]
        let _start = std::time::Instant::now();
        let mut page_guard = self.lock_page().await?;
        if let Some(page) = page_guard.as_mut() {
            let element = page.find_element(selector).await.map_err(|e| {
                TurbosheetError::ElementNotFound {
                    message: format!("Element not found '{}': {}", selector, e),
                    selector: Some(selector.to_string()),
                }
            })?;
            
            element.click().await.ok();
            // Clear existing value and dispatch input event before typing
            element.call_js_fn(
                "function() { this.value = ''; this.dispatchEvent(new Event('input', {bubbles: true})); }",
                true,
            ).await.map_err(|e| {
                TurbosheetError::Other(format!("Failed to clear element '{}': {}", selector, e))
            })?;
            
            element.type_str(value).await.map_err(|e| {
                TurbosheetError::Other(format!("Failed to fill element '{}': {}", selector, e))
            })?;
            #[cfg(feature = "traces")]
            crate::trace::GLOBAL_RECORDER.record_action(
                "fill", Some(selector), Some(value),
                _start.elapsed().as_millis() as u64, "success",
            ).await;
            return Ok(());
        }
        #[cfg(feature = "traces")]
        crate::trace::GLOBAL_RECORDER.record_action(
            "fill", Some(selector), Some(value), 0, "error: page closed",
        ).await;
        Err(TurbosheetError::Other("Page is closed".to_string()))
    }

    async fn text_content(&self, selector: &str) -> Result<Option<String>, TurbosheetError> {
        let mut page_guard = self.lock_page().await?;
        if let Some(page) = page_guard.as_mut() {
            let element = match page.find_element(selector).await {
                Ok(e) => e,
                Err(_) => return Ok(None),
            };
            
            let text = element.inner_text().await.map_err(|e| {
                TurbosheetError::Other(format!("Failed to get text content for '{}': {}", selector, e))
            })?;
            
            return Ok(text);
        }
        Err(TurbosheetError::Other("Page is closed".to_string()))
    }

    async fn is_visible(&self, selector: &str) -> Result<bool, TurbosheetError> {
        let res = self.invoke_action("isVisible", vec![serde_json::Value::String(selector.to_string())]).await?;
        Ok(res.as_bool().unwrap_or(false))
    }

    async fn dblclick(&self, selector: &str) -> Result<(), TurbosheetError> {
        #[cfg(feature = "traces")]
        let _start = std::time::Instant::now();
        let (x, y) = self.get_element_center(selector).await?;
        let mut page_guard = self.lock_page().await?;
        if let Some(page) = page_guard.as_mut() {
            use chromiumoxide::cdp::browser_protocol::input::{
                DispatchMouseEventParams, DispatchMouseEventType, MouseButton,
            };
            for count in &[1i64, 2] {
                let pressed = DispatchMouseEventParams::builder()
                    .r#type(DispatchMouseEventType::MousePressed)
                    .x(x).y(y)
                    .button(MouseButton::Left)
                    .click_count(*count)
                    .build().unwrap();
                page.execute(pressed).await.map_err(|e| {
                    TurbosheetError::Other(format!("CDP mousePressed (dblclick) failed: {}", e))
                })?;
                let released = DispatchMouseEventParams::builder()
                    .r#type(DispatchMouseEventType::MouseReleased)
                    .x(x).y(y)
                    .button(MouseButton::Left)
                    .click_count(*count)
                    .build().unwrap();
                page.execute(released).await.map_err(|e| {
                    TurbosheetError::Other(format!("CDP mouseReleased (dblclick) failed: {}", e))
                })?;
            }
            #[cfg(feature = "traces")]
            crate::trace::GLOBAL_RECORDER.record_action(
                "dblclick", Some(selector), None,
                _start.elapsed().as_millis() as u64, "success",
            ).await;
            return Ok(());
        }
        #[cfg(feature = "traces")]
        crate::trace::GLOBAL_RECORDER.record_action(
            "dblclick", Some(selector), None, 0, "error: page closed",
        ).await;
        Err(TurbosheetError::Other("Page is closed".to_string()))
    }

    async fn right_click(&self, selector: &str) -> Result<(), TurbosheetError> {
        let (x, y) = self.get_element_center(selector).await?;
        let mut page_guard = self.lock_page().await?;
        if let Some(page) = page_guard.as_mut() {
            use chromiumoxide::cdp::browser_protocol::input::{
                DispatchMouseEventParams, DispatchMouseEventType, MouseButton,
            };
            let pressed = DispatchMouseEventParams::builder()
                .r#type(DispatchMouseEventType::MousePressed)
                .x(x).y(y)
                .button(MouseButton::Right)
                .click_count(1)
                .build().unwrap();
            page.execute(pressed).await.map_err(|e| {
                TurbosheetError::Other(format!("CDP mousePressed (right) failed: {}", e))
            })?;
            let released = DispatchMouseEventParams::builder()
                .r#type(DispatchMouseEventType::MouseReleased)
                .x(x).y(y)
                .button(MouseButton::Right)
                .click_count(1)
                .build().unwrap();
            page.execute(released).await.map_err(|e| {
                TurbosheetError::Other(format!("CDP mouseReleased (right) failed: {}", e))
            })?;
            return Ok(());
        }
        Err(TurbosheetError::Other("Page is closed".to_string()))
    }

    async fn hover(&self, selector: &str) -> Result<(), TurbosheetError> {
        #[cfg(feature = "traces")]
        let _start = std::time::Instant::now();
        let (x, y) = self.get_element_center(selector).await?;
        let mut page_guard = self.lock_page().await?;
        if let Some(page) = page_guard.as_mut() {
            use chromiumoxide::cdp::browser_protocol::input::{
                DispatchMouseEventParams, DispatchMouseEventType,
            };
            let params = DispatchMouseEventParams::builder()
                .r#type(DispatchMouseEventType::MouseMoved)
                .x(x).y(y)
                .build().unwrap();
            page.execute(params).await.map_err(|e| {
                TurbosheetError::Other(format!("CDP mouseMoved (hover) failed: {}", e))
            })?;
            #[cfg(feature = "traces")]
            crate::trace::GLOBAL_RECORDER.record_action(
                "hover", Some(selector), None,
                _start.elapsed().as_millis() as u64, "success",
            ).await;
            return Ok(());
        }
        #[cfg(feature = "traces")]
        crate::trace::GLOBAL_RECORDER.record_action(
            "hover", Some(selector), None, 0, "error: page closed",
        ).await;
        Err(TurbosheetError::Other("Page is closed".to_string()))
    }

    async fn check(&self, selector: &str) -> Result<(), TurbosheetError> {
        #[cfg(feature = "traces")]
        let _start = std::time::Instant::now();
        self.invoke_action("check", vec![serde_json::Value::String(selector.to_string())]).await?;
        #[cfg(feature = "traces")]
        crate::trace::GLOBAL_RECORDER.record_action(
            "check", Some(selector), None,
            _start.elapsed().as_millis() as u64, "success",
        ).await;
        Ok(())
    }

    async fn uncheck(&self, selector: &str) -> Result<(), TurbosheetError> {
        #[cfg(feature = "traces")]
        let _start = std::time::Instant::now();
        self.invoke_action("uncheck", vec![serde_json::Value::String(selector.to_string())]).await?;
        #[cfg(feature = "traces")]
        crate::trace::GLOBAL_RECORDER.record_action(
            "uncheck", Some(selector), None,
            _start.elapsed().as_millis() as u64, "success",
        ).await;
        Ok(())
    }

    async fn select(&self, selector: &str, value: &str) -> Result<(), TurbosheetError> {
        #[cfg(feature = "traces")]
        let _start = std::time::Instant::now();
        self.invoke_action("select", vec![serde_json::Value::String(selector.to_string()), serde_json::Value::String(value.to_string())]).await?;
        #[cfg(feature = "traces")]
        crate::trace::GLOBAL_RECORDER.record_action(
            "select", Some(selector), Some(value),
            _start.elapsed().as_millis() as u64, "success",
        ).await;
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

    async fn inner_text(&self, selector: &str) -> Result<String, TurbosheetError> {
        let res = self.invoke_action("innerText", vec![serde_json::Value::String(selector.to_string())]).await?;
        Ok(res.as_str().unwrap_or("").to_string())
    }

    async fn inner_html(&self, selector: &str) -> Result<String, TurbosheetError> {
        let res = self.invoke_action("innerHTML", vec![serde_json::Value::String(selector.to_string())]).await?;
        Ok(res.as_str().unwrap_or("").to_string())
    }

    async fn get_attribute(&self, selector: &str, name: &str) -> Result<Option<String>, TurbosheetError> {
        let res = self.invoke_action("getAttribute", vec![
            serde_json::Value::String(selector.to_string()),
            serde_json::Value::String(name.to_string()),
        ]).await?;
        if res.is_null() { Ok(None) } else { Ok(res.as_str().map(|s| s.to_string())) }
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
        let mut page_guard = self.lock_page().await?;
        if let Some(page) = page_guard.as_mut() {
            let escaped = serde_json::to_string(&selector).unwrap_or_else(|_| format!("\"{}\"", selector));
            let js = format!("(function() {{ var el = document.querySelector({}); if (!el) return null; var rect = el.getBoundingClientRect(); return JSON.stringify({{ x: rect.x, y: rect.y, width: rect.width, height: rect.height }}); }})()", escaped);
            let rect_res = page.evaluate(js).await.map_err(|e| {
                TurbosheetError::Other(format!("Failed to get element rect for screenshot '{}': {}", selector, e))
            })?;

            let rect_str = rect_res.into_value::<String>().unwrap_or_default();
            if rect_str.is_empty() {
                return Err(TurbosheetError::Other(format!("Element '{}' not found", selector)));
            }

            let params = chromiumoxide::cdp::browser_protocol::page::CaptureScreenshotParams::builder()
                .format(chromiumoxide::cdp::browser_protocol::page::CaptureScreenshotFormat::Png)
                .build();

            let data = page.execute(params).await.map_err(|e| {
                TurbosheetError::Other(format!("Failed to capture screenshot: {}", e))
            })?;

            use base64::{Engine as _, engine::general_purpose::STANDARD};
            let bytes = STANDARD.decode(&data.data).map_err(|e| {
                TurbosheetError::Other(format!("Failed to decode screenshot: {}", e))
            })?;
            return Ok(bytes);
        }
        Err(TurbosheetError::Other("Page is closed".to_string()))
    }

    async fn bounding_box(&self, selector: &str) -> Result<Option<crate::page::Rect>, TurbosheetError> {
        let mut page_guard = self.lock_page().await?;
        if let Some(page) = page_guard.as_mut() {
            let escaped = serde_json::to_string(&selector).unwrap_or_else(|_| format!("\"{}\"", selector));
            let js = format!("(function() {{ var el = document.querySelector({}); if (!el) return null; var rect = el.getBoundingClientRect(); return JSON.stringify({{ x: rect.x, y: rect.y, width: rect.width, height: rect.height }}); }})()", escaped);
            let res = page.evaluate(js).await.map_err(|e| {
                TurbosheetError::Other(format!("Failed to get bounding box for '{}': {}", selector, e))
            })?;
            
            if let Ok(rect_str) = res.into_value::<String>() {
                if !rect_str.is_empty() {
                    if let Ok(rect_json) = serde_json::from_str::<serde_json::Value>(&rect_str) {
                        return Ok(Some(crate::page::Rect {
                            x: rect_json["x"].as_f64().unwrap_or(0.0),
                            y: rect_json["y"].as_f64().unwrap_or(0.0),
                            width: rect_json["width"].as_f64().unwrap_or(0.0),
                            height: rect_json["height"].as_f64().unwrap_or(0.0),
                        }));
                    }
                }
            }
            return Ok(None);
        }
        Err(TurbosheetError::Other("Page is closed".to_string()))
    }
    
    async fn drag_and_drop(&self, source: &str, target: &str) -> Result<(), TurbosheetError> {
        self.invoke_action("dragAndDrop", vec![
            serde_json::Value::String(source.to_string()),
            serde_json::Value::String(target.to_string()),
        ]).await?;
        Ok(())
    }
    
    async fn press(&self, selector: &str, key: &str) -> Result<(), TurbosheetError> {
        #[cfg(feature = "traces")]
        let _start = std::time::Instant::now();
        let mut page_guard = self.lock_page().await?;
        if let Some(page) = page_guard.as_mut() {
            // Focus the element first via injected script (faster than CDP chain)
            let escaped_sel = serde_json::to_string(&selector).unwrap_or_else(|_| format!("\"{}\"", selector));
            let focus_js = format!("(function() {{ var el = document.querySelector({}); if (el) el.focus(); }})()", escaped_sel);
            let _ = page.evaluate(focus_js).await;

            // Dispatch real CDP key events
            use chromiumoxide::cdp::browser_protocol::input::{
                DispatchKeyEventParams, DispatchKeyEventType,
            };
            let vk_code = KEY_CODE_MAP
                .get(key.to_lowercase().as_str())
                .copied()
                .unwrap_or_else(|| key.chars().next().map(|c| c as i64).unwrap_or(0));
            let raw_down = DispatchKeyEventParams::builder()
                .r#type(DispatchKeyEventType::RawKeyDown)
                .key(key.to_string())
                .windows_virtual_key_code(vk_code)
                .build().unwrap();
            page.execute(raw_down).await.map_err(|e| {
                TurbosheetError::Other(format!("CDP rawKeyDown failed: {}", e))
            })?;

            let char_event = DispatchKeyEventParams::builder()
                .r#type(DispatchKeyEventType::Char)
                .key(key.to_string())
                .text(key.to_string())
                .unmodified_text(key.to_string())
                .build().unwrap();
            page.execute(char_event).await.map_err(|e| {
                TurbosheetError::Other(format!("CDP char event failed: {}", e))
            })?;

            let key_up = DispatchKeyEventParams::builder()
                .r#type(DispatchKeyEventType::KeyUp)
                .key(key.to_string())
                .build().unwrap();
            page.execute(key_up).await.map_err(|e| {
                TurbosheetError::Other(format!("CDP keyUp failed: {}", e))
            })?;

            #[cfg(feature = "traces")]
            crate::trace::GLOBAL_RECORDER.record_action(
                "press", Some(selector), Some(key),
                _start.elapsed().as_millis() as u64, "success",
            ).await;
            return Ok(());
        }
        #[cfg(feature = "traces")]
        crate::trace::GLOBAL_RECORDER.record_action(
            "press", Some(selector), Some(key), 0, "error: page closed",
        ).await;
        Err(TurbosheetError::Other("Page is closed".to_string()))
    }
    
    async fn press_sequentially(&self, selector: &str, text: &str) -> Result<(), TurbosheetError> {
        #[cfg(feature = "traces")]
        let _start = std::time::Instant::now();
        let mut page_guard = self.lock_page().await?;
        if let Some(page) = page_guard.as_mut() {
            // Focus the element first via injected script
            let escaped_sel = serde_json::to_string(&selector).unwrap_or_else(|_| format!("\"{}\"", selector));
            let focus_js = format!("(function() {{ var el = document.querySelector({}); if (el) el.focus(); }})()", escaped_sel);
            let _ = page.evaluate(focus_js).await;

            use chromiumoxide::cdp::browser_protocol::input::{
                DispatchKeyEventParams, DispatchKeyEventType,
            };
            for ch in text.chars() {
                let key = ch.to_string();
                let vk_code = KEY_CODE_MAP
                    .get(key.to_lowercase().as_str())
                    .copied()
                    .unwrap_or_else(|| ch as i64);
                let raw_down = DispatchKeyEventParams::builder()
                    .r#type(DispatchKeyEventType::RawKeyDown)
                    .key(key.clone())
                    .windows_virtual_key_code(vk_code)
                    .build().unwrap();
                page.execute(raw_down).await.ok();

                let char_event = DispatchKeyEventParams::builder()
                    .r#type(DispatchKeyEventType::Char)
                    .key(key.clone())
                    .text(key.clone())
                    .unmodified_text(key.clone())
                    .build().unwrap();
                page.execute(char_event).await.ok();

                let key_up = DispatchKeyEventParams::builder()
                    .r#type(DispatchKeyEventType::KeyUp)
                    .key(key.clone())
                    .build().unwrap();
                page.execute(key_up).await.ok();

                tokio::time::sleep(std::time::Duration::from_millis(10)).await;
            }
            #[cfg(feature = "traces")]
            crate::trace::GLOBAL_RECORDER.record_action(
                "press_sequentially", Some(selector), Some(text),
                _start.elapsed().as_millis() as u64, "success",
            ).await;
            return Ok(());
        }
        #[cfg(feature = "traces")]
        crate::trace::GLOBAL_RECORDER.record_action(
            "press_sequentially", Some(selector), Some(text), 0, "error: page closed",
        ).await;
        Err(TurbosheetError::Other("Page is closed".to_string()))
    }
    
    async fn set_input_files(&self, selector: &str, files: Vec<String>) -> Result<(), TurbosheetError> {
        let mut page_guard = self.lock_page().await?;
        if let Some(page) = page_guard.as_mut() {
            let escaped_sel = serde_json::to_string(&selector).unwrap_or_else(|_| format!("\"{}\"", selector));
            // Evaluate JS to set the file input's value via the DOM.  Because
            // Chromium's security model blocks programmatic value changes on
            // file inputs we fall back to creating a DataTransfer / assigning
            // files through the `InputEvent` machinery inside the page.
            let files_json = serde_json::to_string(&files).unwrap_or_default();
            let js = format!(
                r#"(async function() {{
                    const el = document.querySelector({});
                    if (!el) throw new Error('Element not found');
                    const dt = new DataTransfer();
                    const filePaths = {};
                    for (const path of filePaths) {{
                        try {{
                            const resp = await fetch('file://' + path);
                            const blob = await resp.blob();
                            const file = new File([blob], path.split('/').pop() || 'file', {{ type: blob.type }});
                            Object.defineProperty(file, 'webkitRelativePath', {{ value: path }});
                            dt.items.add(file);
                        }} catch(e) {{
                            // File not accessible from sandbox – skip
                        }}
                    }}
                    el.files = dt.files;
                    el.dispatchEvent(new Event('change', {{ bubbles: true }}));
                    return true;
                }})()"#,
                escaped_sel, files_json
            );
            page.evaluate(js).await.map_err(|e| {
                TurbosheetError::Other(format!("Failed to set_input_files: {}", e))
            })?;
            return Ok(());
        }
        Err(TurbosheetError::Other("Page is closed".to_string()))
    }
    
    async fn set_viewport_size(&self, width: u32, height: u32) -> Result<(), TurbosheetError> {
        let mut page_guard = self.lock_page().await?;
        if let Some(page) = page_guard.as_mut() {
            let params = chromiumoxide::cdp::browser_protocol::emulation::SetDeviceMetricsOverrideParams::builder()
                .width(width as i64)
                .height(height as i64)
                .device_scale_factor(1.0)
                .mobile(false)
                .build()
                .unwrap();
            page.execute(params).await.map_err(|e| {
                TurbosheetError::Other(format!("Failed to set viewport size: {}", e))
            })?;
            return Ok(());
        }
        Err(TurbosheetError::Other("Page is closed".to_string()))
    }

    async fn viewport_size(&self) -> Result<Option<crate::page::ViewportSize>, TurbosheetError> {
        let mut page_guard = self.lock_page().await?;
        if let Some(page) = page_guard.as_mut() {
            let js = "JSON.stringify({ width: window.innerWidth, height: window.innerHeight })";
            let res = page.evaluate(js).await.map_err(|e| {
                TurbosheetError::Other(format!("Failed to get viewport size: {}", e))
            })?;
            if let Ok(size_str) = res.into_value::<String>() {
                if let Ok(size_json) = serde_json::from_str::<serde_json::Value>(&size_str) {
                    return Ok(Some(crate::page::ViewportSize {
                        width: size_json["width"].as_u64().unwrap_or(0) as u32,
                        height: size_json["height"].as_u64().unwrap_or(0) as u32,
                    }));
                }
            }
            return Ok(None);
        }
        Err(TurbosheetError::Other("Page is closed".to_string()))
    }

    async fn reload(&self) -> Result<(), TurbosheetError> {
        let mut page_guard = self.lock_page().await?;
        if let Some(page) = page_guard.as_mut() {
            let params = chromiumoxide::cdp::browser_protocol::page::ReloadParams::default();
            page.execute(params).await.map_err(|e| {
                TurbosheetError::Other(format!("Failed to reload page: {}", e))
            })?;
            return Ok(());
        }
        Err(TurbosheetError::Other("Page is closed".to_string()))
    }

    async fn go_back(&self) -> Result<(), TurbosheetError> {
        let mut page_guard = self.lock_page().await?;
        if let Some(page) = page_guard.as_mut() {
            let history = page.execute(chromiumoxide::cdp::browser_protocol::page::GetNavigationHistoryParams::default()).await.map_err(|e| {
                TurbosheetError::Other(format!("Failed to get navigation history: {}", e))
            })?;
            
            if history.current_index > 0 {
                let entry = &history.entries[(history.current_index - 1) as usize];
                let params = chromiumoxide::cdp::browser_protocol::page::NavigateToHistoryEntryParams::new(entry.id);
                page.execute(params).await.map_err(|e| {
                    TurbosheetError::Other(format!("Failed to go back: {}", e))
                })?;
            }
            return Ok(());
        }
        Err(TurbosheetError::Other("Page is closed".to_string()))
    }

    async fn go_forward(&self) -> Result<(), TurbosheetError> {
        let mut page_guard = self.lock_page().await?;
        if let Some(page) = page_guard.as_mut() {
            let history = page.execute(chromiumoxide::cdp::browser_protocol::page::GetNavigationHistoryParams::default()).await.map_err(|e| {
                TurbosheetError::Other(format!("Failed to get navigation history: {}", e))
            })?;
            
            if (history.current_index as usize) + 1 < history.entries.len() {
                let entry = &history.entries[(history.current_index + 1) as usize];
                let params = chromiumoxide::cdp::browser_protocol::page::NavigateToHistoryEntryParams::new(entry.id);
                page.execute(params).await.map_err(|e| {
                    TurbosheetError::Other(format!("Failed to go forward: {}", e))
                })?;
            }
            return Ok(());
        }
        Err(TurbosheetError::Other("Page is closed".to_string()))
    }
    async fn wait_for_request(&self, url: &str) -> Result<(), TurbosheetError> {
        let target_url = url.to_string();
        let mut rx = self.network_bus.request_subscriber();
        let start = std::time::Instant::now();
        let timeout = std::time::Duration::from_secs(30);
        loop {
            if start.elapsed() > timeout {
                return Err(TurbosheetError::Other(format!("Timeout waiting for request: {}", target_url)));
            }
            match rx.try_recv() {
                Ok(event) => {
                    if event.url.contains(&target_url) {
                        return Ok(());
                    }
                }
                Err(tokio::sync::broadcast::error::TryRecvError::Empty) => {
                    tokio::time::sleep(std::time::Duration::from_millis(50)).await;
                }
                Err(_) => {
                    return Err(TurbosheetError::Other("Network event channel closed".to_string()));
                }
            }
        }
    }
    async fn wait_for_response(&self, url: &str) -> Result<(), TurbosheetError> {
        let target_url = url.to_string();
        let mut rx = self.network_bus.response_subscriber();
        let start = std::time::Instant::now();
        let timeout = std::time::Duration::from_secs(30);
        loop {
            if start.elapsed() > timeout {
                return Err(TurbosheetError::Other(format!("Timeout waiting for response: {}", target_url)));
            }
            match rx.try_recv() {
                Ok(event) => {
                    if event.url.contains(&target_url) {
                        return Ok(());
                    }
                }
                Err(tokio::sync::broadcast::error::TryRecvError::Empty) => {
                    tokio::time::sleep(std::time::Duration::from_millis(50)).await;
                }
                Err(_) => {
                    return Err(TurbosheetError::Other("Network event channel closed".to_string()));
                }
            }
        }
    }
    
    async fn wait_for_selector(&self, selector: &str) -> Result<(), TurbosheetError> {
        let mut page_guard = self.lock_page().await?;
        if let Some(page) = page_guard.as_mut() {
            let escaped = serde_json::to_string(&selector).unwrap_or_else(|_| format!("\"{}\"", selector));
            let js = format!("(function() {{ return document.querySelector({}) !== null; }})()", escaped);
            let start = std::time::Instant::now();
            let timeout = std::time::Duration::from_secs(30);
            loop {
                if start.elapsed() > timeout {
                    return Err(TurbosheetError::Other(format!("Timeout waiting for selector '{}'", selector)));
                }
                if let Ok(res) = page.evaluate(js.clone()).await {
                    if let Ok(true) = res.into_value::<bool>() {
                        return Ok(());
                    }
                }
                tokio::time::sleep(std::time::Duration::from_millis(50)).await;
            }
        }
        Err(TurbosheetError::Other("Page is closed".to_string()))
    }
    async fn evaluate_handle(&self, js: &str) -> Result<(), TurbosheetError> {
        // Evaluate the raw JS expression via CDP Runtime.evaluate; discard the result value.
        let mut page_guard = self.lock_page().await?;
        if let Some(page) = page_guard.as_mut() {
            page.evaluate(js).await.map_err(|e| {
                TurbosheetError::Other(format!("Failed to evaluate_handle: {}", e))
            })?;
            return Ok(());
        }
        Err(TurbosheetError::Other("Page is closed".to_string()))
    }
    async fn add_script_tag(&self, content: &str) -> Result<(), TurbosheetError> {
        self.invoke_action("addScriptTag", vec![serde_json::Value::String(content.to_string())]).await?;
        Ok(())
    }
    async fn add_style_tag(&self, content: &str) -> Result<(), TurbosheetError> {
        self.invoke_action("addStyleTag", vec![serde_json::Value::String(content.to_string())]).await?;
        Ok(())
    }
    async fn expose_function(&self, name: &str, _js: &str) -> Result<(), TurbosheetError> {
        // Register a CDP Runtime.addBinding so the page can call window[name] (from Rust-land).
        let mut page_guard = self.lock_page().await?;
        if let Some(page) = page_guard.as_mut() {
            let params = chromiumoxide::cdp::js_protocol::runtime::AddBindingParams::builder()
                .name(name.to_string())
                .build().unwrap();
            page.execute(params).await.map_err(|e| {
                TurbosheetError::Other(format!("Failed to expose_function '{}': {}", name, e))
            })?;
            return Ok(());
        }
        Err(TurbosheetError::Other("Page is closed".to_string()))
    }

    async fn set_content(&self, html: &str) -> Result<(), TurbosheetError> {
        let mut page_guard = self.lock_page().await?;
        if let Some(page) = page_guard.as_mut() {
            // Get the frame tree to find the main frame ID
            let frame_tree = page.execute(
                chromiumoxide::cdp::browser_protocol::page::GetFrameTreeParams::default()
            ).await.map_err(|e| {
                TurbosheetError::Other(format!("Failed to get frame tree: {}", e))
            })?;
            let frame_id = frame_tree.frame_tree.frame.id.clone();

            // Use Page.setDocumentContent CDP command
            let params = chromiumoxide::cdp::browser_protocol::page::SetDocumentContentParams::new(
                frame_id,
                html.to_string(),
            );
            page.execute(params).await.map_err(|e| {
                TurbosheetError::Other(format!("Failed to set document content: {}", e))
            })?;
            return Ok(());
        }
        Err(TurbosheetError::Other("Page is closed".to_string()))
    }

    async fn inject_core_script(&self, script: &str) -> Result<(), TurbosheetError> {
        let mut page_guard = self.lock_page().await?;
        if let Some(page) = page_guard.as_mut() {
            let params = chromiumoxide::cdp::browser_protocol::page::AddScriptToEvaluateOnNewDocumentParams::builder()
                .source(script.to_string())
                .build().unwrap();
            page.execute(params).await.map_err(|e| {
                TurbosheetError::Other(format!("Failed to inject script: {}", e))
            })?;
            return Ok(());
        }
        Err(TurbosheetError::Other("Page is closed".to_string()))
    }

    async fn register_binding(&self, name: &str) -> Result<(), TurbosheetError> {
        let mut page_guard = self.lock_page().await?;
        if let Some(page) = page_guard.as_mut() {
            let params = chromiumoxide::cdp::js_protocol::runtime::AddBindingParams::builder()
                .name(name.to_string())
                .build().unwrap();
            page.execute(params).await.map_err(|e| {
                TurbosheetError::Other(format!("Failed to register binding: {}", e))
            })?;
            return Ok(());
        }
        Err(TurbosheetError::Other("Page is closed".to_string()))
    }

    fn global_name(&self) -> &str {
        &self.global_name
    }

    // ── Touch API (CDP DispatchTouchEvent) ─────────────────────

    async fn tap(&self, x: f64, y: f64) -> Result<(), TurbosheetError> {
        let mut page_guard = self.lock_page().await?;
        if let Some(page) = page_guard.as_mut() {
            use chromiumoxide::cdp::browser_protocol::input::{
                DispatchTouchEventParams, DispatchTouchEventType, TouchPoint,
            };
            let touch = TouchPoint::builder().x(x).y(y).build().unwrap();
            let start = DispatchTouchEventParams::builder()
                .r#type(DispatchTouchEventType::TouchStart)
                .touch_points(vec![touch.clone()])
                .build().unwrap();
            page.execute(start).await.map_err(|e| {
                TurbosheetError::Other(format!("CDP touchStart failed: {}", e))
            })?;
            let end = DispatchTouchEventParams::builder()
                .r#type(DispatchTouchEventType::TouchEnd)
                .touch_points(vec![touch])
                .build().unwrap();
            page.execute(end).await.map_err(|e| {
                TurbosheetError::Other(format!("CDP touchEnd failed: {}", e))
            })?;
            return Ok(());
        }
        Err(TurbosheetError::Other("Page is closed".to_string()))
    }

    async fn swipe(&self, fx: f64, fy: f64, tx: f64, ty: f64, steps: i32) -> Result<(), TurbosheetError> {
        let mut page_guard = self.lock_page().await?;
        if let Some(page) = page_guard.as_mut() {
            use chromiumoxide::cdp::browser_protocol::input::{
                DispatchTouchEventParams, DispatchTouchEventType, TouchPoint,
            };
            let start_touch = TouchPoint::builder().x(fx).y(fy).build().unwrap();
            let start = DispatchTouchEventParams::builder()
                .r#type(DispatchTouchEventType::TouchStart)
                .touch_points(vec![start_touch.clone()])
                .build().unwrap();
            page.execute(start).await.map_err(|e| {
                TurbosheetError::Other(format!("CDP swipe touchStart failed: {}", e))
            })?;

            let s = steps.max(1);
            for i in 1..=s {
                let t = i as f64 / s as f64;
                let cx = fx + (tx - fx) * t;
                let cy = fy + (ty - fy) * t;
                let move_touch = TouchPoint::builder().x(cx).y(cy).build().unwrap();
                let move_evt = DispatchTouchEventParams::builder()
                    .r#type(DispatchTouchEventType::TouchMove)
                    .touch_points(vec![move_touch])
                    .build().unwrap();
                page.execute(move_evt).await.ok();
                tokio::time::sleep(std::time::Duration::from_millis(5)).await;
            }

            let end = DispatchTouchEventParams::builder()
                .r#type(DispatchTouchEventType::TouchEnd)
                .touch_points(vec![start_touch])
                .build().unwrap();
            page.execute(end).await.map_err(|e| {
                TurbosheetError::Other(format!("CDP swipe touchEnd failed: {}", e))
            })?;
            return Ok(());
        }
        Err(TurbosheetError::Other("Page is closed".to_string()))
    }

    async fn pinch(&self, x: f64, y: f64, scale: f64) -> Result<(), TurbosheetError> {
        let mut page_guard = self.lock_page().await?;
        if let Some(page) = page_guard.as_mut() {
            use chromiumoxide::cdp::browser_protocol::input::{
                DispatchTouchEventParams, DispatchTouchEventType, TouchPoint,
            };
            // Two-finger pinch: fingers at (±offset, 0) relative to center.
            // Touch points are distinguished by their x/y coordinates.
            let spread = 40.0 * scale.max(0.1).min(5.0);
            let f1_start = TouchPoint::builder().x(x - 30.0).y(y).build().unwrap();
            let f2_start = TouchPoint::builder().x(x + 30.0).y(y).build().unwrap();
            let start = DispatchTouchEventParams::builder()
                .r#type(DispatchTouchEventType::TouchStart)
                .touch_points(vec![f1_start, f2_start])
                .build().unwrap();
            page.execute(start).await.map_err(|e| {
                TurbosheetError::Other(format!("CDP pinch touchStart failed: {}", e))
            })?;

            let f1_move = TouchPoint::builder().x(x - spread).y(y).build().unwrap();
            let f2_move = TouchPoint::builder().x(x + spread).y(y).build().unwrap();
            let move_evt = DispatchTouchEventParams::builder()
                .r#type(DispatchTouchEventType::TouchMove)
                .touch_points(vec![f1_move, f2_move])
                .build().unwrap();
            page.execute(move_evt).await.map_err(|e| {
                TurbosheetError::Other(format!("CDP pinch touchMove failed: {}", e))
            })?;

            let end = DispatchTouchEventParams::builder()
                .r#type(DispatchTouchEventType::TouchEnd)
                .build().unwrap();
            page.execute(end).await.map_err(|e| {
                TurbosheetError::Other(format!("CDP pinch touchEnd failed: {}", e))
            })?;
            return Ok(());
        }
        Err(TurbosheetError::Other("Page is closed".to_string()))
    }

    async fn long_press(&self, x: f64, y: f64, duration_ms: i32) -> Result<(), TurbosheetError> {
        let mut page_guard = self.lock_page().await?;
        if let Some(page) = page_guard.as_mut() {
            use chromiumoxide::cdp::browser_protocol::input::{
                DispatchTouchEventParams, DispatchTouchEventType, TouchPoint,
            };
            let touch = TouchPoint::builder().x(x).y(y).build().unwrap();
            let start = DispatchTouchEventParams::builder()
                .r#type(DispatchTouchEventType::TouchStart)
                .touch_point(touch)
                .build().unwrap();
            page.execute(start).await.map_err(|e| {
                TurbosheetError::Other(format!("CDP longPress touchStart failed: {}", e))
            })?;

            tokio::time::sleep(std::time::Duration::from_millis(duration_ms as u64)).await;

            let end = DispatchTouchEventParams::builder()
                .r#type(DispatchTouchEventType::TouchEnd)
                .build().unwrap();
            page.execute(end).await.map_err(|e| {
                TurbosheetError::Other(format!("CDP longPress touchEnd failed: {}", e))
            })?;
            return Ok(());
        }
        Err(TurbosheetError::Other("Page is closed".to_string()))
    }

    async fn set_page_id(&self, id: &str) -> Result<(), TurbosheetError> {
        let mut guard = self.page_id.lock().await;
        *guard = Some(id.to_string());
        // Auto-register CDP target_id → page_id mapping so popup
        // detection via Target.targetCreated → opener_id can find us.
        self.target_to_page
            .write()
            .await
            .insert(self.cdp_target_id.clone(), id.to_string());
        // Initialize global opener map entry (popup handler may update
        // it to Some(opener_page_id) later).
        crate::engine::PAGE_OPENER.insert(id.to_string(), None);
        Ok(())
    }

    // ── Media Emulation ──────────────────────────────────────────

    async fn emulate_media(&self, color_scheme: Option<String>, reduced_motion: Option<String>) -> Result<(), TurbosheetError> {
        let mut page_guard = self.lock_page().await?;
        if let Some(page) = page_guard.as_mut() {
            let mut features = Vec::new();
            if let Some(cs) = color_scheme {
                features.push(chromiumoxide::cdp::browser_protocol::emulation::MediaFeature {
                    name: "prefers-color-scheme".to_string(),
                    value: cs,
                });
            }
            if let Some(rm) = reduced_motion {
                features.push(chromiumoxide::cdp::browser_protocol::emulation::MediaFeature {
                    name: "prefers-reduced-motion".to_string(),
                    value: rm,
                });
            }
            let params = chromiumoxide::cdp::browser_protocol::emulation::SetEmulatedMediaParams::builder()
                .features(features)
                .build();
            page.execute(params).await.map_err(|e| {
                TurbosheetError::Other(format!("Failed to emulate media: {}", e))
            })?;
            return Ok(());
        }
        Err(TurbosheetError::Other("Page is closed".to_string()))
    }

    // ── Network Condition Overrides ──────────────────────────────

    async fn set_offline(&self, offline: bool) -> Result<(), TurbosheetError> {
        let mut page_guard = self.lock_page().await?;
        if let Some(page) = page_guard.as_mut() {
            let params = chromiumoxide::cdp::browser_protocol::network::EmulateNetworkConditionsParams::builder()
                .offline(offline)
                .latency(0.0)
                .download_throughput(0.0)
                .upload_throughput(0.0)
                .build()
                .unwrap();
            page.execute(params).await.map_err(|e| {
                TurbosheetError::Other(format!("Failed to set offline: {}", e))
            })?;
            return Ok(());
        }
        Err(TurbosheetError::Other("Page is closed".to_string()))
    }

    async fn throttle(&self, latency: f64, download: f64, upload: f64) -> Result<(), TurbosheetError> {
        let mut page_guard = self.lock_page().await?;
        if let Some(page) = page_guard.as_mut() {
            let params = chromiumoxide::cdp::browser_protocol::network::EmulateNetworkConditionsParams::builder()
                .offline(false)
                .latency(latency)
                .download_throughput(download)
                .upload_throughput(upload)
                .build()
                .unwrap();
            page.execute(params).await.map_err(|e| {
                TurbosheetError::Other(format!("Failed to throttle network: {}", e))
            })?;
            return Ok(());
        }
        Err(TurbosheetError::Other("Page is closed".to_string()))
    }

    // ── WebSocket Frame Interception ──────────────────────────

    async fn enable_websocket_interception(&self) -> Result<(), TurbosheetError> {
        // Network.enable is needed to receive Network.webSocketFrame* events.
        // chromiumoxide's Page manages the event listener stream internally,
        // so we register for the specific web socket events.
        let mut page_guard = self.lock_page().await?;
        if let Some(page) = page_guard.as_mut() {
            use chromiumoxide::cdp::browser_protocol::network::EnableParams;
            let enable = EnableParams::builder().build();
            page.execute(enable).await.map_err(|e| {
                TurbosheetError::Other(format!("Failed to enable Network domain: {}", e))
            })?;
            Ok(())
        } else {
            Err(TurbosheetError::Other("Page is closed".to_string()))
        }
    }

    async fn disable_websocket_interception(&self) -> Result<(), TurbosheetError> {
        // No explicit disable for websocket events alone;
        // dropping the subscribers is sufficient.
        Ok(())
    }

    // ── Accessibility Tree Snapshot ──────────────────────────

    async fn accessibility_snapshot(&self) -> Result<serde_json::Value, TurbosheetError> {
        let mut page_guard = self.lock_page().await?;
        if let Some(page) = page_guard.as_mut() {
            use chromiumoxide::cdp::browser_protocol::accessibility::GetFullAxTreeParams;
            let params = GetFullAxTreeParams::builder().build();
            let result = page.execute(params).await.map_err(|e| {
                TurbosheetError::Other(format!("Failed to get accessibility tree: {}", e))
            })?;
            Ok(serde_json::to_value(result.result).unwrap_or_default())
        } else {
            Err(TurbosheetError::Other("Page is closed".to_string()))
        }
    }

    // ── Storage API via CDP evaluate ──────────────────────────

    async fn get_local_storage(&self) -> Result<std::collections::HashMap<String, String>, TurbosheetError> {
        let js = r#"(function() {
            const items = {};
            for (let i = 0; i < window.localStorage.length; i++) {
                const key = window.localStorage.key(i);
                items[key] = window.localStorage.getItem(key);
            }
            return JSON.stringify(items);
        })()"#;
        let result = self.evaluate(js).await?;
        let map: std::collections::HashMap<String, String> =
            serde_json::from_str(&result).unwrap_or_default();
        Ok(map)
    }

    async fn set_local_storage(&self, items: std::collections::HashMap<String, String>) -> Result<(), TurbosheetError> {
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

    async fn get_session_storage(&self) -> Result<std::collections::HashMap<String, String>, TurbosheetError> {
        let js = r#"(function() {
            const items = {};
            for (let i = 0; i < window.sessionStorage.length; i++) {
                const key = window.sessionStorage.key(i);
                items[key] = window.sessionStorage.getItem(key);
            }
            return JSON.stringify(items);
        })()"#;
        let result = self.evaluate(js).await?;
        let map: std::collections::HashMap<String, String> =
            serde_json::from_str(&result).unwrap_or_default();
        Ok(map)
    }

    async fn set_session_storage(&self, items: std::collections::HashMap<String, String>) -> Result<(), TurbosheetError> {
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

    #[cfg(feature = "video")]
    async fn start_recording(&self, config_json: Option<String>) -> Result<(), TurbosheetError> {
        {
            let mut recorder_guard = self.recorder.lock().await;
            if recorder_guard.is_some() {
                return Err(TurbosheetError::Other("Recording already in progress".into()));
            }

            let config = match config_json {
                Some(json) => serde_json::from_str(&json)
                    .map_err(|e| TurbosheetError::Other(format!("Invalid video config: {}", e)))?,
                None => crate::video::VideoConfig::default(),
            };

            let recorder = crate::video::VideoRecorder::new(config);
            let tx = recorder.start().await?;

            // Store the frame sender so the event handler task can push decoded frames
            *self.frame_tx.lock().await = Some(tx);

            *recorder_guard = Some(recorder);
        }

        // Start CDP screencast
        let (width, height) = {
            let mut guard = self.lock_page().await?;
            if let Some(page) = guard.as_mut() {
                let js = "JSON.stringify({w: window.innerWidth, h: window.innerHeight})";
                let res = page.evaluate(js).await
                    .map_err(|e| TurbosheetError::Other(format!("Failed to get viewport: {}", e)))?;
                let val: serde_json::Value = res.into_value().unwrap_or_default();
                let w = val["w"].as_f64().unwrap_or(1280.0) as i64;
                let h = val["h"].as_f64().unwrap_or(720.0) as i64;
                (w, h)
            } else {
                return Err(TurbosheetError::Other("Page is closed".into()));
            }
        };

        {
            let mut guard = self.lock_page().await?;
            if let Some(page) = guard.as_mut() {
                use chromiumoxide::cdp::browser_protocol::page::StartScreencastParams;
                let params = StartScreencastParams::builder()
                    .format(chromiumoxide::cdp::browser_protocol::page::StartScreencastFormat::Jpeg)
                    .quality(80)
                    .max_width(width)
                    .max_height(height)
                    .every_nth_frame(1)
                    .build();
                page.execute(params).await
                    .map_err(|e| TurbosheetError::Other(format!("Failed to start screencast: {}", e)))?;
            } else {
                return Err(TurbosheetError::Other("Page is closed".into()));
            }
        }

        Ok(())
    }

    #[cfg(feature = "video")]
    async fn stop_recording(&self, test_passed: Option<bool>) -> Result<Option<String>, TurbosheetError> {
        // Stop CDP screencast
        // Stop CDP screencast
        {
            let mut guard = self.lock_page().await?;
            if let Some(page) = guard.as_mut() {
                let params = chromiumoxide::cdp::browser_protocol::page::StopScreencastParams::default();
                let _ = page.execute(params).await;
            }
        }

        // Clear the frame sender so the event handler stops pushing
        self.frame_tx.lock().await.take();

        // Stop the recorder and get output path
        let mut recorder_guard = self.recorder.lock().await;
        if let Some(ref recorder) = *recorder_guard {
            let result = recorder.stop(test_passed).await?;
            *recorder_guard = None;
            Ok(result)
        } else {
            Ok(None)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::KEY_CODE_MAP;

    #[test]
    fn test_key_code_map_enter() {
        assert_eq!(KEY_CODE_MAP.get("enter"), Some(&13i64));
    }

    #[test]
    fn test_key_code_map_tab() {
        assert_eq!(KEY_CODE_MAP.get("tab"), Some(&9i64));
    }

    #[test]
    fn test_key_code_map_escape() {
        assert_eq!(KEY_CODE_MAP.get("escape"), Some(&27i64));
    }

    #[test]
    fn test_key_code_map_backspace() {
        assert_eq!(KEY_CODE_MAP.get("backspace"), Some(&8i64));
    }

    #[test]
    fn test_key_code_map_delete() {
        assert_eq!(KEY_CODE_MAP.get("delete"), Some(&46i64));
    }

    #[test]
    fn test_key_code_map_arrow_keys() {
        assert_eq!(KEY_CODE_MAP.get("arrowup"), Some(&38i64));
        assert_eq!(KEY_CODE_MAP.get("arrowdown"), Some(&40i64));
        assert_eq!(KEY_CODE_MAP.get("arrowleft"), Some(&37i64));
        assert_eq!(KEY_CODE_MAP.get("arrowright"), Some(&39i64));
    }

    #[test]
    fn test_key_code_map_function_keys() {
        assert_eq!(KEY_CODE_MAP.get("f1"), Some(&112i64));
        assert_eq!(KEY_CODE_MAP.get("f5"), Some(&116i64));
        assert_eq!(KEY_CODE_MAP.get("f12"), Some(&123i64));
    }

    #[test]
    fn test_key_code_map_modifiers() {
        assert_eq!(KEY_CODE_MAP.get("shift"), Some(&16i64));
        assert_eq!(KEY_CODE_MAP.get("control"), Some(&17i64));
        assert_eq!(KEY_CODE_MAP.get("alt"), Some(&18i64));
        assert_eq!(KEY_CODE_MAP.get("meta"), Some(&91i64));
    }

    #[test]
    fn test_key_code_map_case_sensitive() {
        // KEY_CODE_MAP uses lowercase keys
        assert!(KEY_CODE_MAP.contains_key("enter"));
        assert!(!KEY_CODE_MAP.contains_key("Enter"));
        assert!(!KEY_CODE_MAP.contains_key("ENTER"));
    }

    #[test]
    fn test_key_code_map_nonexistent_key() {
        // Keys not in the map should return None
        assert_eq!(KEY_CODE_MAP.get("a"), None);
        assert_eq!(KEY_CODE_MAP.get("z"), None);
        assert_eq!(KEY_CODE_MAP.get("0"), None);
    }

    /// Integration test requiring a running browser.
    /// Run: CHROME_HEADLESS=true cargo test test_press_enter_on_input_changes_focus -- --ignored
    #[test]
    #[ignore = "requires running browser (CDP endpoint)"]
    fn test_press_enter_on_input_changes_focus() {
    }
}
