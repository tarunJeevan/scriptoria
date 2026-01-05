// Integration tests for Scriptoria Chunk 0
// Tests encryption + database + document repository working together

use scriptoria_lib::db::documents::DocumentRepository;
use scriptoria_lib::encryption::{EncryptionService, KeyManager, PasswordValidator};
use scriptoria_lib::models::{CreateDocumentParams, DocumentMetadata, UpdateDocumentParams};

use sqlx::SqlitePool;
use std::fs;
use std::path::PathBuf;

// ============================================================================
// TEST SETUP HELPERS
// ============================================================================

/// Create a temporary test database with schema applied
async fn setup_test_db() -> (SqlitePool, PathBuf) {
    let temp_dir = std::env::temp_dir();
    let db_path = temp_dir.join(format!("scriptoria_test_{}.db", uuid::Uuid::new_v4()));

    // Create database file if it doesn't already exist
    let options = sqlx::sqlite::SqliteConnectOptions::new()
        .filename(db_path.clone())
        .create_if_missing(true)
        .foreign_keys(true);

    // Create database connection
    let pool = SqlitePool::connect_with(options).await.unwrap();

    // Apply schema
    let schema = include_str!("../migrations/20251220220048_initial_schema.sql");
    sqlx::query(schema).execute(&pool).await.unwrap();

    // Insert initial parent rows required by tests to avoid Foreign Key (FK) errors
    sqlx::query!(
    r#"
    INSERT OR IGNORE INTO projects (id, user_id, title, created_at, updated_at, display_order, archived, metadata)
    VALUES (1, 1, 'Default Project 1', datetime('now'), datetime('now'), 0, 0, '{}')
    "#
    )
    .execute(&pool)
    .await
    .unwrap();

    sqlx::query!(
    r#"
    INSERT OR IGNORE INTO projects (id, user_id, title, created_at, updated_at, display_order, archived, metadata)
    VALUES (2, 1, 'Default Project 2', datetime('now'), datetime('now'), 0, 0, '{}')
    "#
    )
    .execute(&pool)
    .await
    .unwrap();

    (pool, db_path)
}

/// Cleanup test database
async fn cleanup_test_db(pool: SqlitePool, db_path: PathBuf) {
    pool.close().await;
    let _ = fs::remove_file(&db_path);
    let _ = fs::remove_file(db_path.with_extension("db-shm"));
    let _ = fs::remove_file(db_path.with_extension("db-wal"));
}

// ============================================================================
// INTEGRATION TESTS
// ============================================================================

fn gen_salt_with_retry(max_attempts: usize) -> Vec<u8> {
    for attempt in 1..=max_attempts {
        match EncryptionService::generate_salt() {
            Ok(salt) => return salt,
            Err(e) => {
                eprintln!("generate_salt attempt {attempt}/{max_attempts} failed: {e}");
                // Pause for 50 milliseconds before trying again
                if attempt < max_attempts {
                    std::thread::sleep(std::time::Duration::from_millis(50));
                }
            }
        }
    }
    panic!("Failed to generate salt after {max_attempts} attempts");
}

#[tokio::test]
async fn test_end_to_end_document_lifecycle() {
    let (pool, db_path) = setup_test_db().await;
    let repo = DocumentRepository::new(pool.clone());

    // Derive encryption key from password
    let password = "Test_Password_123!";
    let salt = gen_salt_with_retry(3);
    let master_key = EncryptionService::derive_master_key(password, &salt).unwrap();
    let encryption = EncryptionService::new(master_key);

    // Create document
    let create_params = CreateDocumentParams {
        project_id: 1,
        title: "My Secret Novel".to_string(),
        content: "This is the beginning of a great story...".to_string(),
        doc_type: Some("prose".to_string()),
        entity_type: None,
        parent_id: None,
        display_order: Some(0),
    };

    let doc = repo.create(create_params, &encryption).await.unwrap();
    assert_eq!(doc.title, "My Secret Novel");
    assert_eq!(doc.content, "This is the beginning of a great story...");
    assert_eq!(doc.word_count, 8);

    // Retrieve document
    let retrieved = repo.get_by_id(doc.id, &encryption).await.unwrap();
    assert_eq!(retrieved.content, doc.content);

    // Update document
    let update_params = UpdateDocumentParams {
        title: Some("My Secret Novel - Revised".to_string()),
        content: Some("This is the revised beginning of an epic tale...".to_string()),
        entity_type: None,
        parent_id: None,
        display_order: None,
        metadata: None,
    };

    let updated = repo
        .update(doc.id, update_params, &encryption)
        .await
        .unwrap();
    assert_eq!(updated.title, "My Secret Novel - Revised");
    assert_eq!(
        updated.content,
        "This is the revised beginning of an epic tale..."
    );
    assert_eq!(updated.word_count, 9);

    // Delete document
    repo.delete(doc.id).await.unwrap();

    // Verify deleted
    let deleted_result = repo.get_by_id(doc.id, &encryption).await;
    assert!(deleted_result.is_err());

    cleanup_test_db(pool, db_path).await;
}

