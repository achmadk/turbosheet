pub mod config;
pub mod config_loader;
pub mod discovery;
pub mod executor;
pub mod extractor;
pub mod fixtures;
pub mod hooks;
pub mod ipc;
pub mod registry;
pub mod worker;

use std::sync::{Mutex, OnceLock};
use napi::bindgen_prelude::*;
use napi_derive::napi;
use crate::reporters::{AggregatedTestResult, Reporter, parse_reporters, ReporterType};
use self::registry::{TestRegistry, TestModifier, SuiteType};

static REGISTRY: OnceLock<Mutex<TestRegistry>> = OnceLock::new();

fn with_registry<F, R>(f: F) -> R
where
    F: FnOnce(&mut TestRegistry) -> R,
{
    let reg = REGISTRY.get_or_init(|| Mutex::new(TestRegistry::new()));
    let mut guard = reg.lock().expect("Failed to lock test registry");
    f(&mut guard)
}

#[napi]
pub async fn run_tests(config: config::TestConfig) -> Result<Vec<executor::TestResult>> {
    // Clear any previously registered tests from module-load-time registrations
    with_registry(|reg| reg.clear());

    let test_dir = config.test_dir.clone().unwrap_or_else(|| ".".to_string());
    let test_match = config.test_match.clone().unwrap_or_else(|| {
        vec![
            "**/*.tsheet.ts".to_string(),
            "**/*.tsheet.spec.ts".to_string(),
        ]
    });

    let mut files = discovery::discover_tests(&test_dir, &test_match, config.grep.as_deref())?;

    if let Some(shard) = &config.shard {
        if shard.total > 0 && shard.current > 0 && shard.current <= shard.total {
            let chunk_size = (files.len() + shard.total as usize - 1) / shard.total as usize;
            let start = (shard.current as usize - 1) * chunk_size;
            let end = std::cmp::min(start + chunk_size, files.len());
            if start < files.len() {
                files = files[start..end].to_vec();
            } else {
                files = Vec::new();
            }
        }
    }

    let executor = executor::TestExecutor::new(config.clone());

    // Build reporter from CLI config
    let reporter: Option<Box<dyn Reporter>> = config.reporter.as_ref().map(|reporter_str| {
        let reporter_types = parse_reporters(reporter_str);
        // For a single reporter type, return that reporter directly.
        // For multiple reporters, we compose them. Currently only single reporter is used.
        let reporter_type = reporter_types.first().copied().unwrap_or(ReporterType::List);
        match reporter_type {
            ReporterType::Dot => Box::new(crate::reporters::dot::DotReporter) as Box<dyn Reporter>,
            ReporterType::Line => Box::new(crate::reporters::line::LineReporter) as Box<dyn Reporter>,
            ReporterType::List => Box::new(crate::reporters::list::ListReporter) as Box<dyn Reporter>,
            ReporterType::Json => Box::new(crate::reporters::json::JsonReporter) as Box<dyn Reporter>,
            ReporterType::Junit => Box::new(crate::reporters::junit::JunitReporter) as Box<dyn Reporter>,
            ReporterType::Html => Box::new(crate::reporters::html::HtmlReporter) as Box<dyn Reporter>,
            ReporterType::Github => Box::new(crate::reporters::github::GithubReporter) as Box<dyn Reporter>,
        }
    });

    let results = executor.execute(files, reporter.as_deref()).await?;

    // Handle HTML and JUnit file output (beyond stdout from on_complete)
    if let Some(ref reporter_str) = config.reporter {
        let reporter_types = parse_reporters(reporter_str);
        let aggregated = AggregatedTestResult::from_test_results(results.clone());

        for reporter_type in reporter_types {
            match reporter_type {
                ReporterType::Html => {
                    use crate::reporters::html::write_to_file;
                    let html = crate::reporters::html::HtmlReporter::write(&aggregated);
                    let html_output_path = config.output_dir.clone().map(|d| format!("{}/report.html", d)).unwrap_or_else(|| "test-results/report.html".to_string());
                    if let Err(e) = write_to_file(&html, &html_output_path) {
                        eprintln!("Warning: Failed to write HTML report: {}", e);
                    } else {
                        println!("  HTML report written to {}", html_output_path);
                    }
                },
                ReporterType::Junit => {
                    use crate::reporters::html::write_to_file;
                    let junit_xml = crate::reporters::junit::JunitReporter::write(&aggregated);
                    match config.output_dir.clone() {
                        Some(dir) => {
                            let junit_path = format!("{}/results.xml", dir);
                            if let Err(e) = write_to_file(&junit_xml, &junit_path) {
                                eprintln!("Warning: Failed to write JUnit report: {}", e);
                            } else {
                                println!("  JUnit report written to {}", junit_path);
                            }
                        },
                        None => {
                            println!("{}", junit_xml);
                        }
                    }
                },
                _ => {} // All other reporters handled via trait on_test_result/on_complete
            }
        }
    }

    Ok(results)
}

