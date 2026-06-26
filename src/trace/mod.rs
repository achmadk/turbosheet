pub mod recorder;
pub mod serializer;
pub mod viewer;

pub use recorder::{TraceRecorder, TraceEvent, ActionEvent, NetworkEvent, ConsoleEvent, DomSnapshot, AccessibilityNode, ElementData, AiSnapshotMetadata, ScreencastFrameEvent, GLOBAL_RECORDER};
pub use serializer::{serialize_trace, deserialize_trace, trace_to_json_string, snapshot_to_accessibility_tree, TraceMetadata};
