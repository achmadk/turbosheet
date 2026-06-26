use serde::{Deserialize, Serialize};
use std::collections::{HashMap, VecDeque};
use std::sync::Arc;
use tokio::sync::RwLock;
use tokio::time::{interval, Duration};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthMonitorConfig {
    pub enabled: bool,
    pub health_check_interval_secs: u64,
    pub missed_heartbeats_threshold: u32,
    pub stale_job_timeout_secs: u64,
    pub dead_worker_cleanup_interval_secs: u64,
}

impl Default for HealthMonitorConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            health_check_interval_secs: 30,
            missed_heartbeats_threshold: 3,
            stale_job_timeout_secs: 600,
            dead_worker_cleanup_interval_secs: 60,
        }
    }
}

#[derive(Debug, Clone)]
pub struct WorkerHealth {
    pub worker_id: String,
    pub last_heartbeat: u64,
    pub missed_heartbeats: u32,
    pub consecutive_failures: u32,
    pub is_healthy: bool,
    pub current_job_id: Option<String>,
    pub job_started_at: Option<u64>,
}

impl WorkerHealth {
    pub fn new(worker_id: String) -> Self {
        Self {
            worker_id,
            last_heartbeat: now_ms(),
            missed_heartbeats: 0,
            consecutive_failures: 0,
            is_healthy: true,
            current_job_id: None,
            job_started_at: None,
        }
    }

    pub fn record_heartbeat(&mut self) {
        self.last_heartbeat = now_ms();
        self.missed_heartbeats = 0;
    }

    pub fn record_missed_heartbeat(&mut self) {
        self.missed_heartbeats += 1;
    }

    pub fn record_job_started(&mut self, job_id: String) {
        self.current_job_id = Some(job_id);
        self.job_started_at = Some(now_ms());
    }

    pub fn record_job_completed(&mut self) {
        self.current_job_id = None;
        self.job_started_at = None;
        self.consecutive_failures = 0;
    }

    pub fn record_job_failure(&mut self) {
        self.consecutive_failures += 1;
    }

    pub fn is_dead(&self, threshold: u32) -> bool {
        self.missed_heartbeats >= threshold
    }

    pub fn has_stale_job(&self, timeout_secs: u64) -> bool {
        if let (Some(_), Some(started)) = (&self.current_job_id, self.job_started_at) {
            let elapsed = (now_ms() - started) / 1000;
            return elapsed >= timeout_secs;
        }
        false
    }
}

pub struct HealthMonitor {
    config: HealthMonitorConfig,
    workers: Arc<RwLock<HashMap<String, WorkerHealth>>>,
    dead_job_queue: Arc<RwLock<VecDeque<ReassignableJob>>>,
}

#[derive(Debug, Clone)]
pub struct ReassignableJob {
    pub job_id: String,
    pub original_worker_id: String,
    pub test_file: String,
    pub browser: String,
    pub priority: u8,
    pub timeout_secs: u64,
    pub failed_attempts: u32,
    pub created_at: u64,
}

#[derive(Debug, Clone)]
pub enum HealthEvent {
    WorkerBecameUnhealthy { worker_id: String },
    WorkerBecameHealthy { worker_id: String },
    WorkerDead { worker_id: String },
    JobStale { job_id: String, worker_id: String },
    JobReassigned { job_id: String, from: String, to: Option<String> },
}

impl HealthMonitor {
    pub fn new(config: HealthMonitorConfig) -> Self {
        Self {
            config,
            workers: Arc::new(RwLock::new(HashMap::new())),
            dead_job_queue: Arc::new(RwLock::new(VecDeque::new())),
        }
    }

    pub async fn register_worker(&self, worker_id: String) {
        let mut workers = self.workers.write().await;
        workers.insert(worker_id.clone(), WorkerHealth::new(worker_id));
    }

    pub async fn unregister_worker(&self, worker_id: &str) {
        let mut workers = self.workers.write().await;
        if let Some(health) = workers.remove(worker_id) {
            if let Some(job_id) = health.current_job_id {
                drop(workers);
                self.mark_job_for_reassignment(job_id, worker_id).await;
            }
        }
    }

    pub async fn record_heartbeat(&self, worker_id: &str) {
        let mut workers = self.workers.write().await;
        if let Some(health) = workers.get_mut(worker_id) {
            health.record_heartbeat();
            if !health.is_healthy {
                health.is_healthy = true;
            }
        }
    }

    pub async fn record_job_started(&self, worker_id: &str, job_id: String) {
        let mut workers = self.workers.write().await;
        if let Some(health) = workers.get_mut(worker_id) {
            health.record_job_started(job_id);
        }
    }

    pub async fn record_job_completed(&self, worker_id: &str) {
        let mut workers = self.workers.write().await;
        if let Some(health) = workers.get_mut(worker_id) {
            health.record_job_completed();
        }
    }

    pub async fn record_job_failed(&self, worker_id: &str) {
        let mut workers = self.workers.write().await;
        if let Some(health) = workers.get_mut(worker_id) {
            health.record_job_failure();
        }
    }

