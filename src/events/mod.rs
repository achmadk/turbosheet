use dashmap::DashMap;
use serde_json::Value;
use tokio::sync::mpsc;

#[derive(Debug, Clone)]
pub enum EventType {
    FrameNavigated,
    FrameDetached,
    DomContentLoaded,
    LoadEventFired,
    JavascriptDialogOpening,
    ConsoleAPICalled,
    BindingCalled,
    RequestWillBeSent,
    ResponseReceived,
    LoadingFinished,
    TargetCreated,
    DownloadWillBegin,
    FileChooserOpened,
}

#[derive(Debug, Clone)]
pub struct CdpEvent {
    pub event_type: EventType,
    pub payload: Value,
}

pub struct EventDispatcher {
    subscribers: DashMap<String, Vec<mpsc::UnboundedSender<CdpEvent>>>,
    url_cache: DashMap<String, String>,
}

impl EventDispatcher {
    pub fn new() -> Self {
        Self {
            subscribers: DashMap::new(),
            url_cache: DashMap::new(),
        }
    }

    pub fn subscribe(&self, page_id: &str) -> mpsc::UnboundedReceiver<CdpEvent> {
        let (tx, rx) = mpsc::unbounded_channel();
        self.subscribers
            .entry(page_id.to_string())
            .or_insert_with(Vec::new)
            .push(tx);
        rx
    }

    pub fn handle_event(&self, page_id: &str, event_type: EventType, payload: Value) {
        tracing::debug!("Handling event {:?} for page {}", event_type, page_id);
        
        if let EventType::FrameNavigated = event_type {
            if let Some(url) = payload.get("frame").and_then(|f| f.get("url")).and_then(|u| u.as_str()) {
                self.url_cache.insert(page_id.to_string(), url.to_string());
            }
        }

        if let Some(mut subs) = self.subscribers.get_mut(page_id) {
            let event = CdpEvent {
                event_type,
                payload,
            };
            subs.retain(|tx| tx.send(event.clone()).is_ok());
        }
    }

    pub fn unsubscribe_all(&self, page_id: &str) {
        self.subscribers.remove(page_id);
        self.url_cache.remove(page_id);
    }
    
    pub async fn get_url(&self, page_id: &str) -> Option<String> {
        self.url_cache.get(page_id).map(|r| r.clone())
    }

    /// Sync version of get_url — used by `PageEngine::url()` which is trait-bounded
    /// to `fn url(&self) -> String` (no async).  Returns `None` if no URL has been
    /// cached yet for `page_id`.
    pub fn try_get_url(&self, page_id: &str) -> Option<String> {
        self.url_cache.get(page_id).map(|r| r.clone())
    }
}

impl Default for EventDispatcher {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[tokio::test]
    async fn test_subscribe_and_receive_event() {
        let dispatcher = EventDispatcher::new();
        let mut rx = dispatcher.subscribe("page_1");

        let payload = json!({"key": "value"});
        dispatcher.handle_event("page_1", EventType::FrameNavigated, payload.clone());

        let event = rx.recv().await.expect("Should receive event");
        assert!(matches!(event.event_type, EventType::FrameNavigated));
        assert_eq!(event.payload, payload);
    }

    #[tokio::test]
    async fn test_frame_navigated_updates_url_cache() {
        let dispatcher = EventDispatcher::new();

        let payload = json!({"frame": {"url": "https://example.com/page1"}});
        dispatcher.handle_event("page_1", EventType::FrameNavigated, payload);

        let cached = dispatcher.get_url("page_1").await;
        assert_eq!(cached, Some("https://example.com/page1".to_string()));

        // Sync try_get_url should return the same
        assert_eq!(dispatcher.try_get_url("page_1"), Some("https://example.com/page1".to_string()));
    }

    #[tokio::test]
    async fn test_multiple_subscribers_all_receive() {
        let dispatcher = EventDispatcher::new();
        let mut rx1 = dispatcher.subscribe("page_1");
        let mut rx2 = dispatcher.subscribe("page_1");

        let payload = json!({"msg": "hello"});
        dispatcher.handle_event("page_1", EventType::ConsoleAPICalled, payload);

        let ev1 = rx1.recv().await.unwrap();
        let ev2 = rx2.recv().await.unwrap();
        assert_eq!(ev1.payload, ev2.payload);
        assert!(matches!(ev1.event_type, EventType::ConsoleAPICalled));
    }

    #[tokio::test]
    async fn test_unsubscribe_removes_all_channels() {
        let dispatcher = EventDispatcher::new();
        let mut rx = dispatcher.subscribe("page_1");

        dispatcher.unsubscribe_all("page_1");

        // Subscriber should be dropped — channel closed
        dispatcher.handle_event("page_1", EventType::LoadEventFired, json!({}));
        let result = rx.recv().await;
        assert!(result.is_none(), "Channel should be closed after unsubscribe_all");
    }

    #[tokio::test]
    async fn test_dead_subscriber_does_not_block() {
        let dispatcher = EventDispatcher::new();
        let rx = dispatcher.subscribe("page_1");
        drop(rx); // Drop receiver immediately

        // handle_event should not panic — it removes dead senders via retain
        dispatcher.handle_event("page_1", EventType::FrameNavigated, json!({"frame": {"url": "about:blank"}}));

        // Subscriber list should have been cleaned up (dead senders removed)
        let subs = dispatcher.subscribers.get("page_1");
        if let Some(list) = subs {
            assert!(list.is_empty(), "Dead subscribers should be removed");
        }
    }

