use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum RuntimeType {
    Workerd,
    Deno,
    Bun,
    Node,
    BareMetal,
    Unknown,
}

#[derive(Debug, Clone)]
pub struct RuntimeInfo {
    pub runtime_type: RuntimeType,
    pub version: Option<String>,
    pub features: RuntimeFeatures,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct RuntimeFeatures {
    pub supports_webassembly: bool,
    pub supports_wasi: bool,
    pub supports_threads: bool,
    pub supports_shared_memory: bool,
    pub supports_simd: bool,
    pub max_memory_mb: Option<u64>,
    pub cpu_count: Option<u32>,
}

impl RuntimeInfo {
    pub fn detect() -> Self {
        #[cfg(target_arch = "wasm32")]
        {
            Self::detect_wasm()
        }
        #[cfg(not(target_arch = "wasm32"))]
        {
            Self::detect_native()
        }
    }

    #[cfg(target_arch = "wasm32")]
    fn detect_wasm() -> Self {
        let (runtime_type, version) = Self::detect_wasm_runtime();

        let features = RuntimeFeatures {
            supports_webassembly: true,
            supports_wasi: true,
            supports_threads: Self::has_threads_support(),
            supports_shared_memory: Self::has_shared_memory(),
            supports_simd: Self::has_simd_support(),
            max_memory_mb: Self::get_wasm_memory_limit(),
            cpu_count: None,
        };

        Self {
            runtime_type,
            version,
            features,
        }
    }

    #[cfg(target_arch = "wasm32")]
    fn detect_wasm_runtime() -> (RuntimeType, Option<String>) {
        if Self::is_workerd() {
            (RuntimeType::Workerd, Some("workerd".to_string()))
        } else if Self::is_deno() {
            (RuntimeType::Deno, Some("deno".to_string()))
        } else if Self::is_bun() {
            (RuntimeType::Bun, Some("bun".to_string()))
        } else if Self::is_node() {
            (RuntimeType::Node, Self::get_node_version())
        } else {
            (RuntimeType::Unknown, None)
        }
    }

    #[cfg(target_arch = "wasm32")]
    fn is_workerd() -> bool {
        Self::has_js_builtin("__workerd__")
    }

    #[cfg(target_arch = "wasm32")]
    fn is_deno() -> bool {
        Self::has_js_builtin("__deno__")
    }

    #[cfg(target_arch = "wasm32")]
    fn is_bun() -> bool {
        Self::has_js_builtin("__bun__")
    }

    #[cfg(target_arch = "wasm32")]
    fn is_node() -> bool {
        Self::has_js_builtin("__nodejs__")
    }

    #[cfg(target_arch = "wasm32")]
    fn has_js_builtin(name: &str) -> bool {
        #[link_section = ".custom_section.__jsc"]
        extern "C" {
            static __jsc_internals: u8;
        }
        false
    }

    #[cfg(target_arch = "wasm32")]
    fn get_node_version() -> Option<String> {
        None
    }

    #[cfg(target_arch = "wasm32")]
    fn has_threads_support() -> bool {
        #[cfg(feature = "wasm_threads")]
        return true;
        #[cfg(not(feature = "wasm_threads"))]
        return false;
    }

    #[cfg(target_arch = "wasm32")]
    fn has_shared_memory() -> bool {
        true
    }

    #[cfg(target_arch = "wasm32")]
    fn has_simd_support() -> bool {
        #[cfg(feature = "wasm_simd")]
        return true;
        #[cfg(not(feature = "wasm_simd"))]
        return false;
    }

    #[cfg(target_arch = "wasm32")]
    fn get_wasm_memory_limit() -> Option<u64> {
        Some(512)
    }

    #[cfg(not(target_arch = "wasm32"))]
    fn detect_native() -> Self {
        Self {
            runtime_type: RuntimeType::BareMetal,
            version: Some(String::from("native")),
            features: RuntimeFeatures {
                supports_webassembly: true,
                supports_wasi: false,
                supports_threads: true,
                supports_shared_memory: true,
                supports_simd: true,
                max_memory_mb: None,
                cpu_count: Some(num_cpus::get() as u32),
            },
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RuntimeConfig {
    pub runtime_type: RuntimeType,
    pub max_concurrent_jobs: usize,
    pub memory_limit_mb: u64,
    pub enable_assertions_in_wasm: bool,
    pub grid_proxy_url: Option<String>,
}

impl Default for RuntimeConfig {
    fn default() -> Self {
        Self {
            runtime_type: RuntimeType::Unknown,
            max_concurrent_jobs: 10,
            memory_limit_mb: 256,
            enable_assertions_in_wasm: true,
            grid_proxy_url: None,
        }
    }
}

impl RuntimeConfig {
    pub fn for_workerd() -> Self {
        Self {
            runtime_type: RuntimeType::Workerd,
            max_concurrent_jobs: 5,
            memory_limit_mb: 128,
            enable_assertions_in_wasm: true,
            grid_proxy_url: None,
        }
    }

    pub fn for_deno() -> Self {
        Self {
            runtime_type: RuntimeType::Deno,
            max_concurrent_jobs: 10,
            memory_limit_mb: 512,
            enable_assertions_in_wasm: true,
            grid_proxy_url: None,
        }
    }

    pub fn for_bare_metal() -> Self {
        Self {
            runtime_type: RuntimeType::BareMetal,
            max_concurrent_jobs: 50,
            memory_limit_mb: 1024,
            enable_assertions_in_wasm: false,
            grid_proxy_url: None,
        }
    }
}

pub struct EdgeRuntime {
    info: RuntimeInfo,
    config: RuntimeConfig,
}

impl EdgeRuntime {
    pub fn new(config: RuntimeConfig) -> Self {
        Self {
            info: RuntimeInfo::detect(),
            config,
        }
    }

    pub fn info(&self) -> &RuntimeInfo {
        &self.info
    }

    pub fn config(&self) -> &RuntimeConfig {
        &self.config
    }

    pub fn can_run_assertions(&self) -> bool {
        self.config.enable_assertions_in_wasm && self.info.features.supports_webassembly
    }

    pub fn recommended_max_jobs(&self) -> usize {
        let base = self.config.max_concurrent_jobs;
        let memory_factor = self.config.memory_limit_mb / 64;
        base.min(memory_factor as usize)
    }

    pub fn should_proxy_to_grid(&self) -> bool {
        matches!(
            self.info.runtime_type,
            RuntimeType::Workerd | RuntimeType::Deno | RuntimeType::Bun
        ) && !self.config.enable_assertions_in_wasm
    }
}