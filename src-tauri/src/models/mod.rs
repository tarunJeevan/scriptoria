// Database models for Scriptoria Phase 1
//
// This module will contain type-safe representations of database entities:
// - Document, DecryptedDocument, DocumentVersion
// - Project, ProjectMetadata
// - AiContextChunk, VectorEmbedding
// - ChatSession, ChatMessage
// - EncryptedContent (encryption wrapper)
// - Query parameter structs (CreateDocumentParams, UpdateDocumentParams)
//
// Models will use:
// - sqlx::FromRow for database mapping
// - serde::{Serialize, Deserialize} for Tauri IPC
// - Custom types for encrypted content (nonce, tag, ciphertext)
//
// See: scriptoria-phase-1-rust-database-models.rs in project docs

// use blake3::hazmat::Mode;
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
// use time::OffsetDateTime;

// Re-export submodules
pub mod ai;
pub mod chat;
pub mod document;
pub mod encryption;
pub mod project;

// ============================================================================
// COMMON TYPES
// ============================================================================

/// Encrypted content wrapper with metadata
#[derive(Debug, Clone)]
pub struct EncryptedContent {
    pub ciphertext: Vec<u8>,
    pub nonce: [u8; 12], // ChaCha20-Poly1305 nonce
    pub tag: [u8; 16],   // Authentication tag
}

impl EncryptedContent {
    pub fn to_blob(&self) -> Vec<u8> {
        // Serialize as: [nonce(12) || ciphertext || tag(16)]
        let mut blob = Vec::with_capacity(12 + self.ciphertext.len() + 16);
        blob.extend_from_slice(&self.nonce);
        blob.extend_from_slice(&self.ciphertext);
        blob.extend_from_slice(&self.tag);
        blob
    }

    pub fn from_blob(blob: &[u8]) -> Result<Self> {
        if blob.len() < 28 {
            return Err("Blob too short for encrypted content".into());
        }

        let mut nonce = [0u8; 12];
        let mut tag = [0u8; 16];

        nonce.copy_from_slice(&blob[0..12]);
        let ciphertext = blob[12..blob.len() - 16].to_vec();
        tag.copy_from_slice(&blob[blob.len() - 16..]);

        Ok(Self {
            ciphertext,
            nonce,
            tag,
        })
    }
}

// ============================================================================
// USER MODEL
// ============================================================================

#[derive(Debug, Clone, FromRow, Serialize, Deserialize)]
pub struct User {
    pub id: i64,
    pub username: String,
    pub created_at: String,
    pub last_login: Option<String>,
    pub preferences: String, // JSON
}

impl User {
    pub fn preferences_json(&self) -> serde_json::Value {
        serde_json::from_str(&self.preferences).unwrap_or(serde_json::json!({}))
    }
}

// ============================================================================
// PROJECT MODEL
// ============================================================================

