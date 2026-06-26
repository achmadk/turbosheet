use serde::{Deserialize, Serialize};
use serde_json::Value;

/// Supported BiDi event methods.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BidiEventMethod {
    NetworkBeforeRequestSent,
    NetworkResponseCompleted,
    NetworkFetchError,
    ScriptMessage,
    Unknown(String),
}

impl From<&str> for BidiEventMethod {
    fn from(s: &str) -> Self {
        match s {
            "network.beforeRequestSent" => BidiEventMethod::NetworkBeforeRequestSent,
            "network.responseCompleted" => BidiEventMethod::NetworkResponseCompleted,
            "network.fetchError" => BidiEventMethod::NetworkFetchError,
            "script.message" => BidiEventMethod::ScriptMessage,
            other => BidiEventMethod::Unknown(other.to_string()),
        }
    }
}

/// Network request metadata from BiDi events.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkRequest {
    pub request: RequestDetails,
}

/// Request details from BiDi network events.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RequestDetails {
    pub request: String,
    pub url: String,
    pub method: String,
    pub headers: Option<Value>,
}

/// Response details from BiDi network events.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkResponse {
    pub request: RequestDetails,
    pub response: ResponseDetails,
}

/// Response status details.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResponseDetails {
    pub url: String,
    pub status: u16,
    pub statusText: Option<String>,
    pub headers: Option<Value>,
}

/// Script message from BiDi events.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScriptMessage {
    pub message: String,
    pub realm: Option<String>,
}