    pub async fn mark_job_for_reassignment(&self, job_id: String, original_worker: &str) {
        let dead_job = ReassignableJob {
            job_id,
            original_worker_id: original_worker.to_string(),
            test_file: String::new(),
            browser: "chromium".to_string(),
            priority: 5,
            timeout_secs: 300,
            failed_attempts: 1,
            created_at: now_ms(),
        };
        let mut queue = self.dead_job_queue.write().await;
        queue.push_back(dead_job);
    }

    pub async fn get_next_reassignable_job(&self) -> Option<ReassignableJob> {
        let mut queue = self.dead_job_queue.write().await;
        queue.pop_front()
    }

    pub async fn get_dead_workers(&self) -> Vec<String> {
        let workers = self.workers.read().await;
        workers
            .iter()
            .filter(|(_, h)| h.is_dead(self.config.missed_heartbeats_threshold))
            .map(|(id, _)| id.clone())
            .collect()
    }

    pub async fn get_stale_jobs(&self) -> Vec<(String, String)> {
        let workers = self.workers.read().await;
        workers
            .iter()
            .filter(|(_, h)| h.has_stale_job(self.config.stale_job_timeout_secs))
            .filter_map(|(id, h)| {
                h.current_job_id
                    .as_ref()
                    .map(|jid| (jid.clone(), id.clone()))
            })
            .collect()
    }

    pub async fn check_health(&self) -> Vec<HealthEvent> {
        let mut events = Vec::new();
        let threshold = self.config.missed_heartbeats_threshold;
        let timeout = self.config.stale_job_timeout_secs;
        
        let mut to_reassign = Vec::new();

        {
            let mut workers = self.workers.write().await;

            for (worker_id, health) in workers.iter_mut() {
                if !health.is_healthy && health.missed_heartbeats < threshold {
                    health.is_healthy = true;
                    events.push(HealthEvent::WorkerBecameHealthy {
                        worker_id: worker_id.clone(),
                    });
                }

                if health.is_dead(threshold) && health.is_healthy {
                    health.is_healthy = false;
                    events.push(HealthEvent::WorkerDead {
                        worker_id: worker_id.clone(),
                    });

                    if let Some(job_id) = health.current_job_id.clone() {
                        to_reassign.push((job_id.clone(), worker_id.clone()));
                        events.push(HealthEvent::JobStale {
                            job_id,
                            worker_id: worker_id.clone(),
                        });
                    }
                } else if health.has_stale_job(timeout) {
                    if let Some(job_id) = health.current_job_id.clone() {
                        events.push(HealthEvent::JobStale {
                            job_id,
                            worker_id: worker_id.clone(),
                        });
                    }
                }
            }
        }
        
        for (job_id, worker_id) in to_reassign {
            self.mark_job_for_reassignment(job_id, &worker_id).await;
        }

        events
    }

    pub async fn start_monitoring(&self) {
        if !self.config.enabled {
            return;
        }

        let workers = self.workers.clone();
        let config = self.config.clone();
        let dead_job_queue = self.dead_job_queue.clone();

        tokio::spawn(async move {
            let mut ticker = interval(Duration::from_secs(config.health_check_interval_secs));

            loop {
                ticker.tick().await;

                let mut workers_guard = workers.write().await;

                for (worker_id, health) in workers_guard.iter_mut() {
                    let elapsed = (now_ms() - health.last_heartbeat) / 1000;
                    let interval_ms = config.health_check_interval_secs * 1000;

                    if elapsed >= interval_ms {
                        health.record_missed_heartbeat();
                    }
                }

                drop(workers_guard);

                let dead_workers = {
                    let workers = workers.read().await;
                    workers
                        .iter()
                        .filter(|(_, h)| h.is_dead(config.missed_heartbeats_threshold))
                        .filter(|(_, h)| h.current_job_id.is_some())
                        .map(|(id, h)| (id.clone(), h.current_job_id.clone().unwrap()))
                        .collect::<Vec<_>>()
                };

                for (worker_id, job_id) in dead_workers {
                    let mut queue = dead_job_queue.write().await;
                    queue.push_back(ReassignableJob {
                        job_id,
                        original_worker_id: worker_id,
                        test_file: String::new(),
                        browser: "chromium".to_string(),
                        priority: 5,
                        timeout_secs: 300,
                        failed_attempts: 1,
                        created_at: now_ms(),
                    });
                }
            }
        });
    }

    pub async fn get_health_stats(&self) -> HealthStats {
        let workers = self.workers.read().await;
        let total = workers.len();
        let healthy = workers.values().filter(|h| h.is_healthy).count();
        let unhealthy = total - healthy;
        let queue = self.dead_job_queue.read().await;
        let pending_reassign = queue.len();

        HealthStats {
            total_workers: total,
            healthy_workers: healthy,
            unhealthy_workers: unhealthy,
            pending_reassign_jobs: pending_reassign,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthStats {
    pub total_workers: usize,
    pub healthy_workers: usize,
    pub unhealthy_workers: usize,
    pub pending_reassign_jobs: usize,
}

fn now_ms() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis() as u64
}