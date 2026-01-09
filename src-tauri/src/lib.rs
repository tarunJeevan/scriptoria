pub mod db;
pub mod encryption;
pub mod models;

pub use models::*;

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

            println!("Database URL: {}", db_url);

            // TODO: Initialize database pool here

            Ok(())
        })
        // NOTE: Add plugins here if necessary
        // NOTE: Add invoke_handlers here if necessary
        .run(tauri::generate_context!())
        .expect("Error while running tauri application");
}
