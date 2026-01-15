// Document repository with encryption integration

use crate::encryption::EncryptionService;
use crate::models::{
    CreateDocumentParams, DecryptedDocument, Document, DocumentListItem, DocumentMetadata,
    DocumentVersion, EncryptedContent, ModelError, Result, UpdateDocumentParams,
};
use sqlx::SqlitePool;

// ============================================================================
// DOCUMENT REPOSITORY
// ============================================================================

#[derive(Clone)]
pub struct DocumentRepository {
    pool: SqlitePool,
}

impl DocumentRepository {
    pub fn new(pool: SqlitePool) -> Self {
        Self { pool }
    }

    /// Create a new document with encrypted content
    pub async fn create(
        &self,
        params: CreateDocumentParams,
        encryption: &EncryptionService,
    ) -> Result<DecryptedDocument> {
        let encrypted = encryption
            .encrypt(&params.content)
            .map_err(|e| ModelError::Encryption(e.to_string()))?;

        let content_blob = encrypted.to_blob();

        let word_count = params.content.split_whitespace().count() as i64;
        let char_count = params.content.chars().count() as i64;

        let doc_type = params.doc_type.unwrap_or_else(|| "prose".to_string());
        let display_order = params.display_order.unwrap_or(0_i64);

        let metadata = DocumentMetadata::default();
        let metadata_json = serde_json::to_string(&metadata)?;

        let id = sqlx::query!(
            r#"
            INSERT INTO documents (
            	project_id, title, content_encrypted, word_count, char_count,
             	doc_type, entity_type, parent_id, display_order, metadata
            )
            VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
            "#,
            params.project_id,
            params.title,
            content_blob,
            word_count,
            char_count,
            doc_type,
            params.entity_type,
            params.parent_id,
            display_order,
            metadata_json,
        )
        .execute(&self.pool)
        .await?
        .last_insert_rowid();

        // Fetch the created document
        self.get_by_id(id, encryption).await
    }

    /// Get document by ID with decrypted content
    pub async fn get_by_id(
        &self,
        id: i64,
        encryption: &EncryptionService,
    ) -> Result<DecryptedDocument> {
        let doc = sqlx::query_as!(
            Document,
            r#"
            SELECT *
            FROM documents
            WHERE id = ? AND deleted = 0
            "#,
            id
        )
        .fetch_optional(&self.pool)
        .await?
        .ok_or_else(|| ModelError::NotFound(format!("Document {}", id)))?;

        self.decrypt_document(doc, encryption)
    }

    /// Update document content and/or metadata
    pub async fn update(
        &self,
        id: i64,
        params: UpdateDocumentParams,
        encryption: &EncryptionService,
    ) -> Result<DecryptedDocument> {
        let mut conn = self.pool.acquire().await?;

        // Fetch current document
        let current = sqlx::query_as!(
            Document,
            "SELECT * FROM documents WHERE id = ? AND deleted = 0",
            id
        )
        .fetch_optional(&mut *conn)
        .await?
        .ok_or_else(|| ModelError::NotFound(format!("Document {}", id)))?;

        // Determine what to update
        let content_blob = if let Some(new_content) = &params.content {
            let encrypted = encryption
                .encrypt(new_content)
                .map_err(|e| ModelError::Encryption(e.to_string()))?;
            Some(encrypted.to_blob())
        } else {
            None
        };

        let title = params.title.as_ref().unwrap_or(&current.title);
        let entity_type = params.entity_type.or(current.entity_type);
        let parent_id = params.parent_id.or(current.parent_id);
        let display_order = params.display_order.unwrap_or(current.display_order);

        let metadata_json = if let Some(meta) = params.metadata {
            serde_json::to_string(&meta)?
        } else {
            current.metadata.unwrap_or_else(|| "{}".to_string())
        };

        if let Some(blob) = content_blob {
            let word_count = params
                .content
                .as_ref()
                .map(|c| c.split_whitespace().count() as i64)
                .unwrap_or(current.word_count);

            let char_count = params
                .content
                .as_ref()
                .map(|c| c.chars().count() as i64)
                .unwrap_or(current.char_count);

            sqlx::query!(
                r#"
             	UPDATE documents
                SET title = ?, content_encrypted = ?, word_count = ?, char_count = ?,
                    entity_type = ?, parent_id = ?, display_order = ?, metadata = ?
                WHERE id = ?
                "#,
                title,
                blob,
                word_count,
                char_count,
                entity_type,
                parent_id,
                display_order,
                metadata_json,
                id,
            )
            .execute(&mut *conn)
            .await?;
        } else {
            // Update only metadata, no content change
            sqlx::query!(
                r#"
                UPDATE documents
                SET title = ?, entity_type = ?, parent_id = ?, display_order = ?, metadata = ?
                WHERE id = ?
                "#,
                title,
                entity_type,
                parent_id,
                display_order,
                metadata_json,
                id,
            )
            .execute(&mut *conn)
            .await?;
        }

        drop(conn);
        self.get_by_id(id, encryption).await
    }

