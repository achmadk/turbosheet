use dashmap::DashMap;
use serde::{Deserialize, Serialize};
use tokio::sync::oneshot;

/// Default timeout for waiting on a binding response (30 seconds).
pub const BINDING_TIMEOUT: std::time::Duration = std::time::Duration::from_secs(30);

#[derive(Serialize, Deserialize, Debug)]
pub struct BindingMessage {
    pub id: String,
    pub ok: Option<bool>,
    pub error: Option<String>,
    pub data: Option<serde_json::Value>,
}

/// Tracks pending binding calls from injected JavaScript.
///
/// Each `invoke_action` call registers a oneshot sender keyed by a UUID.
/// When the injected script calls the binding function, the CDP
/// `EventBindingCalled` listener fires and routes the payload to
/// `dispatch()`, which resolves the corresponding sender.
///
/// Entries are removed on dispatch. Orphans (stale entries from
/// crashed/unresponsive pages) are cleaned up via `remove()`.
pub struct BindingRegistry {
    pub pending: DashMap<String, oneshot::Sender<Result<serde_json::Value, String>>>,
}

impl Default for BindingRegistry {
    fn default() -> Self {
        Self::new()
    }
}

impl BindingRegistry {
    pub fn new() -> Self {
        Self {
            pending: DashMap::new(),
        }
    }

    /// Register a oneshot sender for a binding call ID.
    /// Removes any prior entry for the same ID (safety net).
    pub fn register(&self, id: String, tx: oneshot::Sender<Result<serde_json::Value, String>>) {
        self.pending.insert(id, tx);
    }

    /// Remove and drop a pending entry (e.g., on evaluate failure or timeout)
    /// without sending a response. Returns true if the entry existed.
    pub fn remove(&self, id: &str) -> bool {
        self.pending.remove(id).is_some()
    }

    /// Dispatch an incoming binding payload from the injected script.
    /// Parses the JSON message, looks up the pending ID, and sends the
    /// result through the oneshot channel.
    pub fn dispatch(&self, payload: &str) {
        if let Ok(msg) = serde_json::from_str::<BindingMessage>(payload) {
            if let Some((_, tx)) = self.pending.remove(&msg.id) {
                if let Some(true) = msg.ok {
                    let _ = tx.send(Ok(msg.data.unwrap_or_default()));
                } else {
                    let err = msg.error.unwrap_or_else(|| "Unknown error from injected script".to_string());
                    let _ = tx.send(Err(err));
                }
            }
        }
    }

    /// Number of currently pending binding calls.
    pub fn pending_count(&self) -> usize {
        self.pending.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_register_and_dispatch_ok() {
        let reg = BindingRegistry::new();
        let (tx, mut rx) = oneshot::channel();
        let id = "test-1".to_string();

        reg.register(id.clone(), tx);
        assert_eq!(reg.pending_count(), 1);

        let payload = r#"{"id":"test-1","ok":true,"data":{"value":42}}"#;
        reg.dispatch(payload);

        let result = rx.try_recv().unwrap().unwrap();
        assert_eq!(result, serde_json::json!({"value": 42}));
        assert_eq!(reg.pending_count(), 0);
    }

    #[test]
    fn test_register_and_dispatch_error() {
        let reg = BindingRegistry::new();
        let (tx, mut rx) = oneshot::channel();
        let id = "test-err".to_string();

        reg.register(id.clone(), tx);
        let payload = r#"{"id":"test-err","ok":false,"error":"Something broke"}"#;
        reg.dispatch(payload);

        let result = rx.try_recv().unwrap().unwrap_err();
        assert_eq!(result, "Something broke");
        assert_eq!(reg.pending_count(), 0);
    }

    #[test]
    fn test_dispatch_unknown_id_does_not_crash() {
        let reg = BindingRegistry::new();
        let payload = r#"{"id":"nonexistent","ok":true,"data":null}"#;
        reg.dispatch(payload);
        assert_eq!(reg.pending_count(), 0);
    }

    #[test]
    fn test_dispatch_invalid_json_does_not_crash() {
        let reg = BindingRegistry::new();
        reg.dispatch("not valid json");
        assert_eq!(reg.pending_count(), 0);
    }

    #[test]
    fn test_dispatch_missing_fields_does_not_crash() {
        let reg = BindingRegistry::new();
        reg.dispatch(r#"{"id":"x"}"#);
        assert_eq!(reg.pending_count(), 0);
    }

    #[test]
    fn test_remove_cleans_up_entry() {
        let reg = BindingRegistry::new();
        let (tx, _rx) = oneshot::channel();
        reg.register("to-remove".to_string(), tx);
        assert_eq!(reg.pending_count(), 1);

        assert!(reg.remove("to-remove"));
        assert_eq!(reg.pending_count(), 0);

        assert!(!reg.remove("to-remove"));
    }

    #[test]
    fn test_register_replaces_existing_id() {
        let reg = BindingRegistry::new();
        let (tx1, _rx1) = oneshot::channel();
        let (tx2, mut rx2) = oneshot::channel();

        reg.register("same-id".to_string(), tx1);
        assert_eq!(reg.pending_count(), 1);

        reg.register("same-id".to_string(), tx2);
        assert_eq!(reg.pending_count(), 1);

        reg.dispatch(r#"{"id":"same-id","ok":true,"data":true}"#);
        let result = rx2.try_recv().unwrap().unwrap();
        assert_eq!(result, serde_json::json!(true));
        assert_eq!(reg.pending_count(), 0);
    }

    #[test]
    fn test_concurrent_dispatch() {
        let reg = BindingRegistry::new();
        let count = 10;
        let mut receivers = Vec::with_capacity(count);

        for i in 0..count {
            let (tx, rx) = oneshot::channel();
            let id = format!("concurrent-{}", i);
            reg.register(id.clone(), tx);
            receivers.push((id, rx));
        }

        assert_eq!(reg.pending_count(), count);

        for (id, _) in receivers.iter().rev() {
            let payload = format!(r#"{{"id":"{}","ok":true,"data":"{}"}}"#, id, id);
            reg.dispatch(&payload);
        }

        assert_eq!(reg.pending_count(), 0);

        for (id, mut rx) in receivers {
            let result = rx.try_recv().unwrap().unwrap();
            assert_eq!(result, serde_json::json!(id));
        }
    }
}
