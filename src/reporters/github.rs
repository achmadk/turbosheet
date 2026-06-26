use super::{AggregatedTestResult, TestCaseResult};
use std::fmt::Write;

pub struct GithubReporter;

impl GithubReporter {
    pub fn write(result: &AggregatedTestResult) -> String {
        let mut output = String::new();

        for test in &result.tests {
            if test.status == "failed" {
                Self::write_annotation(&mut output, test);
            }
        }

        output
    }

    fn write_annotation(output: &mut String, test: &TestCaseResult) {
        let error_msg = test.error.as_deref().unwrap_or("Test failed");
        let escaped_msg = escape_gha(error_msg);

        let _ = writeln!(output, "::error title=Test Failure::{},{}::{}",
            test.title,
            test.title,
            escaped_msg
        );
    }

    pub fn print_to_stdout(result: &AggregatedTestResult) {
        let output = Self::write(result);
        println!("{}", output);
    }
}

fn escape_gha(s: &str) -> String {
    s.replace('\n', "%0A")
        .replace('\r', "%0D")
}
