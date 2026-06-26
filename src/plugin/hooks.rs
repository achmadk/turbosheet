use napi::bindgen_prelude::*;
use napi_derive::napi;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TestHook {
    pub test_name: String,
    pub test_file: String,
    pub status: Option<String>,
    pub duration_ms: Option<u64>,
    pub error: Option<String>,
}

#[derive(Default)]
pub struct LifecycleHooks {
    pub on_test_start: Vec<HookCallback>,
    pub on_test_end: Vec<HookCallback>,
    pub on_test_failed: Vec<HookCallback>,
}

pub struct HookCallback {
    pub name: String,
    pub callback: Box<dyn Fn(TestHook) + Send + Sync>,
}

impl LifecycleHooks {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn register_test_start<F>(&mut self, name: String, callback: F)
    where
        F: Fn(TestHook) + Send + Sync + 'static,
    {
        self.on_test_start.push(HookCallback {
            name,
            callback: Box::new(callback),
        });
    }

    pub fn register_test_end<F>(&mut self, name: String, callback: F)
    where
        F: Fn(TestHook) + Send + Sync + 'static,
    {
        self.on_test_end.push(HookCallback {
            name,
            callback: Box::new(callback),
        });
    }

    pub fn register_test_failed<F>(&mut self, name: String, callback: F)
    where
        F: Fn(TestHook) + Send + Sync + 'static,
    {
        self.on_test_failed.push(HookCallback {
            name,
            callback: Box::new(callback),
        });
    }

    pub fn trigger_test_start(&self, hook: TestHook) {
        for cb in &self.on_test_start {
            (cb.callback)(hook.clone());
        }
    }

    pub fn trigger_test_end(&self, hook: TestHook) {
        for cb in &self.on_test_end {
            (cb.callback)(hook.clone());
        }
    }

    pub fn trigger_test_failed(&self, hook: TestHook) {
        for cb in &self.on_test_failed {
            (cb.callback)(hook.clone());
        }
    }
}

#[napi(object)]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginConfig {
    pub name: String,
    pub enabled: Option<bool>,
    pub options: Option<HashMap<String, String>>,
}

#[napi(object)]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HookRegistration {
    pub event: String,
    pub callback: String,
}
