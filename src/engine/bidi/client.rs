use std::collections::HashMap;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use tokio::sync::{Mutex, oneshot};
use tokio::task::JoinHandle;
use crate::engine::bidi::transport::{BidiTransport, SharedTransport};
use crate::engine::bidi::types::{self, BidiCommand, BidiEvent, BidiMessage, BidiResponse};
use serde_json::Value;

/// A URL-pattern waiter that resolves when a BiDi network event matches.
pub struct UrlWaiter {
    pub pattern: String,
    pub kind: RequestKind,
    pub sender: oneshot::Sender<Value>,
    pub created_at: tokio::time::Instant,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RequestKind {
    Request,
    Response,
    Error,
}

/// Registry of pending URL waiters on a BidiClient.
pub struct UrlWaiterRegistry {
    waiters: Vec<UrlWaiter>,
    timeout: std::time::Duration,
}

impl UrlWaiterRegistry {
    pub fn new(timeout: std::time::Duration) -> Self {
        Self {
            waiters: Vec::new(),
            timeout,
        }
    }

    pub fn register(&mut self, pattern: String, kind: RequestKind) -> oneshot::Receiver<Value> {
        let (tx, rx) = oneshot::channel();
        self.waiters.push(UrlWaiter {
            pattern,
            kind,
            sender: tx,
            created_at: tokio::time::Instant::now(),
        });
        rx
    }

    pub fn resolve_matching(&mut self, event: &BidiEvent) {
        let url = event.params.get("request")
            .and_then(|r| r.get("url"))
            .and_then(|u| u.as_str())
            .unwrap_or("");
        let kind = match event.method.as_str() {
            "network.beforeRequestSent" => RequestKind::Request,
            "network.responseCompleted" => RequestKind::Response,
            "network.fetchError" => RequestKind::Error,
            _ => return,
        };

        let mut i = 0;
        while i < self.waiters.len() {
            if self.waiters[i].kind == kind && url.contains(&self.waiters[i].pattern) {
                let waiter = self.waiters.remove(i);
                let metadata = event.params.clone();
                let _ = waiter.sender.send(metadata);
            } else {
                i += 1;
            }
        }
    }

    pub fn cleanup_expired(&mut self) {
        let now = tokio::time::Instant::now();
        self.waiters.retain(|w| now.duration_since(w.created_at) < self.timeout);
    }
}

/// High-level BiDi client managing command/event interaction over a WebSocket.
pub struct BidiClient {
    transport: SharedTransport,
    next_id: AtomicU64,
    pending: Arc<Mutex<HashMap<u64, oneshot::Sender<BidiResponse>>>>,
    waiter_registry: Arc<Mutex<UrlWaiterRegistry>>,
    event_handler: Mutex<Option<JoinHandle<()>>>,
    session_id: Mutex<Option<String>>,
}

impl BidiClient {
    /// Connect to a BiDi WebSocket URL and start the event listener.
    pub async fn connect(url: &str) -> Result<Arc<Self>, crate::error::TurbosheetError> {
        let transport = BidiTransport::connect(url).await?;
        let shared = SharedTransport::new(transport);

        let client = Arc::new(Self {
            transport: shared,
            next_id: AtomicU64::new(1),
            pending: Arc::new(Mutex::new(HashMap::new())),
            waiter_registry: Arc::new(Mutex::new(UrlWaiterRegistry::new(std::time::Duration::from_secs(30)))),
            event_handler: Mutex::new(None),
            session_id: Mutex::new(None),
        });

        let handler = start_event_listener(
            client.transport.clone_inner(),
            client.pending.clone(),
            client.waiter_registry.clone(),
        );
        *client.event_handler.lock().await = Some(handler);

        Ok(client)
    }

    /// Create a BidiClient by extracting the BiDi WebSocket URL from
    /// WebDriver capabilities and connecting.
    pub async fn from_webdriver_session(
        webdriver_url: &str,
        session_id: &str,
    ) -> Result<Arc<Self>, crate::error::TurbosheetError> {
        let ws_url = format!(
            "{}/session/{}",
            webdriver_url
                .replace("http://", "ws://")
                .replace("https://", "wss://")
                .trim_end_matches('/'),
            session_id
        );

        let client = Self::connect(&ws_url).await?;
        *client.session_id.lock().await = Some(session_id.to_string());

        client.subscribe(&[
            "network.beforeRequestSent",
            "network.responseCompleted",
            "network.fetchError",
        ]).await?;

        Ok(client)
    }

    /// Send a BiDi command and wait for the response.
    pub async fn send_command(
        &self,
        method: &str,
        params: Option<Value>,
    ) -> Result<Value, crate::error::TurbosheetError> {
        let id = self.next_id.fetch_add(1, Ordering::SeqCst);
        let (tx, rx) = oneshot::channel();

        {
            let mut pending = self.pending.lock().await;
            pending.insert(id, tx);
        }

        let cmd = BidiCommand {
            id,
            method: method.to_string(),
            params,
        };
        let raw = serde_json::to_string(&cmd)
            .map_err(|e| crate::error::TurbosheetError::Other(format!("BiDi serialize error: {}", e)))?;
        self.transport.send(&raw).await?;

        let resp = tokio::time::timeout(std::time::Duration::from_secs(30), rx)
            .await
            .map_err(|_| crate::error::TurbosheetError::Other("BiDi command timeout".into()))?
            .map_err(|_| crate::error::TurbosheetError::Other("BiDi channel closed".into()))?;

        if let Some(err) = resp.error {
            return Err(crate::error::TurbosheetError::Other(
                format!("BiDi error ({}): {}", err.code, err.message),
            ));
        }

        Ok(resp.result.unwrap_or(Value::Null))
    }

