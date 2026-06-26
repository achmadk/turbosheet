use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::{mpsc, RwLock};
use tokio::time::{interval, Duration};

use crate::swarm::{GridController, GridConfig, Job, JobResult, JobStatus, WorkerCapabilities, WorkerRegistration, WorkerStatus};

pub struct WorkerNode {
    worker_id: String,
    controller_url: String,
    capabilities: WorkerCapabilities,
    status: Arc<RwLock<WorkerStatus>>,
    active_jobs: Arc<RwLock<Vec<String>>>,
    shutdown_tx: Option<mpsc::Sender<()>>,
}

impl WorkerNode {
    pub fn new(worker_id: String, controller_url: String, capabilities: WorkerCapabilities) -> Self {
        Self {
            worker_id,
            controller_url,
            capabilities,
            status: Arc::new(RwLock::new(WorkerStatus::Disconnected)),
            active_jobs: Arc::new(RwLock::new(Vec::new())),
            shutdown_tx: None,
        }
    }

    pub async fn start(&mut self) -> Result<(), WorkerError> {
        let registration = WorkerRegistration {
            worker_id: self.worker_id.clone(),
            capabilities: self.capabilities.clone(),
            status: WorkerStatus::Idle,
        };

        let (shutdown_tx, mut shutdown_rx) = mpsc::channel::<()>(1);
        self.shutdown_tx = Some(shutdown_tx);

        let worker_id = self.worker_id.clone();
        let controller_url = self.controller_url.clone();
        let status = self.status.clone();

        tokio::spawn(async move {
            Self::heartbeat_loop(worker_id, controller_url, shutdown_rx).await;
        });

        let mut st = self.status.write().await;
        *st = WorkerStatus::Idle;

        Ok(())
    }

    async fn heartbeat_loop(worker_id: String, controller_url: String, mut shutdown: mpsc::Receiver<()>) {
        let mut ticker = interval(Duration::from_secs(30));

        loop {
            tokio::select! {
                _ = ticker.tick() => {
                    if let Err(e) = Self::send_heartbeat(&worker_id, &controller_url).await {
                        eprintln!("Heartbeat failed: {}", e);
                    }
                }
                _ = shutdown.recv() => {
                    break;
                }
            }
        }
    }

    async fn send_heartbeat(worker_id: &str, controller_url: &str) -> Result<(), WorkerError> {
        Ok(())
    }

    pub async fn claim_job(&self) -> Result<Option<Job>, WorkerError> {
        let active = self.active_jobs.read().await;
        if active.len() >= self.capabilities.max_concurrent {
            return Err(WorkerError::NotIdle);
        }

        // Simulating network fetch from controller
        Ok(None)
    }

    pub async fn complete_job(&self, result: JobResult) -> Result<(), WorkerError> {
        let mut active = self.active_jobs.write().await;
        active.retain(|id| id != &result.job_id);

        if active.is_empty() {
            let mut status = self.status.write().await;
            *status = WorkerStatus::Idle;
        }

        Ok(())
    }

    pub async fn update_status(&self, new_status: WorkerStatus) {
        let mut status = self.status.write().await;
        *status = new_status;
    }

    pub async fn shutdown(&self) {
        if let Some(tx) = &self.shutdown_tx {
            let _ = tx.send(()).await;
        }
    }

    pub fn worker_id(&self) -> &str {
        &self.worker_id
    }

    pub async fn get_status(&self) -> WorkerStatus {
        *self.status.read().await
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkerConfig {
    pub worker_id: String,
    pub controller_url: String,
    pub browser_types: Vec<String>,
    pub max_concurrent: usize,
    pub memory_limit_mb: u64,
    pub cpu_cores: u32,
    pub tags: Vec<String>,
}

impl Default for WorkerConfig {
    fn default() -> Self {
        Self {
            worker_id: uuid::Uuid::new_v4().to_string(),
            controller_url: "http://localhost:8080".to_string(),
            browser_types: vec!["chromium".to_string()],
            max_concurrent: 5,
            memory_limit_mb: 2048,
            cpu_cores: 2,
            tags: vec![],
        }
    }
}

#[derive(Debug)]
pub enum WorkerError {
    NotIdle,
    AlreadyHasJob,
    ControllerUnavailable,
    JobExecutionFailed(String),
}

impl std::fmt::Display for WorkerError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            WorkerError::NotIdle => write!(f, "Worker is not idle"),
            WorkerError::AlreadyHasJob => write!(f, "Worker already has a job"),
            WorkerError::ControllerUnavailable => write!(f, "Controller is unavailable"),
            WorkerError::JobExecutionFailed(s) => write!(f, "Job execution failed: {}", s),
        }
    }
}

impl std::error::Error for WorkerError {}
