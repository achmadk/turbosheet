use napi::bindgen_prelude::*;
use napi_derive::napi;
use serde::{Deserialize, Serialize};

#[napi(object)]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TestConfig {
    pub test_dir: Option<String>,
    pub test_match: Option<Vec<String>>,
    pub workers: Option<u32>,
    pub retries: Option<u32>,
    pub timeout: Option<u32>,
    pub reporter: Option<String>,
    pub use_options: Option<String>,
    pub projects: Option<Vec<ProjectReference>>,
    pub grep: Option<String>,
    pub shard: Option<ShardConfig>,
    pub global_setup: Option<String>,
    pub global_teardown: Option<String>,
    pub screenshot_on_failure: Option<bool>,
    pub screenshot_dir: Option<String>,
    pub video_on_failure: Option<bool>,
    pub video_dir: Option<String>,
    pub webserver: Option<WebserverConfig>,
    pub dependencies: Option<Vec<ProjectDependency>>,
    pub update_snapshots: Option<bool>,
    pub headless: Option<bool>,
    pub browser: Option<String>,
    pub output_dir: Option<String>,
}

#[napi(object)]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectReference {
    pub name: String,
    pub test_dir: String,
    pub test_match: Option<Vec<String>>,
    pub dependencies: Option<Vec<String>>,
}

#[napi(object)]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WebserverConfig {
    pub port: Option<u16>,
    pub host: Option<String>,
    pub reuse_existing: Option<bool>,
}

#[napi(object)]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectDependency {
    pub name: String,
    pub workspace: String,
}

#[napi(object)]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ShardConfig {
    pub current: u32,
    pub total: u32,
}

impl Default for TestConfig {
    fn default() -> Self {
        Self {
            test_dir: Some(".".to_string()),
            test_match: Some(vec!["**/*.tsheet.ts".to_string(), "**/*.tsheet.spec.ts".to_string()]),
            workers: Some(std::thread::available_parallelism().map(|n| n.get()).unwrap_or(1) as u32),
            retries: Some(0),
            timeout: Some(30000),
            reporter: Some("list".to_string()),
            use_options: None,
            projects: None,
            global_setup: None,
            global_teardown: None,
            grep: None,
            shard: None,
            screenshot_on_failure: Some(true),
            screenshot_dir: Some("test-results/screenshots".to_string()),
            video_on_failure: Some(false),
            video_dir: Some("test-results/videos".to_string()),
            webserver: None,
            dependencies: None,
            update_snapshots: Some(false),
            headless: None,
            browser: None,
            output_dir: None,
        }
    }
}
