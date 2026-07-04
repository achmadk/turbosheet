use std::time::Duration;
use tokio::time::{sleep, Instant};
use crate::error::TurbosheetError;

pub struct AssertionEngine {
    pub timeout: Duration,
    pub initial_interval: Duration,
    pub max_interval: Duration,
}

impl Default for AssertionEngine {
    fn default() -> Self {
        Self {
            timeout: Duration::from_millis(30000), // Configurable globally
            initial_interval: Duration::from_millis(100),
            max_interval: Duration::from_millis(500),
        }
    }
}

impl AssertionEngine {
    fn compute_jitter(interval: Duration) -> Duration {
        use std::time::{SystemTime, UNIX_EPOCH};
        let interval_ms = interval.as_millis() as u64;
        let max_jitter = interval_ms / 10;
        if max_jitter == 0 {
            return Duration::from_millis(0);
        }
        let time = SystemTime::now().duration_since(UNIX_EPOCH).unwrap_or_default().as_micros() as u64;
        Duration::from_millis(time % max_jitter)
    }

    pub async fn poll<F, Fut, T>(&self, condition: F) -> Result<T, TurbosheetError>
    where
        F: FnMut() -> Fut,
        Fut: std::future::Future<Output = Result<T, String>>,
    {
        self.poll_with_timeout(self.timeout, condition).await
    }

    pub async fn poll_with_timeout<F, Fut, T>(&self, custom_timeout: Duration, mut condition: F) -> Result<T, TurbosheetError>
    where
        F: FnMut() -> Fut,
        Fut: std::future::Future<Output = Result<T, String>>,
    {
        let start = Instant::now();
        let mut interval = self.initial_interval;
        let mut last_error: Option<String> = None;
        let mut poll_count: u32 = 0;

        loop {
            poll_count += 1;
            match condition().await {
                Ok(result) => return Ok(result),
                Err(e) => {
                    last_error = Some(e);
                }
            }

            if start.elapsed() >= custom_timeout {
                return Err(TurbosheetError::Other(format!(
                    "Assertion timeout after {}ms ({} polls). {}",
                    custom_timeout.as_millis(),
                    poll_count,
                    last_error.unwrap_or_default()
                )));
            }

            let jitter = Self::compute_jitter(interval);
            sleep(interval + jitter).await;
            interval = std::cmp::min(interval * 2, self.max_interval);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;

    #[tokio::test]
    async fn test_poll_immediate_success() {
        let engine = AssertionEngine {
            timeout: Duration::from_millis(100),
            ..Default::default()
        };

        let result = engine
            .poll(|| async { Ok::<_, String>(42) })
            .await;
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 42);
    }

    #[tokio::test]
    async fn test_poll_with_timeout_error() {
        let engine = AssertionEngine {
            timeout: Duration::from_millis(100),
            initial_interval: Duration::from_millis(10),
            max_interval: Duration::from_millis(50),
        };

        let result = engine
            .poll_with_timeout(Duration::from_millis(100), || async {
                Err::<(), String>("not ready".to_string())
            })
            .await;

        assert!(result.is_err());
        let err = result.unwrap_err().to_string();
        assert!(
            err.contains("timeout"),
            "Expected timeout in error message, got: {}",
            err
        );
    }

    #[tokio::test]
    async fn test_jitter_within_bounds() {
        // compute_jitter is private but accessible in cfg(test) via `use super::*`
        for interval_ms in [1u64, 10, 50, 100, 200, 500] {
            let interval = Duration::from_millis(interval_ms);
            let max_jitter = interval_ms / 10;

            // Run 100 samples to verify the jitter property
            for _ in 0..100 {
                let jitter = AssertionEngine::compute_jitter(interval);
                assert!(
                    jitter.as_millis() as u64 <= max_jitter,
                    "jitter {}ms exceeds 10% of {}ms interval (max {}ms)",
                    jitter.as_millis(),
                    interval_ms,
                    max_jitter
                );
            }
        }
    }

    #[tokio::test]
    async fn test_exponential_backoff() {
        let engine = AssertionEngine {
            timeout: Duration::from_secs(10),
            initial_interval: Duration::from_millis(10),
            max_interval: Duration::from_millis(50),
        };

        let start = Instant::now();
        let mut call_count = 0u32;
        let result = engine
            .poll_with_timeout(Duration::from_millis(500), || {
                call_count += 1;
                async { Err::<(), String>("not yet".to_string()) }
            })
            .await;

        assert!(result.is_err());
        // With 10ms initial + jitter, doubling up to 50ms max, should get ~5-15 polls in 500ms
        assert!(
            call_count >= 3,
            "Expected at least 3 polls with backoff, got {}",
            call_count
        );
    }

    #[tokio::test]
    async fn test_poll_count_in_error() {
        let engine = AssertionEngine {
            timeout: Duration::from_millis(500),
            initial_interval: Duration::from_millis(10),
            max_interval: Duration::from_millis(50),
        };

        let mut call_count = 0u32;
        let result = engine
            .poll_with_timeout(Duration::from_millis(200), || {
                call_count += 1;
                async { Err::<(), String>("fail".to_string()) }
            })
            .await;

        assert!(result.is_err());
        let err = result.unwrap_err().to_string();
        // The error message includes the poll count, e.g. "Assertion timeout after 200ms (5 polls). fail"
        assert!(
            err.contains("polls"),
            "Expected poll count in error message, got: {}",
            err
        );
    }
}