#[napi]
pub fn test(name: String, _fn_callback: Function<'_>) {
    with_registry(|reg| reg.add_test(name, TestModifier::Normal));
}

#[napi(js_name = "test.only")]
pub fn test_only(name: String, _fn_callback: Function<'_>) {
    with_registry(|reg| reg.add_test(name, TestModifier::Only));
}

#[napi(js_name = "test.skip")]
pub fn test_skip(name: String, _fn_callback: Function<'_>) {
    with_registry(|reg| reg.add_test(name, TestModifier::Skip));
}

#[napi(js_name = "test.fixme")]
pub fn test_fixme(name: String, _fn_callback: Function<'_>) {
    with_registry(|reg| reg.add_test(name, TestModifier::Fixme));
}

#[napi(js_name = "test.fail")]
pub fn test_fail(name: String, _fn_callback: Function<'_>) {
    with_registry(|reg| reg.add_test(name, TestModifier::Fail));
}

#[napi(js_name = "test.slow")]
pub fn test_slow(name: String, _fn_callback: Function<'_>) {
    with_registry(|reg| reg.add_test(name, TestModifier::Slow));
}

#[napi(js_name = "describe")]
pub fn test_describe(name: String, _fn_callback: Function<'_>) {
    // Suite registration only — callback execution happens in worker processes.
    // The callback would trigger nested test()/describe() calls in the worker's
    // own napi context, not in the main process registry.
    with_registry(|reg| reg.begin_suite(name, SuiteType::Default));
    with_registry(|reg| reg.end_suite());
}

#[napi(js_name = "describe.serial")]
pub fn test_describe_serial(name: String, _fn_callback: Function<'_>) {
    with_registry(|reg| reg.begin_suite(name, SuiteType::Serial));
    with_registry(|reg| reg.end_suite());
}

#[napi(js_name = "describe.parallel")]
pub fn test_describe_parallel(name: String, _fn_callback: Function<'_>) {
    with_registry(|reg| reg.begin_suite(name, SuiteType::Parallel));
    with_registry(|reg| reg.end_suite());
}

#[napi(js_name = "describe.skip")]
pub fn test_describe_skip(name: String, _fn_callback: Function<'_>) {
    with_registry(|reg| reg.begin_suite(name, SuiteType::Skip));
    with_registry(|reg| reg.end_suite());
}

#[napi(js_name = "describe.only")]
pub fn test_describe_only(name: String, _fn_callback: Function<'_>) {
    with_registry(|reg| reg.begin_suite(name, SuiteType::Only));
    with_registry(|reg| reg.end_suite());
}

#[napi(js_name = "beforeAll")]
pub fn test_before_all(_fn_callback: Function<'_>) {
    with_registry(|reg| reg.add_hook("beforeAll".to_string(), String::new(), None));
}

#[napi(js_name = "afterAll")]
pub fn test_after_all(_fn_callback: Function<'_>) {
    with_registry(|reg| reg.add_hook("afterAll".to_string(), String::new(), None));
}

#[napi(js_name = "beforeEach")]
pub fn test_before_each(_fn_callback: Function<'_>) {
    with_registry(|reg| reg.add_hook("beforeEach".to_string(), String::new(), None));
}

#[napi(js_name = "afterEach")]
pub fn test_after_each(_fn_callback: Function<'_>) {
    with_registry(|reg| reg.add_hook("afterEach".to_string(), String::new(), None));
}

#[napi(js_name = "test.extend")]
pub fn test_extend(_fixtures: Object) {
    // TODO: In a future phase, parse the fixtures object to create typed test variants.
    // The Object parameter contains fixture definitions keyed by name.
    // For now, this is a no-op placeholder.
}