    #[tokio::test]
    async fn test_subscribers_isolated_by_page_id() {
        let dispatcher = EventDispatcher::new();
        let mut rx_a = dispatcher.subscribe("page_a");
        let mut rx_b = dispatcher.subscribe("page_b");

        dispatcher.handle_event("page_a", EventType::DomContentLoaded, json!({"a": 1}));

        assert!(rx_a.recv().await.is_some(), "page_a should receive");
        assert!(rx_b.try_recv().is_err(), "page_b should NOT receive");

        dispatcher.handle_event("page_b", EventType::LoadEventFired, json!({"b": 2}));
        assert!(rx_b.recv().await.is_some(), "page_b should now receive");
    }

    #[tokio::test]
    async fn test_url_cache_multiple_navigations() {
        let dispatcher = EventDispatcher::new();

        dispatcher.handle_event("page_1", EventType::FrameNavigated, json!({"frame": {"url": "https://a.com"}}));
        assert_eq!(dispatcher.get_url("page_1").await.unwrap(), "https://a.com");

        dispatcher.handle_event("page_1", EventType::FrameNavigated, json!({"frame": {"url": "https://b.com"}}));
        assert_eq!(dispatcher.get_url("page_1").await.unwrap(), "https://b.com");
    }

    #[tokio::test]
    async fn test_url_cache_per_page_isolation() {
        let dispatcher = EventDispatcher::new();

        dispatcher.handle_event("page_x", EventType::FrameNavigated, json!({"frame": {"url": "https://x.com"}}));
        dispatcher.handle_event("page_y", EventType::FrameNavigated, json!({"frame": {"url": "https://y.com"}}));

        assert_eq!(dispatcher.get_url("page_x").await.unwrap(), "https://x.com");
        assert_eq!(dispatcher.get_url("page_y").await.unwrap(), "https://y.com");
    }

    #[tokio::test]
    async fn test_url_cache_no_navigation_returns_none() {
        let dispatcher = EventDispatcher::new();
        assert!(dispatcher.get_url("unknown").await.is_none());
        assert!(dispatcher.try_get_url("unknown").is_none());
    }

    #[tokio::test]
    async fn test_dialog_event_payload_format() {
        let dispatcher = EventDispatcher::new();
        let mut rx = dispatcher.subscribe("page_1");

        let payload = json!({
            "type": "Alert",
            "message": "Hello!",
            "url": "https://example.com",
        });
        dispatcher.handle_event("page_1", EventType::JavascriptDialogOpening, payload.clone());

        let event = rx.recv().await.unwrap();
        assert_eq!(event.payload["type"], "Alert");
        assert_eq!(event.payload["message"], "Hello!");
        assert_eq!(event.payload["url"], "https://example.com");
    }

    #[tokio::test]
    async fn test_unsubscribe_only_removes_target_page() {
        let dispatcher = EventDispatcher::new();
        let mut rx1 = dispatcher.subscribe("page_1");
        let mut rx2 = dispatcher.subscribe("page_2");

        dispatcher.unsubscribe_all("page_1");

        dispatcher.handle_event("page_1", EventType::LoadEventFired, json!({}));
        assert!(rx1.recv().await.is_none(), "page_1 channel should be closed");

        dispatcher.handle_event("page_2", EventType::LoadEventFired, json!({}));
        assert!(rx2.recv().await.is_some(), "page_2 should still receive events");
    }

    #[tokio::test]
    async fn test_console_event_round_trip() {
        let dispatcher = EventDispatcher::new();
        let mut rx = dispatcher.subscribe("page_1");

        let payload = json!({
            "type": "log",
            "args": ["hello", "world"],
            "stackTrace": null,
        });
        dispatcher.handle_event("page_1", EventType::ConsoleAPICalled, payload.clone());

        let event = rx.recv().await.unwrap();
        assert_eq!(event.payload["type"], "log");
        assert_eq!(event.payload["args"][0], "hello");
    }

    #[tokio::test]
    async fn test_request_event_payload() {
        let dispatcher = EventDispatcher::new();
        let mut rx = dispatcher.subscribe("page_1");

        let payload = json!({
            "url": "https://example.com/api",
            "method": "POST",
            "headers": {"Content-Type": "application/json"},
        });
        dispatcher.handle_event("page_1", EventType::RequestWillBeSent, payload.clone());

        let event = rx.recv().await.unwrap();
        assert_eq!(event.payload["url"], "https://example.com/api");
        assert_eq!(event.payload["method"], "POST");
        assert_eq!(event.payload["headers"]["Content-Type"], "application/json");
    }

    #[tokio::test]
    async fn test_response_event_payload() {
        let dispatcher = EventDispatcher::new();
        let mut rx = dispatcher.subscribe("page_1");

        let payload = json!({
            "url": "https://example.com/api",
            "status": 200,
            "headers": {"content-type": "application/json"},
        });
        dispatcher.handle_event("page_1", EventType::ResponseReceived, payload.clone());

        let event = rx.recv().await.unwrap();
        assert_eq!(event.payload["url"], "https://example.com/api");
        assert_eq!(event.payload["status"], 200);
    }

    #[tokio::test]
    async fn test_nonevent_type_does_not_affect_cache() {
        let dispatcher = EventDispatcher::new();

        // Non-FrameNavigated events should NOT update URL cache
        dispatcher.handle_event("page_1", EventType::ConsoleAPICalled, json!({"msg": "hi"}));
        assert!(dispatcher.get_url("page_1").await.is_none());
    }

    #[tokio::test]
    async fn test_subscribe_on_demand() {
        // Subscribe after handle_event — existing events should NOT be replayed
        let dispatcher = EventDispatcher::new();

        dispatcher.handle_event("page_1", EventType::FrameNavigated, json!({"frame": {"url": "https://old.com"}}));

        let mut rx = dispatcher.subscribe("page_1");
        let result = rx.try_recv();
        assert!(result.is_err(), "Late subscriber should not receive past events");
    }
}
