use std::path::PathBuf;

use crate::error::AppError;
use crate::models::config::{ConfigStatus, Configuration};

/// Returns the path to `~/.qwenimage/setting.json`.
fn config_path() -> Result<PathBuf, AppError> {
    let home = dirs::home_dir()
        .ok_or_else(|| AppError::Config("Cannot determine home directory".to_string()))?;
    Ok(home.join(".qwenimage").join("setting.json"))
}

/// Load and validate the configuration file from `~/.qwenimage/setting.json`.
///
/// Returns a `ConfigStatus` indicating whether loading succeeded, which models
/// are available, and any error message if loading failed.
pub fn load_config() -> ConfigStatus {
    match load_config_inner() {
        Ok((_config, models)) => ConfigStatus {
            loaded: true,
            available_models: models,
            error_message: None,
        },
        Err(err) => ConfigStatus {
            loaded: false,
            available_models: vec![],
            error_message: Some(err.to_string()),
        },
    }
}

/// Attempt to load and parse the configuration, returning the parsed
/// `Configuration` and the list of available model names.
pub fn load_config_inner() -> Result<(Configuration, Vec<String>), AppError> {
    let path = config_path()?;

    if !path.exists() {
        return Err(AppError::Config(format!(
            "Configuration file not found: {}. Please create it with your API key and model endpoints.",
            path.display()
        )));
    }

    let content = std::fs::read_to_string(&path).map_err(|e| {
        AppError::Config(format!(
            "Failed to read configuration file {}: {}",
            path.display(),
            e
        ))
    })?;

    let config: Configuration = serde_json::from_str(&content).map_err(|e| {
        AppError::Config(format!(
            "Configuration file format error: {}. Please check that {} is valid JSON.",
            e,
            path.display()
        ))
    })?;

    // Validate that the "qwen" provider exists
    let provider = config.providers.get("qwen").ok_or_else(|| {
        AppError::Config(
            "Missing 'qwen' provider in configuration. Expected providers.qwen with apiKey and models."
                .to_string(),
        )
    })?;

    // Validate API key is non-empty
    if provider.api_key.trim().is_empty() {
        return Err(AppError::Config(
            "Missing apiKey: providers.qwen.apiKey is empty. Please set your DashScope API key."
                .to_string(),
        ));
    }

    // Validate at least one model is configured
    if provider.models.is_empty() {
        return Err(AppError::Config(
            "Missing models: providers.qwen.models is empty. Please configure at least one model."
                .to_string(),
        ));
    }

    // Validate each model has a non-empty and valid URL
    for (name, model_config) in &provider.models {
        if model_config.url.trim().is_empty() {
            return Err(AppError::Config(format!(
                "Model '{}' has an empty URL. Please set the API endpoint URL.",
                name
            )));
        }
        if !model_config.url.starts_with("http://") && !model_config.url.starts_with("https://") {
            return Err(AppError::Config(format!(
                "Model '{}' URL must start with http:// or https://. Got: {}",
                name, model_config.url
            )));
        }
    }

    let model_names: Vec<String> = provider.models.keys().cloned().collect();
    Ok((config, model_names))
}
