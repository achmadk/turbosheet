use std::sync::Arc;
use crate::swarm::autoscaler::AutoscalerController;

pub struct MetricsServer {
    controller: Arc<dyn AutoscalerController>,
    port: u16,
}

impl MetricsServer {
    pub fn new(controller: Arc<dyn AutoscalerController>, port: u16) -> Self {
        Self { controller, port }
    }

    pub async fn start(&self) {
        println!("Swarm Metrics server disabled due to hyper 1.0 upgrade");
    }
}
