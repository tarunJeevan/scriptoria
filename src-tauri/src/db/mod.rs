pub mod documents;
// Placeholder for Chunk 0: Database Operations
//
// This module will contain:
// - SQLite connection pool setup (via sqlx)
// - Migration runner (sqlx::migrate!)
// - Repository pattern implementations:
//   - DocumentRepository (CRUD + versioning)
//   - ProjectRepository (hierarchy management)
//   - AiContextRepository (chunk management)
//
// Functions to implement:
// - pub async fn create_pool(database_url: &str) -> Result<SqlitePool>
// - pub async fn run_migrations(pool: &SqlitePool) -> Result<()>
//
// Configuration:
// - WAL mode enabled (concurrent reads)
// - Busy timeout: 5 seconds
// - Max connections: 5
//
// See: scriptoria-database-setup-guide.md and scriptoria-phase-1-document-repo.rs

use sqlx::sqlite::{SqlitePool, SqlitePoolOptions};
use std::time::Duration;

pub async fn create_pool(db_url: &str) -> Result<SqlitePool, sqlx::Error> {
    SqlitePoolOptions::new()
        .max_connections(5)
        .acquire_timeout(Duration::from_secs(3))
        .connect(db_url)
        .await
}

pub async fn run_migrations(pool: &SqlitePool) -> Result<(), sqlx::migrate::MigrateError> {
    sqlx::migrate!("./migrations").run(pool).await
}
