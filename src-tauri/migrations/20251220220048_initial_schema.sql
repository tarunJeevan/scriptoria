-- Scriptoria Phase 1 Database Schema
-- SQLite (Standard, not SQLCipher for Phase 1)
-- Version: 1.0.0
-- Last Updated: 2025-12-14

-- ============================================================================
-- METADATA & CONFIGURATION
-- ============================================================================

-- Schema versioning for migrations
CREATE TABLE schema_version (
    version INTEGER PRIMARY KEY,
    applied_at TEXT NOT NULL DEFAULT (datetime('now')),
    description TEXT
);

INSERT INTO schema_version (version, description)
VALUES (1, 'Initial Phase 1 schema');

-- Application settings (non-encrypted metadata)
CREATE TABLE app_settings (
    key TEXT PRIMARY KEY,
    value TEXT NOT NULL,
    updated_at TEXT NOT NULL DEFAULT (datetime('now'))
);

-- Store KDF parameters, salt (plaintext), cipher config
INSERT INTO app_settings (key, value) VALUES
    ('kdf_algorithm', 'argon2id'),
    ('kdf_memory_kb', '65536'),  -- 64MB
    ('kdf_iterations', '3'),
    ('cipher', 'chacha20-poly1305'),
    ('schema_version', '1');

-- ============================================================================
-- USER MANAGEMENT (Phase 1: Single User)
-- ============================================================================

CREATE TABLE users (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    username TEXT NOT NULL UNIQUE,
    -- Master key derived from password via Argon2id (never stored)
    -- Salt stored in system keyring via KeyManager
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    last_login TEXT,
    preferences TEXT  -- JSON blob for UI preferences
);

-- Default user for single-user installations
INSERT INTO users (id, username, preferences)
VALUES (1, 'default_user', '{}');

-- ============================================================================
-- PROJECT MANAGEMENT
-- ============================================================================

CREATE TABLE projects (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    user_id INTEGER NOT NULL,
    title TEXT NOT NULL,
    description TEXT,	-- Optional project summary. NULL = no description
    parent_id INTEGER,	-- Parent project for nested hierarchy (e.g., Series -> Book 1). NULL = top-level project
    display_order INTEGER NOT NULL DEFAULT 0,	-- Display order within parent (for sorting sibling projects)
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at TEXT NOT NULL DEFAULT (datetime('now')),
    archived BOOLEAN NOT NULL DEFAULT 0,
    metadata TEXT,  -- JSON: tags, color, custom fields, project type. NULL = no custom metadata
    FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE CASCADE,
    FOREIGN KEY (parent_id) REFERENCES projects(id) ON DELETE CASCADE
);

CREATE INDEX idx_projects_user ON projects(user_id);
CREATE INDEX idx_projects_parent ON projects(parent_id);
CREATE INDEX idx_projects_archived ON projects(archived);

-- ============================================================================
-- DOCUMENT MANAGEMENT
-- ============================================================================

CREATE TABLE documents (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    project_id INTEGER NOT NULL,
    title TEXT NOT NULL,
    -- Content stored encrypted (Tier 1 data)
    -- Application-level encryption: ChaCha20-Poly1305
    content_encrypted BLOB NOT NULL,
    word_count INTEGER NOT NULL DEFAULT 0,	-- Plaintext metadata for search/sorting (Tier 2)
    char_count INTEGER NOT NULL DEFAULT 0,	-- Plaintext metadata for search/sorting (Tier 2)
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at TEXT NOT NULL DEFAULT (datetime('now')),
    last_edited_at TEXT,	-- Distinct from updated_at for version tracking. NULL = no edits
    doc_type TEXT NOT NULL DEFAULT 'prose',	-- Document type: 'prose', 'outline', 'note' (extensible)
    entity_type TEXT,	-- NULL == regular doc, 'character_stub' || 'timeline_stub' || 'map_stub' == placeholders
    parent_id INTEGER,	-- Parent doc for hierarchical organization (e.g., chapters in book). NULL = top-level doc
    display_order INTEGER NOT NULL DEFAULT 0,	-- Display order within parent/project
    deleted BOOLEAN NOT NULL DEFAULT 0,	-- Soft delete flag
    metadata TEXT,	-- JSON: tags, custom fields, editor state. NULL = no custom metadata
    FOREIGN KEY (project_id) REFERENCES projects(id) ON DELETE CASCADE,
    FOREIGN KEY (parent_id) REFERENCES documents(id) ON DELETE SET NULL
);

