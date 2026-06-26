use std::sync::Arc;
use std::time::Instant;
use tokio::sync::mpsc;
use tokio::task::JoinHandle;
use tracing::{info, error, warn};
use crate::error::TurbosheetError;
use super::config::TestConfig;
use super::discovery::TestFile;
use super::worker::{WorkerProcess, WorkerTestResult};

use napi_derive::napi;

#[napi(string_enum)]
#[derive(Debug, Clone)]
pub enum TestStatus {
    Passed,
    Failed,
    Skipped,
    Timeout,
}

#[napi(object)]
#[derive(Debug, Clone)]
pub struct TestResult {
    pub file: String,
    pub name: String,
    pub status: TestStatus,
    pub error_message: Option<String>,
    pub duration_ms: u32,
    pub retries: u32,
    pub screenshot_paths: Option<Vec<String>>,
    pub trace_data: Option<String>,
    pub video_paths: Option<Vec<String>>,
}

pub struct TestExecutor {
    config: Arc<TestConfig>,
}

impl TestExecutor {
    pub fn new(config: TestConfig) -> Self {
        Self {
            config: Arc::new(config),
        }
    }

    pub async fn execute(&self, files: Vec<TestFile>) -> Result<Vec<TestResult>, TurbosheetError> {
        let workers = self.config.workers.unwrap_or(1) as usize;
        let screenshot_on_failure = self.config.screenshot_on_failure.unwrap_or(true);
        let screenshot_dir = self.config.screenshot_dir.clone()
            .unwrap_or_else(|| "test-results/screenshots".to_string());
        let video_on_failure = self.config.video_on_failure.unwrap_or(false);
        let video_dir = self.config.video_dir.clone()
            .unwrap_or_else(|| "test-results/videos".to_string());

        info!("Starting test execution with {} workers", workers);
        info!("Screenshot on failure: {}, dir: {}", screenshot_on_failure, screenshot_dir);
        info!("Video on failure: {}, dir: {}", video_on_failure, video_dir);

        // Ensure output directories exist
        if let Err(e) = std::fs::create_dir_all(&screenshot_dir) {
            warn!("Failed to create screenshot directory {}: {}", screenshot_dir, e);
        }
        if video_on_failure {
            if let Err(e) = std::fs::create_dir_all(&video_dir) {
                warn!("Failed to create video directory {}: {}", video_dir, e);
            }
        }

        if let Some(ref setup_file) = self.config.global_setup {
            self.run_global_setup(setup_file).await?;
        }

        let result = self.execute_tests(files, workers, screenshot_on_failure, &screenshot_dir, video_on_failure, &video_dir).await;

        if let Some(ref teardown_file) = self.config.global_teardown {
            self.run_global_teardown(teardown_file).await;
        }

        result
    }

    async fn run_global_setup(&self, setup_file: &str) -> Result<(), TurbosheetError> {
        info!("Running global setup: {}", setup_file);
        let path = std::path::Path::new(setup_file);
        if path.exists() {
            // Execute global setup via a worker process
            let mut worker = WorkerProcess::spawn().await.map_err(|e| {
                warn!("Failed to spawn worker for global setup, skipping: {}", e);
                e
            })?;

            let results = worker.execute_test(setup_file, 60000, None, None).await;
            let _ = worker.shutdown().await;

            match results {
                Ok(r) => {
                    let failures: Vec<_> = r.iter().filter(|t| t.status == "failed").collect();
                    if !failures.is_empty() {
                        return Err(TurbosheetError::Other(format!(
                            "Global setup failed: {}",
                            failures[0].error.as_deref().unwrap_or("unknown error")
                        )));
                    }
                    info!("Global setup executed successfully");
                }
                Err(e) => {
                    warn!("Global setup execution error (continuing): {}", e);
                }
            }
        } else {
            error!("Global setup file not found: {}", setup_file);
        }
        Ok(())
    }

