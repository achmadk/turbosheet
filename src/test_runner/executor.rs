use std::collections::HashMap;
use std::sync::Arc;
use std::time::Instant;
use tokio::sync::mpsc;
use tokio::task::JoinHandle;
use tracing::{info, error, warn};
use crate::error::TurbosheetError;
use crate::reporters::Reporter;
use super::config::TestConfig;
use super::discovery::TestFile;
use super::ipc::{CollectedTest, CollectedHook, CollectedSuite, ExecutionPlan, ExtractResponse, PlanResponse};
use super::worker::WorkerProcess;

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

    /// Execute tests and return results. If `reporter` is provided,
    /// its `on_test_result` is called with each `TestResult` as it arrives,
    /// and `on_complete` is called with the aggregate summary after all tests finish.
    pub async fn execute(
        &self,
        files: Vec<TestFile>,
        reporter: Option<&dyn Reporter>,
    ) -> Result<Vec<TestResult>, TurbosheetError> {
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

        let result = self.execute_tests(files, workers, screenshot_on_failure, &screenshot_dir, video_on_failure, &video_dir, reporter).await;

        if let Some(ref teardown_file) = self.config.global_teardown {
            self.run_global_teardown(teardown_file).await;
        }

        // Notify reporter of completion with aggregate summary
        if let (Ok(ref results), Some(reporter)) = (&result, reporter) {
            let summary = crate::reporters::AggregatedTestResult::from_test_results(results.clone());
            reporter.on_complete(&summary);
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
        reporter: Option<&dyn Reporter>,
    ) -> Result<Vec<TestResult>, TurbosheetError> {
        if files.is_empty() {
            return Ok(Vec::new());
        }

        let timeout_ms = self.config.timeout.unwrap_or(30000);
        let max_retries = self.config.retries.unwrap_or(0);

        // ── Phase 1: Extract test definitions from all files ──────────────
        info!("Extracting test definitions from {} files ({} workers)", files.len(), workers);
        let extract_responses = self.extract_all_files(&files, workers, timeout_ms).await?;

        if extract_responses.is_empty() {
            info!("No test definitions extracted — skipping execution");
            return Ok(Vec::new());
        }

        // ── Phase 2: Build execution plans via PlanBuilder ────────────────
        let mut plan_builder = PlanBuilder::new(timeout_ms, max_retries);
        for response in extract_responses {
            plan_builder.add_extracted_file(response);
        }
        let all_plans = plan_builder.build();
        let total_plans = all_plans.len();
        info!("Built {} execution plans from extracted definitions", total_plans);

        if total_plans == 0 {
            return Ok(Vec::new());
        }

        // ── Phase 3: Execute plans ────────────────────────────────────────
        self.execute_plans(
            all_plans,
            workers,
            screenshot_on_failure,
            screenshot_dir,
            video_on_failure,
            video_dir,
            reporter,
            max_retries,
        ).await
    }

    /// Phase 1: Spawn workers, extract test definitions from each file,
    /// return collected `ExtractResponse`s.
    async fn extract_all_files(
        &self,
        files: &[TestFile],
        workers: usize,
        timeout_ms: u32,
    ) -> Result<Vec<ExtractResponse>, TurbosheetError> {
        let file_count = files.len();
        let chunk_size = (file_count + workers - 1) / workers;
        let mut all_responses = Vec::with_capacity(file_count);

        // Collect responses sequentially by chunks; parallel extraction
        // is handled inside each chunk via the worker pool
        let mut chunk_buffers: Vec<Vec<ExtractResponse>> = Vec::new();

        // Extract in parallel chunks using one worker per chunk
        let mut handles = Vec::new();
        for chunk in files.chunks(chunk_size) {
            let chunk: Vec<TestFile> = chunk.to_vec();
            let timeout_ms = timeout_ms;

            let handle = tokio::spawn(async move {
                let mut responses = Vec::with_capacity(chunk.len());
                let mut worker = match WorkerProcess::spawn().await {
                    Ok(w) => w,
                    Err(e) => {
                        error!("Failed to spawn extract worker: {}", e);
                        return responses;
                    }
                };

                for file in &chunk {
                    let file_path = file.path.to_string_lossy().to_string();
                    match worker.extract_tests(&file_path, timeout_ms).await {
                        Ok(response) => {
                            let count = response.tests.len();
                            info!("Extracted {} tests from {}", count, file_path);
                            responses.push(response);
                        }
                        Err(e) => {
                            error!("Failed to extract tests from {}: {}", file_path, e);
                            // Push an empty response so we still register the file
                            responses.push(ExtractResponse {
                                tests: Vec::new(),
                                hooks: Vec::new(),
                                suites: Vec::new(),
                            });
                        }
                    }
                }

                // Clean up worker
                let _ = worker.shutdown().await;
                responses
            });

            handles.push(handle);
        }

        for handle in handles {
            let chunk_responses = handle.await.unwrap_or_default();
            chunk_buffers.push(chunk_responses);
        }

        for mut buf in chunk_buffers {
            all_responses.append(&mut buf);
        }

        Ok(all_responses)
    }

    /// Phase 3: Execute a list of `ExecutionPlan`s using workers.
    /// Each plan is run independently with retry support.
    /// Tracks suite-level beforeAll failures to cascade skips
    /// to subsequent tests in the same suite.
    async fn execute_plans(
        &self,
        plans: Vec<ExecutionPlan>,
        workers: usize,
        screenshot_on_failure: bool,
        screenshot_dir: &str,
        video_on_failure: bool,
        video_dir: &str,
        reporter: Option<&dyn Reporter>,
        max_retries: u32,
    ) -> Result<Vec<TestResult>, TurbosheetError> {
        let plan_count = plans.len();
        let chunk_size = (plan_count + workers - 1) / workers;
        let (tx, mut rx) = mpsc::channel(100);
        let mut handles: Vec<JoinHandle<()>> = Vec::new();

        // Shared state for suite-level beforeAll failure cascading.
        // Contains suite_paths whose beforeAll has failed.
        let blocked_suites = Arc::new(tokio::sync::Mutex::new(
            std::collections::HashSet::<Vec<String>>::new(),
        ));

        for chunk in plans.into_iter().collect::<Vec<_>>().chunks(chunk_size) {
            let chunk = chunk.to_vec();
            let tx = tx.clone();
            let screenshot_on_failure = screenshot_on_failure;
            let _screenshot_dir = screenshot_dir.to_string();
            let video_on_failure = video_on_failure;
            let video_dir = video_dir.to_string();
            let max_retries = max_retries;
            let blocked_suites = Arc::clone(&blocked_suites);

            let handle = tokio::spawn(async move {
                let mut worker = match WorkerProcess::spawn().await {
                    Ok(w) => w,
                    Err(e) => {
                        error!("Failed to spawn execution worker: {}", e);
                        for plan in chunk {
                            let result = TestResult {
                                file: plan.file_path.clone(),
                                name: plan.test_name.clone(),
                                status: TestStatus::Failed,
                                error_message: Some(format!("Worker unavailable: {}", e)),
                                duration_ms: 0,
                                retries: 0,
                                screenshot_paths: None,
                                trace_data: None,
                                video_paths: None,
                            };
                            let _ = tx.send(result).await;
                        }
                        return;
                    }
                };

                for plan in chunk {
                    // Check if this plan's suite is blocked by a beforeAll failure
                    {
                        let blocked = blocked_suites.lock().await;

                        // Walk the suite path ancestry — any ancestor suite being
                        // blocked means this test is also blocked
                        let is_ancestor_blocked = (0..=plan.suite_path.len()).any(|len| {
                            let prefix: Vec<String> = plan.suite_path[..len].to_vec();
                            blocked.contains(&prefix)
                        });

                        if is_ancestor_blocked {
                            let result = TestResult {
                                file: plan.file_path.clone(),
                                name: plan.test_name.clone(),
                                status: TestStatus::Skipped,
                                error_message: Some(
                                    "Skipped: beforeAll failed in parent suite".to_string(),
                                ),
                                duration_ms: 0,
                                retries: 0,
                                screenshot_paths: None,
                                trace_data: None,
                                video_paths: None,
                            };
                            let _ = tx.send(result).await;
                            continue;
                        }
                    }

                    let mut attempt = 0;

                    loop {
                        match worker.run_plan(
                            plan.clone(),
                            Some(video_on_failure),
                            Some(&video_dir),
                        ).await {
                            Ok(plan_response) => {
                                let has_failure = plan_response.status == "failed"
                                    || plan_response.status == "timeout";

                                if has_failure && attempt < max_retries {
                                    attempt += 1;
                                    info!(
                                        "Retrying plan '{}' ({}/{})",
                                        plan.test_name,
                                        attempt + 1,
                                        max_retries + 1,
                                    );
                                    continue;
                                }

                                // If this test had run_before_all and failed,
                                // block the suite so remaining tests are skipped
                                if has_failure && plan.run_before_all {
                                    blocked_suites.lock().await.insert(plan.suite_path.clone());
                                }

                                let status = match plan_response.status.as_str() {
                                    "passed" => TestStatus::Passed,
                                    "failed" => TestStatus::Failed,
                                    "skipped" => TestStatus::Skipped,
                                    "timeout" => TestStatus::Timeout,
                                    _ => TestStatus::Failed,
                                };

                                let should_screenshot = screenshot_on_failure
                                    && matches!(status, TestStatus::Failed | TestStatus::Timeout);

                                let result = TestResult {
                                    file: plan.file_path.clone(),
                                    name: plan.test_name.clone(),
                                    status,
                                    error_message: plan_response.error,
                                    duration_ms: plan_response.duration_ms,
                                    retries: attempt,
                                    screenshot_paths: if should_screenshot {
                                        None // TODO: capture screenshot via CDP
                                    } else {
                                        None
                                    },
                                    trace_data: None,
                                    video_paths: None,
                                };

                                let _ = tx.send(result).await;
                                break;
                            }
                            Err(e) => {
                                error!("Plan execution failed for '{}': {}", plan.test_name, e);

                                // Worker error also blocks the suite if beforeAll was involved
                                if plan.run_before_all {
                                    blocked_suites.lock().await.insert(plan.suite_path.clone());
                                }

                                let result = TestResult {
                                    file: plan.file_path.clone(),
                                    name: plan.test_name.clone(),
                                    status: TestStatus::Failed,
                                    error_message: Some(format!("Execution error: {}", e)),
                                    duration_ms: 0,
                                    retries: attempt,
                                    screenshot_paths: None,
                                    trace_data: None,
                                    video_paths: None,
                                };
                                let _ = tx.send(result).await;
                                break;
                            }
                        }
                    }
                }

                // Gracefully shut down the worker
                if let Err(e) = worker.shutdown().await {
                    warn!("Failed to gracefully shutdown execution worker: {}", e);
                    let _ = worker.kill().await;
                }
            });

            handles.push(handle);
        }

        drop(tx);

        let mut results = Vec::new();
        while let Some(result) = rx.recv().await {
            if let Some(reporter) = reporter {
                reporter.on_test_result(&result);
            }
            results.push(result);
        }

        for handle in handles {
            let _ = handle.await;
        }

        info!("Completed execution of {} test plans", results.len());
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
                let results = self.execute_tests(files, 1, false, "test-results", false, "test-results/videos", None).await?;
                all_results.extend(results);
            }
        }

        Ok(all_results)
    }
}

