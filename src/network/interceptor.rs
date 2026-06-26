use tokio::sync::broadcast;

#[derive(Clone, Debug)]
pub struct NetworkRequestEvent {
    pub url: String,
    pub method: String,
    pub headers: std::collections::HashMap<String, String>,
    pub post_data: Option<String>,
    pub timestamp: f64,
}

#[derive(Clone, Debug)]
pub struct NetworkResponseEvent {
    pub url: String,
    pub status: u16,
    pub headers: std::collections::HashMap<String, String>,
    pub timestamp: f64,
}

pub struct NetworkEventBus {
    pub request_tx: broadcast::Sender<NetworkRequestEvent>,
    pub response_tx: broadcast::Sender<NetworkResponseEvent>,
}

impl NetworkEventBus {
    pub fn new() -> Self {
        let (request_tx, _) = broadcast::channel(256);
        let (response_tx, _) = broadcast::channel(256);
        Self { request_tx, response_tx }
    }

    pub fn request_subscriber(&self) -> broadcast::Receiver<NetworkRequestEvent> {
        self.request_tx.subscribe()
    }

    pub fn response_subscriber(&self) -> broadcast::Receiver<NetworkResponseEvent> {
        self.response_tx.subscribe()
    }

    pub fn emit_request(&self, event: NetworkRequestEvent) {
        let _ = self.request_tx.send(event);
    }

    pub fn emit_response(&self, event: NetworkResponseEvent) {
        let _ = self.response_tx.send(event);
    }
}