    async fn run_global_teardown(&self, teardown_file: &str) {
        info!("Running global teardown: {}", teardown_file);
        let path = std::path::Path::new(teardown_file);
        if path.exists() {
            match WorkerProcess::spawn().await {
                Ok(mut worker) => {
                    let _ = worker.execute_test(teardown_file, 60000, None, None).await;
                    let _ = worker.shutdown().await;
                    info!("Global teardown executed successfully");
                }
                Err(e) => {
                    error!("Failed to spawn worker for global teardown: {}", e);
                }
            }
        } else {
            error!("Global teardown file not found: {}", teardown_file);
        }
    }

    async fn execute_tests(
        &self,
        files: Vec<TestFile>,
        workers: usize,
        screenshot_on_failure: bool,
        screenshot_dir: &str,
        video_on_failure: bool,
        video_dir: &str,
    ) -> Result<Vec<TestResult>, TurbosheetError> {
        let (tx, mut rx) = mpsc::channel(100);
        let mut handles: Vec<JoinHandle<()>> = Vec::new();

        let chunk_size = (files.len() + workers - 1) / workers;
        if chunk_size == 0 {
            return Ok(Vec::new());
        }

        let timeout_ms = self.config.timeout.unwrap_or(30000);
        let max_retries = self.config.retries.unwrap_or(0);

        for chunk in files.chunks(chunk_size) {
            let chunk = chunk.to_vec();
            let tx = tx.clone();
            let timeout_ms = timeout_ms;
            let max_retries = max_retries;
            let screenshot_on_failure = screenshot_on_failure;
            let _screenshot_dir = screenshot_dir.to_string();
            let video_on_failure = video_on_failure;
            let video_dir = video_dir.to_string();

            let handle = tokio::spawn(async move {
                // Each worker task spawns its own Node.js worker process
                let mut worker = match WorkerProcess::spawn().await {
                    Ok(w) => w,
                    Err(e) => {
                        error!("Failed to spawn worker: {}. Falling back to stub results.", e);
                        // Fallback: produce stub results if worker can't be spawned
                        for file in chunk {
                            let start_time = Instant::now();
                            let file_name = file.path.file_name()
                                .map(|n| n.to_string_lossy().to_string())
                                .unwrap_or_else(|| "unknown".to_string());

                            let test_name = file_name.replace(".tsheet.ts", "")
                                .replace(".spec.ts", "")
                                .replace('_', " ")
                                .replace('-', " ");

                            let result = TestResult {
                                file: file.path.to_string_lossy().to_string(),
                                name: test_name,
                                status: TestStatus::Failed,
                                error_message: Some(format!("Worker process unavailable: {}", e)),
                                duration_ms: start_time.elapsed().as_millis() as u32,
                                retries: 0,
                                screenshot_paths: None,
                                trace_data: None,
                                video_paths: None,
                            };

                            if let Err(send_err) = tx.send(result).await {
                                error!("Failed to send test result: {}", send_err);
                            }
                        }
                        return;
                    }
                };

                for file in chunk {
                    let file_path = file.path.to_string_lossy().to_string();
                    let mut attempt = 0;
                    let mut last_results: Vec<WorkerTestResult> = Vec::new();

                    loop {
                        match worker.execute_test(&file_path, timeout_ms, Some(video_on_failure), Some(&video_dir)).await {
                            Ok(worker_results) => {
                                last_results = worker_results;

                                // Check if any tests failed and we have retries left
                                let has_failures = last_results.iter().any(|r| {
                                    r.status == "failed" || r.status == "timeout"
                                });

                                if has_failures && attempt < max_retries {
                                    attempt += 1;
                                    info!("Retrying file {} (attempt {}/{})", file_path, attempt + 1, max_retries + 1);
                                    continue;
                                }

                                // Convert worker results to TestResult
                                for wr in &last_results {
                                    let status = match wr.status.as_str() {
                                        "passed" => TestStatus::Passed,
                                        "failed" => TestStatus::Failed,
                                        "skipped" => TestStatus::Skipped,
                                        "timeout" => TestStatus::Timeout,
                                        _ => TestStatus::Failed,
                                    };

                                    let screenshot_paths = if screenshot_on_failure
                                        && matches!(status, TestStatus::Failed | TestStatus::Timeout)
                                    {
                                        // TODO: capture screenshot via CDP when page is available
                                        None
                                    } else {
                                        None
                                    };

                                    let video_paths = if video_on_failure
                                        && matches!(status, TestStatus::Failed | TestStatus::Timeout)
                                    {
                                        wr.video_paths.clone()
                                    } else if !video_on_failure {
                                        // If video is always on, still capture paths
                                        wr.video_paths.clone()
                                    } else {
                                        None
                                    };

                                    let result = TestResult {
                                        file: file_path.clone(),
                                        name: wr.name.clone(),
                                        status,
                                        error_message: wr.error.clone(),
                                        duration_ms: wr.duration_ms,
                                        retries: attempt,
                                        screenshot_paths,
                                        trace_data: None,
                                        video_paths,
                                    };

                                    if let Err(e) = tx.send(result).await {
                                        error!("Failed to send test result: {}", e);
                                    }
                                }
                                break;
                            }
                            Err(e) => {
                                error!("Worker execution failed for {}: {}", file_path, e);
                                let file_name = file.path.file_name()
                                    .map(|n| n.to_string_lossy().to_string())
                                    .unwrap_or_else(|| "unknown".to_string());

                                let result = TestResult {
                                    file: file_path.clone(),
                                    name: file_name,
                                    status: TestStatus::Failed,
                                    error_message: Some(format!("Worker error: {}", e)),
                                    duration_ms: 0,
                                    retries: attempt,
                                    screenshot_paths: None,
                                    trace_data: None,
                                    video_paths: None,
                                };

                                if let Err(send_err) = tx.send(result).await {
                                    error!("Failed to send test result: {}", send_err);
                                }
                                break;
                            }
                        }
                    }
                }

                // Gracefully shut down the worker
                if let Err(e) = worker.shutdown().await {
                    warn!("Failed to gracefully shutdown worker: {}", e);
                    let _ = worker.kill().await;
                }
            });

            handles.push(handle);
        }

        drop(tx);

        let mut results = Vec::new();
        while let Some(result) = rx.recv().await {
            results.push(result);
        }

        for handle in handles {
            let _ = handle.await;
        }

        info!("Completed execution of {} tests", results.len());
        Ok(results)
    }

