// src-tauri/src/commands/ai.rs

// use std::collections::HashMap;
// use std::sync::Arc;

use tauri::{
    AppHandle,
    Emitter,
    // Manager,
    State,
};
// use tokio::sync::Mutex;
use tokio_util::sync::CancellationToken;

use crate::ai::types::{AiPreferences, InferenceConfig, ModelInfo, StreamChunk};
use crate::error::AppError;
use crate::state::AppState;

// Model management

/// Returns all locally available Ollama models.
#[tauri::command]
pub async fn list_models(state: State<'_, AppState>) -> Result<Vec<ModelInfo>, AppError> {
    state.ollama.list_models().await
}

/// Initiates a model pull (download). Progress is emitted as Tauri events: "ai://pull/progress" -> { model, status_line }
#[tauri::command]
pub async fn download_model(
    model_name: String,
    app: AppHandle,
    state: State<'_, AppState>,
) -> Result<(), AppError> {
    let cancel = CancellationToken::new();
    let model_name_clone = model_name.clone();
    let app_clone = app.clone();

    state
        .ollama
        .pull_model(
            &model_name,
            move |status| {
                let _ = app_clone.emit(
                    "ai://pull/progress",
                    serde_json::json!({
                        "model": model_name_clone,
                        "status": status,
                    }),
                );
            },
            cancel,
        )
        .await
}

/// Check if Ollama is running and reachable.
#[tauri::command]
pub async fn check_ollama_status(state: State<'_, AppState>) -> Result<String, AppError> {
    state.ollama.health_check().await
}

// Inference

/// Non-streaming inference. Use for short inline suggestions where latency target is <2s.
/// Returns the full response string.
#[tauri::command]
pub async fn generate_text(
    prompt: String,
    model: String,
    config: InferenceConfig,
    state: State<'_, AppState>,
) -> Result<String, AppError> {
    let cancel = CancellationToken::new();
    state
        .ollama
        .generate(&model, &prompt, &config, cancel)
        .await
}

/// Streaming inference. Chunks are emitted as Tauri events: "ai://stream/{request_id}" -> { request_id, delta, done, token_count? }
/// Returns `request_id` immediately.
#[tauri::command]
pub async fn stream_text(
    prompt: String,
    model: String,
    config: InferenceConfig,
    app: AppHandle,
    state: State<'_, AppState>,
) -> Result<String, AppError> {
    let on_chunk = {
        let app = app.clone();
        move |chunk: StreamChunk| {
            let event = format!("ai://stream/{}", chunk.request_id);
            let _ = app.emit(&event, &chunk);
        }
    };

    let (request_id, _result_rx, cancel) = state
        .inference_queue
        .enqueue(model, prompt, config, Box::new(on_chunk))
        .await;

    // Track cancellation token so cancel_inference can reach it
    state
        .active_cancellations
        .lock()
        .await
        .insert(request_id.clone(), cancel);

    Ok(request_id)
}

/// Cancel an in-flight or queued stremaing request.
#[tauri::command]
pub async fn cancel_inference(
    request_id: String,
    state: State<'_, AppState>,
) -> Result<(), AppError> {
    let mut active = state.active_cancellations.lock().await;
    if let Some(token) = active.remove(&request_id) {
        token.cancel();
    }
    state.inference_queue.cancel(&request_id).await;
    Ok(())
}

// AI Preferences

/// Returns the current AI preferences (model tier, model tags, Ollama URL)
#[tauri::command]
pub async fn get_ai_preferences(state: State<'_, AppState>) -> Result<AiPreferences, AppError> {
    let prefs = state.ai_preferences.lock().await;
    Ok(prefs.clone())
}

/// Persists updated AI preferences to the app config file.
/// NOTE: Chunk 7 is the UI owner. This command is the persistence layer.
#[tauri::command]
pub async fn set_ai_preferences(
    preferences: AiPreferences,
    state: State<'_, AppState>,
) -> Result<(), AppError> {
    let mut prefs = state.ai_preferences.lock().await;
    *prefs = preferences.clone();

    // Persist to config file (plaintext - Tier 3, not sensitive)
    let config_path = state.config_dir.join("ai_preferences.json");
    let json = serde_json::to_string_pretty(&preferences)
        .map_err(|e| AppError::Serialization(e.to_string()))?;

    tokio::fs::write(&config_path, json)
        .await
        .map_err(|e| AppError::Io(e.to_string()))?;

    Ok(())
}
