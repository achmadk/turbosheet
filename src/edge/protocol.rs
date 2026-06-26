use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EdgeMessage {
    pub version: u8,
    pub msg_type: MessageType,
    pub request_id: u64,
    pub payload: Vec<u8>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum MessageType {
    Ping = 0x01,
    Pong = 0x02,
    SubmitJob = 0x10,
    JobSubmitted = 0x11,
    ClaimJob = 0x12,
    JobClaimed = 0x13,
    JobResult = 0x14,
    GridCommand = 0x20,
    CommandResponse = 0x21,
    AssertionCheck = 0x30,
    AssertionResult = 0x31,
    Error = 0xFF,
}

impl EdgeMessage {
    pub fn new(msg_type: MessageType, request_id: u64, payload: Vec<u8>) -> Self {
        Self {
            version: 1,
            msg_type,
            request_id,
            payload,
        }
    }

    pub fn ping() -> Self {
        Self::new(MessageType::Ping, 0, Vec::new())
    }

    pub fn pong() -> Self {
        Self::new(MessageType::Pong, 0, Vec::new())
    }

    pub fn submit_job(job: &EdgeJobPayload) -> Result<Self, ProtocolError> {
        let payload = serde_json::to_vec(job).map_err(|_| ProtocolError::SerializationError)?;
        Ok(Self::new(MessageType::SubmitJob, next_request_id(), payload))
    }

    pub fn claim_job(worker_id: &str) -> Self {
        Self::new(MessageType::ClaimJob, next_request_id(), worker_id.as_bytes().to_vec())
    }

    pub fn job_result(result: &JobResultPayload) -> Result<Self, ProtocolError> {
        let payload = serde_json::to_vec(result).map_err(|_| ProtocolError::SerializationError)?;
        Ok(Self::new(MessageType::JobResult, 0, payload))
    }

    pub fn grid_command(cmd: &GridCommand) -> Result<Self, ProtocolError> {
        let payload = serde_json::to_vec(cmd).map_err(|_| ProtocolError::SerializationError)?;
        Ok(Self::new(MessageType::GridCommand, next_request_id(), payload))
    }

    pub fn assertion_check(assertion: &AssertionCheckPayload) -> Result<Self, ProtocolError> {
        let payload = serde_json::to_vec(assertion).map_err(|_| ProtocolError::SerializationError)?;
        Ok(Self::new(MessageType::AssertionCheck, next_request_id(), payload))
    }

    pub fn encode(&self) -> Vec<u8> {
        let mut bytes = Vec::new();
        bytes.push(self.version);
        bytes.push(self.msg_type as u8);
        bytes.extend_from_slice(&self.request_id.to_be_bytes());

        let len = self.payload.len() as u32;
        bytes.extend_from_slice(&len.to_be_bytes());
        bytes.extend_from_slice(&self.payload);

        bytes
    }

    pub fn decode(bytes: &[u8]) -> Result<Self, ProtocolError> {
        if bytes.len() < 10 {
            return Err(ProtocolError::InvalidMessage);
        }

        let version = bytes[0];
        let msg_type = match bytes[1] {
            0x01 => MessageType::Ping,
            0x02 => MessageType::Pong,
            0x10 => MessageType::SubmitJob,
            0x11 => MessageType::JobSubmitted,
            0x12 => MessageType::ClaimJob,
            0x13 => MessageType::JobClaimed,
            0x14 => MessageType::JobResult,
            0x20 => MessageType::GridCommand,
            0x21 => MessageType::CommandResponse,
            0x30 => MessageType::AssertionCheck,
            0x31 => MessageType::AssertionResult,
            0xFF => MessageType::Error,
            _ => return Err(ProtocolError::UnknownMessageType),
        };

        let request_id = u64::from_be_bytes([
            bytes[2], bytes[3], bytes[4], bytes[5], bytes[6], bytes[7], bytes[8], bytes[9]
        ]);

        let payload_len = u32::from_be_bytes([bytes[10], bytes[11], bytes[12], bytes[13]]) as usize;

        if bytes.len() < 14 + payload_len {
            return Err(ProtocolError::InvalidMessage);
        }

        let payload = bytes[14..14 + payload_len].to_vec();

        Ok(Self {
            version,
            msg_type,
            request_id,
            payload,
        })
    }
}

use std::sync::atomic::{AtomicU64, Ordering};
static REQUEST_COUNTER: AtomicU64 = AtomicU64::new(1);

fn next_request_id() -> u64 {
    REQUEST_COUNTER.fetch_add(1, Ordering::Relaxed)
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EdgeJobPayload {
    pub id: String,
    pub test_file: String,
    pub browser: String,
    pub priority: u8,
    pub timeout_secs: u64,
    pub tenant_id: Option<String>,
    pub assertions: Vec<AssertionSpecPayload>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AssertionSpecPayload {
    pub kind: String,
    pub target: String,
    pub expected: String,
    pub timeout_ms: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JobResultPayload {
    pub job_id: String,
    pub status: String,
    pub duration_ms: u64,
    pub error: Option<String>,
    pub assertions_passed: u32,
    pub assertions_failed: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GridCommand {
    pub command_type: GridCommandType,
    pub params: CommandParams,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum GridCommandType {
    Navigate,
    Click,
    Fill,
    Select,
    Hover,
    Screenshot,
    Evaluate,
    GetAttribute,
    GetText,
    WaitForSelector,
    WaitForNavigation,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommandParams {
    pub selector: Option<String>,
    pub url: Option<String>,
    pub value: Option<String>,
    pub timeout_ms: Option<u64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GridResponse {
    pub request_id: u64,
    pub success: bool,
    pub data: Option<ResponseData>,
    pub error: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ResponseData {
    String(String),
    Bool(bool),
    Number(f64),
    ElementHandle(String),
    None,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AssertionCheckPayload {
    pub assertion_id: String,
    pub job_id: String,
    pub kind: String,
    pub target: String,
    pub expected: String,
    pub timeout_ms: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AssertionResultPayload {
    pub assertion_id: String,
    pub passed: bool,
    pub actual: Option<String>,
    pub error: Option<String>,
    pub duration_ms: u64,
}

#[derive(Debug)]
pub enum ProtocolError {
    InvalidMessage,
    UnknownMessageType,
    SerializationError,
    DeserializationError,
}

impl std::fmt::Display for ProtocolError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ProtocolError::InvalidMessage => write!(f, "Invalid message format"),
            ProtocolError::UnknownMessageType => write!(f, "Unknown message type"),
            ProtocolError::SerializationError => write!(f, "Failed to serialize data"),
            ProtocolError::DeserializationError => write!(f, "Failed to deserialize data"),
        }
    }
}

impl std::error::Error for ProtocolError {}

pub struct MessageBuilder;

impl MessageBuilder {
    pub fn build_ping() -> EdgeMessage {
        EdgeMessage::ping()
    }

    pub fn build_pong() -> EdgeMessage {
        EdgeMessage::pong()
    }

    pub fn build_navigate(url: &str, timeout_ms: u64) -> Result<GridCommand, ProtocolError> {
        Ok(GridCommand {
            command_type: GridCommandType::Navigate,
            params: CommandParams {
                selector: None,
                url: Some(url.to_string()),
                value: None,
                timeout_ms: Some(timeout_ms),
            },
        })
    }

    pub fn build_click(selector: &str, timeout_ms: u64) -> Result<GridCommand, ProtocolError> {
        Ok(GridCommand {
            command_type: GridCommandType::Click,
            params: CommandParams {
                selector: Some(selector.to_string()),
                url: None,
                value: None,
                timeout_ms: Some(timeout_ms),
            },
        })
    }

    pub fn build_fill(selector: &str, value: &str, timeout_ms: u64) -> Result<GridCommand, ProtocolError> {
        Ok(GridCommand {
            command_type: GridCommandType::Fill,
            params: CommandParams {
                selector: Some(selector.to_string()),
                url: None,
                value: Some(value.to_string()),
                timeout_ms: Some(timeout_ms),
            },
        })
    }
}