CREATE INDEX idx_documents_project ON documents(project_id);
CREATE INDEX idx_documents_parent ON documents(parent_id);
CREATE INDEX idx_documents_updated ON documents(updated_at DESC);
CREATE INDEX idx_documents_deleted ON documents(deleted);

-- ============================================================================
-- DOCUMENT VERSIONING
-- ============================================================================

CREATE TABLE document_versions (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    document_id INTEGER NOT NULL,
    version_number INTEGER NOT NULL,  -- Incremental version counter
    content_encrypted BLOB NOT NULL,
    -- Snapshot metadata
    word_count INTEGER,
    char_count INTEGER,
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    -- Version label (e.g., "Draft 1", "Final", auto-generated timestamp)
    label TEXT,
    -- Diff summary: added/removed lines, change description
    diff_summary TEXT,  -- JSON blob
    -- Optional: store diffs instead of full snapshots (Phase 2 optimization)
    is_delta BOOLEAN NOT NULL DEFAULT 0,
    FOREIGN KEY (document_id) REFERENCES documents(id) ON DELETE CASCADE
);

CREATE INDEX idx_versions_document ON document_versions(document_id);
CREATE INDEX idx_versions_created ON document_versions(created_at DESC);

-- ============================================================================
-- FILE ATTACHMENTS (Encrypted with per-file keys)
-- ============================================================================

CREATE TABLE document_attachments (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    document_id INTEGER NOT NULL,
    filename TEXT NOT NULL,
    -- Encrypted file content (ChaCha20-Poly1305)
    file_encrypted BLOB NOT NULL,
    -- Derived key identifier (hash of master_key + file_hash)
    key_derivation_info TEXT NOT NULL,
    mime_type TEXT,
    file_size INTEGER,  -- Original size before encryption
    uploaded_at TEXT NOT NULL DEFAULT (datetime('now')),
    metadata TEXT,  -- JSON: alt text, captions, tags
    FOREIGN KEY (document_id) REFERENCES documents(id) ON DELETE CASCADE
);

CREATE INDEX idx_attachments_document ON document_attachments(document_id);

-- ============================================================================
-- AI CONTEXT MANAGEMENT
-- ============================================================================

-- Document chunks for AI retrieval (semantic search)
CREATE TABLE ai_context_chunks (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    document_id INTEGER NOT NULL,
    chunk_index INTEGER NOT NULL,  -- Order within document
    -- Chunk strategy: Phase 1 uses 'semantic' (paragraph-based) only
    chunk_strategy TEXT NOT NULL DEFAULT 'semantic',
    -- Encrypted chunk content (Tier 1)
    content_encrypted BLOB NOT NULL,
    -- Plaintext metadata for retrieval logic
    token_count INTEGER,
    char_count INTEGER,
    -- Chunk boundaries (start/end char offsets in original document)
    start_offset INTEGER,
    end_offset INTEGER,
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    -- Flag for stale chunks (document updated since chunk creation)
    is_stale BOOLEAN NOT NULL DEFAULT 0,
    FOREIGN KEY (document_id) REFERENCES documents(id) ON DELETE CASCADE,
    UNIQUE(document_id, chunk_index, chunk_strategy)
);

CREATE INDEX idx_chunks_document ON ai_context_chunks(document_id);
CREATE INDEX idx_chunks_stale ON ai_context_chunks(is_stale);

-- ============================================================================
-- VECTOR EMBEDDINGS (Local FAISS/Chroma integration)
-- ============================================================================

