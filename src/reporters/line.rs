use super::{AggregatedTestResult, TestCaseResult};
use std::fmt::Write;

pub struct LineReporter;

impl LineReporter {
    pub fn write(results: &[TestCaseResult]) -> String {
        let mut output = String::new();
        for result in results {
            let status_symbol = match result.status.as_str() {
                "passed" => "✓",
                "failed" => "✗",
                "skipped" => "-",
                _ => "?",
            };
            let error_suffix = if let Some(ref err) = result.error {
                format!(" - {}", err.lines().next().unwrap_or(""))
            } else {
                String::new()
            };
            let _ = writeln!(
                output,
                "{} {} ({:?}){}",
                status_symbol,
                result.title,
                std::time::Duration::from_millis(result.duration_ms as u64),
                error_suffix
            );
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
