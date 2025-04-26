use std::error::Error;
use std::fmt;

// Custom error type for Nextcloud sync operations
#[derive(Debug)]
pub enum SyncError {
    Request(String),
    Response(String),
    FileSystem(String),
    WebDav(String),
    Configuration(String),
}

impl fmt::Display for SyncError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            SyncError::Request(msg) => write!(f, "Request error: {}", msg),
            SyncError::Response(msg) => write!(f, "Response error: {}", msg),
            SyncError::FileSystem(msg) => write!(f, "File system error: {}", msg),
            SyncError::WebDav(msg) => write!(f, "WebDAV error: {}", msg),
            SyncError::Configuration(msg) => write!(f, "Configuration error: {}", msg),
        }
    }
}

impl Error for SyncError {}

// Convert generic errors to SyncError
impl From<std::io::Error> for SyncError {
    fn from(error: std::io::Error) -> Self {
        SyncError::FileSystem(error.to_string())
    }
}

impl From<reqwest::Error> for SyncError {
    fn from(error: reqwest::Error) -> Self {
        if error.is_timeout() {
            SyncError::Request(format!("Request timeout: {}", error))
        } else if error.is_connect() {
            SyncError::Request(format!("Connection error: {}", error))
        } else if error.is_status() {
            SyncError::Response(format!("HTTP error: {}", error))
        } else {
            SyncError::Request(format!("Request error: {}", error))
        }
    }
}

impl From<serde_json::Error> for SyncError {
    fn from(error: serde_json::Error) -> Self {
        SyncError::Configuration(format!("JSON parsing error: {}", error))
    }
}

impl From<url::ParseError> for SyncError {
    fn from(error: url::ParseError) -> Self {
        SyncError::Configuration(format!("URL parsing error: {}", error))
    }
}
