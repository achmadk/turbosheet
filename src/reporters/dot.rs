use super::{AggregatedTestResult, Reporter, ReporterWithProgress, TestCaseResult};
use std::time::Duration;

pub struct DotReporter;

impl Reporter for DotReporter {
    fn on_test_result(&self, result: &crate::test_runner::executor::TestResult) {
        let tc = TestCaseResult {
            title: result.name.clone(),
            status: format!("{:?}", result.status).to_lowercase(),
            duration_ms: result.duration_ms,
            error: result.error_message.clone(),
            retry: result.retries,
            screenshot_paths: result.screenshot_paths.clone().unwrap_or_default(),
            trace_data: result.trace_data.clone(),
        };
        print!("{}", Self::write_single(&tc));
    }

    fn on_complete(&self, summary: &AggregatedTestResult) {
        Self::print_summary(summary);
    }
}

impl ReporterWithProgress for DotReporter {
    fn on_progress(&self, _elapsed: Duration, _completed: usize, _total: usize) {
        // DotReporter already outputs per-test, progress is inherent
    }
}

impl DotReporter {
    pub fn write(results: &[TestCaseResult]) -> String {
        let mut output = String::new();
        for result in results {
            output.push_str(&Self::write_single(result));
        }
        output
    }

    pub fn write_single(result: &TestCaseResult) -> String {
        match result.status.as_str() {
            "passed" => ".".to_string(),
            "failed" => "F".to_string(),
            "skipped" => "-".to_string(),
            _ => "?".to_string(),
        }
    }

    pub fn print_summary(result: &AggregatedTestResult) {
        println!();
        println!(
            "  {} passed, {} failed, {} skipped ({:?})",
            result.passes,
            result.failures,
            result.skipped,
            std::time::Duration::from_millis(result.duration_ms as u64)
        );
    }
}
