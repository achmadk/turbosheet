use super::{AggregatedTestResult, Reporter, ReporterWithProgress, TestCaseResult};
use std::time::Duration;

pub struct ListReporter;

impl Reporter for ListReporter {
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

impl ReporterWithProgress for ListReporter {
    fn on_progress(&self, _elapsed: Duration, _completed: usize, _total: usize) {
        // ListReporter outputs per-test, no additional progress needed
    }
}

impl ListReporter {
    pub fn write(results: &[TestCaseResult]) -> String {
        let mut output = String::new();

        for result in results {
            output.push_str(&Self::write_single(result));
        }

        output
    }

    pub fn write_single(result: &TestCaseResult) -> String {
        let mut output = String::new();
        let symbol = match result.status.as_str() {
            "passed" => "\u{2713}",
            "failed" => "\u{2717}",
            "skipped" => "-",
            _ => "?",
        };

        output.push_str(&format!("    {} {} ({}ms)\n", symbol, result.title, result.duration_ms));

        if result.status == "failed" {
            if let Some(ref error) = result.error {
                for line in error.lines() {
                    output.push_str(&format!("      {}\n", line));
                }
            }
        }

        output
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