    /// Subscribe to BiDi events.
    pub async fn subscribe(&self, events: &[&str]) -> Result<(), crate::error::TurbosheetError> {
        let params = serde_json::json!({
            "events": events
        });
        self.send_command("session.subscribe", Some(params)).await?;
        Ok(())
    }

    /// Register a wait_for_request waiter.
    pub async fn wait_for_request(&self, url: &str) -> Result<Value, crate::error::TurbosheetError> {
        let rx = {
            let mut registry = self.waiter_registry.lock().await;
            registry.register(url.to_string(), RequestKind::Request)
        };
        tokio::time::timeout(std::time::Duration::from_secs(30), rx)
            .await
            .map_err(|_| crate::error::TurbosheetError::Other("wait_for_request timeout".into()))?
            .map_err(|_| crate::error::TurbosheetError::Other("wait_for_request channel closed".into()))
    }

    /// Register a wait_for_response waiter.
    pub async fn wait_for_response(&self, url: &str) -> Result<Value, crate::error::TurbosheetError> {
        let rx = {
            let mut registry = self.waiter_registry.lock().await;
            registry.register(url.to_string(), RequestKind::Response)
        };
        tokio::time::timeout(std::time::Duration::from_secs(30), rx)
            .await
            .map_err(|_| crate::error::TurbosheetError::Other("wait_for_response timeout".into()))?
            .map_err(|_| crate::error::TurbosheetError::Other("wait_for_response channel closed".into()))
    }

    /// Add a preload script that runs on every page load.
    pub async fn add_preload_script(&self, js: &str) -> Result<String, crate::error::TurbosheetError> {
        let params = serde_json::json!({
            "script": js
        });
        let result = self.send_command("script.addPreloadScript", Some(params)).await?;
        let script_id = result.get("script")
            .and_then(|v| v.as_str())
            .ok_or_else(|| crate::error::TurbosheetError::Other("Missing script ID in addPreloadScript response".into()))?;
        Ok(script_id.to_string())
    }

    /// Remove a preload script.
    pub async fn remove_preload_script(&self, script_id: &str) -> Result<(), crate::error::TurbosheetError> {
        let params = serde_json::json!({
            "script": script_id
        });
        self.send_command("script.removePreloadScript", Some(params)).await?;
        Ok(())
    }

    /// Call a function in the page context.
    pub async fn call_function(
        &self,
        function_declaration: &str,
        args: Vec<Value>,
    ) -> Result<Value, crate::error::TurbosheetError> {
        let params = serde_json::json!({
            "functionDeclaration": function_declaration,
            "arguments": args,
            "awaitPromise": true,
            "target": {
                "type": "realm",
                "realmId": serde_json::Value::Null
            }
        });
        self.send_command("script.callFunctionOn", Some(params)).await
    }

