pub mod snapshot;
pub mod compare;

pub use snapshot::{ScreenshotOptions, ScreenshotType, SnapshotResult};
pub use compare::{PixelDiff, DiffOptions, ComparisonResult, SemanticDiff};
