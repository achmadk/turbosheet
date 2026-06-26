use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::RwLock;
use tokio::time::{interval, Duration};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AutoscalingConfig {
    pub enabled: bool,
    pub min_workers: usize,
    pub max_workers: usize,
    pub scale_up_threshold: usize,
    pub scale_down_threshold: usize,
    pub scale_up_cooldown_secs: u64,
    pub scale_down_cooldown_secs: u64,
    pub scale_factor: usize,
}

impl Default for AutoscalingConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            min_workers: 1,
            max_workers: 100,
            scale_up_threshold: 10,
            scale_down_threshold: 3,
            scale_up_cooldown_secs: 60,
            scale_down_cooldown_secs: 300,
            scale_factor: 2,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ScalingAction {
    ScaleUp(usize),
    ScaleDown(usize),
    NoChange,
}

pub struct AutoScaler {
    config: AutoscalingConfig,
    last_scale_action: Arc<RwLock<ScalingAction>>,
    last_scale_time: Arc<RwLock<u64>>,
    current_workers: Arc<RwLock<usize>>,
}

impl AutoScaler {
    pub fn new(config: AutoscalingConfig, initial_workers: usize) -> Self {
        Self {
            config,
            last_scale_action: Arc::new(RwLock::new(ScalingAction::NoChange)),
            last_scale_time: Arc::new(RwLock::new(now_ms())),
            current_workers: Arc::new(RwLock::new(initial_workers)),
        }
    }

    pub async fn evaluate(&self, queue_depth: usize, idle_workers: usize) -> ScalingAction {
        if !self.config.enabled {
            return ScalingAction::NoChange;
        }

        let last_action = *self.last_scale_action.read().await;
        let last_time = *self.last_scale_time.read().await;
        let current_time = now_ms();
        let cooldown = match last_action {
            ScalingAction::ScaleUp(_) => self.config.scale_up_cooldown_secs * 1000,
            ScalingAction::ScaleDown(_) => self.config.scale_down_cooldown_secs * 1000,
            ScalingAction::NoChange => 0,
        };

        if current_time - last_time < cooldown {
            return ScalingAction::NoChange;
        }

        let current = *self.current_workers.read().await;

        if queue_depth >= self.config.scale_up_threshold && idle_workers == 0 {
            let scale_amount = (self.config.scale_factor)
                .min(self.config.max_workers.saturating_sub(current));
            if scale_amount > 0 {
                return ScalingAction::ScaleUp(scale_amount);
            }
        }

        if queue_depth <= self.config.scale_down_threshold && idle_workers > self.config.scale_down_threshold {
            let scale_amount = (self.config.scale_factor)
                .min(current.saturating_sub(self.config.min_workers));
            if scale_amount > 0 {
                return ScalingAction::ScaleDown(scale_amount);
            }
        }

        ScalingAction::NoChange
    }

    pub async fn record_scale_action(&self, action: ScalingAction) {
        let mut last_action = self.last_scale_action.write().await;
        let mut last_time = self.last_scale_time.write().await;
        *last_action = action;
        *last_time = now_ms();

        match action {
            ScalingAction::ScaleUp(n) => {
                let mut current = self.current_workers.write().await;
                *current = (*current + n).min(self.config.max_workers);
            }
            ScalingAction::ScaleDown(n) => {
                let mut current = self.current_workers.write().await;
                *current = current.saturating_sub(n).max(self.config.min_workers);
            }
            ScalingAction::NoChange => {}
        }
    }

    pub async fn get_current_workers(&self) -> usize {
        *self.current_workers.read().await
    }
}

pub struct AutoscalerRunner {
    auto_scaler: Arc<AutoScaler>,
    controller: Arc<dyn AutoscalerController>,
}

#[async_trait::async_trait]
pub trait AutoscalerController: Send + Sync {
    async fn get_queue_depth(&self) -> usize;
    async fn get_idle_workers(&self) -> usize;
    async fn request_worker_scale(&self, count: usize) -> Result<(), AutoscalerError>;
}

#[derive(Debug)]
pub enum AutoscalerError {
    ScalingFailed(String),
    MaxWorkersReached,
}

impl std::fmt::Display for AutoscalerError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AutoscalerError::ScalingFailed(s) => write!(f, "Scaling failed: {}", s),
            AutoscalerError::MaxWorkersReached => write!(f, "Maximum workers reached"),
        }
    }
}

impl std::error::Error for AutoscalerError {}

impl AutoScaler {
    pub fn start_monitor<C: AutoscalerController + 'static>(
        &self,
        controller: Arc<C>,
    ) {
        let auto_scaler = Arc::new(self.clone());
        let ctrl = controller;

        tokio::spawn(async move {
            let mut ticker = interval(Duration::from_secs(10));
            loop {
                ticker.tick().await;
                let queue_depth = ctrl.get_queue_depth().await;
                let idle_workers = ctrl.get_idle_workers().await;

                let action = auto_scaler.evaluate(queue_depth, idle_workers).await;

                match action {
                    ScalingAction::ScaleUp(n) => {
                        if let Err(e) = ctrl.request_worker_scale(n).await {
                            eprintln!("Failed to scale up: {}", e);
                        } else {
                            auto_scaler.record_scale_action(ScalingAction::ScaleUp(n)).await;
                            println!("Autoscaler: scaled up by {} workers", n);
                        }
                    }
                    ScalingAction::ScaleDown(n) => {
                        auto_scaler.record_scale_action(ScalingAction::ScaleDown(n)).await;
                        println!("Autoscaler: scaled down by {} workers", n);
                    }
                    ScalingAction::NoChange => {}
                }
            }
        });
    }
}

impl Clone for AutoScaler {
    fn clone(&self) -> Self {
        Self {
            config: self.config.clone(),
            last_scale_action: self.last_scale_action.clone(),
            last_scale_time: self.last_scale_time.clone(),
            current_workers: self.current_workers.clone(),
        }
    }
}

fn now_ms() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis() as u64
}