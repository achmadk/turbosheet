use serde::{Deserialize, Serialize};
use std::collections::{HashMap, VecDeque};
use std::sync::Arc;
use tokio::sync::{broadcast, mpsc, RwLock};
use tokio::time::{interval, Duration};

use super::{AutoscalerController, AutoscalerError, AutoScaler, AutoscalingConfig, HealthMonitor, HealthMonitorConfig, ReassignableJob};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GridConfig {
    pub controller_host: String,
    pub controller_port: u16,
    pub max_workers: usize,
    pub idle_timeout_secs: u64,
    pub health_check_interval_secs: u64,
    pub job_timeout_secs: u64,
}

impl Default for GridConfig {
    fn default() -> Self {
        Self {
            controller_host: "0.0.0.0".to_string(),
            controller_port: 8080,
            max_workers: 100,
            idle_timeout_secs: 300,
            health_check_interval_secs: 30,
            job_timeout_secs: 600,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkerRegistration {
    pub worker_id: String,
    pub capabilities: WorkerCapabilities,
    pub status: WorkerStatus,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum WorkerStatus {
    Idle,
    Busy,
    Disconnected,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkerCapabilities {
    pub browser_types: Vec<String>,
    pub max_concurrent: usize,
    pub memory_limit_mb: u64,
    pub cpu_cores: u32,
    pub tags: Vec<String>,
}

use crate::swarm::job::{Job, JobResult, JobStatus};

pub struct GridController {
    config: GridConfig,
    workers: Arc<RwLock<HashMap<String, WorkerInfo>>>,
    job_queue: Arc<RwLock<VecDeque<Job>>>,
    running_jobs: Arc<RwLock<HashMap<String, Job>>>,
    results: Arc<RwLock<Vec<JobResult>>>,
    duration_stats: Arc<RwLock<HashMap<String, u64>>>,
    event_sender: broadcast::Sender<GridEvent>,
    auto_scaler: Option<Arc<AutoScaler>>,
    health_monitor: Option<Arc<HealthMonitor>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum GridEvent {
    WorkerRegistered(String),
    WorkerDisconnected(String),
    JobQueued(String),
    JobStarted { job_id: String, worker_id: String },
    JobCompleted(JobResult),
    WorkerHealthChanged { worker_id: String, healthy: bool },
}

#[derive(Debug, Clone)]
pub struct WorkerInfo {
    pub registration: WorkerRegistration,
    pub last_health_check: u64,
    pub active_jobs: usize,
}

impl GridController {
    pub fn new(config: GridConfig) -> Self {
        let (sender, _) = broadcast::channel(1000);
        Self {
            config: config.clone(),
            workers: Arc::new(RwLock::new(HashMap::new())),
            job_queue: Arc::new(RwLock::new(VecDeque::new())),
            running_jobs: Arc::new(RwLock::new(HashMap::new())),
            results: Arc::new(RwLock::new(Vec::new())),
            duration_stats: Arc::new(RwLock::new(HashMap::new())),
            event_sender: sender,
            auto_scaler: None,
            health_monitor: None,
        }
    }

    pub fn with_autoscaler(mut self, config: AutoscalingConfig) -> Self {
        let worker_count = config.min_workers;
        self.auto_scaler = Some(Arc::new(AutoScaler::new(config, worker_count)));
        self
    }

    pub fn with_health_monitor(mut self, config: HealthMonitorConfig) -> Self {
        self.health_monitor = Some(Arc::new(HealthMonitor::new(config)));
        self
    }

    pub fn get_auto_scaler(&self) -> Option<Arc<AutoScaler>> {
        self.auto_scaler.clone()
    }

    pub fn get_health_monitor(&self) -> Option<Arc<HealthMonitor>> {
        self.health_monitor.clone()
    }

    pub async fn register_worker(&self, registration: WorkerRegistration) -> Result<(), GridError> {
        let mut workers = self.workers.write().await;

        if workers.len() >= self.config.max_workers {
            return Err(GridError::MaxWorkersReached);
        }

        workers.insert(registration.worker_id.clone(), WorkerInfo {
            registration: registration.clone(),
            last_health_check: now_ms(),
            active_jobs: 0,
        });

        let _ = self.event_sender.send(GridEvent::WorkerRegistered(registration.worker_id));

        Ok(())
    }

    pub async fn unregister_worker(&self, worker_id: &str) {
        let mut workers = self.workers.write().await;
        if workers.remove(worker_id).is_some() {
            let _ = self.event_sender.send(GridEvent::WorkerDisconnected(worker_id.to_string()));
        }
    }

    pub async fn submit_job(&self, job: Job) -> Result<(), GridError> {
        let mut queue = self.job_queue.write().await;
        let stats = self.duration_stats.read().await;

        let job_duration = stats.get(&job.test_file).copied().unwrap_or(0);

        // Sort by LPT: longest processing time first, then by priority
        let insert_pos = queue.iter()
            .position(|j| {
                let j_duration = stats.get(&j.test_file).copied().unwrap_or(0);
                if j.priority != job.priority {
                    j.priority < job.priority
                } else {
                    j_duration < job_duration // Insert before shorter jobs
                }
            })
            .unwrap_or(queue.len());

        queue.insert(insert_pos, job.clone());

        let _ = self.event_sender.send(GridEvent::JobQueued(job.id.clone()));

        Ok(())
    }

    pub async fn claim_next_job(&self, worker_id: &str) -> Option<Job> {
        let worker = self.workers.read().await.get(worker_id)?.clone();

        if worker.active_jobs >= worker.registration.capabilities.max_concurrent {
            return None; // Resource limit hit
        }

        let mut queue = self.job_queue.write().await;
        let mut to_remove = None;

        for (i, job) in queue.iter().enumerate() {
            if worker.registration.capabilities.browser_types.contains(&job.browser) {
                to_remove = Some(i);
                break;
            }
        }

        if let Some(idx) = to_remove {
            let job = queue.remove(idx).unwrap();

            let mut workers = self.workers.write().await;
            if let Some(info) = workers.get_mut(worker_id) {
                info.active_jobs += 1;
            }

            let mut running = self.running_jobs.write().await;
            running.insert(job.id.clone(), job.clone());

            let _ = self.event_sender.send(GridEvent::JobStarted {
                job_id: job.id.clone(),
                worker_id: worker_id.to_string(),
            });

            return Some(job);
        }

        None
    }

    pub async fn complete_job(&self, result: JobResult) {
        let mut workers = self.workers.write().await;
        if let Some(info) = workers.get_mut(&result.worker_id) {
            info.active_jobs = info.active_jobs.saturating_sub(1);
        }

        let mut running = self.running_jobs.write().await;
        let completed_job = running.remove(&result.job_id);

        if let Some(job) = completed_job {
            let mut stats = self.duration_stats.write().await;
            let entry = stats.entry(job.test_file.clone()).or_insert(result.duration_ms);
            *entry = (*entry + result.duration_ms) / 2; // Simple moving average
        }

        let mut results = self.results.write().await;
        results.push(result.clone());

        let _ = self.event_sender.send(GridEvent::JobCompleted(result));
    }

    pub async fn get_worker_stats(&self) -> WorkerStats {
        let workers = self.workers.read().await;
        let queue = self.job_queue.read().await;
        let running = self.running_jobs.read().await;
        let results = self.results.read().await;

        let idle = workers.values().filter(|w| w.active_jobs == 0).count();
        let busy = workers.values().filter(|w| w.active_jobs > 0).count();

        let passed = results.iter().filter(|r| r.status == JobStatus::Passed).count();
        let failed = results.iter().filter(|r| r.status == JobStatus::Failed).count();

        WorkerStats {
            total_workers: workers.len(),
            idle_workers: idle,
            busy_workers: busy,
            queued_jobs: queue.len(),
            running_jobs: running.len(),
            completed_jobs: results.len(),
            passed_jobs: passed,
            failed_jobs: failed,
        }
    }

    pub fn subscribe(&self) -> broadcast::Receiver<GridEvent> {
        self.event_sender.subscribe()
    }

    pub async fn get_queue_depth(&self) -> usize {
        self.job_queue.read().await.len()
    }

    pub async fn get_idle_workers(&self) -> usize {
        self.workers.read().await
            .values()
            .filter(|w| w.active_jobs < w.registration.capabilities.max_concurrent)
            .count()
    }

    pub async fn request_worker_scale(&self, count: usize) -> Result<(), AutoscalerError> {
        if self.workers.read().await.len() + count > self.config.max_workers {
            return Err(AutoscalerError::MaxWorkersReached);
        }
        Ok(())
    }

    pub async fn reassign_stale_jobs(&self) -> Vec<String> {
        let mut reassigned = Vec::new();

        if let Some(health) = &self.health_monitor {
            let stale_jobs = health.get_stale_jobs().await;

            for (job_id, worker_id) in stale_jobs {
                if let Some(job) = self.running_jobs.write().await.remove(&job_id) {
                    let mut queue = self.job_queue.write().await;
                    queue.push_front(job);

                    if let Some(info) = self.workers.write().await.get_mut(&worker_id) {
                        info.active_jobs = info.active_jobs.saturating_sub(1);
                    }

                    reassigned.push(job_id);
                }
            }
        }

        reassigned
    }

    pub async fn start_monitoring(&self) {
        if let Some(health) = &self.health_monitor {
            health.start_monitoring().await;
        }

        if let Some(auto_scaler) = &self.auto_scaler {
            if let Some(controller) = self.get_auto_scaler_controller() {
                auto_scaler.start_monitor(controller);
            }
        }
    }

    fn get_auto_scaler_controller(&self) -> Option<Arc<GridController>> {
        Some(Arc::new(self.clone()))
    }
}

#[async_trait::async_trait]
impl AutoscalerController for GridController {
    async fn get_queue_depth(&self) -> usize {
        self.get_queue_depth().await
    }

    async fn get_idle_workers(&self) -> usize {
        self.get_idle_workers().await
    }

    async fn request_worker_scale(&self, count: usize) -> Result<(), AutoscalerError> {
        self.request_worker_scale(count).await
    }
}

impl Clone for GridController {
    fn clone(&self) -> Self {
        Self {
            config: self.config.clone(),
            workers: self.workers.clone(),
            job_queue: self.job_queue.clone(),
            running_jobs: self.running_jobs.clone(),
            results: self.results.clone(),
            event_sender: self.event_sender.clone(),
            auto_scaler: self.auto_scaler.clone(),
            health_monitor: self.health_monitor.clone(),
            duration_stats: self.duration_stats.clone(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkerStats {
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
pub enum GridError {
    MaxWorkersReached,
    WorkerNotFound,
    JobNotFound,
    InvalidJob,
}

impl std::fmt::Display for GridError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            GridError::MaxWorkersReached => write!(f, "Maximum workers reached"),
            GridError::WorkerNotFound => write!(f, "Worker not found"),
            GridError::JobNotFound => write!(f, "Job not found"),
            GridError::InvalidJob => write!(f, "Invalid job"),
        }
    }
}

impl std::error::Error for GridError {}

fn now_ms() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis() as u64
}
