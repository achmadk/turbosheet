use serde::{Deserialize, Serialize};
use std::sync::Arc;
use crate::swarm::state::{RedisStateManager, ShardStatus};
use crate::error::TurbosheetError;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ShardingConfig {
    pub total_shards: usize,
    pub shard_index: usize,
    pub run_id: String,
    pub worker_id: String,
}

pub struct SwarmOrchestrator {
    state: Arc<RedisStateManager>,
    config: ShardingConfig,
}

impl SwarmOrchestrator {
    pub fn new(state: Arc<RedisStateManager>, config: ShardingConfig) -> Self {
        Self { state, config }
    }

    pub async fn fetch_and_claim_shards(&self, test_files: Vec<String>) -> Result<Vec<String>, TurbosheetError> {
        let mut claimed_files = Vec::new();
        
        for (i, file) in test_files.iter().enumerate() {
            let file_shard_idx = i % self.config.total_shards;
            
            // Only try to claim tests mapped to our index
            if file_shard_idx != self.config.shard_index {
                continue;
            }
            let file_shard_id = format!("file_{}", file);
            let claimed = self.state.claim_shard(&self.config.run_id, &file_shard_id, &self.config.worker_id).await?;
            
            if claimed {
                claimed_files.push(file.clone());
                
                let status = ShardStatus {
                    shard_id: file_shard_id.clone(),
                    worker_id: self.config.worker_id.clone(),
                    status: "running".to_string(),
                    started_at: now_ms(),
                };
                self.state.update_shard_status(&self.config.run_id, &file_shard_id, &status).await?;
            }
        }
        
        Ok(claimed_files)
    }

    pub async fn report_shard_complete(&self, file: &str, success: bool) -> Result<(), TurbosheetError> {
        let file_shard_id = format!("file_{}", file);
        let status_str = if success { "completed" } else { "failed" };
        
        let status = ShardStatus {
            shard_id: file_shard_id.clone(),
            worker_id: self.config.worker_id.clone(),
            status: status_str.to_string(),
            started_at: now_ms(),
        };
        
        self.state.update_shard_status(&self.config.run_id, &file_shard_id, &status).await?;
        Ok(())
    }
}

fn now_ms() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis() as u64
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Arc;

    #[tokio::test]
    async fn test_sharding_distribution() {
        let state = Arc::new(RedisStateManager::new(None)); // In-memory mock
        
        let orchestrator1 = SwarmOrchestrator::new(state.clone(), ShardingConfig {
            total_shards: 2,
            shard_index: 0,
            run_id: "test_run".to_string(),
            worker_id: "worker_1".to_string(),
        });
        
        let orchestrator2 = SwarmOrchestrator::new(state.clone(), ShardingConfig {
            total_shards: 2,
            shard_index: 1,
            run_id: "test_run".to_string(),
            worker_id: "worker_2".to_string(),
        });
        
        let files = vec![
            "test1.spec.ts".to_string(),
            "test2.spec.ts".to_string(),
            "test3.spec.ts".to_string(),
            "test4.spec.ts".to_string(),
        ];
        
        let claimed1 = orchestrator1.fetch_and_claim_shards(files.clone()).await.unwrap();
        let claimed2 = orchestrator2.fetch_and_claim_shards(files.clone()).await.unwrap();
        
        assert_eq!(claimed1, vec!["test1.spec.ts".to_string(), "test3.spec.ts".to_string()]);
        assert_eq!(claimed2, vec!["test2.spec.ts".to_string(), "test4.spec.ts".to_string()]);
    }
}