#[tokio::test]
async fn test_versioning_with_encryption() {
    let (pool, db_path) = setup_test_db().await;
    let repo = DocumentRepository::new(pool.clone());

    let password = "Secure_Pass_456!";
    let salt = gen_salt_with_retry(3);
    let master_key = EncryptionService::derive_master_key(password, &salt).unwrap();
    let encryption = EncryptionService::new(master_key);

    // Create initial document
    let create_params = CreateDocumentParams {
        project_id: 1,
        title: "Versioned Document".to_string(),
        content: "Version 1 content".to_string(),
        doc_type: None,
        entity_type: None,
        parent_id: None,
        display_order: None,
    };

    let doc = repo.create(create_params, &encryption).await.unwrap();

    // Create version 1
    let v1_id = repo
        .create_version(doc.id, Some("Draft 1".to_string()), &encryption)
        .await
        .unwrap();

    // Update document
    let update1 = UpdateDocumentParams {
        title: None,
        content: Some("Version 2 content".to_string()),
        entity_type: None,
        parent_id: None,
        display_order: None,
        metadata: None,
    };
    repo.update(doc.id, update1, &encryption).await.unwrap();

    // Create version 2
    let v2_id = repo
        .create_version(doc.id, Some("Draft 2".to_string()), &encryption)
        .await
        .unwrap();

    // Update document again
    let update2 = UpdateDocumentParams {
        title: None,
        content: Some("Version 3 content".to_string()),
        entity_type: None,
        parent_id: None,
        display_order: None,
        metadata: None,
    };
    repo.update(doc.id, update2, &encryption).await.unwrap();

    // List all versions
    let versions = repo.list_versions(doc.id).await.unwrap();
    assert_eq!(versions.len(), 2);
    assert_eq!(versions[0].label, Some("Draft 2".to_string()));
    assert_eq!(versions[1].label, Some("Draft 1".to_string()));

    // Retrieve specific versions
    let (_, v1_content) = repo.get_version(v1_id, &encryption).await.unwrap();
    assert_eq!(v1_content, "Version 1 content");

    let (_, v2_content) = repo.get_version(v2_id, &encryption).await.unwrap();
    assert_eq!(v2_content, "Version 2 content");

    // Current document should have latest content
    let current = repo.get_by_id(doc.id, &encryption).await.unwrap();
    assert_eq!(current.content, "Version 3 content");

    cleanup_test_db(pool, db_path).await;
}

#[tokio::test]
async fn test_wrong_password_fails_decryption() {
    let (pool, db_path) = setup_test_db().await;
    let repo = DocumentRepository::new(pool.clone());

    // Create document with password 1
    let password1 = "Correct_Password_789!";
    let salt = gen_salt_with_retry(3);
    let master_key1 = EncryptionService::derive_master_key(password1, &salt).unwrap();
    let encryption1 = EncryptionService::new(master_key1);

    let create_params = CreateDocumentParams {
        project_id: 1,
        title: "Secret Document".to_string(),
        content: "Top secret information".to_string(),
        doc_type: None,
        entity_type: None,
        parent_id: None,
        display_order: None,
    };

    let doc = repo.create(create_params, &encryption1).await.unwrap();

    // Try to decrypt with wrong password
    let password2 = "Wrong_Password_999!";
    let master_key2 = EncryptionService::derive_master_key(password2, &salt).unwrap();
    let encryption2 = EncryptionService::new(master_key2);

    let decrypt_result = repo.get_by_id(doc.id, &encryption2).await;
    assert!(decrypt_result.is_err());

    cleanup_test_db(pool, db_path).await;
}

