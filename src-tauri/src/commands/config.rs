use crate::models::config::ConfigStatus;
use crate::services::config_loader;

/// Tauri command: load the user's API configuration from `~/.qwenimage/setting.json`.
///
/// Returns `ConfigStatus` with `loaded`, `available_models`, and optional `error_message`.
#[tauri::command]
pub fn load_config() -> ConfigStatus {
    config_loader::load_config()
}
