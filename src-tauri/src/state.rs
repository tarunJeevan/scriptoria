// src-tauri/src/state.rs
//
// Centralized Tauri managed state.
//
// AppState (previoudly located in `command/documents.rs`) is defined here and re-exported from there so all existing call sites continue to compile.

use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::Arc;

use sqlx::SqlitePool;
use tokio::sync::Mutex;
use tokio_util::sync::CancellationToken;

use crate::ai::inference::InferenceQueue;
use crate::ai::ollama::OllamaClient;
use crate::ai::types::AiPreferences;
use crate::encryption::EncryptionService;

// AppState

/// Tauri managed state - one instance per app lifetime.
///
/// Constructed in `lib.rs` setup closure and registered via `app.manage(AppState { ... })`.
///
/// Cloning policy:
/// - `pool` is cheaply cloneable (`Arc`-backed internally by sqlx).
/// - `encryption` is `Clone` (holds a 32-byte key copy).
/// - AI fields use `Arc<_>` so the queue qorker and command handlers can share ownership without copying.
pub struct AppState {
    /// SQLite connection pool (SQLCipher-encrypted)
    pub pool: SqlitePool,

    /// Encryption service - wraps ChaCha20-Poly1305 + key material
    pub encryption: EncryptionService,

    /// Ollama HTTP client (shared, connection-pooled)
    pub ollama: Arc<OllamaClient>,

    /// Sequential inference queue
    pub inference_queue: Arc<InferenceQueue>,

    /// Active CancellationTokens keyed by request_id
    pub active_cancellations: Arc<Mutex<HashMap<String, CancellationToken>>>,

    /// Current AI preferences (also persisted to config file)
    pub ai_preferences: Arc<Mutex<AiPreferences>>,

    /// Path to app config directory (for persisting preferences)
    pub config_dir: std::path::PathBuf,
}

impl AppState {
    /// Build `AppState` from its components.
    ///
    /// Called from `lib.rs` after the database pool and encryption service have been set up. Loads persisted AI preferences (falling back to defauls if the config file is absent or corrupted), constructs the Ollama client with the configured base URL, and spawns the inference queue worker.
    pub async fn new(pool: SqlitePool, encryption: EncryptionService, config_dir: PathBuf) -> Self {
        let prefs = load_ai_preferences(&config_dir).await;

        let ollama = Arc::new(OllamaClient::new(&prefs.ollama_base_url));
        let inference_queue = Arc::new(InferenceQueue::new());

        // Start the background worker that drains the inference queue.
        // The worker holds a shared reference to the Ollama client.
        inference_queue.start_worker(Arc::clone(&ollama));

        Self {
            pool,
            encryption,
            ollama,
            inference_queue,
            active_cancellations: Arc::new(Mutex::new(HashMap::new())),
            ai_preferences: Arc::new(Mutex::new(prefs)),
            config_dir,
        }
    }
}

// Helpers

/// Load `ai_preferences.json` from the config directory.
/// Returns `AiPreferences::default()` on any failure - startup must never panic due to a missing or malformed preferences file.
async fn load_ai_preferences(config_dir: &Path) -> AiPreferences {
    let path = config_dir.join("ai_preferences.json");

    let raw = match tokio::fs::read_to_string(&path).await {
        Ok(s) => s,
        Err(_) => return AiPreferences::default(),
    };

    serde_json::from_str(&raw).unwrap_or_else(|e| {
        // Log malformed config but don't crash = silently fall back.
        eprintln!(
            "[state] Failed to parse ai_preferences.json: {e}. \
			Using defaults."
        );
        AiPreferences::default()
    })
}
