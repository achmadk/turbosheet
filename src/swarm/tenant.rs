use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TenantConfig {
    pub tenant_id: String,
    pub name: String,
    pub cpu_limit: f32,
    pub memory_limit_mb: u64,
    pub max_concurrent_jobs: usize,
    pub network_isolation: bool,
    pub allowed_browsers: Vec<String>,
    pub priority: u8,
}

impl TenantConfig {
    pub fn new(tenant_id: String, name: String) -> Self {
        Self {
            tenant_id,
            name,
            cpu_limit: 1.0,
            memory_limit_mb: 1024,
            max_concurrent_jobs: 10,
            network_isolation: true,
            allowed_browsers: vec!["chromium".to_string()],
            priority: 5,
        }
    }

    pub fn with_resources(mut self, cpu: f32, memory_mb: u64) -> Self {
        self.cpu_limit = cpu;
        self.memory_limit_mb = memory_mb;
        self
    }

    pub fn with_concurrency(mut self, max: usize) -> Self {
        self.max_concurrent_jobs = max;
        self
    }
}

#[derive(Debug, Clone)]
pub struct Tenant {
    config: TenantConfig,
    active_jobs: Arc<RwLock<usize>>,
    container_ids: Arc<RwLock<Vec<String>>>,
}

impl Tenant {
    pub fn new(config: TenantConfig) -> Self {
        Self {
            config,
            active_jobs: Arc::new(RwLock::new(0)),
            container_ids: Arc::new(RwLock::new(Vec::new())),
        }
    }

    pub fn id(&self) -> &str {
        &self.config.tenant_id
    }

    pub fn config(&self) -> &TenantConfig {
        &self.config
    }

    pub async fn can_start_job(&self) -> bool {
        let active = *self.active_jobs.read().await;
        active < self.config.max_concurrent_jobs
    }

    pub async fn increment_jobs(&self) -> Result<(), TenantError> {
        let mut active = self.active_jobs.write().await;
        if *active >= self.config.max_concurrent_jobs {
            return Err(TenantError::ConcurrencyLimitReached(self.config.tenant_id.clone()));
        }
        *active += 1;
        Ok(())
    }

    pub async fn decrement_jobs(&self) {
        let mut active = self.active_jobs.write().await;
        *active = active.saturating_sub(1);
    }

    pub async fn get_active_jobs(&self) -> usize {
        *self.active_jobs.read().await
    }

    pub async fn register_container(&self, container_id: String) {
        let mut containers = self.container_ids.write().await;
        containers.push(container_id);
    }

    pub async fn unregister_container(&self, container_id: &str) {
        let mut containers = self.container_ids.write().await;
        containers.retain(|id| id != container_id);
    }

    pub async fn get_containers(&self) -> Vec<String> {
        self.container_ids.read().await.clone()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContainerSpec {
    pub image: String,
    pub cpu_limit: f32,
    pub memory_limit_mb: u64,
    pub network_mode: NetworkMode,
    pub volumes: Vec<VolumeMount>,
    pub environment: HashMap<String, String>,
}

impl ContainerSpec {
    pub fn default_browser() -> Self {
        Self {
            image: "rashiq-browser:latest".to_string(),
            cpu_limit: 1.0,
            memory_limit_mb: 1024,
            network_mode: NetworkMode::Bridge,
            volumes: vec![],
            environment: HashMap::new(),
        }
    }

    pub fn with_network(mut self, mode: NetworkMode) -> Self {
        self.network_mode = mode;
        self
    }

    pub fn with_volume(mut self, host_path: String, container_path: String) -> Self {
        self.volumes.push(VolumeMount {
            host_path,
            container_path,
            read_only: false,
        });
        self
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum NetworkMode {
    Bridge,
    Host,
    None,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VolumeMount {
    pub host_path: String,
    pub container_path: String,
    pub read_only: bool,
}

pub struct DockerTenantManager {
    tenants: Arc<RwLock<HashMap<String, Tenant>>>,
    default_tenant: Tenant,
}

impl DockerTenantManager {
    pub fn new() -> Self {
        let default_config = TenantConfig::new("default".to_string(), "Default".to_string());
        Self {
            tenants: Arc::new(RwLock::new(HashMap::new())),
            default_tenant: Tenant::new(default_config),
        }
    }

    pub async fn create_tenant(&self, config: TenantConfig) -> Result<(), TenantError> {
        let mut tenants = self.tenants.write().await;
        if tenants.contains_key(&config.tenant_id) {
            return Err(TenantError::TenantAlreadyExists(config.tenant_id));
        }
        tenants.insert(config.tenant_id.clone(), Tenant::new(config));
        Ok(())
    }

    pub async fn get_tenant(&self, tenant_id: &str) -> Option<Arc<Tenant>> {
        let tenants = self.tenants.read().await;
        tenants.get(tenant_id).map(|t| Arc::new(t.clone()))
    }

    pub async fn get_or_default(&self, tenant_id: Option<&str>) -> Arc<Tenant> {
        match tenant_id {
            Some(id) => {
                let tenants = self.tenants.read().await;
                tenants.get(id).map(|t| Arc::new(t.clone()))
                    .unwrap_or_else(|| Arc::new(self.default_tenant.clone()))
            }
            None => Arc::new(self.default_tenant.clone()),
        }
    }

    pub async fn remove_tenant(&self, tenant_id: &str) -> Result<(), TenantError> {
        if tenant_id == "default" {
            return Err(TenantError::CannotRemoveDefaultTenant);
        }
        let mut tenants = self.tenants.write().await;
        tenants.remove(tenant_id)
            .ok_or(TenantError::TenantNotFound(tenant_id.to_string()))?;
        Ok(())
    }

    pub async fn list_tenants(&self) -> Vec<String> {
        let tenants = self.tenants.read().await;
        let mut ids: Vec<String> = tenants.keys().cloned().collect();
        ids.push("default".to_string());
        ids
    }
}

impl Default for DockerTenantManager {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug)]
pub enum TenantError {
    TenantAlreadyExists(String),
    TenantNotFound(String),
    ConcurrencyLimitReached(String),
    CannotRemoveDefaultTenant,
    ContainerError(String),
}

impl std::fmt::Display for TenantError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TenantError::TenantAlreadyExists(id) => write!(f, "Tenant {} already exists", id),
            TenantError::TenantNotFound(id) => write!(f, "Tenant {} not found", id),
            TenantError::ConcurrencyLimitReached(id) => write!(f, "Concurrency limit reached for tenant {}", id),
            TenantError::CannotRemoveDefaultTenant => write!(f, "Cannot remove default tenant"),
            TenantError::ContainerError(s) => write!(f, "Container error: {}", s),
        }
    }
}

impl std::error::Error for TenantError {}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TenantStats {
    pub tenant_id: String,
    pub active_jobs: usize,
    pub max_jobs: usize,
    pub cpu_limit: f32,
    pub memory_limit_mb: u64,
    pub containers: usize,
}

impl Tenant {
    pub async fn get_stats(&self) -> TenantStats {
        TenantStats {
            tenant_id: self.config.tenant_id.clone(),
            active_jobs: self.get_active_jobs().await,
            max_jobs: self.config.max_concurrent_jobs,
            cpu_limit: self.config.cpu_limit,
            memory_limit_mb: self.config.memory_limit_mb,
            containers: self.get_containers().await.len(),
        }
    }
}