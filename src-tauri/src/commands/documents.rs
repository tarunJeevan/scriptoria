// src-tauri/src/commands/documents.rs

use crate::db::documents::DocumentRepository;
use crate::encryption::EncryptionService;
use crate::models::{
    CreateDocumentParams, DecryptedDocument, DocumentListItem, DocumentMetadata, DocumentVersion,
    UpdateDocumentParams,
};

use sqlx::SqlitePool;

use tauri::State;

// =========================================================================
// ERROR TYPES
// =========================================================================

#[derive(Debug, thiserror::Error)]
pub enum CommandError {
    #[error("Database error: {0}")]
    Database(#[from] sqlx::Error),

    #[error("Model error: {0}")]
    Model(#[from] crate::models::ModelError),

    #[error("Encryption error: {0}")]
    Encryption(String),

    #[error("Validation error: {0}")]
    Validation(String),

    #[error("Not found: {0}")]
    NotFound(String),
}

impl serde::Serialize for CommandError {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(&self.to_string())
    }
}

type CommandResult<T> = Result<T, CommandError>;

// =========================================================================
// APP STATE (Managed by Tauri)
// =========================================================================

pub struct AppState {
    pub pool: SqlitePool,
    pub encryption: EncryptionService,
}

// =========================================================================
// TAURI COMMANDS
// =========================================================================

/// Create a new document
#[tauri::command]
pub async fn create_document(
    project_id: i64,
    title: String,
    content: String,
    doc_type: Option<String>,
    entity_type: Option<String>,
    parent_id: Option<i64>,
    state: State<'_, AppState>,
) -> CommandResult<DecryptedDocument> {
    // Validation
    if title.trim().is_empty() {
        return Err(CommandError::Validation(
            "Title cannot be empty".to_string(),
        ));
    }

    if title.len() > 500 {
        return Err(CommandError::Validation(
            "Title too long (max 500 characters".to_string(),
        ));
    }

    // Document creation
    let repo = DocumentRepository::new(state.pool.clone());

    let params = CreateDocumentParams {
        project_id,
        title,
        content,
        doc_type,
        entity_type,
        parent_id,
        display_order: None, // Auto-assign based on sibling count
    };

    let doc = repo.create(params, &state.encryption).await?;

    Ok(doc)
}

/// Read a document by ID
#[tauri::command]
pub async fn read_document(
    document_id: i64,
    state: State<'_, AppState>,
) -> CommandResult<DecryptedDocument> {
    let repo = DocumentRepository::new(state.pool.clone());
    let doc = repo.get_by_id(document_id, &state.encryption).await?;

    Ok(doc)
}

/// Update a document
#[allow(clippy::too_many_arguments)]
#[tauri::command]
pub async fn update_document(
    document_id: i64,
    title: Option<String>,
    content: Option<String>,
    entity_type: Option<String>,
    parent_id: Option<i64>,
    display_order: Option<i64>,
    metadata: Option<DocumentMetadata>,
    state: State<'_, AppState>,
) -> CommandResult<DecryptedDocument> {
    // Validation
    if let Some(ref t) = title {
        if t.trim().is_empty() {
            return Err(CommandError::Validation(
                "Title cannot be empty".to_string(),
            ));
        }
        if t.len() > 500 {
            return Err(CommandError::Validation(
                "Title too long (max 500 characters".to_string(),
            ));
        }
    }

    // Document update
    let repo = DocumentRepository::new(state.pool.clone());

    let params = UpdateDocumentParams {
        title,
        content,
        entity_type,
        parent_id,
        display_order,
        metadata,
    };

    let doc = repo.update(document_id, params, &state.encryption).await?;

    Ok(doc)
}

/// Soft delete a document
#[tauri::command]
pub async fn delete_document(document_id: i64, state: State<'_, AppState>) -> CommandResult<()> {
    let repo = DocumentRepository::new(state.pool.clone());
    repo.delete(document_id).await?;

    Ok(())
}

/// List all documents in a project
#[tauri::command]
pub async fn list_documents(
    project_id: i64,
    state: State<'_, AppState>,
) -> CommandResult<Vec<DocumentListItem>> {
    let repo = DocumentRepository::new(state.pool.clone());
    let docs = repo.list_by_project(project_id).await?;

    Ok(docs)
}

/// List documents by entity type
#[tauri::command]
pub async fn list_documents_by_type(
    project_id: i64,
    entity_type: Option<String>,
    state: State<'_, AppState>,
) -> CommandResult<Vec<DocumentListItem>> {
    let repo = DocumentRepository::new(state.pool.clone());
    let docs = repo
        .list_by_entity_type(project_id, entity_type.as_deref())
        .await?;

    Ok(docs)
}

/// Count documents by entity type
#[tauri::command]
pub async fn count_documents_by_type(
    project_id: i64,
    entity_type: Option<String>,
    state: State<'_, AppState>,
) -> CommandResult<i64> {
    let repo = DocumentRepository::new(state.pool.clone());
    let count = repo
        .count_by_entity_type(project_id, entity_type.as_deref())
        .await?;

    Ok(count)
}

// =========================================================================
// VERSION MANAGEMENT COMMANDS
// =========================================================================

/// Create a version snapshot of a document
#[tauri::command]
pub async fn create_version(
    document_id: i64,
    label: Option<String>,
    state: State<'_, AppState>,
) -> CommandResult<i64> {
    let repo = DocumentRepository::new(state.pool.clone());
    let version_id = repo
        .create_version(document_id, label, &state.encryption)
        .await?;

    Ok(version_id)
}

/// List all versions for a document
#[tauri::command]
pub async fn list_versions(
    document_id: i64,
    state: State<'_, AppState>,
) -> CommandResult<Vec<DocumentVersion>> {
    let repo = DocumentRepository::new(state.pool.clone());
    let versions = repo.list_versions(document_id).await?;

    Ok(versions)
}

/// Get a specific version (with decrypted document)
#[tauri::command]
pub async fn get_version(
    version_id: i64,
    state: State<'_, AppState>,
) -> CommandResult<(DocumentVersion, String)> {
    let repo = DocumentRepository::new(state.pool.clone());
    let result = repo.get_version(version_id, &state.encryption).await?;

    Ok(result)
}

// =========================================================================
// UTILITY COMMANDS
// =========================================================================

/// Restore a document from a version
#[tauri::command]
pub async fn restore_from_version(
    document_id: i64,
    version_id: i64,
    state: State<'_, AppState>,
) -> CommandResult<DecryptedDocument> {
    let repo = DocumentRepository::new(state.pool.clone());

    // Get version content
    let (_version, content) = repo.get_version(version_id, &state.encryption).await?;

    // Update document with version content
    let params = UpdateDocumentParams {
        title: None,
        content: Some(content),
        entity_type: None,
        parent_id: None,
        display_order: None,
        metadata: None,
    };

    let doc = repo.update(document_id, params, &state.encryption).await?;

    Ok(doc)
}

/// Permanently delete a document (bypassing soft delete)
#[tauri::command]
pub async fn permanently_delete_document(
    document_id: i64,
    state: State<'_, AppState>,
) -> CommandResult<()> {
    let result = sqlx::query!("DELETE FROM documents WHERE id = ?", document_id)
        .execute(&state.pool)
        .await?;

    if result.rows_affected() == 0 {
        return Err(CommandError::NotFound(format!(
            "Document {} not found",
            document_id
        )));
    }

    Ok(())
}
