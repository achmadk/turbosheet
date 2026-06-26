use super::{AggregatedTestResult, TestSuiteResult, TestCaseResult};
use std::fmt::Write;

pub struct JunitReporter;

impl JunitReporter {
    pub fn write(result: &AggregatedTestResult) -> String {
        let mut output = String::new();

        let _ = writeln!(output, r#"<?xml version="1.0" encoding="UTF-8"?>"#);
        let _ = writeln!(output, r#"<testsuites name="TurboSheet" failures="{}" tests="{}" time="{:.3}">"#,
            result.failures,
            result.passes + result.failures + result.skipped,
            result.duration_ms as f64 / 1000.0
        );

        for suite in &result.suites {
            Self::write_suite(&mut output, suite);
        }

        let _ = writeln!(output, "</testsuites>");
        output
    }

    fn write_suite(output: &mut String, suite: &TestSuiteResult) {
        let failures = suite.tests.iter().filter(|t| t.status == "failed").count();
        let time = suite.duration_ms as f64 / 1000.0;

        let _ = writeln!(output, r#"  <testsuite name="{}" failures="{}" tests="{}" time="{:.3}">"#,
            escape_xml(&suite.name),
            failures,
            suite.tests.len(),
            time
        );

        for test in &suite.tests {
            Self::write_testcase(output, test);
        }

        let _ = writeln!(output, "  </testsuite>");
    }

    fn write_testcase(output: &mut String, test: &TestCaseResult) {
        let classname = "tsheet.tests";
        let _ = write!(output, r#"    <testcase name="{}" classname="{}" time="{:.3}""#,
            escape_xml(&test.title),
            classname,
            test.duration_ms as f64 / 1000.0
        );

        if let Some(ref error) = test.error {
            let _ = writeln!(output, ">");
            let _ = writeln!(output, r#"      <failure message="test failed">{}</failure>"#, escape_xml(error));
            let _ = writeln!(output, "    </testcase>");
        } else {
            let _ = writeln!(output, " />");
        }
    }
}

fn escape_xml(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&apos;")
}
