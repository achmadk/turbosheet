use serde::{Deserialize, Serialize};

/// Controls runtime name-visibility trade-offs for the injected scripts.
///
/// When enabled, generated global names avoid the `__ts_` prefix and the
/// public proxy is set as non-enumerable so it doesn't appear in
/// `Object.getOwnPropertyNames(window)` or `for..in` enumeration.
///
/// Stealth mode is **default-enabled** (`StealthConfig::default()`).
/// Disable for debugging with `StealthConfig::disabled()`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StealthConfig {
    /// Whether stealth transformations are applied.
    /// Default: `true`
    pub enabled: bool,

    /// Style of generated global / binding names.
    /// Default: `StealthNameStyle::Simple`
    pub name_style: StealthNameStyle,
}

/// How injected-global names should look on `window`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum StealthNameStyle {
    /// `__ts_g_{uuid}` / `__ts_b_{uuid}` — obvious but still unique per page.
    /// Useful for debugging.
    Explicit,
    /// Short names: `_a{short_uuid}`, `_b{short_uuid}` — less obvious but
    /// still recognisable in devtools.
    Simple,
    /// Realistic-looking property names drawn from a pool (e.g. `jQuery`,
    /// `_n`, `__WEBPACK__`) — blend in with real page globals.
    Realistic,
}

impl Default for StealthConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            name_style: StealthNameStyle::Simple,
        }
    }
}

impl StealthConfig {
    /// Convenience constructor for fully disabled stealth (debugging).
    pub fn disabled() -> Self {
        Self {
            enabled: false,
            name_style: StealthNameStyle::Explicit,
        }
    }

    /// Generate a global-name and binding-name pair according to the config.
    pub fn generate_names(&self) -> (String, String) {
        if !self.enabled {
            let g = format!("__ts_g_{}", uuid::Uuid::new_v4().simple());
            let b = format!("__ts_b_{}", uuid::Uuid::new_v4().simple());
            return (g, b);
        }
        match self.name_style {
            StealthNameStyle::Explicit => {
                let g = format!("__ts_g_{}", uuid::Uuid::new_v4().simple());
                let b = format!("__ts_b_{}", uuid::Uuid::new_v4().simple());
                (g, b)
            }
            StealthNameStyle::Simple => {
                let short = &uuid::Uuid::new_v4().simple().to_string()[..8];
                let g = format!("_a{}", short);
                let b = format!("_b{}", short);
                (g, b)
            }
            StealthNameStyle::Realistic => {
                let short = &uuid::Uuid::new_v4().simple().to_string()[..6];
                let pool = [
                    "_n", "_t", "jQuery", "_react", "vue", "_s", "_l",
                    "_w", "_h", "_d", "_r", "__WEBPACK__", "__NEXT_DATA__",
                    "_paq", "_ga", "ga", "Stripe", "paypal",
                ];
                let idx = (short.as_bytes()[0] as usize) % pool.len();
                let g = format!("{}{}", pool[idx], short);
                let b = format!("_{}", short);
                (g, b)
            }
        }
    }

    /// Process a raw script by replacing `__TS_GLOBAL__` and `__TS_BINDING__`
    /// placeholders with generated names.
    #[inline]
    pub fn process_script(&self, script: &str, global_name: &str, binding_name: &str) -> String {
        script
            .replace("__TS_GLOBAL__", global_name)
            .replace("__TS_BINDING__", binding_name)
    }

    /// Build a JS expression that accesses the public proxy.
    ///
    /// For stealth modes, uses bracket notation `window[<name>]` which works
    /// regardless of whether the property is non-enumerable.
    pub fn global_expr(&self, global_name: &str) -> String {
        if !self.enabled || self.name_style == StealthNameStyle::Explicit {
            format!("window.{}", global_name)
        } else {
            format!("window['{}']", global_name)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_is_enabled_simple() {
        let cfg = StealthConfig::default();
        assert!(cfg.enabled);
        assert_eq!(cfg.name_style, StealthNameStyle::Simple);
    }

    #[test]
    fn test_disabled_config() {
        let cfg = StealthConfig::disabled();
        assert!(!cfg.enabled);
        assert_eq!(cfg.name_style, StealthNameStyle::Explicit);
    }

    #[test]
    fn test_generate_names_explicit() {
        let cfg = StealthConfig { enabled: true, name_style: StealthNameStyle::Explicit };
        let (g, b) = cfg.generate_names();
        assert!(g.starts_with("__ts_g_"));
        assert!(b.starts_with("__ts_b_"));
        assert!(g.len() > 10);
    }

    #[test]
    fn test_generate_names_simple() {
        let cfg = StealthConfig { enabled: true, name_style: StealthNameStyle::Simple };
        let (g, b) = cfg.generate_names();
        assert!(g.starts_with("_a"));
        assert!(b.starts_with("_b"));
        assert_eq!(g.len(), 10);
        assert_eq!(b.len(), 10);
    }

    #[test]
    fn test_generate_names_realistic() {
        let cfg = StealthConfig { enabled: true, name_style: StealthNameStyle::Realistic };
        let (g, b) = cfg.generate_names();
        assert!(!g.starts_with("__ts"));
        assert!(b.starts_with("_"));
        assert!(g.len() > 6);
    }

    #[test]
    fn test_process_script_placeholder_replacement() {
        let cfg = StealthConfig::default();
        let script = "window.__TS_GLOBAL__.dispatch('abc', 'click'); (window.__TS_BINDING__)('payload');";
        let (global, binding) = cfg.generate_names();
        let result = cfg.process_script(script, &global, &binding);
        assert!(!result.contains("__TS_GLOBAL__"));
        assert!(!result.contains("__TS_BINDING__"));
        assert!(result.contains(&global));
        assert!(result.contains(&binding));
    }

    #[test]
    fn test_process_script_disabled() {
        let cfg = StealthConfig::disabled();
        let script = "window.__TS_GLOBAL__.dispatch('abc');";
        let result = cfg.process_script(script, "__ts_g_abc123", "__ts_b_abc123");
        assert!(result.contains("__ts_g_abc123"));
        assert!(!result.contains("__TS_GLOBAL__"));
    }

    #[test]
    fn test_global_expr_stealth_uses_bracket_notation() {
        let cfg = StealthConfig::default();
        assert_eq!(cfg.global_expr("_abc123"), "window['_abc123']");
    }

    #[test]
    fn test_global_expr_disabled_uses_dot_notation() {
        let cfg = StealthConfig::disabled();
        assert_eq!(cfg.global_expr("__ts_g_abc"), "window.__ts_g_abc");
    }

    #[test]
    fn test_global_expr_explicit_uses_dot_notation() {
        let cfg = StealthConfig { enabled: true, name_style: StealthNameStyle::Explicit };
        assert_eq!(cfg.global_expr("__ts_g_abc"), "window.__ts_g_abc");
    }

    #[test]
    fn test_generate_names_non_stealth_uses_ts_prefix() {
        let cfg = StealthConfig::disabled();
        let (g, b) = cfg.generate_names();
        assert!(g.starts_with("__ts_g_"));
        assert!(b.starts_with("__ts_b_"));
    }
}
