use super::{AggregatedTestResult, TestCaseResult};

pub struct DotReporter;

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