#[tokio::test]
async fn test_multiple_projects_and_documents() {
    let (pool, db_path) = setup_test_db().await;
    let repo = DocumentRepository::new(pool.clone());

    let password = "Multi_Project_Pass!";
    let salt = gen_salt_with_retry(3);
    let master_key = EncryptionService::derive_master_key(password, &salt).unwrap();
    let encryption = EncryptionService::new(master_key);

    // Create documents in project 1
    for i in 1..=3 {
        let params = CreateDocumentParams {
            project_id: 1,
            title: format!("Project 1 - Document {}", i),
            content: format!("Content for document {}", i),
            doc_type: None,
            entity_type: None,
            parent_id: None,
            display_order: Some(i as i64),
        };
        repo.create(params, &encryption).await.unwrap();
    }

    // Create documents in project 2
    for i in 1..=2 {
        let params = CreateDocumentParams {
            project_id: 2,
            title: format!("Project 2 - Document {}", i),
            content: format!("Content for document {}", i),
            doc_type: None,
            entity_type: None,
            parent_id: None,
            display_order: Some(i as i64),
        };
        repo.create(params, &encryption).await.unwrap();
    }

    // List documents by project
    let project1_docs = repo.list_by_project(1).await.unwrap();
    assert_eq!(project1_docs.len(), 3);
    assert_eq!(project1_docs[0].title, "Project 1 - Document 1");

    let project2_docs = repo.list_by_project(2).await.unwrap();
    assert_eq!(project2_docs.len(), 2);
    assert_eq!(project2_docs[0].title, "Project 2 - Document 1");

    cleanup_test_db(pool, db_path).await;
}

#[tokio::test]
async fn test_entity_type_filtering() {
    let (pool, db_path) = setup_test_db().await;
    let repo = DocumentRepository::new(pool.clone());

    let password = "Entity_Type_Test!";
    let salt = gen_salt_with_retry(3);
    let master_key = EncryptionService::derive_master_key(password, &salt).unwrap();
    let encryption = EncryptionService::new(master_key);

    // Create regular documents
    for i in 1..=2 {
        let params = CreateDocumentParams {
            project_id: 1,
            title: format!("Regular Doc {}", i),
            content: format!("Regular content {}", i),
            doc_type: Some("prose".to_string()),
            entity_type: None,
            parent_id: None,
            display_order: Some(i as i64),
        };
        repo.create(params, &encryption).await.unwrap();
    }

    // Create character stubs
    for i in 1..=3 {
        let params = CreateDocumentParams {
            project_id: 1,
            title: format!("Character {}", i),
            content: format!("Character bio {}", i),
            doc_type: Some("character".to_string()),
            entity_type: Some("character_stub".to_string()),
            parent_id: None,
            display_order: Some(i as i64),
        };
        repo.create(params, &encryption).await.unwrap();
    }

    // List regular documents (entity_type = NULL)
    let regular_docs = repo.list_by_entity_type(1, None).await.unwrap();
    assert_eq!(regular_docs.len(), 2);
    assert!(regular_docs.iter().all(|d| d.title.starts_with("Regular")));

    // List character stubs
    let char_stubs = repo
        .list_by_entity_type(1, Some("character_stub"))
        .await
        .unwrap();
    assert_eq!(char_stubs.len(), 3);
    assert!(char_stubs.iter().all(|d| d.title.starts_with("Character")));

    // Count by type
    let regular_count = repo.count_by_entity_type(1, None).await.unwrap();
    assert_eq!(regular_count, 2);

    let char_count = repo
        .count_by_entity_type(1, Some("character_stub"))
        .await
        .unwrap();
    assert_eq!(char_count, 3);

    cleanup_test_db(pool, db_path).await;
}