    /// Soft delete a document
    pub async fn delete(&self, id: i64) -> Result<()> {
        let result = sqlx::query!("UPDATE documents SET deleted = 1 WHERE id = ?", id)
            .execute(&self.pool)
            .await?;

        if result.rows_affected() == 0 {
            return Err(ModelError::NotFound(format!("Document {}", id)));
        }

        Ok(())
    }

    /// List all documents in a project
    pub async fn list_by_project(&self, project_id: i64) -> Result<Vec<DocumentListItem>> {
        // Query all relevant rows
        let rows = sqlx::query!(
            r#"
        SELECT *
        FROM documents
        WHERE project_id = ? AND deleted = 0
        ORDER BY parent_id, display_order, title
        "#,
            project_id
        )
        .fetch_all(&self.pool)
        .await?;

        // Map query result to a vector of DocumentListItems
        let items: Vec<DocumentListItem> = rows
            .into_iter()
            .map(|row| DocumentListItem {
                id: row.id.unwrap(),
                title: row.title,
                doc_type: row.doc_type,
                word_count: row.word_count,
                updated_at: row.updated_at,
                parent_id: row.parent_id,
                display_order: row.display_order,
            })
            .collect();

        Ok(items)
    }

    /// Create a new version snapshot
    pub async fn create_version(
        &self,
        document_id: i64,
        label: Option<String>,
        _encryption: &EncryptionService,
    ) -> Result<i64> {
        let mut conn = self.pool.acquire().await?;

        // Get current document
        let doc = sqlx::query_as!(
            Document,
            "SELECT * FROM documents WHERE id = ? AND deleted = 0",
            document_id
        )
        .fetch_optional(&mut *conn)
        .await?
        .ok_or_else(|| ModelError::NotFound(format!("Document {}", document_id)))?;

        // Get next version number
        let version_number: i64 = sqlx::query_scalar!(
            "SELECT COALESCE(MAX(version_number), 0) + 1 FROM document_versions WHERE document_id = ?",
            document_id
        )
        .fetch_one(&mut *conn)
        .await?;

        // Insert version
        let id = sqlx::query!(
            r#"
            INSERT INTO document_versions (
                document_id, version_number, content_encrypted,
                word_count, char_count, label
            )
            VALUES (?, ?, ?, ?, ?, ?)
            "#,
            document_id,
            version_number,
            doc.content_encrypted,
            doc.word_count,
            doc.char_count,
            label,
        )
        .execute(&mut *conn)
        .await?
        .last_insert_rowid();

        Ok(id)
    }

    /// List all versions for a document
    pub async fn list_versions(&self, document_id: i64) -> Result<Vec<DocumentVersion>> {
        let versions = sqlx::query_as!(
            DocumentVersion,
            r#"
            SELECT * FROM document_versions
            WHERE (document_id = ? OR (document_id IS NULL AND ? IS NULL))
            ORDER BY version_number DESC
            "#,
            document_id,
            document_id
        )
        .fetch_all(&self.pool)
        .await?;

        Ok(versions)
    }

