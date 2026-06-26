use serde::{Deserialize, Serialize};
use serde_json::Value;

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_parse_command_response_with_result() {
        let raw = r#"{"id": 1, "result": {"value": "ok"}}"#;
        let msg = parse_message(raw).unwrap();
        match msg {
            BidiMessage::Command(resp) => {
                assert_eq!(resp.id, 1);
                assert_eq!(resp.result.unwrap(), json!({"value": "ok"}));
                assert!(resp.error.is_none());
            }
            _ => panic!("Expected Command message"),
        }
    }

    #[test]
    fn test_parse_response_with_error() {
        let raw = r#"{"id": 42, "error": {"code": "invalid argument", "message": "Bad params"}}"#;
        let msg = parse_message(raw).unwrap();
        match msg {
            BidiMessage::Command(resp) => {
                assert_eq!(resp.id, 42);
                assert!(resp.result.is_none());
                let err = resp.error.unwrap();
                assert_eq!(err.code, "invalid argument");
                assert_eq!(err.message, "Bad params");
                assert!(err.stacktrace.is_none());
            }
            _ => panic!("Expected Command message"),
        }
    }

    #[test]
    fn test_parse_event() {
        let raw = r#"{"method": "network.beforeRequestSent", "params": {"request": {"url": "https://example.com"}}}"#;
        let msg = parse_message(raw).unwrap();
        match msg {
            BidiMessage::Event(event) => {
                assert_eq!(event.method, "network.beforeRequestSent");
                assert_eq!(event.params["request"]["url"], "https://example.com");
            }
            _ => panic!("Expected Event message"),
        }
    }

    #[test]
    fn test_parse_event_missing_method_fails() {
        let raw = r#"{"params": {"key": "value"}}"#;
        let result = parse_message(raw);
        assert!(result.is_err(), "Should fail when event has no method field");
    }

    #[test]
    fn test_parse_invalid_json_fails() {
        let result = parse_message("not valid json");
        assert!(result.is_err(), "Should fail on malformed JSON");
    }

    #[test]
    fn test_parse_empty_json_fails() {
        let result = parse_message("{}");
        assert!(result.is_err(), "Should fail on empty object (neither command nor event)");
    }

    #[test]
    fn test_bidi_command_serialization() {
        let cmd = BidiCommand {
            id: 5,
            method: "session.subscribe".to_string(),
            params: Some(json!({"events": ["network.beforeRequestSent"]})),
        };
        let json = serde_json::to_string(&cmd).unwrap();
        assert!(json.contains(r#""id":5"#));
        assert!(json.contains(r#""method":"session.subscribe""#));
        assert!(json.contains(r#""events":["network.beforeRequestSent"]"#));
    }

    #[test]
    fn test_bidi_command_serialization_no_params() {
        let cmd = BidiCommand {
            id: 99,
            method: "session.end".to_string(),
            params: None,
        };
        let json = serde_json::to_string(&cmd).unwrap();
        assert!(json.contains(r#""id":99"#));
        assert!(json.contains(r#""method":"session.end""#));
        // params field should be skipped when None
        assert!(!json.contains("params"));
    }

    #[test]
    fn test_bidi_response_deserialization() {
        let raw = r#"{"id": 10, "result": {"script": "abc123"}}"#;
        let resp: BidiResponse = serde_json::from_str(raw).unwrap();
        assert_eq!(resp.id, 10);
        assert_eq!(resp.result.unwrap()["script"], "abc123");
        assert!(resp.error.is_none());
    }

    #[test]
    fn test_bidi_error_with_stacktrace() {
        let raw = r#"{"code": "timeout", "message": "Request timed out", "stacktrace": "at line 42"}"#;
        let err: BidiError = serde_json::from_str(raw).unwrap();
        assert_eq!(err.code, "timeout");
        assert_eq!(err.message, "Request timed out");
        assert_eq!(err.stacktrace, Some("at line 42".to_string()));
    }

    #[test]
    fn test_parse_message_roundtrip_command() {
        // Send a command, serialize it, parse it back
        let cmd = BidiCommand {
            id: 7,
            method: "script.callFunctionOn".to_string(),
            params: Some(json!({"functionDeclaration": "() => 42"})),
        };
        let raw = serde_json::to_string(&cmd).unwrap();
        
        // parse_message sees an "id" field and routes it as Command
        let msg = parse_message(&raw).unwrap();
        match msg {
            BidiMessage::Command(resp) => {
                assert_eq!(resp.id, 7);
                // Note: parse_message routes based on "id" presence,
                // and then deserializes the full JSON as BidiResponse
                // which picks up id and result/error.
                // Since the original cmd has no "result" or "error",
                // they'll be None
                assert!(resp.result.is_none());
                assert!(resp.error.is_none());
            }
            _ => panic!("Expected Command message"),
        }
    }
}

/// A BiDi command message sent to the browser.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BidiCommand {
    pub id: u64,
    pub method: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub params: Option<Value>,
}

/// A BiDi response message received from the browser.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BidiResponse {
    pub id: u64,
    #[serde(default)]
    pub result: Option<Value>,
    #[serde(default)]
    pub error: Option<BidiError>,
}

/// A BiDi error structure.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BidiError {
    pub code: String,
    pub message: String,
    #[serde(default)]
    pub stacktrace: Option<String>,
}

/// An event message pushed from the browser (no `id` field).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BidiEvent {
    pub method: String,
    pub params: Value,
}

/// Union type for incoming WebSocket messages.
#[derive(Debug, Clone)]
pub enum BidiMessage {
    Command(BidiResponse),
    Event(BidiEvent),
}

/// Attempt to parse a JSON string into a BidiMessage.
///
/// BiDi messages with an `id` field are command responses;
/// messages without an `id` field are events.
pub fn parse_message(raw: &str) -> Result<BidiMessage, crate::error::TurbosheetError> {
    let v: Value = serde_json::from_str(raw)
        .map_err(|e| crate::error::TurbosheetError::Other(format!("BiDi parse error: {}", e)))?;

    if v.get("id").is_some() {
        let resp: BidiResponse = serde_json::from_value(v)
            .map_err(|e| crate::error::TurbosheetError::Other(format!("BiDi response parse error: {}", e)))?;
        Ok(BidiMessage::Command(resp))
    } else {
        let event: BidiEvent = serde_json::from_value(v)
            .map_err(|e| crate::error::TurbosheetError::Other(format!("BiDi event parse error: {}", e)))?;
        Ok(BidiMessage::Event(event))
    }
}
