use std::sync::Arc;
use tokio::sync::Mutex;
use dashmap::DashMap;
use uuid::Uuid;

use crate::engine::ContextEngine;
use crate::error::TurbosheetError;

lazy_static::lazy_static! {
    pub static ref CONTEXT_POOL: ContextPool = ContextPool::new();
}

pub struct ContextPool {
    free_contexts: DashMap<String, Vec<Arc<dyn ContextEngine>>>,
    active_contexts: DashMap<String, Arc<dyn ContextEngine>>,
    pool_lock: Mutex<()>,
}

impl ContextPool {
    pub fn new() -> Self {
        Self {
            free_contexts: DashMap::new(),
            active_contexts: DashMap::new(),
            pool_lock: Mutex::new(()),
        }
    }

    pub async fn acquire(
        &self,
        browser_id: &str,
        factory: impl std::future::Future<Output = Result<Arc<dyn ContextEngine>, TurbosheetError>>,
    ) -> Result<String, TurbosheetError> {
        let _guard = self.pool_lock.lock().await;

        let context = if let Some(mut free_list) = self.free_contexts.get_mut(browser_id) {
            if let Some(ctx) = free_list.pop() {
                // Reset context (e.g. clear cookies) before reuse
                ctx.clear_cookies().await?;
                ctx
            } else {
                factory.await?
            }
        } else {
            factory.await?
        };

        let context_id = Uuid::new_v4().to_string();
        self.active_contexts.insert(context_id.clone(), context);

        Ok(context_id)
    }

    pub async fn release(&self, context_id: &str, browser_id: &str) -> Result<(), TurbosheetError> {
        let _guard = self.pool_lock.lock().await;

        if let Some((_, context)) = self.active_contexts.remove(context_id) {
            // Close all pages before pooling
            let pages = context.pages().await?;
            for page in pages {
                page.close().await?;
            }

            self.free_contexts
                .entry(browser_id.to_string())
                .or_insert_with(Vec::new)
                .push(context);
        }

        Ok(())
    }

    pub fn get(&self, context_id: &str) -> Option<Arc<dyn ContextEngine>> {
        self.active_contexts.get(context_id).map(|c| c.clone())
    }
}