    pub fn should_capture_screenshot_on_failure(&self) -> bool {
        self.config.screenshot_on_failure.unwrap_or(true)
    }

    pub fn get_screenshot_dir(&self) -> String {
        self.config.screenshot_dir.clone()
            .unwrap_or_else(|| "test-results/screenshots".to_string())
    }

    pub fn create_test_result(
        file: &std::path::Path,
        name: String,
        status: TestStatus,
        error_message: Option<String>,
        duration_ms: u32,
        retries: u32,
    ) -> TestResult {
        TestResult {
            file: file.to_string_lossy().to_string(),
            name,
            status,
            error_message,
            duration_ms,
            retries,
            screenshot_paths: None,
            trace_data: None,
            video_paths: None,
        }
    }

    pub fn status_from_error(error: Option<&str>) -> TestStatus {
        match error {
            Some(msg) if msg.contains("timeout") => TestStatus::Timeout,
            Some(_) => TestStatus::Failed,
            None => TestStatus::Passed,
        }
    }

    pub async fn execute_project_refs(&self) -> Result<Vec<TestResult>, TurbosheetError> {
        let mut all_results = Vec::new();

        if let Some(ref projects) = self.config.projects {
            for project in projects {
                info!("Executing project: {}", project.name);
                let files = super::discovery::discover_tests(&project.test_dir, &project.test_match.clone().unwrap_or_else(|| vec!["**/*.tsheet.ts".to_string()]), None)?;
                let results = self.execute_tests(files, 1, false, "test-results", false, "test-results/videos").await?;
                all_results.extend(results);
            }
        }

        Ok(all_results)
    }
}
