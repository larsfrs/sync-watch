use thiserror::Error;

#[derive(Debug, Clone, serde::Serialize, Error)]
#[serde(tag = "kind", content = "message")]
pub enum RcloneError {
    #[error("rclone not found on PATH")]
    NotInstalled,
    #[error("rclone version check failed: {0}")]
    InstalledCheckFail(String),
    #[error("daemon failed to start: {0}")]
    DaemonFailedToStart(String),
    #[error("daemon failed to stop: {0}")]
    DaemonFailedToStop(String),
    #[error("daemon is not running")]
    DaemonNotRunning,
    #[error("rc call failed: {0}")]
    RcCallFailed(String),
    #[error("unexpected response: {0}")]
    UnexpectedResponse(String),
    #[error("timed out")]
    Timeout,
}

#[derive(Debug, Clone, serde::Serialize, Error)]
#[serde(tag = "kind", content = "message")]
pub enum StoreError {
    #[error("read failed: {0}")]
    ReadFailed(String),
    #[error("write failed: {0}")]
    WriteFailed(String),
    #[error("unexpected error: {0}")]
    Other(String),
}

#[derive(Debug, Clone, serde::Serialize, Error)]
#[serde(tag = "kind", content = "message")]
pub enum BackendError {
    #[error(transparent)]
    Rclone(#[from] RcloneError),
    #[error(transparent)]
    Store(#[from] StoreError),
    #[error("unexpected error: {0}")]
    Other(String),
}