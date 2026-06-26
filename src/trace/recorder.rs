use std::sync::Arc;
use tokio::sync::Mutex;
use serde::{Deserialize, Serialize};
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActionEvent {
    pub timestamp: u64,
    pub action_type: String,
    pub selector: Option<String>,
    pub value: Option<String>,
    pub duration_ms: u64,
    pub result: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkEvent {
    pub timestamp: u64,
    pub event_type: String,
    pub url: String,
    pub method: String,
    pub status: Option<u16>,
    pub timing: Option<u64>,
    pub headers: Vec<(String, String)>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConsoleEvent {
    pub timestamp: u64,
    pub level: String,
    pub message: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AccessibilityNode {
    pub role: String,
    pub name: String,
    pub value: Option<String>,
    pub description: Option<String>,
    pub state: Vec<String>,
    pub children: Vec<usize>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ElementData {
    pub selector: String,
    pub tag: String,
    pub attrs: std::collections::HashMap<String, String>,
    pub rect: Option<Rect>,
    pub is_visible: bool,
    pub is_interactive: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Rect {
    pub x: f64,
    pub y: f64,
    pub width: f64,
    pub height: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AiSnapshotMetadata {
    pub viewport_width: u32,
    pub viewport_height: u32,
    pub url: String,
    pub title: String,
    pub focus_element: Option<String>,
    pub interactive_count: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DomSnapshot {
    pub timestamp: u64,
    pub snapshot_type: String,
    pub accessibility_tree: Vec<AccessibilityNode>,
    pub elements: Vec<ElementData>,
    pub html: String,
    pub ai_metadata: AiSnapshotMetadata,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScreencastFrameEvent {
    pub timestamp: u64,
    pub frame_index: u64,
    pub frame_size: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum TraceEvent {
    Action(ActionEvent),
    Network(NetworkEvent),
    Console(ConsoleEvent),
    Snapshot(DomSnapshot),
    Screencast(ScreencastFrameEvent),
}

#[derive(Default)]
pub struct TraceRecorderState {
    pub events: Vec<TraceEvent>,
    pub action_count: usize,
    pub network_count: usize,
    pub console_count: usize,
    pub screencast_count: usize,
}

pub struct TraceRecorder {
    state: Arc<Mutex<TraceRecorderState>>,
    max_events: usize,
}

impl TraceRecorder {
    pub fn new(max_events: Option<usize>) -> Self {
        Self {
            state: Arc::new(Mutex::new(TraceRecorderState::default())),
            max_events: max_events.unwrap_or(100_000),
        }
    }

    pub async fn record_action(
        &self,
        action_type: &str,
        selector: Option<&str>,
        value: Option<&str>,
        duration_ms: u64,
        result: &str,
    ) {
        let mut state = self.state.lock().await;
        if state.events.len() < self.max_events {
            state.events.push(TraceEvent::Action(ActionEvent {
                timestamp: now_ms(),
                action_type: action_type.to_string(),
                selector: selector.map(|s| s.to_string()),
                value: value.map(|s| s.to_string()),
                duration_ms,
                result: result.to_string(),
            }));
            state.action_count += 1;
        }
    }

    pub async fn record_network(
        &self,
        event_type: &str,
        url: &str,
        method: &str,
        status: Option<u16>,
        timing: Option<u64>,
        headers: Vec<(String, String)>,
    ) {
        let mut state = self.state.lock().await;
        if state.events.len() < self.max_events {
            state.events.push(TraceEvent::Network(NetworkEvent {
                timestamp: now_ms(),
                event_type: event_type.to_string(),
                url: url.to_string(),
                method: method.to_string(),
                status,
                timing,
                headers,
            }));
            state.network_count += 1;
        }
    }

    pub async fn record_console(&self, level: &str, message: &str) {
        let mut state = self.state.lock().await;
        if state.events.len() < self.max_events {
            state.events.push(TraceEvent::Console(ConsoleEvent {
                timestamp: now_ms(),
                level: level.to_string(),
                message: message.to_string(),
            }));
            state.console_count += 1;
        }
    }

    pub async fn record_snapshot(&self, snapshot_type: &str, _accessibility_tree: &str, html: &str) {
        let mut state = self.state.lock().await;
        if state.events.len() < self.max_events {
            state.events.push(TraceEvent::Snapshot(DomSnapshot {
                timestamp: now_ms(),
                snapshot_type: snapshot_type.to_string(),
                accessibility_tree: vec![],
                elements: vec![],
                html: html.to_string(),
                ai_metadata: AiSnapshotMetadata {
                    viewport_width: 1920,
                    viewport_height: 1080,
                    url: String::new(),
                    title: String::new(),
                    focus_element: None,
                    interactive_count: 0,
                },
            }));
        }
    }

    pub async fn record_screencast_frame(&self, frame_index: u64, frame_size: u32) {
        let mut state = self.state.lock().await;
        if state.events.len() < self.max_events {
            state.events.push(TraceEvent::Screencast(ScreencastFrameEvent {
                timestamp: now_ms(),
                frame_index,
                frame_size,
            }));
            state.screencast_count += 1;
        }
    }

    pub async fn record_snapshot_ai(
        &self,
        snapshot_type: &str,
        accessibility_tree: Vec<AccessibilityNode>,
        elements: Vec<ElementData>,
        html: &str,
        viewport_width: u32,
        viewport_height: u32,
        url: &str,
        title: &str,
        focus_element: Option<&str>,
        interactive_count: usize,
    ) {
        let mut state = self.state.lock().await;
        if state.events.len() < self.max_events {
            state.events.push(TraceEvent::Snapshot(DomSnapshot {
                timestamp: now_ms(),
                snapshot_type: snapshot_type.to_string(),
                accessibility_tree,
                elements,
                html: html.to_string(),
                ai_metadata: AiSnapshotMetadata {
                    viewport_width,
                    viewport_height,
                    url: url.to_string(),
                    title: title.to_string(),
                    focus_element: focus_element.map(|s| s.to_string()),
                    interactive_count,
                },
            }));
        }
    }

    pub async fn get_events(&self) -> Vec<TraceEvent> {
        let state = self.state.lock().await;
        state.events.clone()
    }

    pub async fn clear(&self) {
        let mut state = self.state.lock().await;
        state.events.clear();
        state.action_count = 0;
        state.network_count = 0;
        state.console_count = 0;
        state.screencast_count = 0;
    }

    pub async fn stats(&self) -> TraceStats {
        let state = self.state.lock().await;
        TraceStats {
            total_events: state.events.len(),
            action_count: state.action_count,
            network_count: state.network_count,
            console_count: state.console_count,
            screencast_count: state.screencast_count,
        }
    }
}

#[derive(Debug, Clone)]
pub struct TraceStats {
    pub total_events: usize,
    pub action_count: usize,
    pub network_count: usize,
    pub console_count: usize,
    pub screencast_count: usize,
}

fn now_ms() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis() as u64
}

lazy_static::lazy_static! {
    pub static ref GLOBAL_RECORDER: TraceRecorder = TraceRecorder::new(None);
}
