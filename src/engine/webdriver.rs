use crate::error::TurbosheetError;

pub struct WebDriverClient {
    pub client: reqwest::Client,
    pub base_url: String,
    pub session_id: Option<String>,
}

impl WebDriverClient {
    pub fn new(base_url: String) -> Self {
        Self {
            client: reqwest::Client::new(),
            base_url,
            session_id: None,
        }
    }

    pub async fn create_session(&mut self, capabilities: serde_json::Value) -> Result<String, TurbosheetError> {
        let res = self.client.post(&format!("{}/session", self.base_url))
            .json(&capabilities)
            .send()
            .await
            .map_err(|e| TurbosheetError::LaunchFailed(format!("Session failed: {}", e)))?;
        let data: serde_json::Value = res.json().await.unwrap_or_default();
        let session_id = data["value"]["sessionId"].as_str()
            .or(data["sessionId"].as_str())
            .unwrap_or("")
            .to_string();
        self.session_id = Some(session_id.clone());
        Ok(session_id)
    }

    pub async fn delete_session(&mut self) -> Result<(), TurbosheetError> {
        if let Some(sid) = &self.session_id {
            let _ = self.client.delete(&format!("{}/session/{}", self.base_url, sid)).send().await;
        }
        self.session_id = None;
        Ok(())
    }

    pub async fn execute_command(&self, method: reqwest::Method, path: &str, body: Option<serde_json::Value>) -> Result<serde_json::Value, TurbosheetError> {
        let sid = self.session_id.as_ref().ok_or_else(|| {
            TurbosheetError::Other("No active session".to_string())
        })?;
        let url = format!("{}/session/{}{}", self.base_url, sid, path);
        let req = self.client.request(method.clone(), &url);
        let req = if let Some(b) = body {
            req.json(&b)
        } else {
            req
        };
        let res = req.send().await.map_err(|e| {
            TurbosheetError::Other(format!("WebDriver command failed: {}", e))
        })?;
        let data: serde_json::Value = res.json().await.unwrap_or_default();
        Ok(data)
    }
}

/// BiDi connection (requires tokio-tungstenite feature)
/// Not yet implemented — Firefox console events use polling fallback
pub struct BidiConnection;
