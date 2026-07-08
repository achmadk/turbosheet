use serde::{Deserialize, Serialize};

// ── JSON-RPC primitives ────────────────────────────────────────────────

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct JsonRpcRequest {
    pub jsonrpc: String,
    pub id: String,
    pub method: String,
    pub params: serde_json::Value,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct JsonRpcResponse {
    pub jsonrpc: String,
    pub id: String,
    pub result: Option<serde_json::Value>,
    pub error: Option<JsonRpcError>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct JsonRpcError {
    pub code: i32,
    pub message: String,
    pub data: Option<serde_json::Value>,
}

impl JsonRpcRequest {
    pub fn new(method: &str, params: serde_json::Value) -> Self {
        Self {
            jsonrpc: "2.0".to_string(),
            id: uuid::Uuid::new_v4().to_string(),
            method: method.to_string(),
            params,
        }
    }
}

// ── Two-phase IPC protocol types ──────────────────────────────────────

/// A single test definition collected from the worker during extraction phase.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct CollectedTest {
    pub name: String,
    /// One of "normal", "only", "skip", "fixme", "fail", "slow"
    pub modifier: String,
    /// Suite path from root to the containing describe block, e.g. ["Root", "Child"]
    pub suite_path: Vec<String>,
    /// The test function body as a JavaScript source string (fn.toString())
    pub fn_body: String,
}

/// A hook definition collected from the worker during extraction phase.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct CollectedHook {
    /// One of "beforeAll", "afterAll", "beforeEach", "afterEach"
    pub hook_type: String,
    /// Suite path at the point of hook registration
    pub suite_path: Vec<String>,
    /// The hook function body as a JavaScript source string
    pub fn_body: String,
}

/// A suite definition collected from the worker during extraction phase.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct CollectedSuite {
    pub name: String,
    /// One of "default", "serial", "parallel", "skip", "only"
    pub suite_type: String,
    /// Suite path from root to this suite, including this suite's name
    pub suite_path: Vec<String>,
}

/// Request sent to worker for the extraction phase (no browser).
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ExtractRequest {
    pub file_path: String,
    pub timeout_ms: u32,
}

/// Response from the extraction phase — all collected definitions.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ExtractResponse {
    pub tests: Vec<CollectedTest>,
    pub hooks: Vec<CollectedHook>,
    pub suites: Vec<CollectedSuite>,
}

/// A single test execution plan computed by the Rust orchestrator.
/// Contains everything the worker needs to execute one test in isolation.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ExecutionPlan {
    /// Test identification for result reporting
    pub test_name: String,
    pub suite_path: Vec<String>,
    pub file_path: String,

    /// The test function body (JavaScript source string)
    pub test_fn_body: String,

    /// Hook function bodies to run around the test (resolved by PlanBuilder)
    pub before_all_hooks: Vec<String>,
    pub after_all_hooks: Vec<String>,
    pub before_each_hooks: Vec<String>,
    pub after_each_hooks: Vec<String>,

    /// Timeout for this test in milliseconds (may be tripled for slow tests)
    pub timeout_ms: u32,

    /// Modifier behavior flags
    pub is_fail: bool,
    pub is_fixme: bool,
    pub is_slow: bool,

    /// Whether to run the suite's beforeAll hooks before this test.
    /// Only true for the first test in a suite.
    pub run_before_all: bool,

    /// Whether to run the suite's afterAll hooks after this test.
    /// Only true for the last test in a suite (or on suite failure).
    pub run_after_all: bool,
}

