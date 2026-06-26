pub mod bindings;
pub mod scripts;
pub mod stealth;

/// Manages the lifecycle of injected scripts (core + actions) for a page.
///
/// Delegates the actual CDP calls (`addScriptToEvaluateOnNewDocument`,
/// `Runtime.evaluate`) to the caller, but owns the script strings and
/// exposes a high-level `inject_core()` / `call_action()` API.
pub struct InjectionManager {
    core_script: &'static str,
    actions_script: &'static str,
}

impl InjectionManager {
    /// Creates a new manager wrapping the compiled JS blobs.
    pub fn new() -> Self {
        Self {
            core_script: scripts::CORE_SCRIPT,
            actions_script: scripts::ACTIONS_SCRIPT,
        }
    }

    /// Returns the raw core script bytes for injection via
    /// `addScriptToEvaluateOnNewDocument` (auto-run on every new document).
    pub fn core_script(&self) -> &'static str {
        self.core_script
    }

    /// Returns the actions script (lazy-loaded on first action call).
    pub fn actions_script(&self) -> &'static str {
        self.actions_script
    }

    /// Build an `invoke_action` JS snippet that dispatches an action through
    /// the binding bridge and returns the result as a string expression.
    pub fn build_action_call(
        &self,
        global_name: &str,
        binding_name: &str,
        action: &str,
        args_serialized: &str,
    ) -> String {
        let action_id = uuid::Uuid::new_v4().to_string();
        format!(
            r#"(async function() {{
                if (!window.{} || !window.{}.content) {{
                    {}
                }}
                window.{}.dispatch('{}', '{}{}{}');
            }})()"#,
            global_name,
            global_name,
            self.actions_script
                .replace("__TS_GLOBAL__", global_name)
                .replace("__TS_BINDING__", binding_name),
            global_name,
            action_id,
            action,
            if args_serialized.is_empty() { "" } else { ", " },
            args_serialized,
        )
    }
}

impl Default for InjectionManager {
    fn default() -> Self {
        Self::new()
    }
}