#[tokio::test]
async fn test_hierarchical_documents() {
    let (pool, db_path) = setup_test_db().await;
    let repo = DocumentRepository::new(pool.clone());

    let password = "Hierarchy_Test!";
    let salt = gen_salt_with_retry(3);
    let master_key = EncryptionService::derive_master_key(password, &salt).unwrap();
    let encryption = EncryptionService::new(master_key);

    // Create parent document (book)
    let book_params = CreateDocumentParams {
        project_id: 1,
        title: "My Novel".to_string(),
        content: "Book overview".to_string(),
        doc_type: Some("prose".to_string()),
        entity_type: None,
        parent_id: None,
        display_order: Some(0),
    };
    let book = repo.create(book_params, &encryption).await.unwrap();

    // Create child documents (chapters)
    for i in 1..=3 {
        let chapter_params = CreateDocumentParams {
            project_id: 1,
            title: format!("Chapter {}", i),
            content: format!("Chapter {} content", i),
            doc_type: Some("prose".to_string()),
            entity_type: None,
            parent_id: Some(book.id),
            display_order: Some(i as i64),
        };
        repo.create(chapter_params, &encryption).await.unwrap();
    }

    // List all documents in project
    let all_docs = repo.list_by_project(1).await.unwrap();
    assert_eq!(all_docs.len(), 4); // 1 book + 3 chapters

    // Verify parent relationships
    let chapters: Vec<_> = all_docs
        .iter()
        .filter(|d| d.parent_id == Some(book.id))
        .collect();
    assert_eq!(chapters.len(), 3);

    // Verify ordering
    assert_eq!(chapters[0].title, "Chapter 1");
    assert_eq!(chapters[1].title, "Chapter 2");
    assert_eq!(chapters[2].title, "Chapter 3");

    cleanup_test_db(pool, db_path).await;
}

#[tokio::test]
async fn test_password_validation() {
    // Valid passwords
    assert!(PasswordValidator::validate("Valid_Password_123").is_ok());
    assert!(PasswordValidator::validate("Another$Strong1Pass").is_ok());
    assert!(PasswordValidator::validate("Complex!Pass123Word").is_ok());

    // Too short
    assert!(PasswordValidator::validate("Short1!").is_err());
    assert!(PasswordValidator::validate("TooShort1").is_err());

    // Missing complexity
    assert!(PasswordValidator::validate("NoDigitsOrSpecialChars").is_err());
    assert!(PasswordValidator::validate("nouppercaseorspecial123").is_err());
    assert!(PasswordValidator::validate("NOLOWERCASEORSPECIAL123").is_err());

    // Edge cases
    assert!(PasswordValidator::validate("ExactlyTwelve1!").is_ok()); // 15 chars

    // Too long (>128 chars)
    let long_password = "A".repeat(129) + "1!";
    assert!(PasswordValidator::validate(&long_password).is_err());
}

#[tokio::test]
async fn test_key_derivation_determinism() {
    let password = "Deterministic_Test!";
    let salt = gen_salt_with_retry(3);

    // Derive key multiple times
    let key1 = EncryptionService::derive_master_key(password, &salt).unwrap();
    let key2 = EncryptionService::derive_master_key(password, &salt).unwrap();
    let key3 = EncryptionService::derive_master_key(password, &salt).unwrap();

    // All keys should be identical
    assert_eq!(key1, key2);
    assert_eq!(key2, key3);

    // Different password should produce different key
    let different_password = "Different_Password!";
    let key4 = EncryptionService::derive_master_key(different_password, &salt).unwrap();
    assert_ne!(key1, key4);

    // Different salt should produce different key
    let different_salt = gen_salt_with_retry(3);
    let key5 = EncryptionService::derive_master_key(password, &different_salt).unwrap();
    assert_ne!(key1, key5);
}

