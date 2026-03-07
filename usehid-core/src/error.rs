//! Error types for useHID

use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
    #[error("Device not created")]
    DeviceNotCreated,
    
    #[error("Device already exists")]
    DeviceAlreadyExists,
    
    #[error("Failed to create device: {0}")]
    CreateFailed(String),
    
    #[error("Failed to send report: {0}")]
    SendFailed(String),
    
    #[error("Platform not supported: {0}")]
    PlatformNotSupported(String),
    
    #[error("Invalid key: {0}")]
    InvalidKey(String),
    
    #[error("Invalid action: {0}")]
    InvalidAction(String),
    
    #[error("Query failed: {0}")]
    QueryFailed(String),
    
    #[error("Move failed: {0}")]
    MoveFailed(String),
    
    #[error("Failsafe triggered: {0}")]
    FailsafeTriggered(String),

    #[error("Screenshot failed: {0}")]
    ScreenshotFailed(String),

    #[error("Accessibility failed: {0}")]
    AccessibilityFailed(String),
    
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    
    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),
}

pub type Result<T> = std::result::Result<T, Error>;
