use napi::bindgen_prelude::*;
use napi::threadsafe_function::{ThreadsafeFunction, ThreadsafeFunctionCallMode};
use napi_derive::napi;
use std::collections::HashMap;
use dashmap::DashMap;
use tokio::sync::broadcast;
use lazy_static::lazy_static;

/// N-API event payload passed to `page.on('request', handler)`.
#[napi(object)]
#[derive(Clone)]
pub struct JsNetworkRequestEvent {
    pub url: String,
    pub method: String,
    pub headers: HashMap<String, String>,
    pub post_data: Option<String>,
}

/// N-API event payload passed to `page.on('response', handler)`.
#[napi(object)]
#[derive(Clone)]
pub struct JsNetworkResponseEvent {
    pub url: String,
    pub status: u16,
    pub headers: HashMap<String, String>,
}

/// Opaque JSON payload used internally for ThreadsafeFunction dispatch.
/// Event listeners registered from JavaScript receive this string and
/// deserialize it on the JS side into a plain object.
pub type EventCallback = ThreadsafeFunction<String>;

lazy_static! {
    /// Global listener storage keyed by `"{page_id}:{event_type}"`.
    /// Only `request` and `response` event types are supported initially.
    pub static ref PAGE_EVENT_LISTENERS: DashMap<String, Vec<EventCallback>> = DashMap::new();

    /// Context-level event listeners keyed by `"{context_id}:{event_type}"`.
    /// Used for `context.on('page', handler)` and `context.waitForEvent('page')`.
    pub static ref CONTEXT_EVENT_LISTENERS: DashMap<String, Vec<EventCallback>> = DashMap::new();

    /// Context-level popup broadcast senders, keyed by context_id.
    /// Each context can have at most one sender; receivers subscribe for
    /// `waitForEvent('page')`.
    pub static ref CONTEXT_POPUP_TX: DashMap<String, broadcast::Sender<String>> = DashMap::new();
}

/// Register a listener for a named event on a page.
pub fn register_listener(page_id: &str, event: &str, callback: EventCallback) {
    let key = format!("{}:{}", page_id, event);
    PAGE_EVENT_LISTENERS
        .entry(key)
        .or_insert_with(Vec::new)
        .push(callback);
}

/// Unregister all listeners for a (page_id, event) pair.
pub fn unregister_listeners(page_id: &str, event: &str) {
    let key = format!("{}:{}", page_id, event);
    PAGE_EVENT_LISTENERS.remove(&key);
}

/// Unregister ALL listeners for a page across all event types.
/// Called from `page.close()` so no stale callbacks fire after a page is destroyed.
pub fn unregister_all_for_page(page_id: &str) {
    let prefix = format!("{}:", page_id);
    PAGE_EVENT_LISTENERS.retain(|key, _| !key.starts_with(&prefix));
}

/// Dispatch a JSON payload to every listener registered for `page_id:event_type`.
pub fn dispatch_event(page_id: &str, event_type: &str, json_payload: &str) {
    let key = format!("{}:{}", page_id, event_type);
    if let Some(listeners) = PAGE_EVENT_LISTENERS.get(&key) {
        for cb in listeners.value().iter() {
            let _ = cb.call(
                Ok(json_payload.to_string()),
                ThreadsafeFunctionCallMode::NonBlocking,
            );
        }
    }
}

/// Register a listener for a named event on a context (e.g. 'page').
pub fn register_context_listener(context_id: &str, event: &str, callback: EventCallback) {
    let key = format!("{}:{}", context_id, event);
    CONTEXT_EVENT_LISTENERS
        .entry(key)
        .or_insert_with(Vec::new)
        .push(callback);
}

/// Unregister all listeners for a (context_id, event) pair.
pub fn unregister_context_listeners(context_id: &str, event: &str) {
    let key = format!("{}:{}", context_id, event);
    CONTEXT_EVENT_LISTENERS.remove(&key);
}

/// Unregister ALL context listeners for a given context.
pub fn unregister_all_context_events(context_id: &str) {
    let prefix = format!("{}:", context_id);
    CONTEXT_EVENT_LISTENERS.retain(|key, _| !key.starts_with(&prefix));
}

/// Dispatch a JSON payload to every context-level listener for `context_id:event_type`.
pub fn dispatch_context_event(context_id: &str, event_type: &str, json_payload: &str) {
    let key = format!("{}:{}", context_id, event_type);
    if let Some(listeners) = CONTEXT_EVENT_LISTENERS.get(&key) {
        for cb in listeners.value().iter() {
            let _ = cb.call(
                Ok(json_payload.to_string()),
                ThreadsafeFunctionCallMode::NonBlocking,
            );
        }
    }
}

/// Build a request-event JSON string from the standard event fields.
pub fn request_event_json(
    url: &str,
    method: &str,
    headers: &HashMap<String, String>,
    post_data: Option<&str>,
) -> String {
    let mut obj = serde_json::json!({
        "url": url,
        "method": method,
        "headers": headers,
    });
    if let Some(pd) = post_data {
        obj["postData"] = serde_json::Value::String(pd.to_string());
    }
    obj.to_string()
}

/// Build a response-event JSON string from the standard event fields.
pub fn response_event_json(
    url: &str,
    status: u16,
    headers: &HashMap<String, String>,
) -> String {
    serde_json::json!({
        "url": url,
        "status": status,
        "headers": headers,
    })
    .to_string()
}