// ── PlanBuilder ────────────────────────────────────────────────────────

/// Builds per-test `ExecutionPlan`s from extracted test definitions.
///
/// 1. Accumulates `ExtractResponse` data from multiple files.
/// 2. On `build()`: applies modifier cascade, resolves hook inheritance,
///    determines beforeAll/afterAll scheduling, and produces flat plan list.
pub struct PlanBuilder {
    tests: Vec<CollectedTest>,
    hooks: Vec<CollectedHook>,
    suites: Vec<CollectedSuite>,
    /// Default timeout for tests that don't specify one
    default_timeout_ms: u32,
    /// Maximum retries per test
    max_retries: u32,
}

impl PlanBuilder {
    pub fn new(default_timeout_ms: u32, max_retries: u32) -> Self {
        Self {
            tests: Vec::new(),
            hooks: Vec::new(),
            suites: Vec::new(),
            default_timeout_ms,
            max_retries,
        }
    }

    /// Add extracted test definitions from a single file.
    pub fn add_extracted_file(&mut self, response: ExtractResponse) {
        self.tests.extend(response.tests);
        self.hooks.extend(response.hooks);
        self.suites.extend(response.suites);
    }

    /// Consume the builder and produce a list of execution plans.
    pub fn build(mut self) -> Vec<ExecutionPlan> {
        if self.tests.is_empty() {
            return Vec::new();
        }

        // Step 1: Remove skipped tests
        self.tests.retain(|t| t.modifier != "skip");

        // Step 2: Check for "only" focus
        let has_only_test = self.tests.iter().any(|t| t.modifier == "only");
        let has_only_suite = self.suites.iter().any(|s| s.suite_type == "only");

        if has_only_test || has_only_suite {
            let only_suite_paths: Vec<Vec<String>> = self.suites.iter()
                .filter(|s| s.suite_type == "only")
                .map(|s| s.suite_path.clone())
                .collect();

            self.tests.retain(|t| {
                if t.modifier == "only" {
                    return true;
                }
                // Check if test belongs to an "only" suite
                only_suite_paths.iter().any(|sp| {
                    sp.len() <= t.suite_path.len()
                        && sp.iter().zip(t.suite_path.iter()).all(|(a, b)| a == b)
                })
            });
        }

        // Step 3: Remove tests in "skip" suites
        let skip_suite_paths: Vec<Vec<String>> = self.suites.iter()
            .filter(|s| s.suite_type == "skip")
            .map(|s| s.suite_path.clone())
            .collect();

        if !skip_suite_paths.is_empty() {
            self.tests.retain(|t| {
                !skip_suite_paths.iter().any(|sp| {
                    sp.len() <= t.suite_path.len()
                        && sp.iter().zip(t.suite_path.iter()).all(|(a, b)| a == b)
                })
            });
        }

        // Step 4: Determine suite membership for scheduling
        // Group tests by their suite path to find first/last test per suite
        let mut suite_test_indices: HashMap<Vec<String>, Vec<usize>> = HashMap::new();
        for (i, test) in self.tests.iter().enumerate() {
            suite_test_indices
                .entry(test.suite_path.clone())
                .or_default()
                .push(i);
        }

        // Step 5: Build per-test execution plans with resolved hooks
        let mut plans = Vec::with_capacity(self.tests.len());
        let timeout_ms = self.default_timeout_ms;

        for (i, test) in self.tests.iter().enumerate() {
            let suite_path = &test.suite_path;

            // Resolve hook inheritance by walking suite path ancestry
            let before_all = self.resolve_hooks_for_path("beforeAll", suite_path);
            let after_all = self.resolve_hooks_for_path("afterAll", suite_path);
            let before_each = self.resolve_before_each(suite_path);
            let after_each = self.resolve_after_each(suite_path);

            // Determine beforeAll scheduling: first test in this suite path
            let run_before_all = suite_test_indices
                .get(suite_path)
                .map(|indices| indices.first() == Some(&i))
                .unwrap_or(false);

            // Determine afterAll scheduling: last test in this suite path
            let run_after_all = suite_test_indices
                .get(suite_path)
                .map(|indices| indices.last() == Some(&i))
                .unwrap_or(false);

            // Apply test.slow: triple timeout
            let test_timeout = if test.modifier == "slow" {
                timeout_ms * 3
            } else {
                timeout_ms
            };

            plans.push(ExecutionPlan {
                test_name: test.name.clone(),
                suite_path: suite_path.clone(),
                file_path: String::new(), // Set by caller
                test_fn_body: test.fn_body.clone(),
                before_all_hooks: before_all,
                after_all_hooks: after_all,
                before_each_hooks: before_each,
                after_each_hooks: after_each,
                timeout_ms: test_timeout,
                is_fail: test.modifier == "fail",
                is_fixme: test.modifier == "fixme",
                is_slow: test.modifier == "slow",
                run_before_all,
                run_after_all,
            });
        }

        plans
    }

