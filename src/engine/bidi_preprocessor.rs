use serde_json::Value;
use std::time::{Duration, Instant};
use tokio::sync::mpsc;

pub struct BidiPreprocessor {
    batch_window: Duration,
    last_flush: Instant,
    buffer: Vec<Value>,
    sender: mpsc::Sender<Vec<Value>>,
}

impl BidiPreprocessor {
    pub fn new(batch_window_ms: u64, sender: mpsc::Sender<Vec<Value>>) -> Self {
        Self {
            batch_window: Duration::from_millis(batch_window_ms),
            last_flush: Instant::now(),
            buffer: Vec::new(),
            sender,
        }
    }

    pub async fn process_event(&mut self, event: Value) {
        if let Some(method) = event.get("method").and_then(|m| m.as_str()) {
            if method == "input.mouseMove" || method == "input.scroll" || method == "dom.mutation" {
                return;
            }
        }

        self.buffer.push(event);

        if self.last_flush.elapsed() >= self.batch_window {
            self.flush().await;
        }
    }

    pub async fn flush(&mut self) {
        if !self.buffer.is_empty() {
            let batch = std::mem::take(&mut self.buffer);
            let _ = self.sender.send(batch).await;
            self.last_flush = Instant::now();
        }
    }
}