    /// Get specific version
    pub async fn get_version(
        &self,
        version_id: i64,
        encryption: &EncryptionService,
    ) -> Result<(DocumentVersion, String)> {
        let version = sqlx::query_as!(
            DocumentVersion,
            "SELECT * FROM document_versions WHERE id = ?",
            version_id
        )
        .fetch_optional(&self.pool)
        .await?
        .ok_or_else(|| ModelError::NotFound(format!("Version {}", version_id)))?;

        let encrypted = EncryptedContent::from_blob(&version.content_encrypted)
            .map_err(|e| ModelError::InvalidData(e.to_string()))?;

        let content = encryption
            .decrypt(&encrypted)
            .map_err(|e| ModelError::Encryption(e.to_string()))?;

        Ok((version, content))
    }

    /// Helper: Decrypt a document
    fn decrypt_document(
        &self,
        doc: Document,
        encryption: &EncryptionService,
    ) -> Result<DecryptedDocument> {
        let encrypted = EncryptedContent::from_blob(&doc.content_encrypted)
            .map_err(|e| ModelError::InvalidData(e.to_string()))?;

        let content = encryption
            .decrypt(&encrypted)
            .map_err(|e| ModelError::Encryption(e.to_string()))?;

        let metadata: DocumentMetadata = doc
            .metadata
            .as_ref()
            .and_then(|m| serde_json::from_str(m).ok())
            .unwrap_or_default();

        Ok(DecryptedDocument {
            id: doc.id,
            project_id: doc.project_id,
            title: doc.title,
            content,
            word_count: doc.word_count,
            char_count: doc.char_count,
            created_at: doc.created_at,
            updated_at: doc.updated_at,
            last_edited_at: doc.last_edited_at,
            doc_type: doc.doc_type,
            entity_type: doc.entity_type,
            parent_id: doc.parent_id,
            display_order: doc.display_order,
            deleted: doc.deleted,
            metadata,
        })
    }

    /// List documents by entity type
    pub async fn list_by_entity_type(
        &self,
        project_id: i64,
        entity_type: Option<&str>,
    ) -> Result<Vec<DocumentListItem>> {
        // Query all relevant rows
        let rows = sqlx::query!(
        r#"
        SELECT *
        FROM documents
        WHERE project_id = ? AND deleted = 0 AND (entity_type = ? OR (entity_type IS NULL AND ? IS NULL))
        ORDER BY parent_id, display_order, title
        "#,
        project_id,
        entity_type,
        entity_type
        )
        .fetch_all(&self.pool)
        .await?;

        // Map query results into a vector of DocumentListItems
        let items = rows
            .into_iter()
            .map(|row| DocumentListItem {
                id: row.id.unwrap(),
                title: row.title,
                doc_type: row.doc_type,
                word_count: row.word_count,
                updated_at: row.updated_at,
                parent_id: row.parent_id,
                display_order: row.display_order,
            })
            .collect();

        Ok(items)
    }

    /// Count documents by entity type
    pub async fn count_by_entity_type(
        &self,
        project_id: i64,
        entity_type: Option<&str>,
    ) -> Result<i64> {
        let count = sqlx::query_scalar!(
            r#"
            SELECT COUNT(*) as "count: i64"
            FROM documents
            WHERE project_id = ?
                AND deleted = 0
                AND (entity_type = ? OR (entity_type IS NULL AND ? IS NULL))
            "#,
            project_id,
            entity_type,
            entity_type,
        )
        .fetch_one(&self.pool)
        .await?;

        Ok(count)
    }
}