CREATE TABLE vector_embeddings (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    chunk_id INTEGER NOT NULL,
    -- Embedding model identifier (e.g., 'all-MiniLM-L6-v2')
    model_name TEXT NOT NULL,
    model_version TEXT,
    -- Embedding vector stored as BLOB (f32 array serialized)
    embedding_vector BLOB NOT NULL,
    vector_dimension INTEGER NOT NULL,  -- e.g., 384 for MiniLM
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    FOREIGN KEY (chunk_id) REFERENCES ai_context_chunks(id) ON DELETE CASCADE
);

CREATE INDEX idx_embeddings_chunk ON vector_embeddings(chunk_id);
CREATE INDEX idx_embeddings_model ON vector_embeddings(model_name, model_version);

-- ============================================================================
-- AI MODELS & ADAPTERS (LoRA/QLoRA fine-tuning)
-- ============================================================================

CREATE TABLE ai_models (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    model_name TEXT NOT NULL UNIQUE,  -- e.g., 'llama2-7b', 'mistral-7b-instruct'
    model_path TEXT NOT NULL,  -- Local file path or Ollama model ID
    model_size_bytes INTEGER,
    parameter_count TEXT,  -- e.g., '7B'
    quantization TEXT,  -- e.g., 'Q4_K_M', 'Q5_K_S'
    license TEXT,  -- e.g., 'Apache-2.0', 'MIT'
    downloaded_at TEXT,
    last_used_at TEXT,
    metadata TEXT  -- JSON: config, supported features
);

-- LoRA adapters for style fine-tuning
CREATE TABLE lora_adapters (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    project_id INTEGER,  -- Optional: project-specific adapter
    adapter_name TEXT NOT NULL,
    base_model_id INTEGER NOT NULL,
    -- Adapter weights stored as encrypted BLOB
    weights_encrypted BLOB NOT NULL,
    adapter_size_bytes INTEGER,
    training_config TEXT,  -- JSON: learning rate, epochs, dataset info
    trained_at TEXT NOT NULL DEFAULT (datetime('now')),
    quality_metrics TEXT,  -- JSON: perplexity, BLEU, style similarity
    is_active BOOLEAN NOT NULL DEFAULT 0,  -- Only one adapter active per project
    metadata TEXT,
    FOREIGN KEY (project_id) REFERENCES projects(id) ON DELETE CASCADE,
    FOREIGN KEY (base_model_id) REFERENCES ai_models(id) ON DELETE RESTRICT
);

CREATE INDEX idx_adapters_project ON lora_adapters(project_id);
CREATE INDEX idx_adapters_active ON lora_adapters(is_active);

-- ============================================================================
-- CHAT HISTORY (AI Assistant Conversations)
-- ============================================================================

CREATE TABLE chat_sessions (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    project_id INTEGER NOT NULL,
    session_name TEXT,  -- Optional user-defined name
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    last_message_at TEXT,
    is_archived BOOLEAN NOT NULL DEFAULT 0,
    metadata TEXT,  -- JSON: context settings, model used
    FOREIGN KEY (project_id) REFERENCES projects(id) ON DELETE CASCADE
);

CREATE TABLE chat_messages (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    session_id INTEGER NOT NULL,
    -- Role: 'user', 'assistant', 'system'
    role TEXT NOT NULL,
    -- Message content (encrypted for user/assistant messages)
    content_encrypted BLOB NOT NULL,
    timestamp TEXT NOT NULL DEFAULT (datetime('now')),
    -- Token usage for cost tracking (if using cloud APIs)
    token_count INTEGER,
    -- Context used for this message (references to documents/chunks)
    context_refs TEXT,  -- JSON array of {doc_id, chunk_ids}
    -- Model/adapter used for generation
    model_used TEXT,
    adapter_used INTEGER,  -- FK to lora_adapters
    metadata TEXT,  -- JSON: temperature, top_k, max_tokens
    FOREIGN KEY (session_id) REFERENCES chat_sessions(id) ON DELETE CASCADE,
    FOREIGN KEY (adapter_used) REFERENCES lora_adapters(id) ON DELETE SET NULL
);

