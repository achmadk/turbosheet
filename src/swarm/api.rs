use serde::{Deserialize, Serialize};

use crate::swarm::{JobResult};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SubmitJobRequest {
    pub test_file: String,
    pub browser: Option<String>,
    pub priority: Option<u8>,
    pub timeout_secs: Option<u64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SubmitJobResponse {
    pub job_id: String,
    pub queue_position: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JobStatusResponse {
    pub job_id: String,
    pub status: String,
    pub queue_position: Option<usize>,
    pub result: Option<JobResult>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkerRegisterRequest {
    pub worker_id: String,
    pub capabilities: WorkerCapabilities,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkerCapabilities {
    pub browser_types: Vec<String>,
    pub max_concurrent: usize,
    pub memory_limit_mb: u64,
    pub cpu_cores: u32,
    pub tags: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GridStatsResponse {
    pub total_workers: usize,
    pub idle_workers: usize,
    pub busy_workers: usize,
    pub queued_jobs: usize,
    pub running_jobs: usize,
    pub completed_jobs: usize,
    pub passed_jobs: usize,
    pub failed_jobs: usize,
}

#[derive(Debug)]
pub enum ApiError {
    BindError(std::io::Error),
    ServeError(String),
}

impl std::fmt::Display for ApiError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ApiError::BindError(e) => write!(f, "Failed to bind: {}", e),
            ApiError::ServeError(e) => write!(f, "Serve error: {}", e),
        }
    }
}

impl std::error::Error for ApiError {}
