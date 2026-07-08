pub mod dot;
pub mod line;
pub mod list;
pub mod json;
pub mod junit;
pub mod github;
pub mod html;

use serde::{Deserialize, Serialize};
use std::time::Duration;

/// Trait for reporters that process test results as they complete.
/// Requires `Send + Sync` because reporters are passed across await points
/// in `#[napi]` async functions.
pub trait Reporter: Send + Sync {
    /// Called after each individual test completes with its result.
    fn on_test_result(&self, result: &super::test_runner::executor::TestResult);

    /// Called after all tests finish with the aggregate summary.
    fn on_complete(&self, summary: &AggregatedTestResult);
}

/// Trait for reporters that support live progress updates during test execution.
/// Only streaming reporters (Dot, Line, List) implement this.
pub trait ReporterWithProgress {
    /// Called periodically during test execution to report progress.
    fn on_progress(&self, elapsed: Duration, completed: usize, total: usize);
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReporterConfig {
    pub reporter_type: ReporterType,
    pub output_path: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ReporterType {
    Dot,
    Line,
    List,
    Json,
    Junit,
    Html,
    Github,
}

impl Default for ReporterType {
    fn default() -> Self {
        ReporterType::List
    }
}

impl std::fmt::Display for ReporterType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ReporterType::Dot => write!(f, "dot"),
            ReporterType::Line => write!(f, "line"),
            ReporterType::List => write!(f, "list"),
            ReporterType::Json => write!(f, "json"),
            ReporterType::Junit => write!(f, "junit"),
            ReporterType::Html => write!(f, "html"),
            ReporterType::Github => write!(f, "github"),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TestSuiteResult {
    pub name: String,
    pub tests: Vec<TestCaseResult>,
    pub duration_ms: u32,
    pub error_message: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TestCaseResult {
    pub title: String,
    pub status: String,
    pub duration_ms: u32,
    pub error: Option<String>,
    pub retry: u32,
    pub screenshot_paths: Vec<String>,
    pub trace_data: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AggregatedTestResult {
    pub suites: Vec<TestSuiteResult>,
    pub tests: Vec<TestCaseResult>,
    pub passes: u32,
    pub failures: u32,
    pub skipped: u32,
    pub duration_ms: u32,
    pub timestamp: String,
}

impl AggregatedTestResult {
    pub fn from_test_results(results: Vec<super::test_runner::executor::TestResult>) -> Self {
        let mut passes = 0u32;
        let mut failures = 0u32;
        let mut skipped = 0u32;
        let mut total_duration = 0u32;

        for result in &results {
            total_duration += result.duration_ms;
            match result.status {
                super::test_runner::executor::TestStatus::Passed => passes += 1,
                super::test_runner::executor::TestStatus::Failed => failures += 1,
                super::test_runner::executor::TestStatus::Skipped => skipped += 1,
                super::test_runner::executor::TestStatus::Timeout => failures += 1,
            }
        }

        let tests: Vec<TestCaseResult> = results
            .into_iter()
            .map(|r| TestCaseResult {
                title: r.name,
                status: format!("{:?}", r.status).to_lowercase(),
                duration_ms: r.duration_ms,
                error: r.error_message,
                retry: r.retries,
                screenshot_paths: r.screenshot_paths.unwrap_or_default(),
                trace_data: r.trace_data,
            })
            .collect();

        let suites = vec![TestSuiteResult {
            name: "Default Suite".to_string(),
            tests: tests.clone(),
            duration_ms: total_duration,
            error_message: None,
        }];

        AggregatedTestResult {
            suites,
            tests,
            passes,
            failures,
            skipped,
            duration_ms: total_duration,
            timestamp: chrono_now(),
        }
    }
}

fn chrono_now() -> String {
    use std::time::{SystemTime, UNIX_EPOCH};
    let duration = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default();
    let secs = duration.as_secs();
    let millis = duration.subsec_millis();
    format!("{}.{:03}Z", secs, millis)
}

pub fn parse_reporters(reporter_str: &str) -> Vec<ReporterType> {
    reporter_str
        .split(',')
        .map(|s| s.trim().to_lowercase())
        .filter_map(|s| match s.as_str() {
            "dot" => Some(ReporterType::Dot),
            "line" => Some(ReporterType::Line),
            "list" => Some(ReporterType::List),
            "json" => Some(ReporterType::Json),
            "junit" => Some(ReporterType::Junit),
            "html" => Some(ReporterType::Html),
            "github" => Some(ReporterType::Github),
            _ => None,
        })
        .collect()
}
