use std::sync::Arc;
use tokio::sync::Mutex;
use std::collections::HashMap;
use crate::engine::PageEngine;
use std::time::Duration;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum HookType {
    BeforeAll,
    AfterAll,
    BeforeEach,
    AfterEach,
}

#[derive(Clone)]
pub struct Hook {
    pub hook_type: HookType,
    pub name: Option<String>,
    pub fn_body: String,
    pub timeout_ms: Option<u32>,
}

#[derive(Clone)]
pub struct HookResult {
    pub success: bool,
    pub error: Option<String>,
}

pub struct HookRegistry {
    hooks: Arc<Mutex<Vec<Hook>>>,
    before_all_run: Arc<Mutex<HashMap<String, bool>>>,
    after_all_run: Arc<Mutex<HashMap<String, bool>>>,
}

impl HookRegistry {
    pub fn new() -> Self {
        Self {
            hooks: Arc::new(Mutex::new(Vec::new())),
            before_all_run: Arc::new(Mutex::new(HashMap::new())),
            after_all_run: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    pub async fn add_hook(&self, hook: Hook) {
        let mut hooks = self.hooks.lock().await;
        hooks.push(hook);
    }

    pub async fn get_hooks(&self, hook_type: HookType) -> Vec<Hook> {
        let hooks = self.hooks.lock().await;
        hooks
            .iter()
            .filter(|h| h.hook_type == hook_type)
            .cloned()
            .collect()
    }

    pub async fn get_hooks_for_suite(&self, _suite_id: &str, hook_type: HookType) -> Vec<Hook> {
        let hooks = self.hooks.lock().await;
        hooks
            .iter()
            .filter(|h| h.hook_type == hook_type)
            .cloned()
            .collect()
    }

    pub async fn mark_before_all_run(&self, suite_id: &str) {
        let mut run = self.before_all_run.lock().await;
        run.insert(suite_id.to_string(), true);
    }

    pub async fn is_before_all_run(&self, suite_id: &str) -> bool {
        let run = self.before_all_run.lock().await;
        run.get(suite_id).copied().unwrap_or(false)
    }

    pub async fn mark_after_all_run(&self, suite_id: &str) {
        let mut run = self.after_all_run.lock().await;
        run.insert(suite_id.to_string(), true);
    }

    pub async fn is_after_all_run(&self, suite_id: &str) -> bool {
        let run = self.after_all_run.lock().await;
        run.get(suite_id).copied().unwrap_or(false)
    }

    pub async fn reset(&self) {
        let mut hooks = self.hooks.lock().await;
        hooks.clear();
        let mut before_run = self.before_all_run.lock().await;
        before_run.clear();
        let mut after_run = self.after_all_run.lock().await;
        after_run.clear();
    }
}

pub struct HookExecutor {
    registry: HookRegistry,
}

impl HookExecutor {
    pub fn new(registry: HookRegistry) -> Self {
        Self { registry }
    }

    pub async fn execute_before_all(&self, suite_id: &str, page: &Arc<dyn PageEngine>) -> HookResult {
        if self.registry.is_before_all_run(suite_id).await {
            return HookResult { success: true, error: None };
        }

        let hooks = self.registry.get_hooks(HookType::BeforeAll).await;
        for hook in hooks {
            let result = self.execute_hook_page(&hook, page).await;
            if !result.success {
                return result;
            }
            self.registry.mark_before_all_run(suite_id).await;
        }

        HookResult { success: true, error: None }
    }

    pub async fn execute_after_all(&self, suite_id: &str, page: &Arc<dyn PageEngine>) -> HookResult {
        let hooks = self.registry.get_hooks(HookType::AfterAll).await;
        for hook in hooks {
            let result = self.execute_hook_page(&hook, page).await;
            if !result.success {
                return result;
            }
        }

        self.registry.mark_after_all_run(suite_id).await;
        HookResult { success: true, error: None }
    }

    pub async fn execute_before_each(&self, _suite_id: &str, page: &Arc<dyn PageEngine>) -> HookResult {
        let hooks = self.registry.get_hooks(HookType::BeforeEach).await;
        for hook in hooks {
            let result = self.execute_hook_page(&hook, page).await;
            if !result.success {
                return result;
            }
        }
        HookResult { success: true, error: None }
    }

    pub async fn execute_after_each(&self, _suite_id: &str, page: &Arc<dyn PageEngine>) -> HookResult {
        let hooks = self.registry.get_hooks(HookType::AfterEach).await;
        for hook in hooks {
            let result = self.execute_hook_page(&hook, page).await;
            if !result.success {
                return result;
            }
        }
        HookResult { success: true, error: None }
    }

    pub async fn execute_hook_page(&self, hook: &Hook, page: &Arc<dyn PageEngine>) -> HookResult {
        let timeout_ms = hook.timeout_ms.unwrap_or(30000);
        
        let eval_future = page.evaluate(&hook.fn_body);
        
        match tokio::time::timeout(Duration::from_millis(timeout_ms as u64), eval_future).await {
            Ok(Ok(_)) => HookResult {
                success: true,
                error: None,
            },
            Ok(Err(e)) => HookResult {
                success: false,
                error: Some(format!("Hook execution failed: {}", e)),
            },
            Err(_) => HookResult {
                success: false,
                error: Some(format!("Hook execution timed out after {}ms", timeout_ms)),
            },
        }
    }
}
