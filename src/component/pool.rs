use std::sync::Arc;
use tokio::sync::Mutex;

pub struct ComponentContextPool {
    contexts: Arc<Mutex<Vec<PooledContext>>>,
    max_size: usize,
}

struct PooledContext {
    page_id: String,
    in_use: bool,
    last_used: std::time::Instant,
    framework: String,
}

impl ComponentContextPool {
    pub fn new(max_size: usize) -> Self {
        Self {
            contexts: Arc::new(Mutex::new(Vec::new())),
            max_size,
        }
    }

    pub async fn acquire(&self, framework: &str) -> Option<String> {
        let mut pool = self.contexts.lock().await;

        if let Some(ctx) = pool.iter_mut().find(|ctx| !ctx.in_use && ctx.framework == framework) {
            ctx.in_use = true;
            ctx.last_used = std::time::Instant::now();
            return Some(ctx.page_id.clone());
        }

        if pool.len() < self.max_size {
            let page_id = uuid::Uuid::new_v4().to_string();
            pool.push(PooledContext {
                page_id: page_id.clone(),
                in_use: true,
                last_used: std::time::Instant::now(),
                framework: framework.to_string(),
            });
            return Some(page_id);
        }

        None
    }

    pub async fn release(&self, framework: &str, page_id: &str) {
        let mut pool = self.contexts.lock().await;
        if let Some(ctx) = pool.iter_mut().find(|ctx| ctx.page_id == page_id && ctx.framework == framework) {
            ctx.in_use = false;
            ctx.last_used = std::time::Instant::now();
        }
    }

    pub async fn clear(&self) {
        let mut pool = self.contexts.lock().await;
        pool.retain(|ctx| ctx.in_use);
    }

    pub async fn stats(&self) -> PoolStats {
        let pool = self.contexts.lock().await;
        let total = pool.len();
        let in_use = pool.iter().filter(|ctx| ctx.in_use).count();
        PoolStats {
            total_contexts: total,
            in_use_contexts: in_use,
            available_contexts: total - in_use,
        }
    }
}

impl Default for ComponentContextPool {
    fn default() -> Self {
        Self::new(10)
    }
}

#[derive(Debug, Clone)]
pub struct PoolStats {
    pub total_contexts: usize,
    pub in_use_contexts: usize,
    pub available_contexts: usize,
}
