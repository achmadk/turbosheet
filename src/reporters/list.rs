use super::{AggregatedTestResult, TestCaseResult};

pub struct ListReporter;

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
