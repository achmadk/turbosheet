use std::sync::Arc;
use tokio::sync::Mutex;
use tokio_tungstenite::{connect_async, MaybeTlsStream, WebSocketStream};
use tokio_tungstenite::tungstenite::Message;
use futures::{SinkExt, StreamExt};
use std::time::Duration;

pub struct BidiTransport {
    ws: Option<WebSocketStream<MaybeTlsStream<tokio::net::TcpStream>>>,
    url: String,
    closed: bool,
}

impl BidiTransport {
    /// Connect to a BiDi WebSocket endpoint.
    pub async fn connect(url: &str) -> Result<Self, crate::error::TurbosheetError> {
        let (ws, _) = connect_async(url)
            .await
            .map_err(|e| crate::error::TurbosheetError::Other(format!("BiDi WS connect failed: {}", e)))?;
        Ok(Self {
            ws: Some(ws),
            url: url.to_string(),
            closed: false,
        })
    }

    /// Send a raw JSON string message.
    pub async fn send(&mut self, msg: &str) -> Result<(), crate::error::TurbosheetError> {
        let ws = self.ws.as_mut().ok_or_else(|| {
            crate::error::TurbosheetError::Other("BiDi transport not connected".into())
        })?;
        ws.send(Message::Text(msg.to_string()))
            .await
            .map_err(|e| crate::error::TurbosheetError::Other(format!("BiDi WS send failed: {}", e)))
    }

    /// Receive the next message (JSON text only).
    pub async fn recv(&mut self) -> Result<String, crate::error::TurbosheetError> {
        let ws = self.ws.as_mut().ok_or_else(|| {
            crate::error::TurbosheetError::Other("BiDi transport not connected".into())
        })?;
        loop {
            match ws.next().await {
                Some(Ok(Message::Text(text))) => return Ok(text),
                Some(Ok(Message::Ping(data))) => {
                    if let Err(e) = ws.send(Message::Pong(data)).await {
                        return Err(crate::error::TurbosheetError::Other(
                            format!("BiDi WS pong failed: {}", e),
                        ));
                    }
                }
                Some(Ok(Message::Pong(_))) => {}
                Some(Ok(Message::Close(_))) => {
                    return Err(crate::error::TurbosheetError::Other(
                        "BiDi WS connection closed".into(),
                    ));
                }
                Some(Ok(Message::Binary(_))) => {}
                Some(Ok(Message::Frame(_))) => {}
                Some(Err(e)) => {
                    return Err(crate::error::TurbosheetError::Other(
                        format!("BiDi WS error: {}", e),
                    ));
                }
                None => {
                    return Err(crate::error::TurbosheetError::Other(
                        "BiDi WS stream ended".into(),
                    ));
                }
            }
        }
    }

    /// Attempt reconnection with exponential backoff.
    pub async fn reconnect(&mut self) -> Result<(), crate::error::TurbosheetError> {
        self.close().await;
        let mut delay = Duration::from_millis(100);
        let max_delay = Duration::from_secs(10);
        loop {
            match connect_async(&self.url).await {
                Ok((ws, _)) => {
                    self.ws = Some(ws);
                    return Ok(());
                }
                Err(_) => {
                    tokio::time::sleep(delay).await;
                    delay = std::cmp::min(delay * 2, max_delay);
                }
            }
        }
    }

    /// Close the transport.
    pub async fn close(&mut self) {
        self.closed = true;
        if let Some(mut ws) = self.ws.take() {
            let _ = ws.close(None).await;
        }
    }

    /// Whether the transport was explicitly closed.
    pub fn is_closed(&self) -> bool {
        self.closed
    }
}

/// Thread-safe wrapper around `BidiTransport`.
pub struct SharedTransport {
    inner: Arc<Mutex<BidiTransport>>,
}

impl SharedTransport {
    pub fn new(transport: BidiTransport) -> Self {
        Self {
            inner: Arc::new(Mutex::new(transport)),
        }
    }

    pub async fn send(&self, msg: &str) -> Result<(), crate::error::TurbosheetError> {
        self.inner.lock().await.send(msg).await
    }

    pub async fn recv(&self) -> Result<String, crate::error::TurbosheetError> {
        self.inner.lock().await.recv().await
    }

    pub async fn reconnect(&self) -> Result<(), crate::error::TurbosheetError> {
        self.inner.lock().await.reconnect().await
    }

    pub async fn close(&self) {
        self.inner.lock().await.close().await
    }

    pub fn clone_inner(&self) -> Arc<Mutex<BidiTransport>> {
        self.inner.clone()
    }
}
