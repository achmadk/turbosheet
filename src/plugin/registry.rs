use napi::bindgen_prelude::*;
use napi_derive::napi;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::Mutex;

use super::discovery::PluginInfo;

#[derive(Default)]
pub struct PluginRegistry {
    pub reporters: Vec<ReporterPlugin>,
    pub matchers: Vec<MatcherPlugin>,
    pub plugins: Vec<PluginInfo>,
}

impl PluginRegistry {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn register_reporter(&mut self, reporter: ReporterPlugin) {
        self.reporters.push(reporter);
    }

    pub fn register_matcher(&mut self, matcher: MatcherPlugin) {
        self.matchers.push(matcher);
    }

    pub fn get_reporters(&self) -> &[ReporterPlugin] {
        &self.reporters
    }

    pub fn get_matchers(&self) -> &[MatcherPlugin] {
        &self.matchers
    }
}

lazy_static::lazy_static! {
    pub static ref PLUGIN_REGISTRY: Arc<Mutex<PluginRegistry>> = Arc::new(Mutex::new(PluginRegistry::new()));
}

#[napi(object)]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReporterPlugin {
    pub name: String,
    pub path: String,
    pub enabled: bool,
}

#[napi(object)]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MatcherPlugin {
    pub name: String,
    pub path: String,
    pub matcher_names: Vec<String>,
    pub enabled: bool,
}

#[napi]
pub async fn register_js_reporter(name: String, path: String) -> Result<()> {
    let mut registry = PLUGIN_REGISTRY.lock().await;
    registry.register_reporter(ReporterPlugin {
        name,
        path,
        enabled: true,
    });
    Ok(())
}

#[napi]
pub async fn register_js_matcher(name: String, path: String, matcher_names: Vec<String>) -> Result<()> {
    let mut registry = PLUGIN_REGISTRY.lock().await;
    registry.register_matcher(MatcherPlugin {
        name,
        path,
        matcher_names,
        enabled: true,
    });
    Ok(())
}

#[napi]
pub async fn get_registered_reporters() -> Vec<ReporterPlugin> {
    let registry = PLUGIN_REGISTRY.lock().await;
    registry.get_reporters().to_vec()
}

#[napi]
pub async fn get_registered_matchers() -> Vec<MatcherPlugin> {
    let registry = PLUGIN_REGISTRY.lock().await;
    registry.get_matchers().to_vec()
}

#[napi]
pub async fn enable_plugin(name: String, enabled: bool) -> Result<()> {
    let mut registry = PLUGIN_REGISTRY.lock().await;

    for reporter in &mut registry.reporters {
        if reporter.name == name {
            reporter.enabled = enabled;
            return Ok(());
        }
    }

    for matcher in &mut registry.matchers {
        if matcher.name == name {
            matcher.enabled = enabled;
            return Ok(());
        }
    }

    Err(napi::Error::from_reason(format!("Plugin '{}' not found", name)))
}

#[napi]
pub async fn unregister_plugin(name: String) -> Result<()> {
    let mut registry = PLUGIN_REGISTRY.lock().await;

    if let Some(pos) = registry.reporters.iter().position(|r| r.name == name) {
        registry.reporters.remove(pos);
        return Ok(());
    }

    if let Some(pos) = registry.matchers.iter().position(|m| m.name == name) {
        registry.matchers.remove(pos);
        return Ok(());
    }

    Err(napi::Error::from_reason(format!("Plugin '{}' not found", name)))
}
