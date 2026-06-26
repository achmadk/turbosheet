use super::AggregatedTestResult;

pub struct JsonReporter;

impl JsonReporter {
    pub fn write(result: &AggregatedTestResult) -> String {
        serde_json::to_string_pretty(result).unwrap_or_default()
    }

    pub fn write_to_stdout(result: &AggregatedTestResult) {
        let json = Self::write(result);
        println!("{}", json);
    }
}
