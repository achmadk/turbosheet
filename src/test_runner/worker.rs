use std::process::Stdio;
use tokio::process::{Child, Command};
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use crate::error::TurbosheetError;
use super::ipc::{JsonRpcRequest, JsonRpcResponse};

pub struct WorkerProcess {
    process: Child,
    ready: bool,
}

impl WorkerProcess {
    /// Spawn a Node.js worker process that runs the isolated-vm test executor.
    /// The worker communicates via newline-delimited JSON-RPC on stdin/stdout.
    pub async fn spawn() -> Result<Self, TurbosheetError> {
        // Locate the worker script relative to the package root.
        // The script lives at src/runtime/worker-entry.ts
        let worker_script_path = Self::find_worker_script()?;

        Self::spawn_with_script(&worker_script_path).await
    }

    /// Find the worker-entry script by searching common locations.
    fn find_worker_script() -> Result<String, TurbosheetError> {
        let candidates = [
            // When running from the package directory
            "src/runtime/worker-entry.ts",
            // When installed as a node module
            "node_modules/turbosheet/src/runtime/worker-entry.ts",
        ];

        let base_dir = std::env::current_dir().unwrap_or_default();

        for candidate in &candidates {
            let full_path = base_dir.join(candidate);
            if full_path.exists() {
                return Ok(full_path.to_string_lossy().to_string());
            }
        }

        Err(TurbosheetError::Other(format!(
            "Worker script not found. Searched in: {:?}",
            candidates
                .iter()
                .map(|c| base_dir.join(c).display().to_string())
                .collect::<Vec<_>>()
        )))
    }

    /// Spawn a worker using the compiled JS worker script path directly.
    pub async fn spawn_with_script(script_path: &str) -> Result<Self, TurbosheetError> {
        // Use tsx for TypeScript files, plain node for JavaScript
        let is_ts = script_path.ends_with(".ts");
        let mut cmd = if is_ts {
            // Try npx tsx first (most common way to run TS in Node.js projects)
            let mut c = Command::new("npx");
            c.arg("tsx").arg(script_path);
            c
        } else {
            let mut c = Command::new("node");
            c.arg(script_path);
            c
        };
        cmd.stdin(Stdio::piped());
        cmd.stdout(Stdio::piped());
        cmd.stderr(Stdio::piped());

        if let Ok(cwd) = std::env::current_dir() {
            cmd.current_dir(cwd);
        }

        let process = cmd.spawn().map_err(|e| {
            TurbosheetError::Other(format!(
                "Failed to spawn worker with script {}: {}",
                script_path, e
            ))
        })?;

        let mut worker = Self {
            process,
            ready: false,
        };

        worker.wait_for_ready().await?;
        Ok(worker)
    }

    /// Wait for the worker to send its "ready" notification.
    async fn wait_for_ready(&mut self) -> Result<(), TurbosheetError> {
        let stdout = self.process.stdout.as_mut().ok_or_else(|| {
            TurbosheetError::Other("Worker stdout not available".to_string())
        })?;

        let mut reader = BufReader::new(stdout);
        let mut line = String::new();

        // Read the first line, which should be the ready notification
        let timeout = tokio::time::timeout(
            std::time::Duration::from_secs(10),
            reader.read_line(&mut line),
        )
        .await
        .map_err(|_| TurbosheetError::Other("Worker startup timed out after 10s".to_string()))?
        .map_err(|e| TurbosheetError::Other(format!("Failed to read from worker: {}", e)))?;

        if timeout == 0 {
            return Err(TurbosheetError::Other(
                "Worker process exited before sending ready signal".to_string(),
            ));
        }

        // Parse the ready notification
        if let Ok(msg) = serde_json::from_str::<serde_json::Value>(&line) {
            if msg.get("method").and_then(|m| m.as_str()) == Some("ready") {
                self.ready = true;
                return Ok(());
            }
        }

        Err(TurbosheetError::Other(format!(
            "Unexpected first message from worker: {}",
            line.trim()
        )))
    }