/// Response from a single `runPlan` execution.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct PlanResponse {
    pub name: String,
    /// One of "passed", "failed", "skipped", "timeout", "fixme"
    pub status: String,
    pub error: Option<String>,
    pub duration_ms: u32,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_collected_test_round_trip() {
        let ct = CollectedTest {
            name: "should work".to_string(),
            modifier: "only".to_string(),
            suite_path: vec!["Root".to_string(), "Child".to_string()],
            fn_body: "async ({ page }) => { await page.goto('https://example.com'); }".to_string(),
        };
        let json = serde_json::to_value(&ct).unwrap();
        let deserialized: CollectedTest = serde_json::from_value(json).unwrap();
        assert_eq!(deserialized.name, "should work");
        assert_eq!(deserialized.modifier, "only");
        assert_eq!(deserialized.suite_path, vec!["Root", "Child"]);
        assert!(deserialized.fn_body.contains("page.goto"));
    }

    #[test]
    fn test_collected_hook_round_trip() {
        let ch = CollectedHook {
            hook_type: "beforeEach".to_string(),
            suite_path: vec!["Root".to_string()],
            fn_body: "() => { console.log('setup'); }".to_string(),
        };
        let json = serde_json::to_value(&ch).unwrap();
        let deserialized: CollectedHook = serde_json::from_value(json).unwrap();
        assert_eq!(deserialized.hook_type, "beforeEach");
        assert_eq!(deserialized.suite_path, vec!["Root"]);
    }

    #[test]
    fn test_collected_suite_round_trip() {
        let cs = CollectedSuite {
            name: "Child".to_string(),
            suite_type: "serial".to_string(),
            suite_path: vec!["Root".to_string(), "Child".to_string()],
        };
        let json = serde_json::to_value(&cs).unwrap();
        let deserialized: CollectedSuite = serde_json::from_value(json).unwrap();
        assert_eq!(deserialized.name, "Child");
        assert_eq!(deserialized.suite_type, "serial");
        assert_eq!(deserialized.suite_path, vec!["Root", "Child"]);
    }

    #[test]
    fn test_extract_request_response_round_trip() {
        let req = ExtractRequest {
            file_path: "/tmp/test.tsheet.ts".to_string(),
            timeout_ms: 30000,
        };
        let req_json = serde_json::to_value(&req).unwrap();
        let req_deser: ExtractRequest = serde_json::from_value(req_json).unwrap();
        assert_eq!(req_deser.file_path, "/tmp/test.tsheet.ts");
        assert_eq!(req_deser.timeout_ms, 30000);

        let resp = ExtractResponse {
            tests: vec![
                CollectedTest {
                    name: "test1".to_string(),
                    modifier: "normal".to_string(),
                    suite_path: vec![],
                    fn_body: "() => {}".to_string(),
                },
            ],
            hooks: vec![
                CollectedHook {
                    hook_type: "beforeAll".to_string(),
                    suite_path: vec![],
                    fn_body: "() => { console.log('setup'); }".to_string(),
                },
            ],
            suites: vec![],
        };
        let resp_json = serde_json::to_value(&resp).unwrap();
        let resp_deser: ExtractResponse = serde_json::from_value(resp_json).unwrap();
        assert_eq!(resp_deser.tests.len(), 1);
        assert_eq!(resp_deser.tests[0].name, "test1");
        assert_eq!(resp_deser.hooks.len(), 1);
    }

    #[test]
    fn test_execution_plan_round_trip() {
        let plan = ExecutionPlan {
            test_name: "my test".to_string(),
            suite_path: vec!["Root".to_string()],
            file_path: "/tmp/test.tsheet.ts".to_string(),
            test_fn_body: "async () => { await page.click('#btn'); }".to_string(),
            before_all_hooks: vec!["() => { console.log('before'); }".to_string()],
            after_all_hooks: vec![],
            before_each_hooks: vec![],
            after_each_hooks: vec!["() => { console.log('after'); }".to_string()],
            timeout_ms: 30000,
            is_fail: false,
            is_fixme: false,
            is_slow: true,
            run_before_all: true,
            run_after_all: false,
        };
        let json = serde_json::to_value(&plan).unwrap();
        let deserialized: ExecutionPlan = serde_json::from_value(json).unwrap();
        assert_eq!(deserialized.test_name, "my test");
        assert_eq!(deserialized.timeout_ms, 30000);
        assert!(deserialized.is_slow);
        assert!(deserialized.run_before_all);
        assert!(!deserialized.run_after_all);
        assert!(!deserialized.is_fail);
        assert_eq!(deserialized.before_all_hooks.len(), 1);
        assert_eq!(deserialized.after_each_hooks.len(), 1);
    }

    #[test]
    fn test_plan_response_round_trip() {
        let pr = PlanResponse {
            name: "my test".to_string(),
            status: "failed".to_string(),
            error: Some("something broke".to_string()),
            duration_ms: 1500,
        };
        let json = serde_json::to_value(&pr).unwrap();
        let deserialized: PlanResponse = serde_json::from_value(json).unwrap();
        assert_eq!(deserialized.name, "my test");
        assert_eq!(deserialized.status, "failed");
        assert_eq!(deserialized.error.as_deref(), Some("something broke"));
        assert_eq!(deserialized.duration_ms, 1500);

        let pr2 = PlanResponse {
            name: "known issue".to_string(),
            status: "fixme".to_string(),
            error: None,
            duration_ms: 100,
        };
        let json2 = serde_json::to_value(&pr2).unwrap();
        let deserialized2: PlanResponse = serde_json::from_value(json2).unwrap();
        assert_eq!(deserialized2.status, "fixme");
        assert!(deserialized2.error.is_none());
    }

    #[test]
    fn test_empty_collections() {
        let resp = ExtractResponse {
            tests: vec![],
            hooks: vec![],
            suites: vec![],
        };
        let json = serde_json::to_value(&resp).unwrap();
        let deserialized: ExtractResponse = serde_json::from_value(json).unwrap();
        assert!(deserialized.tests.is_empty());
        assert!(deserialized.hooks.is_empty());
        assert!(deserialized.suites.is_empty());
    }

    #[test]
    fn test_default_modifier_values() {
        let ct = CollectedTest {
            name: "normal test".to_string(),
            modifier: "normal".to_string(),
            suite_path: vec![],
            fn_body: "() => {}".to_string(),
        };
        assert_eq!(ct.modifier, "normal");

        let ct_only = CollectedTest {
            modifier: "only".to_string(),
            ..ct.clone()
        };
        assert_eq!(ct_only.modifier, "only");
    }
}
