
/// Unified error type for TurboSheet operations.
#[derive(Debug, thiserror::Error)]
pub enum TurbosheetError {
    #[error("Browser launch failed: {0}")]
    LaunchFailed(String),

    #[error("Navigation failed: {0}")]
    NavigationFailed(String),

    #[error("Timeout after {0}ms")]
    Timeout(u64),

    #[error("Element not found: {message}")]
    ElementNotFound { message: String, selector: Option<String> },

    #[error("Element not visible: {message}")]
    ElementNotVisible { message: String, selector: Option<String> },

    #[error("Element not unique: {selector} (found {count})")]
    ElementNotUnique { selector: String, count: usize },

    #[error("Assertion failed: expected {expected}, got {actual}")]
    AssertionFailed { expected: String, actual: String },

    #[error("Network error: {0}")]
    Network(String),

    #[error("Protocol error: {0}")]
    Protocol(String),

    #[error("{0}")]
    Other(String),
}

impl From<TurbosheetError> for napi::Error {
    fn from(e: TurbosheetError) -> Self {
        napi::Error::from_reason(e.to_string())
    }
}

impl From<std::io::Error> for TurbosheetError {
    fn from(e: std::io::Error) -> Self {
        TurbosheetError::Other(e.to_string())
    }
}

impl From<zip::result::ZipError> for TurbosheetError {
    fn from(e: zip::result::ZipError) -> Self {
        TurbosheetError::Other(e.to_string())
    }
}
