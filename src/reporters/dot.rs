use super::{AggregatedTestResult, TestCaseResult};

pub struct DotReporter;

impl DotReporter {
    pub fn write(results: &[TestCaseResult]) -> String {
        let mut output = String::new();
        for result in results {
            match result.status.as_str() {
                "passed" => output.push('.'),
                "failed" => output.push('F'),
                "skipped" => output.push('-'),
                _ => output.push('?'),
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
