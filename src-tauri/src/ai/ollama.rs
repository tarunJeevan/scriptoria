// src-tauri/src/ai/ollama.rs

use futures_util::StreamExt;
use reqwest::Client;
use serde_json::json;
use tokio_util::sync::CancellationToken;

use crate::ai::types::{
    InferenceConfig, ModelInfo, OllamaGenerateChunk, OllamaTagsResponse, StreamChunk,
};
use crate::error::AppError;

/// Thin async HTTP wrapper around the Ollama REST API.
/// All methods are stateless - the client holds only the base URL and a shared reqwest::Client (connection pool).
pub struct OllamaClient {
    base_url: String,
    http: Client,
}

impl OllamaClient {
    pub fn new(base_url: impl Into<String>) -> Self {
        let http = Client::builder()
            .timeout(std::time::Duration::from_secs(30))
            .build()
            .expect("Failed to build reqwest client.");

        Self {
            base_url: base_url.into(),
            http,
        }
    }

    /// Check if Ollama is reachable.
    /// Returns Ok(version_string) or Err.
    pub async fn health_check(&self) -> Result<String, AppError> {
        let resp = self
            .http
            .get(format!("{}/api/version", self.base_url))
            .send()
            .await
            .map_err(|e| AppError::OllamaNotReachable(e.to_string()))?;

        let body: serde_json::Value = resp
            .json()
            .await
            .map_err(|e| AppError::OllamaNotReachable(e.to_string()))?;

        Ok(body["version"].as_str().unwrap_or("unknown").to_string())
    }

    /// List locally available models.
    pub async fn list_models(&self) -> Result<Vec<ModelInfo>, AppError> {
        let resp = self
            .http
            .get(format!("{}/api/tags", self.base_url))
            .send()
            .await
            .map_err(|e| AppError::OllamaNotReachable(e.to_string()))?;

        let tags: OllamaTagsResponse = resp
            .json()
            .await
            .map_err(|e| AppError::OllamaProtocol(e.to_string()))?;

        Ok(tags
            .models
            .into_iter()
            .map(|m| ModelInfo {
                name: m.name,
                size_bytes: m.size,
                parameter_count: m.details.as_ref().and_then(|d| d.parameter_size.clone()),
                quantization: m
                    .details
                    .as_ref()
                    .and_then(|d| d.quantization_level.clone()),
                modified_at: m.modified_at,
            })
            .collect())
    }

    /// Pull (download) a model by name.
    /// Streams progress lines. Caller receives log strings via the provided callback.
    pub async fn pull_model(
        &self,
        model_name: &str,
        on_progress: impl Fn(String) + Send,
        cancel: CancellationToken,
    ) -> Result<(), AppError> {
        let body = json!({"name": model_name, "stream": true});

        let resp = self
            .http
            .post(format!("{}/api/pull", self.base_url))
            .json(&body)
            .send()
            .await
            .map_err(|e| AppError::OllamaNotReachable(e.to_string()))?;

        let mut stream = resp.bytes_stream();

        loop {
            tokio::select! {
                _ = cancel.cancelled() => {
                    return Err(AppError::InferenceCancelled);
                }
                chunk = stream.next() => {
                    match chunk {
                        Some(Ok(bytes)) => {
                            if let Ok(s) = std::str::from_utf8(&bytes) {
                                on_progress(s.to_string());
                            }
                        }
                        Some(Err(e)) => {
                            return Err(AppError::OllamaProtocol(e.to_string()));
                        }
                        None => break,
                    }
                }
            }
        }

        Ok(())
    }

    /// Non-streaming generate - returns full response string.
    /// Used for short, latency-sensitive completions (e.g. inline suggestions).
    pub async fn generate(
        &self,
        model: &str,
        prompt: &str,
        config: &InferenceConfig,
        cancel: CancellationToken,
    ) -> Result<String, AppError> {
        let body = build_generate_body(model, prompt, config, false);

        let resp = tokio::select! {
        _ = cancel.cancelled() => return Err(AppError::InferenceCancelled),
        r = self.http
        .post(format!("{}/api/generate", self.base_url))
        .json(&body)
        .send() => r.map_err(|e| AppError::OllamaNotReachable(e.to_string()))?
        };

        let chunk: OllamaGenerateChunk = resp
            .json()
            .await
            .map_err(|e| AppError::OllamaProtocol(e.to_string()))?;

        Ok(chunk.response)
    }

    /// Streaming generate - yields StreamChunk values via callback.
    /// Caller is responsible for forwarding chunks to Tauri events.
    pub async fn stream_generate(
        &self,
        request_id: &str,
        model: &str,
        prompt: &str,
        config: &InferenceConfig,
        on_chunk: impl Fn(StreamChunk) + Send,
        cancel: CancellationToken,
    ) -> Result<(), AppError> {
        let body = build_generate_body(model, prompt, config, true);

        let resp = self
            .http
            .post(format!("{}/api/generate", self.base_url))
            .json(&body)
            .send()
            .await
            .map_err(|e| AppError::OllamaNotReachable(e.to_string()))?;

        let mut stream = resp.bytes_stream();
        let mut buffer = String::new();
        let mut total_tokens: Option<u32> = None;

        loop {
            tokio::select! {
            _ = cancel.cancelled() => {
            return Err(AppError::InferenceCancelled);
            }
            chunk = stream.next() => {
            match chunk {
            Some(Ok(bytes)) => {
            buffer.push_str(
            std::str::from_utf8(&bytes)
            .map_err(|e| AppError::OllamaProtocol(e.to_string()))?
            );

            // Ollama streams newline-delimited JSON
            while let Some(newline_pos) = buffer.find('\n') {
            let line: String = buffer.drain(..=newline_pos).collect();
            let line = line.trim();

            if line.is_empty() {continue;}

            let parsed: OllamaGenerateChunk = serde_json::from_str(line)
            .map_err(|e| AppError::OllamaProtocol(e.to_string()))?;

            if parsed.done {
            total_tokens = parsed.eval_count;
            }

            on_chunk(StreamChunk { request_id: request_id.to_string(), delta: parsed.response, done: parsed.done, token_count: if parsed.done {total_tokens} else {None}, });

            if parsed.done {
            return Ok(());
            }
            }
            }
            Some(Err(e)) => {
            return Err(AppError::OllamaProtocol(e.to_string()));
            }
            None => return Ok(()),
            }
            }
            }
        }
    }
}

// Helper: build Ollama `/api/generate` request body
fn build_generate_body(
    model: &str,
    prompt: &str,
    config: &InferenceConfig,
    stream: bool,
) -> serde_json::Value {
    let mut options = json!({
        "temperature": config.temperature,
        "top_k": config.top_k,
        "top_p": config.top_p,
    });

    if let Some(max_tokens) = config.max_tokens {
        options["num_predict"] = json!(max_tokens);
    }

    let mut body = json!({
        "model": model,
        "prompt": prompt,
        "stream": stream,
        "options": options,
    });

    if let Some(ref sys) = config.system_prompt {
        body["system"] = json!(sys);
    }

    body
}
