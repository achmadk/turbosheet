use super::{AggregatedTestResult, Reporter};
use super::TestCaseResult;

pub struct JsonReporter;

impl Reporter for JsonReporter {
    fn on_test_result(&self, _result: &crate::test_runner::executor::TestResult) {
        // JSON reporter buffers all results, outputs on completion
    }

    fn on_complete(&self, summary: &AggregatedTestResult) {
        let json = Self::write(summary);
        println!("{}", json);
    }
}

impl JsonReporter {
    pub fn write(result: &AggregatedTestResult) -> String {
        serde_json::to_string_pretty(result).unwrap_or_default()
    }

    pub fn write_to_stdout(result: &AggregatedTestResult) {
        let json = Self::write(result);
        println!("{}", json);
    }
}
