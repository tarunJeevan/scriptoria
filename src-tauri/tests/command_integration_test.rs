// src-tauri/tests/command_integration_test.rs

use scriptoria_lib::commands::documents::{AppState, CommandError};
use scriptoria_lib::encryption::EncryptionService;
// use scriptoria_lib::models::DocumentMetadata;

use sqlx::SqlitePool;
use std::path::PathBuf;
use tauri::test::{mock_builder, mock_context, noop_assets, MockRuntime};
use tauri::Manager;

// ==============================================================
// TEST SETUP
// ==============================================================

/// Set up database and return AppState + cleanup path
async fn setup_test_app_state() -> (AppState, PathBuf) {
    let temp_dir = std::env::temp_dir();
    let db_path = temp_dir.join(format!("scriptoria_cmd_test_{}.db", uuid::Uuid::new_v4()));

    let options = sqlx::sqlite::SqliteConnectOptions::new()
        .filename(db_path.clone())
        .create_if_missing(true)
        .foreign_keys(true);

    let pool = SqlitePool::connect_with(options).await.unwrap();

    // Apply schema
    let schema = include_str!("../migrations/20251220220048_initial_schema.sql");
    sqlx::query(schema).execute(&pool).await.unwrap();

    // Insert test project
    sqlx::query!(
		r#"
		INSERT INTO projects (id, user_id, title, created_at, updated_at, display_order, archived, metadata)
		VALUES (1, 1, 'Test Project', datetime('now'), datetime('now'), 0, 0, '{}')
		"#
	)
.execute(&pool)
.await
.unwrap();

    let encryption = EncryptionService::new([0u8; 32]);

    (AppState { pool, encryption }, db_path)
}

/// Set up Tauri mock app runtime with managed state
async fn setup_test_app() -> (tauri::App<MockRuntime>, SqlitePool, PathBuf) {
    let (state, db_path) = setup_test_app_state().await;
    let pool = state.pool.clone(); // Clone needed for cleanup

    let app = mock_builder()
        .build(mock_context(noop_assets()))
        .expect("Failed to build mock app");

    // Manually manage state AFTER build
    app.manage(state);

    (app, pool, db_path)
}

async fn cleanup_test_db(pool: SqlitePool, db_path: PathBuf) {
    pool.close().await;
    let _ = std::fs::remove_file(&db_path);
    let _ = std::fs::remove_file(db_path.with_extension("db-shm"));
    let _ = std::fs::remove_file(db_path.with_extension("db-wal"));
}

// ==============================================================
// COMMAND TESTS
// ==============================================================

#[tokio::test]
async fn test_create_document_command() {
    let (app, pool, db_path) = setup_test_app().await;

    // Simulate Tauri command call
    let result = scriptoria_lib::commands::documents::create_document(
        1,
        "Test Document".to_string(),
        "Content here".to_string(),
        None,
        None,
        None,
        app.state(), // Get State<'_, AppState> from app
    )
    .await;

    assert!(result.is_ok());
    let doc = result.unwrap();
    assert_eq!(doc.title, "Test Document");
    assert_eq!(doc.content, "Content here");
    assert_eq!(doc.word_count, 2);

    cleanup_test_db(pool, db_path).await;
}

#[tokio::test]
async fn test_create_document_validation() {
    let (app, pool, db_path) = setup_test_app().await;

    // Empty title should fail
    let result = scriptoria_lib::commands::documents::create_document(
        1,
        "".to_string(),
        "Content".to_string(),
        None,
        None,
        None,
        app.state(),
    )
    .await;

    assert!(result.is_err());
    assert!(matches!(result.unwrap_err(), CommandError::Validation(_)));

    // Title too long should fail
    let long_title = "a".repeat(501);
    let result = scriptoria_lib::commands::documents::create_document(
        1,
        long_title,
        "Content".to_string(),
        None,
        None,
        None,
        app.state(),
    )
    .await;

    assert!(result.is_err());
    assert!(matches!(result.unwrap_err(), CommandError::Validation(_)));

    cleanup_test_db(pool, db_path).await;
}

#[tokio::test]
async fn test_read_document_command() {
    let (app, pool, db_path) = setup_test_app().await;

    // Create document first
    let doc = scriptoria_lib::commands::documents::create_document(
        1,
        "Read Test".to_string(),
        "Read content".to_string(),
        None,
        None,
        None,
        app.state(),
    )
    .await
    .unwrap();

    // Read it back
    let result = scriptoria_lib::commands::documents::read_document(doc.id, app.state()).await;

    assert!(result.is_ok());
    let retrieved = result.unwrap();
    assert_eq!(retrieved.id, doc.id);
    assert_eq!(retrieved.content, "Read content");

    cleanup_test_db(pool, db_path).await;
}

