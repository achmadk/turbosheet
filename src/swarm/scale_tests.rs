#[cfg(test)]
mod tests {
    use crate::swarm::{GridController, GridConfig, WorkerRegistration, WorkerCapabilities, Job, JobResult, JobStatus};
    use crate::swarm::controller::WorkerStatus;
    use std::time::Duration;

    #[tokio::test]
    async fn test_1000_concurrent_contexts() {
        let config = GridConfig {
            controller_host: "127.0.0.1".to_string(),
            controller_port: 8080,
            max_workers: 200, // Handle up to 200 workers for scaling
            idle_timeout_secs: 300,
            health_check_interval_secs: 30,
            job_timeout_secs: 600,
        };
        
        let controller = GridController::new(config);

        // Register 100 workers, each with max_concurrent = 10 (Total capacity = 1000 concurrent)
        for i in 0..100 {
            let worker_id = format!("worker_{}", i);
            let registration = WorkerRegistration {
                worker_id,
                capabilities: WorkerCapabilities {
                    browser_types: vec!["chromium".to_string()],
                    max_concurrent: 10,
                    memory_limit_mb: 2048,
                    cpu_cores: 4,
                    tags: vec![],
                },
                status: WorkerStatus::Idle,
            };
            controller.register_worker(registration).await.unwrap();
        }

        assert_eq!(controller.get_worker_stats().await.total_workers, 100);

        // Submit 1000 jobs
        for i in 0..1000 {
            let job = Job {
                id: format!("job_{}", i),
                test_file: "huge_suite.spec.ts".to_string(),
                browser: "chromium".to_string(),
                priority: 5,
                timeout_secs: 30,
                created_at: std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_millis() as u64,
                metadata: crate::swarm::job::JobMetadata::default(),
            };
            controller.submit_job(job).await.unwrap();
        }

        assert_eq!(controller.get_queue_depth().await, 1000);

        // Simulate 1000 context claims
        let mut claims = 0;
        for i in 0..100 {
            let worker_id = format!("worker_{}", i);
            for _ in 0..10 { // Each worker claims its max capacity
                if let Some(job) = controller.claim_next_job(&worker_id).await {
                    assert!(job.id.starts_with("job_"));
                    claims += 1;
                }
            }
            
            // 11th claim should fail because capacity is maxed out
            let failed_claim = controller.claim_next_job(&worker_id).await;
            assert!(failed_claim.is_none());
        }

        assert_eq!(claims, 1000);
        assert_eq!(controller.get_queue_depth().await, 0);

        let stats = controller.get_worker_stats().await;
        assert_eq!(stats.idle_workers, 0);
        assert_eq!(stats.busy_workers, 100);
        assert_eq!(stats.running_jobs, 1000);

        // Complete all jobs
        for i in 0..100 {
            let worker_id = format!("worker_{}", i);
            for j in 0..10 {
                let job_id = format!("job_{}", i * 10 + j);
                let result = crate::swarm::job::JobResult::success(
                    job_id,
                    worker_id.clone(),
                    100
                );
                controller.complete_job(result).await;
            }
        }

        let final_stats = controller.get_worker_stats().await;
        assert_eq!(final_stats.running_jobs, 0);
        assert_eq!(final_stats.completed_jobs, 1000);
        assert_eq!(final_stats.passed_jobs, 1000);
        assert_eq!(final_stats.idle_workers, 100);
    }
}
