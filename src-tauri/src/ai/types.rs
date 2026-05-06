// src-tauri/src/ai/types.rs

use serde::{Deserialize, Serialize};

// Model info returned from Ollama /api/tags
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelInfo {
    pub name: String,
    pub size_bytes: u64,
    pub parameter_count: Option<String>, // e.g. "1.2B", "2B", "7B"
    pub quantization: Option<String>,    // e.g."Q4_K_M"
    pub modified_at: String,
}

// Inference configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InferenceConfig {
    /// Sampling temperature (0.0 = deterministic, 1.0 = creative)
    #[serde(default = "default_temperature")]
    pub temperature: f32,

    /// Top-k sampling (0 = disabled)
    #[serde(default = "default_top_k")]
    pub top_k: u32,

    /// Top-p nucleus sampling
    #[serde(default = "default_top_p")]
    pub top_p: f32,

    /// Max tokens to generate (None = model default)
    pub max_tokens: Option<u32>,

    /// System prompt injected before used prompt
    pub system_prompt: Option<String>,
}

impl Default for InferenceConfig {
    fn default() -> Self {
        Self {
            temperature: default_temperature(),
            top_k: default_top_k(),
            top_p: default_top_p(),
            max_tokens: None,
            system_prompt: None,
        }
    }
}

fn default_temperature() -> f32 {
    0.7
}
fn default_top_k() -> u32 {
    40
}
fn default_top_p() -> f32 {
    0.9
}

// Streaming event payload emitted to frontend
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StreamChunk {
    pub request_id: String,
    pub delta: String, // incremental token text
    pub done: bool,
    pub token_count: Option<u32>, // populated on final chunk
}

// AI preferences (persisted to user settings)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AiPreferences {
    /// "1B" | "7B" | custom model name
    #[serde(default = "default_model_tier")]
    pub preferred_model_tier: String,

    /// Resolved Ollama model tag for the preferred tier
    #[serde(default = "default_model_1b")]
    pub model_1b: String,

    /// Resovled Ollama model tag for the 7B tier
    #[serde(default = "default_model_7b")]
    pub model_7b: String,

    /// Base URL for Ollama (allows non-default port)
    #[serde(default = "default_ollama_url")]
    pub ollama_base_url: String,
}

impl Default for AiPreferences {
    fn default() -> Self {
        Self {
            preferred_model_tier: default_model_tier(),
            model_1b: default_model_1b(),
            model_7b: default_model_7b(),
            ollama_base_url: default_ollama_url(),
        }
    }
}

fn default_model_tier() -> String {
    "1b".to_string()
}
fn default_model_1b() -> String {
    "llama3.2:1b".to_string()
}
fn default_model_7b() -> String {
    "llama3.2:7b".to_string()
}
fn default_ollama_url() -> String {
    "http://127.0.0.1:11434".to_string()
}

// Internal Ollama API response shapes

/// Response shape from Ollama /api/generate (NDJSON stream)
#[derive(Debug, Deserialize)]
pub struct OllamaGenerateChunk {
    pub response: String,
    pub done: bool,
    pub eval_count: Option<u32>, // total tokens generated (on done=true)
    pub prompt_eval_count: Option<u32>,
}

/// Single model entry from Ollama /api/tags
#[derive(Debug, Deserialize)]
pub struct OllamaModelEntry {
    pub name: String,
    pub size: u64,
    pub modified_at: String,
    pub details: Option<OllamaModelDetails>,
}

#[derive(Debug, Deserialize)]
pub struct OllamaModelDetails {
    pub parameter_size: Option<String>,
    pub quantization_level: Option<String>,
}

/// Response from Ollama /api/tags
#[derive(Debug, Deserialize)]
pub struct OllamaTagsResponse {
    pub models: Vec<OllamaModelEntry>,
}