    /// Close the client and its event handler.
    pub async fn close(&self) {
        self.transport.close().await;
    }
}

/// Background task that reads events from the WebSocket and dispatches them.
fn start_event_listener(
    transport: Arc<Mutex<BidiTransport>>,
    pending: Arc<Mutex<HashMap<u64, oneshot::Sender<BidiResponse>>>>,
    waiter_registry: Arc<Mutex<UrlWaiterRegistry>>,
) -> JoinHandle<()> {
    tokio::spawn(async move {
        loop {
            let msg = {
                let mut t = transport.lock().await;
                match t.recv().await {
                    Ok(msg) => msg,
                    Err(_) => {
                        tokio::time::sleep(std::time::Duration::from_millis(100)).await;
                        continue;
                    }
                }
            };

            match types::parse_message(&msg) {
                Ok(BidiMessage::Command(resp)) => {
                    let mut p = pending.lock().await;
                    if let Some(tx) = p.remove(&resp.id) {
                        let _ = tx.send(resp);
                    }
                }
                Ok(BidiMessage::Event(event)) => {
                    let mut r = waiter_registry.lock().await;
                    r.resolve_matching(&event);
                }
                Err(_) => {}
            }
        }
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn make_registry() -> UrlWaiterRegistry {
        UrlWaiterRegistry::new(std::time::Duration::from_secs(30))
    }

    fn make_event(method: &str, url: &str) -> BidiEvent {
        BidiEvent {
            method: method.to_string(),
            params: json!({"request": {"url": url}}),
        }
    }

    // -- Registration + resolution -------------------------------------------------

    #[tokio::test]
    async fn test_resolve_request() {
        let mut registry = make_registry();
        let mut rx = registry.register("example.com".to_string(), RequestKind::Request);
        registry.resolve_matching(&make_event("network.beforeRequestSent", "https://example.com/page"));
        let result = tokio::time::timeout(std::time::Duration::from_millis(200), &mut rx).await;
        assert!(result.is_ok(), "Request waiter should resolve on matching beforeRequestSent");
    }

    #[tokio::test]
    async fn test_resolve_response() {
        let mut registry = make_registry();
        let mut rx = registry.register("api.example.com".to_string(), RequestKind::Response);
        registry.resolve_matching(&make_event("network.responseCompleted", "https://api.example.com/data"));
        let result = tokio::time::timeout(std::time::Duration::from_millis(200), &mut rx).await;
        assert!(result.is_ok(), "Response waiter should resolve on matching responseCompleted");
    }

    #[tokio::test]
    async fn test_resolve_fetch_error() {
        let mut registry = make_registry();
        let mut rx = registry.register("cdn.example.com".to_string(), RequestKind::Error);
        registry.resolve_matching(&make_event("network.fetchError", "https://cdn.example.com/broken.js"));
        let result = tokio::time::timeout(std::time::Duration::from_millis(200), &mut rx).await;
        assert!(result.is_ok(), "Error waiter should resolve on matching fetchError");
    }

    // -- Non-matching cases --------------------------------------------------------

    #[tokio::test]
    async fn test_no_match_wrong_kind() {
        let mut registry = make_registry();
        let mut rx = registry.register("example.com".to_string(), RequestKind::Request);
        // Same URL but wrong event method (response instead of request)
        registry.resolve_matching(&make_event("network.responseCompleted", "https://example.com/page"));
        let result = tokio::time::timeout(std::time::Duration::from_millis(100), &mut rx).await;
        assert!(result.is_err(), "Request waiter should NOT resolve on response event");
    }

    #[tokio::test]
    async fn test_no_match_wrong_url() {
        let mut registry = make_registry();
        let mut rx = registry.register("example.com".to_string(), RequestKind::Request);
        registry.resolve_matching(&make_event("network.beforeRequestSent", "https://other.com/page"));
        let result = tokio::time::timeout(std::time::Duration::from_millis(100), &mut rx).await;
        assert!(result.is_err(), "Waiter should NOT resolve on non-matching URL");
    }

    #[test]
    fn test_unknown_method_event_ignored() {
        let mut registry = make_registry();
        let _rx = registry.register("test".to_string(), RequestKind::Request);
        let event = BidiEvent {
            method: "browsingContext.load".to_string(),
            params: json!({"context": "abc", "url": "https://test.com"}),
        };
        registry.resolve_matching(&event);
        assert_eq!(registry.waiters.len(), 1, "Unknown event method should not resolve any waiters");
    }

    // -- URL substring matching ----------------------------------------------------

    #[tokio::test]
    async fn test_url_substring_match() {
        let mut registry = make_registry();
        let mut rx = registry.register("page/42".to_string(), RequestKind::Request);
        registry.resolve_matching(&make_event("network.beforeRequestSent", "https://example.com/page/42/details"));
        let result = tokio::time::timeout(std::time::Duration::from_millis(200), &mut rx).await;
        assert!(result.is_ok(), "Should match on URL substring");
    }

    // -- Multiple waiters ----------------------------------------------------------

    #[tokio::test]
    async fn test_multiple_waiters_only_matching_removed() {
        let mut registry = make_registry();
        let _rx1 = registry.register("alpha".to_string(), RequestKind::Request);
        let mut rx2 = registry.register("beta".to_string(), RequestKind::Request);
        registry.resolve_matching(&make_event("network.beforeRequestSent", "https://beta.com/page"));
        let result = tokio::time::timeout(std::time::Duration::from_millis(100), &mut rx2).await;
        assert!(result.is_ok(), "Matching waiter should resolve");
        assert_eq!(registry.waiters.len(), 1, "Only non-matching waiter should remain");
        assert_eq!(registry.waiters[0].pattern, "alpha");
    }

    // -- Cleanup -------------------------------------------------------------------

    #[test]
    fn test_cleanup_expired_removes_stale() {
        let short = std::time::Duration::from_millis(1);
        let mut registry = UrlWaiterRegistry::new(short);
        let _rx = registry.register("test".to_string(), RequestKind::Request);
        // Wait longer than the timeout so the waiter expires
        std::thread::sleep(std::time::Duration::from_millis(10));
        assert_eq!(registry.waiters.len(), 1, "Should still have waiter before cleanup");
        registry.cleanup_expired();
        assert_eq!(registry.waiters.len(), 0, "Expired waiter should be removed after cleanup");
    }

    #[test]
    fn test_cleanup_keeps_fresh_waiters() {
        let mut registry = make_registry();
        let _rx = registry.register("fresh".to_string(), RequestKind::Response);
        registry.cleanup_expired();
        assert_eq!(registry.waiters.len(), 1, "Fresh waiter should survive cleanup");
    }

    // -- Registry construction -----------------------------------------------------

    #[test]
    fn test_registry_new_starts_empty() {
        let registry = make_registry();
        assert_eq!(registry.waiters.len(), 0, "New registry should be empty");
    }
}