    /// Find all hooks of a given type whose suite_path is an ancestor of
    /// or equal to the test's suite_path. Parent-first order.
    fn resolve_hooks_for_path(
        &self,
        hook_type: &str,
        test_path: &[String],
    ) -> Vec<String> {
        let mut matching: Vec<(&CollectedHook, usize)> = self.hooks
            .iter()
            .filter(|h| h.hook_type == hook_type)
            .filter(|h| is_path_ancestor_or_self(&h.suite_path, test_path))
            .map(|h| (h, h.suite_path.len()))
            .collect();

        // Sort by path depth: shallow (parent) first for beforeAll, deep (child) first for afterAll
        matching.sort_by_key(|(_, depth)| *depth);

        matching.into_iter().map(|(h, _)| h.fn_body.clone()).collect()
    }

    /// Resolve beforeEach hooks parent-first (ancestor → child order)
    fn resolve_before_each(&self, test_path: &[String]) -> Vec<String> {
        let mut matching: Vec<(&CollectedHook, usize)> = self.hooks
            .iter()
            .filter(|h| h.hook_type == "beforeEach")
            .filter(|h| is_path_ancestor_or_self(&h.suite_path, test_path))
            .map(|h| (h, h.suite_path.len()))
            .collect();

        matching.sort_by_key(|(_, depth)| *depth); // parent first
        matching.into_iter().map(|(h, _)| h.fn_body.clone()).collect()
    }

    /// Resolve afterEach hooks child-first (reverse depth order)
    fn resolve_after_each(&self, test_path: &[String]) -> Vec<String> {
        let mut matching: Vec<(&CollectedHook, usize)> = self.hooks
            .iter()
            .filter(|h| h.hook_type == "afterEach")
            .filter(|h| is_path_ancestor_or_self(&h.suite_path, test_path))
            .map(|h| (h, h.suite_path.len()))
            .collect();

        matching.sort_by_key(|(_, depth)| std::cmp::Reverse(*depth)); // child first
        matching.into_iter().map(|(h, _)| h.fn_body.clone()).collect()
    }
}

/// Returns true if `ancestor_path` is equal to or a prefix of `test_path`.
/// Empty ancestor_path (root-level hooks) applies to all tests.
fn is_path_ancestor_or_self(ancestor_path: &[String], test_path: &[String]) -> bool {
    if ancestor_path.is_empty() {
        return true;
    }
    if ancestor_path.len() > test_path.len() {
        return false;
    }
    ancestor_path
        .iter()
        .zip(test_path.iter())
        .all(|(a, b)| a == b)
}
