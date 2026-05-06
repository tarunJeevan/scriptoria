// src-tauri/src/error.rs
//
// Unified error type for all Tauri commands and internal services.
//
// Design notes:
// - All variants implement `thiserror::Error` for structured messages.
// - `serde::Serialize` is required by Tauri - Command return types must be serializable so errors propagate to the frontend as JSON strings.
// - `CommandError` in commands/documents.rs is replaced by a type alias ponting here, so existing call sites require zero changes.

use serde::Serialize;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum AppError {
    // Database
    #[error("Database error: {0}")]
    Database(#[from] sqlx::Error),

    // Model
    #[error("Model error: {0}")]
    Model(#[from] crate::models::ModelError),

    // Encryption
    #[error("Encryption error: {0}")]
    Encryption(#[from] crate::encryption::EncryptionError),

    // Input validation
    #[error("Validation error: {0}")]
    Validation(String),

    // Not found
    #[error("Not found: {0}")]
    NotFound(String),

    // Ollama / AI
    /// Ollama process is not running or not reachable on the expected port.
    #[error(
        "Ollama is not reachable: {0}. \
		Is Ollama installed and running? \
		Visit https://ollama.com to install it."
    )]
    OllamaNotReachable(String),

    /// Ollama returned an unexpected response shape or non-200 status.
    #[error("Ollama protocol error: {0}")]
    OllamaProtocol(String),

    /// A streaming or blocking inference request was cancelled by the user or the app (e.g. on navigation away from a document).
    #[error("Inference was cancelled")]
    InferenceCancelled,

    /// Requested model is not present in the local Ollama store.
    /// User must `download_model` first.
    #[error("Model not found: {0}. Use download_model to pull it first.")]
    ModelNotFound(String),

    // Serialization
    #[error("Serialization error: {0}")]
    Serialization(String),

    // I/O
    #[error("IO error: {0}")]
    Io(String),
}

// Required by Tauri: command return types must be Serializable so that Err(AppError) serializes to a JSON string on the frontend.
impl Serialize for AppError {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(&self.to_string())
    }
}

// Convenience alias - command modules use `CommandResult<T>` internally.
pub type CommandResult<T> = Result<T, AppError>;

// From impls for types that don't auto-derive
impl From<std::io::Error> for AppError {
    fn from(value: std::io::Error) -> Self {
        AppError::Io(value.to_string())
    }
}

impl From<serde_json::Error> for AppError {
    fn from(value: serde_json::Error) -> Self {
        AppError::Serialization(value.to_string())
    }
}