// ============================================================================
// TESTS
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use crate::encryption::EncryptionService;

    async fn setup_test_db() -> SqlitePool {
        let pool = SqlitePool::connect(":memory:").await.unwrap();

        // Run migrations (simplified for test)
        sqlx::query(include_str!(
            "../../migrations/20251220220048_initial_schema.sql"
        ))
        .execute(&pool)
        .await
        .unwrap();

        pool
    }

    /// Helper function to create a test project
    async fn create_test_project(pool: &SqlitePool) -> i64 {
        sqlx::query!(
            r#"
            INSERT INTO projects (user_id, title, description)
            VALUES (1, 'Test Project', 'A test project for unit tests')
            "#
        )
        .execute(pool)
        .await
        .unwrap()
        .last_insert_rowid()
    }

    #[tokio::test]
    async fn test_create_and_retrieve_document() {
        let pool = setup_test_db().await;
        let project_id = create_test_project(&pool).await;
        let repo = DocumentRepository::new(pool);
        let encryption = EncryptionService::new([0u8; 32]);

        let params = CreateDocumentParams {
            project_id,
            title: "Test Document".to_string(),
            content: "Hello, world!".to_string(),
            doc_type: None,
            entity_type: None, // Regular document
            parent_id: None,
            display_order: None,
        };

        let doc = repo.create(params, &encryption).await.unwrap();
        assert_eq!(doc.title, "Test Document");
        assert_eq!(doc.content, "Hello, world!");
        assert_eq!(doc.word_count, 2);

        let retrieved = repo.get_by_id(doc.id, &encryption).await.unwrap();
        assert_eq!(retrieved.content, doc.content);
    }

    #[tokio::test]
    async fn test_update_document() {
        let pool = setup_test_db().await;
        let project_id = create_test_project(&pool).await;
        let repo = DocumentRepository::new(pool);
        let encryption = EncryptionService::new([1u8; 32]);

        let params = CreateDocumentParams {
            project_id,
            title: "Original".to_string(),
            content: "Original content".to_string(),
            doc_type: None,
            entity_type: None, // Regular document
            parent_id: None,
            display_order: None,
        };

        let doc = repo.create(params, &encryption).await.unwrap();

        let update_params = UpdateDocumentParams {
            title: Some("Updated".to_string()),
            content: Some("Updated content".to_string()),
            entity_type: None,
            parent_id: None,
            display_order: None,
            metadata: None,
        };

        let updated = repo
            .update(doc.id, update_params, &encryption)
            .await
            .unwrap();
        assert_eq!(updated.title, "Updated");
        assert_eq!(updated.content, "Updated content");
    }

    #[tokio::test]
    async fn test_version_creation() {
        let pool = setup_test_db().await;
        let project_id = create_test_project(&pool).await;
        let repo = DocumentRepository::new(pool);
        let encryption = EncryptionService::new([2u8; 32]);

        let params = CreateDocumentParams {
            project_id,
            title: "Versioned Doc".to_string(),
            content: "Version 1".to_string(),
            doc_type: None,
            entity_type: None,
            parent_id: None,
            display_order: None,
        };

        let doc = repo.create(params, &encryption).await.unwrap();

        // Create first version
        let v1_id = repo
            .create_version(doc.id, Some("Draft 1".to_string()), &encryption)
            .await
            .unwrap();

        // Update document
        let update = UpdateDocumentParams {
            title: None,
            content: Some("Version 2".to_string()),
            entity_type: None,
            parent_id: None,
            display_order: None,
            metadata: None,
        };
        repo.update(doc.id, update, &encryption).await.unwrap();

        // Create second version
        repo.create_version(doc.id, Some("Draft 2".to_string()), &encryption)
            .await
            .unwrap();

        // List versions
        let versions = repo.list_versions(doc.id).await.unwrap();
        assert_eq!(versions.len(), 2);
        assert_eq!(versions[0].label, Some("Draft 2".to_string()));
        assert_eq!(versions[1].label, Some("Draft 1".to_string()));

        // Retrieve specific version
        let (_version, content) = repo.get_version(v1_id, &encryption).await.unwrap();
        assert_eq!(content, "Version 1");
    }
}
