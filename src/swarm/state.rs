use redis::AsyncCommands;
use serde::{Deserialize, Serialize};
use crate::error::TurbosheetError;
use std::sync::Arc;

pub struct RedisStateManager {
    client: Option<redis::Client>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ShardStatus {
    pub shard_id: String,
    pub worker_id: String,
    pub status: String,
    pub started_at: u64,
}

impl RedisStateManager {
    pub fn new(redis_url: Option<&str>) -> Self {
        let client = if let Some(url) = redis_url {
            redis::Client::open(url).ok()
        } else {
            None
        };
        
        Self { client }
    }

    pub async fn register_worker(&self, worker_id: &str) -> Result<(), TurbosheetError> {
        if let Some(client) = &self.client {
            let mut con = client.get_async_connection().await.map_err(|e| TurbosheetError::Other(e.to_string()))?;
            let _: () = con.sadd("tsheet:workers:active", worker_id).await.map_err(|e| TurbosheetError::Other(e.to_string()))?;
        }
        Ok(())
    }

    pub async fn claim_shard(&self, run_id: &str, shard_id: &str, worker_id: &str) -> Result<bool, TurbosheetError> {
        if let Some(client) = &self.client {
            let mut con = client.get_async_connection().await.map_err(|e| TurbosheetError::Other(e.to_string()))?;
            let key = format!("tsheet:run:{}:shard:{}", run_id, shard_id);
            // setnx returns 1 if set, 0 if already exists
            let claimed: bool = redis::cmd("SETNX").arg(&key).arg(worker_id).query_async(&mut con).await.map_err(|e| TurbosheetError::Other(e.to_string()))?;
            Ok(claimed)
        } else {
            // In-memory fallback
            Ok(true)
        }
    }

    pub async fn update_shard_status(&self, run_id: &str, shard_id: &str, status: &ShardStatus) -> Result<(), TurbosheetError> {
        if let Some(client) = &self.client {
            let mut con = client.get_async_connection().await.map_err(|e| TurbosheetError::Other(e.to_string()))?;
            let key = format!("tsheet:run:{}:status", run_id);
            let json = serde_json::to_string(status).unwrap();
            let _: () = con.hset(key, shard_id, json).await.map_err(|e| TurbosheetError::Other(e.to_string()))?;
        }
        Ok(())
    }
}
