use std::fmt;

use serde::Serialize;

pub type Result<T> = std::result::Result<T, AgentskillError>;

#[derive(Debug)]
pub enum AgentskillError {
    Io(std::io::Error),
    InvalidPath(String),
    InvalidArgument(String),
    Json(serde_json::Error),
    Other(String),
}

impl fmt::Display for AgentskillError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Io(error) => write!(f, "{error}"),
            Self::InvalidPath(message) | Self::InvalidArgument(message) | Self::Other(message) => {
                f.write_str(message)
            }
            Self::Json(error) => write!(f, "{error}"),
        }
    }
}

impl std::error::Error for AgentskillError {}

impl From<std::io::Error> for AgentskillError {
    fn from(value: std::io::Error) -> Self {
        Self::Io(value)
    }
}

impl From<serde_json::Error> for AgentskillError {
    fn from(value: serde_json::Error) -> Self {
        Self::Json(value)
    }
}

#[derive(Debug, Serialize)]
pub struct ErrorPayload {
    pub error: String,
    pub script: String,
}

pub fn error_payload(error: impl ToString, script: &str) -> serde_json::Value {
    serde_json::json!({"error": error.to_string(), "script": script})
}

pub fn validate_repo(path: &str) -> Result<std::path::PathBuf> {
    let repo = std::path::Path::new(path)
        .canonicalize()
        .map_err(|_| AgentskillError::InvalidPath(format!("path does not exist: {path}")))?;

    if !repo.is_dir() {
        return Err(AgentskillError::InvalidPath(format!(
            "not a directory: {path}"
        )));
    }

    Ok(repo)
}
