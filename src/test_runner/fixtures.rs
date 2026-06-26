use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::Mutex;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FixtureScope {
    Test,
    Worker,
    Global,
}

#[derive(Debug, Clone)]
pub struct FixtureDefinition {
    pub name: String,
    pub scope: FixtureScope,
    pub dependencies: Vec<String>,
}

pub struct FixtureRegistry {
    fixtures: HashMap<String, FixtureDefinition>,
}

impl FixtureRegistry {
    pub fn new() -> Self {
        Self {
            fixtures: HashMap::new(),
        }
    }

    pub fn register(&mut self, def: FixtureDefinition) {
        self.fixtures.insert(def.name.clone(), def);
    }

    pub fn resolve_dependencies(&self, target: &str) -> Result<Vec<String>, String> {
        let mut resolved = Vec::new();
        let mut visiting = Vec::new();

        self.resolve_recursive(target, &mut resolved, &mut visiting)?;

        Ok(resolved)
    }

    fn resolve_recursive(
        &self,
        target: &str,
        resolved: &mut Vec<String>,
        visiting: &mut Vec<String>,
    ) -> Result<(), String> {
        if resolved.contains(&target.to_string()) {
            return Ok(());
        }

        if visiting.contains(&target.to_string()) {
            return Err(format!("Circular dependency detected: {} -> {}", visiting.join(" -> "), target));
        }

        visiting.push(target.to_string());

        if let Some(def) = self.fixtures.get(target) {
            for dep in &def.dependencies {
                self.resolve_recursive(dep, resolved, visiting)?;
            }
        }

        visiting.pop();
        resolved.push(target.to_string());

        Ok(())
    }
}

pub struct WorkerFixtureCache {
    cache: Arc<Mutex<HashMap<String, serde_json::Value>>>,
}

impl WorkerFixtureCache {
    pub fn new() -> Self {
        Self {
            cache: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    pub async fn get(&self, name: &str) -> Option<serde_json::Value> {
        let cache = self.cache.lock().await;
        cache.get(name).cloned()
    }

    pub async fn set(&self, name: String, value: serde_json::Value) {
        let mut cache = self.cache.lock().await;
        cache.insert(name, value);
    }

    pub async fn clear(&self) {
        let mut cache = self.cache.lock().await;
        cache.clear();
    }
}

pub struct GlobalFixtureCache {
    cache: Arc<Mutex<HashMap<String, serde_json::Value>>>,
}

impl GlobalFixtureCache {
    pub fn new() -> Self {
        Self {
            cache: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    pub async fn get(&self, name: &str) -> Option<serde_json::Value> {
        let cache = self.cache.lock().await;
        cache.get(name).cloned()
    }

    pub async fn set(&self, name: String, value: serde_json::Value) {
        let mut cache = self.cache.lock().await;
        cache.insert(name, value);
    }
}

pub struct TestFixtureCache {
    cache: Arc<Mutex<HashMap<String, serde_json::Value>>>,
}

impl TestFixtureCache {
    pub fn new() -> Self {
        Self {
            cache: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    pub async fn get(&self, name: &str) -> Option<serde_json::Value> {
        let cache = self.cache.lock().await;
        cache.get(name).cloned()
    }

    pub async fn set(&self, name: String, value: serde_json::Value) {
        let mut cache = self.cache.lock().await;
        cache.insert(name, value);
    }

    pub async fn clear(&self) {
        let mut cache = self.cache.lock().await;
        cache.clear();
    }
}