#[derive(Debug, Clone, FromRow, Serialize, Deserialize)]
pub struct Project {
    pub id: i64,
    pub user_id: i64,
    pub title: String,
    pub description: Option<String>,
    pub parent_id: Option<i64>,
    pub display_order: i64,
    pub created_at: String,
    pub updated_at: String,
    pub archived: bool,
    pub metadata: Option<String>, // JSON
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectMetadata {
    pub tags: Vec<String>,
    pub color: Option<String>,
    pub project_type: Option<String>, // "novel", "series", "short_story", etc.
    #[serde(flatten)]
    pub custom: serde_json::Map<String, serde_json::Value>,
}

impl Project {
    pub fn metadata_parsed(&self) -> ProjectMetadata {
        self.metadata
            .as_ref()
            .and_then(|m| serde_json::from_str(m).ok())
            .unwrap_or_default()
    }
}

impl Default for ProjectMetadata {
    fn default() -> Self {
        Self {
            tags: Vec::new(),
            color: None,
            project_type: None,
            custom: serde_json::Map::new(),
        }
    }
}

// ============================================================================
// DOCUMENT MODEL
// ============================================================================

#[derive(Debug, Clone, FromRow)]
pub struct Document {
    pub id: i64,
    pub project_id: i64,
    pub title: String,
    pub content_encrypted: Vec<u8>,
    pub word_count: i64,
    pub char_count: i64,
    pub created_at: String,
    pub updated_at: String,
    pub last_edited_at: Option<String>,
    pub doc_type: String,
    pub entity_type: Option<String>,
    pub parent_id: Option<i64>,
    pub display_order: i64,
    pub deleted: bool,
    pub metadata: Option<String>, // JSON
}

// Decrypted Document for application use
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DecryptedDocument {
    pub id: i64,
    pub project_id: i64,
    pub title: String,
    pub content: String, // Plaintext content
    pub word_count: i64,
    pub char_count: i64,
    pub created_at: String,
    pub updated_at: String,
    pub last_edited_at: Option<String>,
    pub doc_type: String,
    pub entity_type: Option<String>,
    pub parent_id: Option<i64>,
    pub display_order: i64,
    pub deleted: bool,
    pub metadata: DocumentMetadata,
}

// DocumentMetadata struct
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct DocumentMetadata {
    pub tags: Vec<String>,
    pub editor_state: Option<serde_json::Value>,
    #[serde(flatten)]
    pub custom: serde_json::Map<String, serde_json::Value>,
}

// ============================================================================
// DOCUMENT VERSION MODEL
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct DocumentVersion {
    pub id: i64,
    pub document_id: i64,
    pub version_number: i64,
    pub content_encrypted: Vec<u8>,
    pub word_count: Option<i64>,
    pub char_count: Option<i64>,
    pub created_at: String,
    pub label: Option<String>,
    pub diff_summary: Option<String>, // JSON
    pub is_delta: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VersionDiffSummary {
    pub lines_added: usize,
    pub lines_removed: usize,
    pub description: Option<String>,
}

// ============================================================================
// FILE ATTACHMENT MODEL
// ============================================================================

#[derive(Debug, Clone, FromRow)]
pub struct DocumentAttachment {
    pub id: i64,
    pub document_id: i64,
    pub filename: String,
    pub file_encrypted: Vec<u8>,
    pub key_derivation_info: String, // JSON
    pub mime_type: Option<String>,
    pub file_size: Option<i64>,
    pub uploaded_at: String,
    pub metadata: Option<String>, // JSON
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KeyDerivationInfo {
    pub algorithm: String, // "chacha20-poly1305"
    pub file_hash: String, // SHA256 of original file
    pub salt: Vec<u8>,
}

// ============================================================================
// AI CONTEXT CHUNK MODEL
// ============================================================================

#[derive(Debug, Clone, FromRow)]
pub struct AiContextChunk {
    pub id: i64,
    pub document_id: i64,
    pub chunk_index: i64,
    pub chunk_strategy: String,
    pub content_encrypted: Vec<u8>,
    pub token_count: Option<i64>,
    pub char_count: Option<i64>,
    pub start_offset: Option<i64>,
    pub end_offset: Option<i64>,
    pub created_at: String,
    pub is_stale: bool,
}

// ============================================================================
// VECTOR EMBEDDING MODEL
// ============================================================================

#[derive(Debug, Clone, FromRow)]
pub struct VectorEmbedding {
    pub id: i64,
    pub chunk_id: i64,
    pub model_name: String,
    pub model_version: Option<String>,
    pub embedding_vector: Vec<u8>, // Serialized f32 array
    pub vector_dimension: i64,
    pub created_at: String,
}

impl VectorEmbedding {
    /// Deserialize embedding vector from BLOB
    pub fn to_f32_vec(&self) -> Result<Vec<f32>> {
        if !self.embedding_vector.len().is_multiple_of(4) {
            return Err("Invalid vector blob length".into());
        }

        let float_count = self.embedding_vector.len() / 4;
        let mut vec = Vec::with_capacity(float_count);

        for i in 0..float_count {
            let bytes = [
                self.embedding_vector[i * 4],
                self.embedding_vector[i * 4 + 1],
                self.embedding_vector[i * 4 + 2],
                self.embedding_vector[i * 4 + 3],
            ];
            vec.push(f32::from_le_bytes(bytes));
        }

        Ok(vec)
    }

    /// Serialize f32 vector to BLOB
    pub fn from_f32_vec(vec: &[f32]) -> Vec<u8> {
        let mut blob = Vec::with_capacity(vec.len() * 4);
        for &f in vec {
            blob.extend_from_slice(&f.to_le_bytes());
        }
        blob
    }
}

// ============================================================================
// AI MODEL & ADAPTER MODELS
// ============================================================================

#[derive(Debug, Clone, FromRow, Serialize, Deserialize)]
pub struct AiModel {
    pub id: i64,
    pub model_name: String,
    pub model_path: String,
    pub model_size_bytes: Option<i64>,
    pub parameter_count: Option<String>,
    pub quantization: Option<String>,
    pub license: Option<String>,
    pub downloaded_at: Option<String>,
    pub last_used_at: Option<String>,
    pub metadata: Option<String>, // JSON
}

#[derive(Debug, Clone, FromRow)]
pub struct LoraAdapter {
    pub id: i64,
    pub project_id: Option<i64>,
    pub adapter_name: String,
    pub base_model_id: i64,
    pub weights_encrypted: Vec<u8>,
    pub adapter_size_bytes: Option<i64>,
    pub training_config: Option<String>, // JSON
    pub trained_at: String,
    pub quality_metrics: Option<String>, // JSON
    pub is_active: bool,
    pub metadata: Option<String>, // JSON
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrainingConfig {
    pub learning_rate: f32,
    pub epochs: u32,
    pub batch_size: u32,
    pub dataset_size: usize,
    pub lora_rank: u32,
    pub lora_alpha: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QualityMetrics {
    pub perplexity: Option<f32>,
    pub bleu_score: Option<f32>,
    pub style_similarity: Option<f32>,
}

// ============================================================================
// CHAT SESSION & MESSAGE MODELS
// ============================================================================

#[derive(Debug, Clone, FromRow, Serialize, Deserialize)]
pub struct ChatSession {
    pub id: i64,
    pub project_id: i64,
    pub session_name: Option<String>,
    pub created_at: String,
    pub last_message_at: Option<String>,
    pub is_archived: bool,
    pub metadata: Option<String>, // JSON
}

#[derive(Debug, Clone, FromRow)]
pub struct ChatMessage {
    pub id: i64,
    pub session_id: i64,
    pub role: String, // "user", "assistant", "system"
    pub content_encrypted: Vec<u8>,
    pub timestamp: String,
    pub token_count: Option<i64>,
    pub context_refs: Option<String>, // JSON
    pub model_used: Option<String>,
    pub adapter_used: Option<i64>,
    pub metadata: Option<String>, // JSON
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DecryptedChatMessage {
    pub id: i64,
    pub session_id: i64,
    pub role: String,
    pub content: String, // Plaintext
    pub timestamp: String,
    pub token_count: Option<i64>,
    pub context_refs: Vec<ContextRef>,
    pub model_used: Option<String>,
    pub adapter_used: Option<i64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContextRef {
    pub doc_id: i64,
    pub chunk_ids: Vec<i64>,
}

// ============================================================================
// ENCRYPTION KEY MANAGEMENT
// ============================================================================

#[derive(Debug, Clone, FromRow, Serialize, Deserialize)]
pub struct EncryptionKey {
    pub id: i64,
    pub key_purpose: String,
    pub key_identifier: Option<String>,
    pub derivation_info: String, // JSON
    pub created_at: String,
    pub last_used_at: Option<String>,
    pub is_active: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KeyDerivation {
    pub algorithm: String, // "argon2id"
    pub salt: Vec<u8>,
    pub iterations: u32,
    pub memory_kb: u32,
}

// ============================================================================
// CLOUD AUDIT LOG
// ============================================================================

#[derive(Debug, Clone, FromRow, Serialize, Deserialize)]
pub struct CloudAuditLog {
    pub id: i64,
    pub timestamp: String,
    pub operation_type: String,
    pub data_sent_bytes: Option<i64>,
    pub data_summary_encrypted: Option<Vec<u8>>,
    pub user_consent: bool,
    pub request_id: Option<String>,
    pub response_status: Option<i64>,
    pub metadata: Option<String>, // JSON
}

// ============================================================================
// APP SETTINGS
// ============================================================================

#[derive(Debug, Clone, FromRow, Serialize, Deserialize)]
pub struct AppSetting {
    pub key: String,
    pub value: String,
    pub updated_at: String,
}

// ============================================================================
// QUERY HELPERS
// ============================================================================

/// Parameters for creating a new document
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateDocumentParams {
    pub project_id: i64,
    pub title: String,
    pub content: String,
    pub doc_type: Option<String>,
    pub entity_type: Option<String>,
    pub parent_id: Option<i64>,
    pub display_order: Option<i64>,
}

/// Parameters for updating a document
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateDocumentParams {
    pub title: Option<String>,
    pub content: Option<String>,
    pub entity_type: Option<String>,
    pub parent_id: Option<i64>,
    pub display_order: Option<i64>,
    pub metadata: Option<DocumentMetadata>,
}

/// Document list item (for browser views)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DocumentListItem {
    pub id: i64,
    pub title: String,
    pub doc_type: String,
    pub word_count: i64,
    pub updated_at: String,
    pub parent_id: Option<i64>,
    pub display_order: i64,
}

/// Project hierarchy node (for tree views)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectNode {
    pub id: i64,
    pub title: String,
    pub parent_id: Option<i64>,
    pub children: Vec<ProjectNode>,
    pub document_count: i64,
}

// ============================================================================
// ERROR TYPES
// ============================================================================

#[derive(Debug, thiserror::Error)]
pub enum ModelError {
    #[error("Database error: {0}")]
    Database(#[from] sqlx::Error),

    #[error("Encryption error: {0}")]
    Encryption(String),

    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),

    #[error("Not found: {0}")]
    NotFound(String),

    #[error("Invalid data: {0}")]
    InvalidData(String),
}

impl From<String> for ModelError {
    fn from(s: String) -> Self {
        ModelError::InvalidData(s)
    }
}

impl From<&str> for ModelError {
    fn from(s: &str) -> Self {
        ModelError::InvalidData(s.to_string())
    }
}

impl serde::Serialize for ModelError {
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(self.to_string().as_ref())
    }
}

pub type Result<T> = std::result::Result<T, ModelError>;
