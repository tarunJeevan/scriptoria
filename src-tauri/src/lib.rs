pub mod ai; // TODO [Chunk 5]: AI inference
pub mod commands; // TODO [Chunk 3]: Tauri commands
pub mod db;
pub mod encryption;
pub mod models;

pub use models::*;

use commands::documents::AppState;
use encryption::{EncryptionService, KeyManager};

use tauri::Manager;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .setup(|app| {
            // Load environment variables
            dotenvy::dotenv().ok();

            // Get database URL
            let db_url = std::env::var("DATABASE_URL").unwrap_or_else(|_| {
                // NOTE: In production, use app data directory
                let app_dir = app
                    .path()
                    .app_data_dir()
                    .expect("Failed to get app data directory");

                std::fs::create_dir_all(&app_dir).expect("Failed to create app data directory");

                let db_path = app_dir.join("scriptoria.db");
                format!("sqlite://{}", db_path.display())
            });

            #[cfg(debug_assertions)]
            println!("Database URL: {}", db_url);

            // Initialize database pool with tokio runtime
            let runtime = tokio::runtime::Runtime::new().unwrap();
            let pool = runtime.block_on(async {
                db::create_pool(&db_url)
                    .await
                    .expect("Failed to create database pool")
            });

            // Run migrations
            runtime.block_on(async {
                db::run_migrations(&pool)
                    .await
                    .expect("Failed to run migrations")
            });

            // Initialize encryption service
            // NOTE: In production, derive master key from user password
            // For now, use a test key (will be replaced with password auth in Chunk 4)
            let encryption = if cfg!(debug_assertions) {
                // Development: Use test key
                println!("WARNING: Using test encryption key (development mode)");
                EncryptionService::new([0u8; 32])
            } else {
                // Production: check if salt exists, prompt for password
                if KeyManager::has_salt() {
                    // TODO: Implement password prompt UI
                    todo!(
                        "Password prompt UI not implemented yet! Please run in development mode!"
                    );
                } else {
                    // First-time setup: generate salt
                    let salt = EncryptionService::generate_salt().expect("Failed to generate salt");
                    KeyManager::store_salt(&salt).expect("Failed to store salt");

                    // TODO: Derive from user password
                    todo!("Password setup not implemented yet! Please run in development mode!");
                }
            };

            // Store pool in app state
            app.manage(AppState { pool, encryption });

            Ok(())
        })
        // NOTE: Add plugins here if necessary
        .invoke_handler(tauri::generate_handler![
            // Document CRUD
            commands::documents::create_document,
            commands::documents::read_document,
            commands::documents::update_document,
            commands::documents::delete_document,
            commands::documents::list_documents,
            commands::documents::list_documents_by_type,
            commands::documents::count_documents_by_type,
            // Versioning
            commands::documents::create_version,
            commands::documents::list_versions,
            commands::documents::get_version,
            commands::documents::restore_from_version,
            // Utilities
            commands::documents::permanently_delete_document,
        ])
        .run(tauri::generate_context!())
        .expect("Error while running tauri application");
}
