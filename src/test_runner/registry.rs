use std::collections::HashMap;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SuiteType {
    Default,
    Serial,
    Parallel,
    Skip,
    Only,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TestModifier {
    Normal,
    Only,
    Skip,
    Fixme,
    Fail,
    Slow,
}

impl TestModifier {
    pub fn is_focused(&self) -> bool {
        matches!(self, TestModifier::Only)
    }

    pub fn is_skipped(&self) -> bool {
        matches!(self, TestModifier::Skip | TestModifier::Fixme)
    }
}

#[derive(Debug, Clone)]
pub struct RegisteredTest {
    pub id: String,
    pub name: String,
    pub modifier: TestModifier,
    pub suite_id: Option<String>,
    pub line: usize,
    pub column: usize,
}

#[derive(Debug, Clone)]
pub struct RegisteredSuite {
    pub id: String,
    pub name: String,
    pub suite_type: SuiteType,
    pub child_suite_ids: Vec<String>,
    pub test_ids: Vec<String>,
    pub parent_id: Option<String>,
}

#[derive(Debug, Clone)]
pub struct RegisteredHook {
    pub hook_type: String, // beforeAll, afterAll, beforeEach, afterEach
    pub suite_id: Option<String>,
    pub fn_body: String,
    pub timeout_ms: Option<u32>,
}

#[derive(Debug, Clone)]
pub struct TestRegistry {
    tests: HashMap<String, RegisteredTest>,
    suites: HashMap<String, RegisteredSuite>,
    hooks: Vec<RegisteredHook>,
    suite_stack: Vec<String>,
    id_counter: u64,
}

impl TestRegistry {
    pub fn new() -> Self {
        Self {
            tests: HashMap::new(),
            suites: HashMap::new(),
            hooks: Vec::new(),
            suite_stack: Vec::new(),
            id_counter: 0,
        }
    }

    fn next_id(&mut self) -> String {
        self.id_counter += 1;
        format!("t{}", self.id_counter)
    }

    pub fn add_test(&mut self, name: String, modifier: TestModifier) -> String {
        let id = self.next_id();
        let suite_id = self.suite_stack.last().cloned();

        let test = RegisteredTest {
            id: id.clone(),
            name,
            modifier,
            suite_id: suite_id.clone(),
            line: 0,
            column: 0,
        };

        self.tests.insert(id.clone(), test);

        if let Some(sid) = suite_id {
            if let Some(suite) = self.suites.get_mut(&sid) {
                suite.test_ids.push(id.clone());
            }
        }

        id
    }

    pub fn add_test_with_location(
        &mut self,
        name: String,
        modifier: TestModifier,
        line: usize,
        column: usize,
    ) -> String {
        let id = self.next_id();
        let suite_id = self.suite_stack.last().cloned();

        let test = RegisteredTest {
            id: id.clone(),
            name,
            modifier,
            suite_id: suite_id.clone(),
            line,
            column,
        };

        self.tests.insert(id.clone(), test);

        if let Some(sid) = suite_id {
            if let Some(suite) = self.suites.get_mut(&sid) {
                suite.test_ids.push(id.clone());
            }
        }

        id
    }

    pub fn begin_suite(&mut self, name: String, suite_type: SuiteType) -> String {
        let id = self.next_id();
        let parent_id = self.suite_stack.last().cloned();

        let suite = RegisteredSuite {
            id: id.clone(),
            name,
            suite_type,
            child_suite_ids: Vec::new(),
            test_ids: Vec::new(),
            parent_id: parent_id.clone(),
        };

        if let Some(pid) = &parent_id {
            if let Some(parent) = self.suites.get_mut(pid) {
                parent.child_suite_ids.push(id.clone());
            }
        }

        self.suites.insert(id.clone(), suite);
        self.suite_stack.push(id.clone());
        id
    }

    pub fn end_suite(&mut self) {
        self.suite_stack.pop();
    }

    pub fn current_suite_id(&self) -> Option<&str> {
        self.suite_stack.last().map(|s| s.as_str())
    }

    pub fn current_suite_path(&self) -> Vec<String> {
        self.suite_stack.clone()
    }

    pub fn add_hook(
        &mut self,
        hook_type: String,
        fn_body: String,
        timeout_ms: Option<u32>,
    ) {
        let suite_id = self.suite_stack.last().cloned();
        self.hooks.push(RegisteredHook {
            hook_type,
            suite_id,
            fn_body,
            timeout_ms,
        });
    }

    pub fn get_test(&self, id: &str) -> Option<&RegisteredTest> {
        self.tests.get(id)
    }

    pub fn get_suite(&self, id: &str) -> Option<&RegisteredSuite> {
        self.suites.get(id)
    }

    pub fn get_tests(&self) -> Vec<&RegisteredTest> {
        self.tests.values().collect()
    }

    pub fn get_suites(&self) -> Vec<&RegisteredSuite> {
        self.suites.values().collect()
    }

    pub fn get_root_suites(&self) -> Vec<&RegisteredSuite> {
        self.suites
            .values()
            .filter(|s| s.parent_id.is_none())
            .collect()
    }

    pub fn get_hooks(&self) -> &[RegisteredHook] {
        &self.hooks
    }

    pub fn has_focused_tests(&self) -> bool {
        self.tests.values().any(|t| t.modifier.is_focused())
            || self.suites.values().any(|s| s.suite_type == SuiteType::Only)
    }

    pub fn has_focused_suites(&self) -> bool {
        self.suites.values().any(|s| s.suite_type == SuiteType::Only)
    }

    pub fn clear(&mut self) {
        self.tests.clear();
        self.suites.clear();
        self.hooks.clear();
        self.suite_stack.clear();
        self.id_counter = 0;
    }

    pub fn test_count(&self) -> usize {
        self.tests.len()
    }

    pub fn suite_count(&self) -> usize {
        self.suites.len()
    }

    pub fn hook_count(&self) -> usize {
        self.hooks.len()
    }

    /// Collect all tests in a flattened list, optionally filtered by the "only" modifier.
    /// If any test or suite has the Only modifier, only focused items are included.
    pub fn collect_active_tests(&self) -> Vec<&RegisteredTest> {
        let has_focus = self.has_focused_tests();

        if !has_focus {
            // No focus modifiers — include all non-skipped tests
            return self
                .tests
                .values()
                .filter(|t| !t.modifier.is_skipped())
                .collect();
        }

        // Focus mode: only include tests that are focused, or whose suite is focused
        let focused_suite_ids: std::collections::HashSet<&str> = self
            .suites
            .values()
            .filter(|s| s.suite_type == SuiteType::Only)
            .flat_map(|s| collect_suite_and_children_ids(self, &s.id))
            .collect();

        self.tests
            .values()
            .filter(|t| {
                if t.modifier.is_focused() {
                    return true;
                }
                if let Some(ref sid) = t.suite_id {
                    if focused_suite_ids.contains(sid.as_str()) {
                        return true;
                    }
                }
                false
            })
            .collect()
    }
}

fn collect_suite_and_children_ids<'a>(
    registry: &'a TestRegistry,
    suite_id: &str,
) -> std::collections::HashSet<&'a str> {
    let mut ids = std::collections::HashSet::new();
    if let Some(suite) = registry.suites.get(suite_id) {
        ids.insert(suite.id.as_str());
        for child_id in &suite.child_suite_ids {
            ids.extend(collect_suite_and_children_ids(registry, child_id));
        }
    }
    ids
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_registry_add_test() {
        let mut reg = TestRegistry::new();
        let id = reg.add_test("should work".to_string(), TestModifier::Normal);
        assert_eq!(reg.test_count(), 1);
        assert!(reg.get_test(&id).is_some());
        assert_eq!(reg.get_test(&id).unwrap().name, "should work");
    }

    #[test]
    fn test_registry_suite_nesting() {
        let mut reg = TestRegistry::new();
        let suite_id = reg.begin_suite("Root".to_string(), SuiteType::Default);
        reg.add_test("test1".to_string(), TestModifier::Normal);
        reg.add_test("test2".to_string(), TestModifier::Normal);
        reg.end_suite();

        assert_eq!(reg.suite_count(), 1);
        assert_eq!(reg.test_count(), 2);

        let suite = reg.get_suite(&suite_id).unwrap();
        assert_eq!(suite.test_ids.len(), 2);
    }

    #[test]
    fn test_registry_nested_suites() {
        let mut reg = TestRegistry::new();
        reg.begin_suite("Outer".to_string(), SuiteType::Default);
        reg.begin_suite("Inner".to_string(), SuiteType::Default);
        reg.add_test("deep".to_string(), TestModifier::Normal);
        reg.end_suite();
        reg.end_suite();

        assert_eq!(reg.suite_count(), 2);
        assert_eq!(reg.test_count(), 1);

        let root = reg.get_root_suites();
        assert_eq!(root.len(), 1);
        assert_eq!(root[0].child_suite_ids.len(), 1);
    }

    #[test]
    fn test_focused_tests() {
        let mut reg = TestRegistry::new();
        reg.add_test("normal".to_string(), TestModifier::Normal);
        reg.add_test("focused".to_string(), TestModifier::Only);

        assert!(reg.has_focused_tests());
        let active = reg.collect_active_tests();
        assert_eq!(active.len(), 1);
        assert_eq!(active[0].name, "focused");
    }

    #[test]
    fn test_skipped_tests_excluded() {
        let mut reg = TestRegistry::new();
        reg.add_test("normal".to_string(), TestModifier::Normal);
        reg.add_test("skipped".to_string(), TestModifier::Skip);

        let active = reg.collect_active_tests();
        assert_eq!(active.len(), 1);
        assert_eq!(active[0].name, "normal");
    }

    #[test]
    fn test_focused_suite() {
        let mut reg = TestRegistry::new();
        reg.begin_suite("Focused".to_string(), SuiteType::Only);
        reg.add_test("inner".to_string(), TestModifier::Normal);
        reg.end_suite();
        reg.add_test("outside".to_string(), TestModifier::Normal);

        assert!(reg.has_focused_tests());
        let active = reg.collect_active_tests();
        assert_eq!(active.len(), 1);
        assert_eq!(active[0].name, "inner");
    }

    #[test]
    fn test_hooks() {
        let mut reg = TestRegistry::new();
        reg.add_hook("beforeAll".to_string(), "console.log('setup')".to_string(), None);
        reg.add_hook("afterAll".to_string(), "console.log('teardown')".to_string(), None);

        assert_eq!(reg.hook_count(), 2);
    }

    #[test]
    fn test_clear() {
        let mut reg = TestRegistry::new();
        reg.add_test("a".to_string(), TestModifier::Normal);
        reg.add_test("b".to_string(), TestModifier::Normal);
        assert_eq!(reg.test_count(), 2);

        reg.clear();
        assert_eq!(reg.test_count(), 0);
        assert_eq!(reg.suite_count(), 0);
    }
}
