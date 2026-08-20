mod commands;
mod error;
mod models;
mod services;

use std::sync::Arc;

use tauri::Manager;

use services::db::{self, Database};
use services::qwen_client::QwenClient;

/// Application-wide shared state, injected via `tauri::manage()`.
pub struct AppState {
    pub db: Database,
    pub qwen_client: QwenClient,
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        // ── Tauri plugins ──
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_store::Builder::default().build())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_fs::init())
        .plugin(tauri_plugin_clipboard_manager::init())
        // ── App state setup (database + HTTP client) ──
        .setup(|app| {
            let app_data_dir = app
                .path()
                .app_data_dir()
                .expect("Failed to resolve app data directory");

            let db_path = db::resolve_db_path(&app_data_dir);
            let database = Database::new(&db_path).expect("Failed to initialize database");
            let qwen_client = QwenClient::new();

            app.manage(Arc::new(AppState {
                db: database,
                qwen_client,
            }));

            Ok(())
        })
        // ── Cleanup temp files on exit ──
        .on_window_event(|_window, event| {
            if let tauri::WindowEvent::Destroyed = event {
                let temp_dir = std::env::temp_dir().join("qwenimager-clipboard");
                if temp_dir.exists() {
                    let _ = std::fs::remove_dir_all(&temp_dir);
                }
            }
        })
        // ── Tauri commands ──
        .invoke_handler(tauri::generate_handler![
            commands::config::load_config,
            commands::generation::generate_image,
            commands::generation::save_image,
            commands::generation::save_clipboard_image,
            commands::generation::edit_image,
            commands::conversation::create_conversation,
            commands::conversation::get_conversation_messages,
            commands::conversation::get_conversations,
            commands::conversation::delete_conversation,
            commands::translation::translate_image,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
