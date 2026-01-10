//! Error types for Red Queen.

use thiserror::Error;

/// Red Queen error type.
#[derive(Error, Debug)]
pub enum Error {
    /// Configuration error.
    #[error("Configuration error: {0}")]
    Config(String),

    /// Evaluation error.
    #[error("Evaluation error: {0}")]
    Evaluation(String),

    /// Serialization error.
    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),

    /// IO error.
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
}

/// Result type alias.
pub type Result<T> = std::result::Result<T, Error>;