#[tokio::test]
async fn test_concurrent_document_operations() {
    let (pool, db_path) = setup_test_db().await;
    let repo = DocumentRepository::new(pool.clone());

    let password = "Concurrent_Test!";
    let salt = gen_salt_with_retry(3);
    let master_key = EncryptionService::derive_master_key(password, &salt).unwrap();
    let encryption = EncryptionService::new(master_key);

    // Create multiple documents concurrently
    let mut handles = vec![];
    for i in 1..=5 {
        let repo_clone = repo.clone();
        let encryption_clone = EncryptionService::new(master_key);

        let handle = tokio::spawn(async move {
            let params = CreateDocumentParams {
                project_id: 1,
                title: format!("Concurrent Doc {}", i),
                content: format!("Content {}", i),
                doc_type: None,
                entity_type: None,
                parent_id: None,
                display_order: Some(i as i64),
            };
            repo_clone.create(params, &encryption_clone).await
        });
        handles.push(handle);
    }

    // Wait for all operations to complete
    for handle in handles {
        let result = handle.await.unwrap();
        assert!(result.is_ok());
    }

    // Verify all documents were created
    let all_docs = repo.list_by_project(1).await.unwrap();
    assert_eq!(all_docs.len(), 5);

    cleanup_test_db(pool, db_path).await;
}

// ============================================================================
// KEYRING INTEGRATION TESTS (Platform-specific, may require manual testing)
// ============================================================================

#[tokio::test]
#[ignore] // Run manually with: cargo test --test integration_test test_keyring -- --ignored
async fn test_keyring_salt_storage() {
    // Generate salt
    let salt = gen_salt_with_retry(3);

    // Store salt in keyring
    KeyManager::store_salt(&salt).unwrap();

    // Verify salt exists
    assert!(KeyManager::has_salt());

    // Retrieve salt
    let retrieved_salt = KeyManager::retrieve_salt().unwrap();
    assert_eq!(salt, retrieved_salt);

    // Clean up
    KeyManager::delete_salt().unwrap();
    assert!(!KeyManager::has_salt());
}

#[tokio::test]
#[ignore]
async fn test_salt_file_fallback() {
    // Generate salt
    let salt = EncryptionService::generate_salt().unwrap();

    // Store salt (will use file fallback if keyring is unavailable)
    KeyManager::store_salt(&salt).unwrap();

    // Verify salt exists
    assert!(KeyManager::has_salt());

    // Retrieve salt
    let retrieved_salt = KeyManager::retrieve_salt().unwrap();
    assert_eq!(salt, retrieved_salt);

    // Verify file was created
    let salt_path = std::env::var("HOME")
        .or_else(|_| std::env::var("USERPROFILE"))
        .map(|h| {
            std::path::PathBuf::from(h)
                .join(".scriptoria")
                .join("salt.enc")
        })
        .unwrap();

    assert!(salt_path.exists());

    // Verify permissions for Unix
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;

        let perms = std::fs::metadata(&salt_path).unwrap().permissions();
        assert_eq!(perms.mode() & 0o777, 0o600); // -rw-------
    }

    // Clean up
    KeyManager::delete_salt().unwrap();
    assert!(!KeyManager::has_salt());
}

#[tokio::test]
#[ignore] // Run manually with: cargo test --test integration_test test_full_auth_flow -- --ignored
async fn test_full_authentication_flow() {
    let (pool, db_path) = setup_test_db().await;
    let repo = DocumentRepository::new(pool.clone());

    let password = "User_Password_123!";

    // First-time setup: generate and store salt
    let salt = gen_salt_with_retry(3);
    KeyManager::store_salt(&salt).unwrap();

    // Derive master key
    let master_key = EncryptionService::derive_master_key(password, &salt).unwrap();
    let encryption = EncryptionService::new(master_key);

    // Create document
    let params = CreateDocumentParams {
        project_id: 1,
        title: "Protected Document".to_string(),
        content: "Sensitive information".to_string(),
        doc_type: None,
        entity_type: None,
        parent_id: None,
        display_order: None,
    };
    let doc = repo.create(params, &encryption).await.unwrap();

    // Simulate app restart: retrieve salt from keyring
    let retrieved_salt = KeyManager::retrieve_salt().unwrap();
    let retrieved_master_key =
        EncryptionService::derive_master_key(password, &retrieved_salt).unwrap();
    let retrieved_encryption = EncryptionService::new(retrieved_master_key);

    // Decrypt document
    let retrieved_doc = repo.get_by_id(doc.id, &retrieved_encryption).await.unwrap();
    assert_eq!(retrieved_doc.content, "Sensitive information");

    // Cleanup
    KeyManager::delete_salt().unwrap();
    cleanup_test_db(pool, db_path).await;
}
