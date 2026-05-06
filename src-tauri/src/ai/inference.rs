// src-tauri/src/ai/inference.rs

use std::collections::VecDeque;
use std::sync::Arc;

use tokio::sync::{Mutex, Notify};
use tokio_util::sync::CancellationToken;
use uuid::Uuid;

use crate::ai::ollama::OllamaClient;
use crate::ai::types::{InferenceConfig, StreamChunk};
use crate::error::AppError;

/// A pending inference request sitting in the queue.
struct QueuedRequest {
    request_id: String,
    model: String,
    prompt: String,
    config: InferenceConfig,
    cancel: CancellationToken,
    /// Callback invoked for each streamed chunk.
    /// Box<dyn Fn> so the command layer can close over the AppHandle.
    on_chunk: Box<dyn Fn(StreamChunk) + Send + Sync>,
    /// Resolves when the request completes (Ok) or errors out (Err).
    result_tx: tokio::sync::oneshot::Sender<Result<(), AppError>>,
}

/// Sequential inference queue - one request processed at a time.
/// Prevents OOM on minimum-spec hardware (8GB RAM).
pub struct InferenceQueue {
    inner: Arc<Mutex<VecDeque<QueuedRequest>>>,
    notify: Arc<Notify>,
}

impl InferenceQueue {
    pub fn new() -> Self {
        Self {
            inner: Arc::new(Mutex::new(VecDeque::new())),
            notify: Arc::new(Notify::new()),
        }
    }

    /// Spawn the background worker that drains the queue.
    /// Call once during app startup with a shared OllamaClient.
    pub fn start_worker(&self, client: Arc<OllamaClient>) {
        let inner = Arc::clone(&self.inner);
        let notify = Arc::clone(&self.notify);

        tokio::spawn(async move {
            loop {
                // Wait until there's something to do
                notify.notified().await;

                loop {
                    let request = {
                        let mut queue = inner.lock().await;
                        queue.pop_front()
                    };

                    let Some(req) = request else { break };

                    let result = client
                        .stream_generate(
                            &req.request_id,
                            &req.model,
                            &req.prompt,
                            &req.config,
                            req.on_chunk,
                            req.cancel,
                        )
                        .await;

                    // Ignore send error - frontend may have navigated away
                    let _ = req.result_tx.send(result);
                }
            }
        });
    }

    /// Enqueue a streaming inference request.
    /// Returns (request_id, result_rx) immediately.
    /// `result_rx` resolves when the request finishes or errors.
    pub async fn enqueue(
        &self,
        model: String,
        prompt: String,
        config: InferenceConfig,
        on_chunk: Box<dyn Fn(StreamChunk) + Send + Sync>,
    ) -> (
        String,
        tokio::sync::oneshot::Receiver<Result<(), AppError>>,
        CancellationToken,
    ) {
        let request_id = Uuid::new_v4().to_string();
        let cancel = CancellationToken::new();
        let (result_tx, result_rx) = tokio::sync::oneshot::channel();

        let request = QueuedRequest {
            request_id: request_id.clone(),
            model,
            prompt,
            config,
            cancel: cancel.clone(),
            on_chunk,
            result_tx,
        };

        self.inner.lock().await.push_back(request);
        self.notify.notify_one();

        (request_id, result_rx, cancel)
    }

    /// Cancel a queued or in-flight request by ID.
    /// If the request is already complete, this is a no-op.
    pub async fn cancel(&self, request_id: &str) {
        let queue = self.inner.lock().await;
        if let Some(req) = queue.iter().find(|r| r.request_id == request_id) {
            req.cancel.cancel();
        }
        // If `request_id` is not in the queue, it's already running - its CancellationToken was cloned by the caller and can be cancelled via the `active_cancellations` map in `AppState`.
    }
}
