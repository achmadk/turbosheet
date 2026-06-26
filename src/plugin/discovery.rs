use napi::bindgen_prelude::*;
use napi_derive::napi;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginInfo {
    pub name: String,
    pub version: String,
    pub path: String,
    pub plugin_type: String,
}

#[napi(object)]
pub struct JsPluginInfo {
    pub name: String,
    pub version: String,
    pub path: String,
    pub plugin_type: String,
}

impl From<PluginInfo> for JsPluginInfo {
    fn from(p: PluginInfo) -> Self {
        JsPluginInfo {
            name: p.name,
            version: p.version,
            path: p.path,
            plugin_type: p.plugin_type,
        }
    }
}

#[napi]
pub fn discover_plugins() -> Vec<JsPluginInfo> {
    let mut plugins = Vec::new();

    let node_modules = std::env::var("NODE_PATH").unwrap_or_else(|_| "node_modules".to_string());
    let search_path = PathBuf::from(&node_modules);
    if search_path.exists() {
        if let Ok(entries) = std::fs::read_dir(&search_path) {
            for entry in entries.flatten() {
                let name = entry.file_name().to_string_lossy().to_string();
                if name.starts_with("tsheet-plugin-") || name.starts_with("@tsheet/") {
                    let path = entry.path();
                    let package_json_path = path.join("package.json");

                    if package_json_path.exists() {
                        if let Ok(content) = std::fs::read_to_string(&package_json_path) {
                            if let Ok(pkg) = serde_json::from_str::<serde_json::Value>(&content) {
                                plugins.push(JsPluginInfo {
                                    name: pkg.get("name")
                                        .and_then(|v| v.as_str())
                                        .unwrap_or(&name)
                                        .to_string(),
                                    version: pkg.get("version")
                                        .and_then(|v| v.as_str())
                                        .unwrap_or("0.0.0")
                                        .to_string(),
                                    path: path.to_string_lossy().to_string(),
                                    plugin_type: "js".to_string(),
                                });
                            }
                        }
                    }
                }
            }
        }
    }

    if let Some(home) = dirs::home_dir() {
        let tsheet_plugins = home.join(".tsheet").join("plugins");
        if tsheet_plugins.exists() {
            if let Ok(entries) = std::fs::read_dir(&tsheet_plugins) {
                for entry in entries.flatten() {
                    let path = entry.path();
                    if path.extension().map(|e| e == "so" || e == "dylib").unwrap_or(false) {
                        let name = path.file_stem()
                            .map(|s| s.to_string_lossy().to_string())
                            .unwrap_or_else(|| "unknown".to_string());
                        plugins.push(JsPluginInfo {
                            name: name.clone(),
                            version: "0.0.0".to_string(),
                            path: path.to_string_lossy().to_string(),
                            plugin_type: "rust".to_string(),
                        });
                    }
                }
            }
        }
    }

    plugins
}

#[napi]
pub fn get_plugin_exports(plugin_path: String) -> Result<HashMap<String, String>> {
    let mut exports = HashMap::new();

    let path = PathBuf::from(&plugin_path);
    let package_json_path = path.join("package.json");

    if package_json_path.exists() {
        if let Ok(content) = std::fs::read_to_string(&package_json_path) {
            if let Ok(pkg) = serde_json::from_str::<serde_json::Value>(&content) {
                if let Some(ts_sheet) = pkg.get("tsSheet") {
                    if let Some(exports_obj) = ts_sheet.get("exports") {
                        if let Some(obj) = exports_obj.as_object() {
                            for (key, value) in obj {
                                if let Some(v) = value.as_str() {
                                    exports.insert(key.clone(), v.to_string());
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    if exports.is_empty() {
        exports.insert("default".to_string(), "./dist/index.js".to_string());
    }

    Ok(exports)
}
