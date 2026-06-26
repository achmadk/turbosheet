pub mod discovery;
pub mod hooks;
pub mod registry;

pub use discovery::{discover_plugins, PluginInfo};
pub use hooks::{LifecycleHooks, TestHook};
pub use registry::{PluginRegistry, ReporterPlugin, MatcherPlugin};