CREATE INDEX idx_messages_session ON chat_messages(session_id);
CREATE INDEX idx_messages_timestamp ON chat_messages(timestamp DESC);

-- ============================================================================
-- ENCRYPTION KEY MANAGEMENT
-- ============================================================================

-- Key derivation metadata (salt stored in system keyring via KeyManager)
-- Master key never persisted; derived on-demand from user password
CREATE TABLE encryption_keys (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    key_purpose TEXT NOT NULL,  -- 'master', 'document', 'attachment', 'adapter'
    -- For derived keys: identifier (e.g., doc_id + hash)
    key_identifier TEXT UNIQUE,
    -- Key derivation info: parent key, salt, iterations (never stores actual key)
    derivation_info TEXT NOT NULL,  -- JSON
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    last_used_at TEXT,
    -- Flag for key rotation (Phase 3)
    is_active BOOLEAN NOT NULL DEFAULT 1
);

-- ============================================================================
-- AUDIT LOG (Cloud transmission tracking)
-- ============================================================================

CREATE TABLE cloud_audit_log (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    timestamp TEXT NOT NULL DEFAULT (datetime('now')),
    operation_type TEXT NOT NULL,  -- 'ai_inference', 'sync', etc.
    data_sent_bytes INTEGER,
    -- Encrypted summary of transmitted data (never plaintext)
    data_summary_encrypted BLOB,
    user_consent BOOLEAN NOT NULL,  -- Explicit opt-in flag
    request_id TEXT,  -- Random UUID for correlation
    response_status INTEGER,  -- HTTP status or error code
    metadata TEXT  -- JSON: endpoint, duration, error details
);

CREATE INDEX idx_audit_timestamp ON cloud_audit_log(timestamp DESC);
CREATE INDEX idx_audit_operation ON cloud_audit_log(operation_type);

-- ============================================================================
-- PHASE 2 STUBS (Placeholder tables for future features)
-- ============================================================================

-- Notes system (lightweight Markdown notes with bidirectional links)
CREATE TABLE notes (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    project_id INTEGER NOT NULL,
    title TEXT NOT NULL,
    content_encrypted BLOB NOT NULL,
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at TEXT NOT NULL DEFAULT (datetime('now')),
    tags TEXT,  -- JSON array
    metadata TEXT,
    FOREIGN KEY (project_id) REFERENCES projects(id) ON DELETE CASCADE
);

-- Bidirectional links between notes/documents
CREATE TABLE note_links (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    source_id INTEGER NOT NULL,
    target_id INTEGER NOT NULL,
    link_type TEXT,  -- 'reference', 'related', 'child'
    FOREIGN KEY (source_id) REFERENCES notes(id) ON DELETE CASCADE,
    FOREIGN KEY (target_id) REFERENCES notes(id) ON DELETE CASCADE,
    UNIQUE(source_id, target_id)
);

-- ============================================================================
-- TRIGGERS FOR AUTO-UPDATE TIMESTAMPS
-- ============================================================================

-- Update projects.updated_at on document changes
CREATE TRIGGER update_project_timestamp
AFTER UPDATE ON documents
FOR EACH ROW
BEGIN
    UPDATE projects
    SET updated_at = datetime('now')
    WHERE id = NEW.project_id;
END;

-- Update documents.updated_at on content changes
CREATE TRIGGER update_document_timestamp
BEFORE UPDATE ON documents
FOR EACH ROW
WHEN NEW.content_encrypted != OLD.content_encrypted
BEGIN
    UPDATE documents
    SET updated_at = datetime('now'),
        last_edited_at = datetime('now')
    WHERE id = NEW.id;
END;

-- Mark chunks as stale when document is updated
CREATE TRIGGER mark_chunks_stale
AFTER UPDATE ON documents
FOR EACH ROW
WHEN NEW.content_encrypted != OLD.content_encrypted
BEGIN
    UPDATE ai_context_chunks
    SET is_stale = 1
    WHERE document_id = NEW.id;
END;