#[tokio::test]
async fn test_update_document_command() {
    let (app, pool, db_path) = setup_test_app().await;

    // Create document
    let doc = scriptoria_lib::commands::documents::create_document(
        1,
        "Original".to_string(),
        "Original content".to_string(),
        None,
        None,
        None,
        app.state(),
    )
    .await
    .unwrap();

    // Update it
    let result = scriptoria_lib::commands::documents::update_document(
        doc.id,
        Some("Updated".to_string()),
        Some("Updated content".to_string()),
        None,
        None,
        None,
        None,
        app.state(),
    )
    .await;

    assert!(result.is_ok());
    let updated = result.unwrap();
    assert_eq!(updated.title, "Updated");
    assert_eq!(updated.content, "Updated content");

    cleanup_test_db(pool, db_path).await;
}

#[tokio::test]
async fn test_delete_document_command() {
    let (app, pool, db_path) = setup_test_app().await;

    // Create document
    let doc = scriptoria_lib::commands::documents::create_document(
        1,
        "To Delete".to_string(),
        "Delete me".to_string(),
        None,
        None,
        None,
        app.state(),
    )
    .await
    .unwrap();

    // Delete it
    let result = scriptoria_lib::commands::documents::delete_document(doc.id, app.state()).await;
    assert!(result.is_ok());

    // Try to read it (should fail with NotFound)
    let read_result = scriptoria_lib::commands::documents::read_document(doc.id, app.state()).await;
    assert!(read_result.is_err());

    cleanup_test_db(pool, db_path).await;
}

#[tokio::test]
async fn test_list_documents_command() {
    let (app, pool, db_path) = setup_test_app().await;

    // Create multiple documents
    for i in 1..=3 {
        scriptoria_lib::commands::documents::create_document(
            1,
            format!("Doc {}", i),
            format!("Content {}", i),
            None,
            None,
            None,
            app.state(),
        )
        .await
        .unwrap();
    }

    // List them
    let result = scriptoria_lib::commands::documents::list_documents(1, app.state()).await;

    assert!(result.is_ok());
    let docs = result.unwrap();
    assert_eq!(docs.len(), 3);
    assert_eq!(docs[0].title, "Doc 1");

    cleanup_test_db(pool, db_path).await;
}

#[tokio::test]
async fn test_version_commands() {
    let (app, pool, db_path) = setup_test_app().await;

    // Create document
    let doc = scriptoria_lib::commands::documents::create_document(
        1,
        "Versioned".to_string(),
        "Version 1".to_string(),
        None,
        None,
        None,
        app.state(),
    )
    .await
    .unwrap();

    // Create version
    let v1_result = scriptoria_lib::commands::documents::create_version(
        doc.id,
        Some("Draft 1".to_string()),
        app.state(),
    )
    .await;
    assert!(v1_result.is_ok());

    // Update document
    scriptoria_lib::commands::documents::update_document(
        doc.id,
        None,
        Some("Version 2".to_string()),
        None,
        None,
        None,
        None,
        app.state(),
    )
    .await
    .unwrap();

    // Create another version
    scriptoria_lib::commands::documents::create_version(
        doc.id,
        Some("Draft 2".to_string()),
        app.state(),
    )
    .await
    .unwrap();

    // List versions
    let versions_result =
        scriptoria_lib::commands::documents::list_versions(doc.id, app.state()).await;
    assert!(versions_result.is_ok());
    let versions = versions_result.unwrap();
    assert_eq!(versions.len(), 2);

    // Get specific version
    let v1_id = v1_result.unwrap();
    let get_result = scriptoria_lib::commands::documents::get_version(v1_id, app.state()).await;
    assert!(get_result.is_ok());
    let (_version, content) = get_result.unwrap();
    assert_eq!(content, "Version 1");

    cleanup_test_db(pool, db_path).await;
}

#[tokio::test]
async fn test_restore_from_version() {
    let (app, pool, db_path) = setup_test_app().await;

    // Create document with v1 content
    let doc = scriptoria_lib::commands::documents::create_document(
        1,
        "Restore Test".to_string(),
        "Version 1 content".to_string(),
        None,
        None,
        None,
        app.state(),
    )
    .await
    .unwrap();

    // Create version snapshot
    let v1_id = scriptoria_lib::commands::documents::create_version(
        doc.id,
        Some("v1".to_string()),
        app.state(),
    )
    .await
    .unwrap();

    // Update to v2
    scriptoria_lib::commands::documents::update_document(
        doc.id,
        None,
        Some("Version 2 content".to_string()),
        None,
        None,
        None,
        None,
        app.state(),
    )
    .await
    .unwrap();

    // Restore from v1
    let restored =
        scriptoria_lib::commands::documents::restore_from_version(doc.id, v1_id, app.state())
            .await
            .unwrap();

    assert_eq!(restored.content, "Version 1 content");

    cleanup_test_db(pool, db_path).await;
}
