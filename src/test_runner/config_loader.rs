use napi::bindgen_prelude::*;
use napi_derive::napi;
use std::process::Command;
use crate::test_runner::config::TestConfig;

pub struct ConfigLoader;

impl ConfigLoader {
    pub fn load(config_path: &str) -> Result<TestConfig> {
        let worker_script = format!(r#"
            const path = require('path');
            const configPath = path.resolve('{}');
            try {{
                const config = require(configPath).default || require(configPath);
                console.log(JSON.stringify(config));
            }} catch (e) {{
                console.error(e);
                process.exit(1);
            }}
        "#, config_path);
        
        let output = Command::new("npx")
            .args(&["tsx", "-e", &worker_script])
            .output()
            .map_err(|e| Error::from_reason(format!("Failed to execute tsx: {}", e)))?;
            
        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(Error::from_reason(format!("Failed to load config: {}", stderr)));
        }
        
        let stdout = String::from_utf8_lossy(&output.stdout);
        let config: TestConfig = serde_json::from_str(&stdout)
            .map_err(|e| Error::from_reason(format!("Failed to parse config JSON: {}", e)))?;
            
        Ok(config)
    }
}

#[napi]
pub fn load_config(config_path: String) -> Result<TestConfig> {
    ConfigLoader::load(&config_path)
}
