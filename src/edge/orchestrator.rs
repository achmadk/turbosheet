use serde::{Deserialize, Serialize};
use std::collections::{BinaryHeap, VecDeque};
use std::cmp::Ordering;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OrchestratorConfig {
    pub max_pending_jobs: usize,
    pub max_retries: u32,
    pub default_timeout_secs: u64,
    pub assertion_polling_interval_ms: u64,
    pub assertion_max_attempts: u32,
}

impl Default for OrchestratorConfig {
    fn default() -> Self {
        Self {
            max_pending_jobs: 100,
            max_retries: 3,
            default_timeout_secs: 300,
            assertion_polling_interval_ms: 100,
            assertion_max_attempts: 50,
        }
    }
}

#[derive(Debug, Clone)]
pub struct EdgeJob {
    pub id: String,
    pub test_file: String,
    pub browser: String,
    pub priority: u8,
    pub timeout_secs: u64,
    pub retries_remaining: u32,
    pub created_at: u64,
    pub assertions: Vec<AssertionSpec>,
}

impl EdgeJob {
    pub fn new(test_file: String, browser: String) -> Self {
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            test_file,
            browser,
            priority: 5,
            timeout_secs: 300,
            retries_remaining: 3,
            created_at: now_ms(),
            assertions: Vec::new(),
        }
    }

    pub fn with_priority(mut self, priority: u8) -> Self {
        self.priority = priority;
        self
    }

    pub fn with_timeout(mut self, secs: u64) -> Self {
        self.timeout_secs = secs;
        self
    }

    pub fn with_retries(mut self, retries: u32) -> Self {
        self.retries_remaining = retries;
        self
    }

    pub fn with_assertions(mut self, assertions: Vec<AssertionSpec>) -> Self {
        self.assertions = assertions;
        self
    }

    pub fn can_retry(&self) -> bool {
        self.retries_remaining > 0
    }

    pub fn decrement_retries(&mut self) {
        self.retries_remaining = self.retries_remaining.saturating_sub(1);
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AssertionSpec {
    pub kind: AssertionKind,
    pub target: String,
    pub expected: AssertionValue,
    pub timeout_ms: u64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum AssertionKind {
    Visibility,
    Text,
    Attribute,
    Enabled,
    Value,
    Url,
    Title,
    Count,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AssertionValue {
    Exact(String),
    Regex(String),
    Contains(String),
    GreaterThan(i64),
    LessThan(i64),
    Any,
}

pub struct WasmOrchestrator {
    config: OrchestratorConfig,
    pending_jobs: BinaryHeap<PriorityJob>,
    retry_queue: VecDeque<EdgeJob>,
    running_jobs: std::collections::HashMap<String, RunningJob>,
    completed_jobs: VecDeque<JobResult>,
}

#[derive(Debug, Clone)]
struct PriorityJob {
    job: EdgeJob,
    score: i64,
}

impl PartialEq for PriorityJob {
    fn eq(&self, other: &Self) -> bool {
        self.score == other.score
    }
}

impl Eq for PriorityJob {}

impl PartialOrd for PriorityJob {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for PriorityJob {
    fn cmp(&self, other: &Self) -> Ordering {
        other.score.cmp(&self.score)
    }
}

#[derive(Debug, Clone)]
struct RunningJob {
    job: EdgeJob,
    started_at: u64,
    assertions_executed: u32,
    last_assertion_at: Option<u64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JobResult {
    pub job_id: String,
    pub status: JobOutcome,
    pub duration_ms: u64,
    pub assertions_run: u32,
    pub error: Option<String>,
    pub retry_attempt: u32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum JobOutcome {
    Passed,
    Failed,
    Timeout,
    Cancelled,
    Skipped,
}

impl WasmOrchestrator {
    pub fn new(config: OrchestratorConfig) -> Self {
        Self {
            config,
            pending_jobs: BinaryHeap::new(),
            retry_queue: VecDeque::new(),
            running_jobs: std::collections::HashMap::new(),
            completed_jobs: VecDeque::new(),
        }
    }

    pub fn submit(&mut self, job: EdgeJob) -> Result<(), OrchestratorError> {
        if self.pending_jobs.len() + self.running_jobs.len() >= self.config.max_pending_jobs {
            return Err(OrchestratorError::QueueFull);
        }

        let score = Self::compute_priority_score(&job);
        self.pending_jobs.push(PriorityJob { job, score });
        Ok(())
    }

    pub fn submit_for_retry(&mut self, job: EdgeJob) {
        if job.can_retry() {
            self.retry_queue.push_back(job);
        }
    }

    fn compute_priority_score(job: &EdgeJob) -> i64 {
        let now = now_ms() as i64;
        let age = now - job.created_at as i64;
        let priority_factor = (10 - job.priority as i64) * 1000;
        priority_factor + age
    }

    pub fn claim_next(&mut self) -> Option<EdgeJob> {
        if let Some(mut retry_job) = self.retry_queue.pop_front() {
            retry_job.decrement_retries();
            let score = Self::compute_priority_score(&retry_job);
            self.pending_jobs.push(PriorityJob { job: retry_job, score });
        }

        self.pending_jobs.pop().map(|p| {
            let job = p.job;
            self.running_jobs.insert(job.id.clone(), RunningJob {
                job: job.clone(),
                started_at: now_ms(),
                assertions_executed: 0,
                last_assertion_at: None,
            });
            job
        })
    }

    pub fn complete(&mut self, job_id: &str, result: JobResult) {
        self.running_jobs.remove(job_id);
        self.completed_jobs.push_back(result);

        if self.completed_jobs.len() > 1000 {
            self.completed_jobs.pop_front();
        }
    }

    pub fn get_running_count(&self) -> usize {
        self.running_jobs.len()
    }

    pub fn get_pending_count(&self) -> usize {
        self.pending_jobs.len()
    }

    pub fn get_retry_count(&self) -> usize {
        self.retry_queue.len()
    }

    pub fn check_timeout(&mut self) -> Vec<String> {
        let now = now_ms();
        let mut timed_out = Vec::new();

        for (id, running) in &self.running_jobs {
            let elapsed = (now - running.started_at) / 1000;
            if elapsed >= running.job.timeout_secs {
                timed_out.push(id.clone());
            }
        }

        for id in &timed_out {
            if let Some(running) = self.running_jobs.remove(id) {
                let result = JobResult {
                    job_id: id.clone(),
                    status: JobOutcome::Timeout,
                    duration_ms: now - running.started_at,
                    assertions_run: running.assertions_executed,
                    error: Some("Job timed out".to_string()),
                    retry_attempt: 3 - running.job.retries_remaining,
                };
                self.completed_jobs.push_back(result);
            }
        }

        timed_out
    }

    pub fn get_stats(&self) -> OrchestratorStats {
        OrchestratorStats {
            pending: self.pending_jobs.len(),
            running: self.running_jobs.len(),
            retrying: self.retry_queue.len(),
            completed: self.completed_jobs.len(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OrchestratorStats {
    pub pending: usize,
    pub running: usize,
    pub retrying: usize,
    pub completed: usize,
}

#[derive(Debug)]
pub enum OrchestratorError {
    QueueFull,
    JobNotFound,
    InvalidJob,
}

impl std::fmt::Display for OrchestratorError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            OrchestratorError::QueueFull => write!(f, "Job queue is full"),
            OrchestratorError::JobNotFound => write!(f, "Job not found"),
            OrchestratorError::InvalidJob => write!(f, "Invalid job"),
        }
    }
}

impl std::error::Error for OrchestratorError {}

fn now_ms() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis() as u64
}