use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Job {
    pub id: String,
    pub test_file: String,
    pub browser: String,
    pub priority: u8,
    pub timeout_secs: u64,
    pub created_at: u64,
    pub metadata: JobMetadata,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct JobMetadata {
    pub shard: Option<ShardInfo>,
    pub retries: u32,
    pub worker_id: Option<String>,
    pub tags: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ShardInfo {
    pub current: u32,
    pub total: u32,
}

impl Job {
    pub fn new(test_file: String, browser: String) -> Self {
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            test_file,
            browser,
            priority: 5,
            timeout_secs: 300,
            created_at: now_ms(),
            metadata: JobMetadata::default(),
        }
    }

    pub fn with_priority(mut self, priority: u8) -> Self {
        self.priority = priority;
        self
    }

    pub fn with_timeout(mut self, timeout_secs: u64) -> Self {
        self.timeout_secs = timeout_secs;
        self
    }

    pub fn with_shard(mut self, current: u32, total: u32) -> Self {
        self.metadata.shard = Some(ShardInfo { current, total });
        self
    }

    pub fn with_retries(mut self, retries: u32) -> Self {
        self.metadata.retries = retries;
        self
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JobResult {
    pub job_id: String,
    pub worker_id: String,
    pub status: JobStatus,
    pub duration_ms: u64,
    pub started_at: u64,
    pub completed_at: u64,
    pub error: Option<String>,
    pub screenshots: Vec<String>,
    pub trace_path: Option<String>,
    pub console_output: Vec<ConsoleEntry>,
    pub network_events: Vec<NetworkEntry>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum JobStatus {
    Queued,
    Running,
    Passed,
    Failed,
    Timeout,
    Cancelled,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConsoleEntry {
    pub timestamp: u64,
    pub level: String,
    pub message: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkEntry {
    pub timestamp: u64,
    pub method: String,
    pub url: String,
    pub status: Option<u16>,
    pub duration_ms: u64,
}

impl JobResult {
    pub fn success(job_id: String, worker_id: String, duration_ms: u64) -> Self {
        Self {
            job_id,
            worker_id,
            status: JobStatus::Passed,
            duration_ms,
            started_at: 0,
            completed_at: now_ms(),
            error: None,
            screenshots: vec![],
            trace_path: None,
            console_output: vec![],
            network_events: vec![],
        }
    }

    pub fn failure(job_id: String, worker_id: String, error: String, duration_ms: u64) -> Self {
        Self {
            job_id,
            worker_id,
            status: JobStatus::Failed,
            duration_ms,
            started_at: 0,
            completed_at: now_ms(),
            error: Some(error),
            screenshots: vec![],
            trace_path: None,
            console_output: vec![],
            network_events: vec![],
        }
    }

    pub fn timeout(job_id: String, worker_id: String) -> Self {
        Self {
            job_id,
            worker_id,
            status: JobStatus::Timeout,
            duration_ms: 0,
            started_at: 0,
            completed_at: now_ms(),
            error: Some("Job timed out".to_string()),
            screenshots: vec![],
            trace_path: None,
            console_output: vec![],
            network_events: vec![],
        }
    }
}

fn now_ms() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis() as u64
}
