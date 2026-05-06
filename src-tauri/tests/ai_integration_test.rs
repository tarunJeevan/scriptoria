// src-tauri/tests/ai_integration_test.rs

//! Integration tests for AI inference backend.
//! Uses wiremock to simulate Ollama's REST API without a live Ollama process.

use scriptoria_lib::ai::ollama::OllamaClient;
use scriptoria_lib::ai::types::{AiPreferences, InferenceConfig};
use scriptoria_lib::error::AppError;
use tokio_util::sync::CancellationToken;
use wiremock::matchers::{method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};

#[tokio::test]
async fn test_list_models_returns_parsed_model_info() {
    let mock = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/api/tags"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "models": [{
                "name": "llama3.2:1b",
                "size": 1300000000u64,
                "modified_at": "2024-01-01T00:00:00Z",
                "details": {
                    "parameter_size": "1.2B",
                    "quantization_level": "Q4_K_M",
                },
            }]
        })))
        .mount(&mock)
        .await;

    let client = OllamaClient::new(mock.uri());
    let models = client.list_models().await.unwrap();

    assert_eq!(models.len(), 1);
    assert_eq!(models[0].name, "llama3.2:1b");
    assert_eq!(models[0].parameter_count.as_deref(), Some("1.2B"));
}

#[tokio::test]
async fn test_generate_returns_response_text() {
    let mock = MockServer::start().await;

    Mock::given(method("POST"))
        .and(path("/api/generate"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "response": "Once upon a time",
            "done": true,
            "eval_count": 5,
            "prompt_eval_count": 10,
        })))
        .mount(&mock)
        .await;

    let client = OllamaClient::new(mock.uri());
    let cancel = CancellationToken::new();
    let result = client
        .generate(
            "llama3.2:1b",
            "Start a story",
            &InferenceConfig::default(),
            cancel,
        )
        .await
        .unwrap();

    assert_eq!(result, "Once upon a time");
}

#[tokio::test]
async fn test_stream_generate_emits_chunks_and_terminates() {
    let mock = MockServer::start().await;

    // Simulate NDJSON stream
    let ndjson = "{\"response\":\"Hello\",\"done\":false}\n\
		{\"response\":\" world\",\"done\":false}\n\
		{\"response\":\"\",\"done\":true,\"eval_count\":3}\n";

    Mock::given(method("POST"))
        .and(path("/api/generate"))
        .respond_with(ResponseTemplate::new(200).set_body_json(ndjson))
        .mount(&mock)
        .await;

    let client = OllamaClient::new(mock.uri());
    let cancel = CancellationToken::new();
    let mut chunks = vec![];

    client
        .stream_generate(
            "req-001",
            "llama3.2:1b",
            "Say hello",
            &InferenceConfig::default(),
            |chunk| chunks.push(chunk),
            cancel,
        )
        .await
        .unwrap();

    assert_eq!(chunks.len(), 3);
    assert_eq!(chunks[0].delta, "Hello");
    assert_eq!(chunks[1].delta, " world");
    assert!(chunks[2].done);
    assert_eq!(chunks[2].token_count, Some(3));
}

#[tokio::test]
async fn test_cancellation_stops_stream() {
    // Verify CancellationToken fires and stream_generate returns InferenceCancelled
    let mock = MockServer::start().await;

    Mock::given(method("POST"))
        .and(path("/api/generate"))
        .respond_with(
            ResponseTemplate::new(200)
                .set_body_string("{\"response\":\"tok\",\"done\":false}\n")
                .set_delay(std::time::Duration::from_millis(200)),
        )
        .mount(&mock)
        .await;

    let client = OllamaClient::new(mock.uri());
    let cancel = CancellationToken::new();
    let cancel_clone = cancel.clone();

    tokio::spawn(async move {
        tokio::time::sleep(std::time::Duration::from_millis(50)).await;
        cancel_clone.cancel();
    });

    let result = client
        .stream_generate(
            "req-cancel",
            "llama3.2:1b",
            "Test",
            &InferenceConfig::default(),
            |_| {},
            cancel,
        )
        .await;

    assert!(matches!(result, Err(AppError::InferenceCancelled)));
}

#[tokio::test]
async fn test_inference_config_defaults_are_valid() {
    let config = InferenceConfig::default();

    assert!(config.temperature >= 0.0 && config.temperature <= 1.0);
    assert!(config.top_k > 0);
    assert!(config.top_p > 0.0 && config.top_p <= 1.0);
    assert!(config.max_tokens.is_none());
    assert!(config.system_prompt.is_none());
}

#[tokio::test]
async fn test_ai_preferences_defaults() {
    let prefs = AiPreferences::default();

    assert_eq!(prefs.preferred_model_tier, "1b");
    assert_eq!(prefs.model_1b, "llama3.2:1b");
    assert_eq!(prefs.ollama_base_url, "http://127.0.0.1:11434");
}
