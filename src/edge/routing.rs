use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct EdgeRouter {
    nodes: HashMap<String, GridNode>,
    config: RoutingConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RoutingConfig {
    pub enable_latency_routing: bool,
    pub latency_check_interval_secs: u64,
    pub max_latency_threshold_ms: u32,
    pub fallback_to_closest: bool,
}

impl Default for RoutingConfig {
    fn default() -> Self {
        Self {
            enable_latency_routing: true,
            latency_check_interval_secs: 30,
            max_latency_threshold_ms: 500,
            fallback_to_closest: true,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GridNode {
    pub id: String,
    pub endpoint: String,
    pub region: String,
    pub load: NodeLoad,
    pub capabilities: NodeCapabilities,
    pub latency_ms: Option<u32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NodeLoad {
    pub active_jobs: usize,
    pub max_jobs: usize,
    pub queue_depth: usize,
    pub cpu_usage: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NodeCapabilities {
    pub browsers: Vec<String>,
    pub max_concurrent: usize,
    pub tags: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct RouteDecision {
    pub node_id: String,
    pub endpoint: String,
    pub reasoning: RouteReasoning,
}

#[derive(Debug, Clone)]
pub enum RouteReasoning {
    LowestLatency { latency_ms: u32 },
    LowestLoad { load_ratio: f32 },
    CapabilityMatch { matched_browsers: usize },
    Fallback { reason: String },
    NoNodesAvailable,
}

impl EdgeRouter {
    pub fn new(config: RoutingConfig) -> Self {
        Self {
            nodes: HashMap::new(),
            config,
        }
    }

    pub fn add_node(&mut self, node: GridNode) {
        self.nodes.insert(node.id.clone(), node);
    }

    pub fn remove_node(&mut self, node_id: &str) {
        self.nodes.remove(node_id);
    }

    pub fn update_node_latency(&mut self, node_id: &str, latency_ms: u32) {
        if let Some(node) = self.nodes.get_mut(node_id) {
            node.latency_ms = Some(latency_ms);
        }
    }

    pub fn update_node_load(&mut self, node_id: &str, load: NodeLoad) {
        if let Some(node) = self.nodes.get_mut(node_id) {
            node.load = load;
        }
    }

    pub fn route(&self, browser: &str, required_tags: &[String]) -> Option<RouteDecision> {
        if self.nodes.is_empty() {
            return None;
        }

        let candidates: Vec<&GridNode> = self.nodes
            .values()
            .filter(|n| self.node_can_run_browser(n, browser))
            .filter(|n| self.node_has_tags(n, required_tags))
            .filter(|n| self.node_has_capacity(n))
            .collect();

        if candidates.is_empty() {
            return None;
        }

        if self.config.enable_latency_routing {
            self.route_by_latency(&candidates)
        } else {
            self.route_by_load(&candidates)
        }
    }

    fn node_can_run_browser(&self, node: &GridNode, browser: &str) -> bool {
        node.capabilities.browsers.iter().any(|b| b == browser)
    }

    fn node_has_tags(&self, node: &GridNode, required_tags: &[String]) -> bool {
        required_tags.is_empty() || required_tags.iter().all(|tag| {
            node.capabilities.tags.contains(tag)
        })
    }

    fn node_has_capacity(&self, node: &GridNode) -> bool {
        node.load.active_jobs < node.load.max_jobs
    }

    fn route_by_latency(&self, candidates: &[&GridNode]) -> Option<RouteDecision> {
        let mut by_latency: Vec<&GridNode> = candidates
            .iter()
            .filter(|n| n.latency_ms.is_some())
            .cloned()
            .collect();

        by_latency.sort_by_key(|n| n.latency_ms.unwrap_or(u32::MAX));

        if let Some(node) = by_latency.first() {
            let latency = node.latency_ms.unwrap_or(0);
            if latency <= self.config.max_latency_threshold_ms {
                return Some(RouteDecision {
                    node_id: node.id.clone(),
                    endpoint: node.endpoint.clone(),
                    reasoning: RouteReasoning::LowestLatency { latency_ms: latency },
                });
            }
        }

        self.config.fallback_to_closest.then(|| {
            by_latency.first().map(|node| {
                RouteDecision {
                    node_id: node.id.clone(),
                    endpoint: node.endpoint.clone(),
                    reasoning: RouteReasoning::LowestLatency {
                        latency_ms: node.latency_ms.unwrap_or(0)
                    },
                }
            })
        }).flatten()
    }

    fn route_by_load(&self, candidates: &[&GridNode]) -> Option<RouteDecision> {
        let mut by_load: Vec<&GridNode> = candidates.to_vec();
        by_load.sort_by(|a, b| {
            let load_a = a.load.active_jobs as f32 / a.load.max_jobs as f32;
            let load_b = b.load.active_jobs as f32 / b.load.max_jobs as f32;
            load_a.partial_cmp(&load_b).unwrap_or(std::cmp::Ordering::Equal)
        });

        by_load.first().map(|node| {
            RouteDecision {
                node_id: node.id.clone(),
                endpoint: node.endpoint.clone(),
                reasoning: RouteReasoning::LowestLoad {
                    load_ratio: node.load.active_jobs as f32 / node.load.max_jobs as f32
                },
            }
        })
    }

    pub fn get_closest_node(&self) -> Option<RouteDecision> {
        self.nodes.values()
            .filter(|n| n.latency_ms.is_some())
            .min_by_key(|n| n.latency_ms.unwrap_or(u32::MAX))
            .map(|node| RouteDecision {
                node_id: node.id.clone(),
                endpoint: node.endpoint.clone(),
                reasoning: RouteReasoning::LowestLatency {
                    latency_ms: node.latency_ms.unwrap_or(0)
                },
            })
    }

    pub fn get_least_loaded_node(&self) -> Option<RouteDecision> {
        self.nodes.values()
            .min_by(|a, b| {
                let load_a = a.load.active_jobs as f32 / a.load.max_jobs as f32;
                let load_b = b.load.active_jobs as f32 / b.load.max_jobs as f32;
                load_a.partial_cmp(&load_b).unwrap_or(std::cmp::Ordering::Equal)
            })
            .map(|node| RouteDecision {
                node_id: node.id.clone(),
                endpoint: node.endpoint.clone(),
                reasoning: RouteReasoning::LowestLoad {
                    load_ratio: node.load.active_jobs as f32 / node.load.max_jobs as f32
                },
            })
    }

    pub fn get_routing_metrics(&self) -> RoutingMetrics {
        let total_nodes = self.nodes.len();
        let nodes_with_latency = self.nodes.values().filter(|n| n.latency_ms.is_some()).count();
        let avg_latency = self.nodes.values()
            .filter_map(|n| n.latency_ms)
            .reduce(|a, b| a + b)
            .map(|sum| sum / nodes_with_latency as u32);

        RoutingMetrics {
            total_nodes,
            available_nodes: self.nodes.values().filter(|n| self.node_has_capacity(n)).count(),
            nodes_with_latency,
            average_latency_ms: avg_latency,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RoutingMetrics {
    pub total_nodes: usize,
    pub available_nodes: usize,
    pub nodes_with_latency: usize,
    pub average_latency_ms: Option<u32>,
}