    /// Send a JSON-RPC request and read the response.
    pub async fn send_request(
        &mut self,
        req: JsonRpcRequest,
    ) -> Result<JsonRpcResponse, TurbosheetError> {
        let stdin = self.process.stdin.as_mut().ok_or_else(|| {
            TurbosheetError::Other("Worker stdin not available".to_string())
        })?;

        let mut payload = serde_json::to_string(&req).map_err(|e| {
            TurbosheetError::Other(format!("Failed to serialize request: {}", e))
        })?;
        payload.push('\n');

        stdin
            .write_all(payload.as_bytes())
            .await
            .map_err(|e| TurbosheetError::Other(format!("Failed to write to worker stdin: {}", e)))?;
        stdin
            .flush()
            .await
            .map_err(|e| TurbosheetError::Other(format!("Failed to flush worker stdin: {}", e)))?;

        // Read response line
        let stdout = self.process.stdout.as_mut().ok_or_else(|| {
            TurbosheetError::Other("Worker stdout not available".to_string())
        })?;

        let mut reader = BufReader::new(stdout);
        let mut line = String::new();

        let timeout = tokio::time::timeout(
            std::time::Duration::from_secs(60),
            reader.read_line(&mut line),
        )
        .await
        .map_err(|_| {
            TurbosheetError::Other("Worker response timed out after 60s".to_string())
        })?
        .map_err(|e| TurbosheetError::Other(format!("Failed to read worker response: {}", e)))?;

        if timeout == 0 {
            return Err(TurbosheetError::Other(
                "Worker process exited while waiting for response".to_string(),
            ));
        }

        let response: JsonRpcResponse = serde_json::from_str(&line).map_err(|e| {
            TurbosheetError::Other(format!(
                "Failed to parse worker response: {} (raw: {})",
                e,
                line.trim()
            ))
        })?;

        Ok(response)
    }

    /// Send a request to execute a test file and return parsed results.
    /// `video_on_failure` and `video_dir` control video recording during tests.
    pub async fn execute_test(
        &mut self,
        file_path: &str,
        timeout_ms: u32,
        video_on_failure: Option<bool>,
        video_dir: Option<&str>,
    ) -> Result<Vec<WorkerTestResult>, TurbosheetError> {
        let req = JsonRpcRequest::new(
            "executeTest",
            serde_json::json!({
                "filePath": file_path,
                "timeout": timeout_ms,
                "videoOnFailure": video_on_failure,
                "videoDir": video_dir,
            }),
        );

        let response = self.send_request(req).await?;

        if let Some(error) = response.error {
            return Err(TurbosheetError::Other(format!(
                "Worker execution error: {}",
                error.message
            )));
        }

        if let Some(result) = response.result {
            let results: Vec<WorkerTestResult> =
                serde_json::from_value(result["results"].clone()).map_err(|e| {
                    TurbosheetError::Other(format!("Failed to parse test results: {}", e))
                })?;
            Ok(results)
        } else {
            Err(TurbosheetError::Other(
                "Worker returned no result".to_string(),
            ))
        }
    }

    /// Gracefully shut down the worker.
    pub async fn shutdown(&mut self) -> Result<(), TurbosheetError> {
        let req = JsonRpcRequest::new("shutdown", serde_json::json!({}));
        let _ = self.send_request(req).await;
        Ok(())
    }

    /// Forcefully kill the worker process.
    pub async fn kill(&mut self) -> Result<(), TurbosheetError> {
        self.process
            .kill()
            .await
            .map_err(|e| TurbosheetError::Other(e.to_string()))?;
        Ok(())
    }

    pub fn is_ready(&self) -> bool {
        self.ready
    }
}

/// Test result as received from the worker process.
#[derive(Debug, Clone, serde::Deserialize)]
pub struct WorkerTestResult {
    pub name: String,
    pub status: String,
    pub error: Option<String>,
    pub duration_ms: u32,
    /// Paths to recorded video files for this test, if any.
    #[serde(default)]
    pub video_paths: Option<Vec<String>>,
}
