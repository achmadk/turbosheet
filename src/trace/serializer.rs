use serde::{Deserialize, Serialize};
use std::io::Write;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TraceMetadata {
    pub action_count: usize,
    pub network_count: usize,
    pub console_count: usize,
    pub snapshot_count: usize,
    pub screencast_count: usize,
    pub video_path: Option<String>,
    pub compression: String,
    pub original_size: usize,
    pub compressed_size: usize,
}

pub struct TraceWriter<W: std::io::Write> {
    encoder: zstd::Encoder<'static, W>,
}

impl<W: std::io::Write> TraceWriter<W> {
    pub fn new(mut writer: W, metadata: &TraceMetadata) -> Result<Self, TraceError> {
        let metadata_json = serde_json::to_vec(metadata)
            .map_err(|e| TraceError::Serialization(e.to_string()))?;
            
        writer.write_all(b"TTRC\x01\x00\x00\x00").map_err(|e| TraceError::Io(e.to_string()))?;
        writer.write_all(&(metadata_json.len() as u32).to_le_bytes()).map_err(|e| TraceError::Io(e.to_string()))?;
        writer.write_all(&metadata_json).map_err(|e| TraceError::Io(e.to_string()))?;
        
        let encoder = zstd::Encoder::new(writer, 0)
            .map_err(|e| TraceError::Compression(e.to_string()))?;
            
        Ok(Self { encoder })
    }
    
    pub fn write_event(&mut self, event: &super::TraceEvent) -> Result<(), TraceError> {
        ciborium::ser::into_writer(event, &mut self.encoder)
            .map_err(|e| TraceError::Serialization(e.to_string()))?;
        Ok(())
    }
    
    pub fn finish(self) -> Result<(), TraceError> {
        self.encoder.finish().map_err(|e| TraceError::Io(e.to_string()))?;
        Ok(())
    }
}

pub fn serialize_trace(
    events: &[super::TraceEvent],
    _test_name: &str,
    video_path: Option<&str>,
) -> Result<Vec<u8>, TraceError> {
    let mut result = Vec::new();

    let metadata = TraceMetadata {
        action_count: events.iter().filter(|e| matches!(e, super::TraceEvent::Action(_))).count(),
        network_count: events.iter().filter(|e| matches!(e, super::TraceEvent::Network(_))).count(),
        console_count: events.iter().filter(|e| matches!(e, super::TraceEvent::Console(_))).count(),
        snapshot_count: events.iter().filter(|e| matches!(e, super::TraceEvent::Snapshot(_))).count(),
        screencast_count: events.iter().filter(|e| matches!(e, super::TraceEvent::Screencast(_))).count(),
        video_path: video_path.map(|s| s.to_string()),
        compression: "cbor+zstd".to_string(),
        original_size: 0,
        compressed_size: 0,
    };

    let mut writer = TraceWriter::new(&mut result, &metadata)?;
    for event in events {
        writer.write_event(event)?;
    }
    writer.finish()?;

    Ok(result)
}

pub fn deserialize_trace(data: &[u8]) -> Result<(Vec<super::TraceEvent>, TraceMetadata), TraceError> {
    if data.len() < 12 {
        return Err(TraceError::InvalidFormat("Data too short".to_string()));
    }

    if &data[0..8] != b"TTRC\x01\x00\x00\x00" {
        // Fallback for old traces
        if &data[0..8] == b"TSHEETTR" {
            let metadata_len = u32::from_le_bytes(data[8..12].try_into().unwrap()) as usize;
            let events_len = u32::from_le_bytes(data[12..16].try_into().unwrap()) as usize;
            let metadata_json = &data[16..16 + metadata_len];
            let events_compressed = &data[16 + metadata_len..16 + metadata_len + events_len];
            let metadata: TraceMetadata = serde_json::from_slice(metadata_json)
                .map_err(|e| TraceError::Deserialization(e.to_string()))?;
            let events_cbor = zstd::decode_all(std::io::Cursor::new(events_compressed))
                .map_err(|e| TraceError::Decompression(e.to_string()))?;
            let events: Vec<super::TraceEvent> = ciborium::de::from_reader(&events_cbor[..])
                .map_err(|e| TraceError::Deserialization(e.to_string()))?;
            return Ok((events, metadata));
        }
        return Err(TraceError::InvalidFormat("Not a TurboSheet trace file".to_string()));
    }

    let metadata_len = u32::from_le_bytes(data[8..12].try_into().unwrap()) as usize;
    if data.len() < 12 + metadata_len {
        return Err(TraceError::InvalidFormat("Data truncated".to_string()));
    }

    let metadata_json = &data[12..12 + metadata_len];
    let events_compressed = &data[12 + metadata_len..];

    let metadata: TraceMetadata = serde_json::from_slice(metadata_json)
        .map_err(|e| TraceError::Deserialization(e.to_string()))?;

    let mut decoder = zstd::Decoder::new(std::io::Cursor::new(events_compressed))
        .map_err(|e| TraceError::Decompression(e.to_string()))?;
        
    let mut events = Vec::new();
    while let Ok(event) = ciborium::de::from_reader::<super::TraceEvent, _>(&mut decoder) {
        events.push(event);
    }

    Ok((events, metadata))
}

pub fn trace_to_json_string(events: &[super::TraceEvent]) -> Result<String, TraceError> {
    serde_json::to_string_pretty(events)
        .map_err(|e| TraceError::Serialization(e.to_string()))
}

pub fn snapshot_to_accessibility_tree(snapshot: &super::DomSnapshot) -> String {
    let mut result = String::new();
    fn render_node(node: &super::AccessibilityNode, depth: usize, tree: &mut String) {
        let indent = "  ".repeat(depth);
        let state = if node.state.is_empty() {
            String::new()
        } else {
            format!(" [{}]", node.state.join(", "))
        };
        let value = node.value.as_ref().map(|v| format!(" = \"{}\"", v)).unwrap_or_default();
        let name = if node.name.is_empty() {
            String::new()
        } else {
            format!(" \"{}\"", node.name)
        };
        *tree += &format!("{}{}: {}{}{}\n", indent, node.role, name, value, state);
        for child_idx in &node.children {
            if *child_idx < 10000 {
                *tree += &format!("{}  (child {})\n", indent, child_idx);
            }
        }
    }
    for node in &snapshot.accessibility_tree {
        if node.role != "root" {
            render_node(node, 0, &mut result);
        }
    }
    if result.is_empty() {
        result = "[No accessibility tree available]".to_string();
    }
    result
}

#[derive(Debug)]
pub enum TraceError {
    Serialization(String),
    Deserialization(String),
    Compression(String),
    Decompression(String),
    Io(String),
    InvalidFormat(String),
}

impl std::fmt::Display for TraceError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TraceError::Serialization(s) => write!(f, "Serialization error: {}", s),
            TraceError::Deserialization(s) => write!(f, "Deserialization error: {}", s),
            TraceError::Compression(s) => write!(f, "Compression error: {}", s),
            TraceError::Decompression(s) => write!(f, "Decompression error: {}", s),
            TraceError::Io(s) => write!(f, "IO error: {}", s),
            TraceError::InvalidFormat(s) => write!(f, "Invalid format: {}", s),
        }
    }
}

impl std::error::Error for TraceError {}
