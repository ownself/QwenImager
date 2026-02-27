use std::path::PathBuf;

use crate::error::AppError;
use crate::models::config::{
    ApiMode, ConfigStatus, Configuration, ModelConfig, ModelInfo, ServiceType,
};
use crate::services::template_engine::infer_service_type;

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
/// `Configuration` and the list of available model info.
pub fn load_config_inner() -> Result<(Configuration, Vec<ModelInfo>), AppError> {
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

    // Validate that at least one provider exists
    if config.providers.is_empty() {
        return Err(AppError::Config(
            "No providers configured. Please add at least one provider with apiKey and models."
                .to_string(),
        ));
    }

    // Validate all providers and build ModelInfo list
    let mut model_infos: Vec<ModelInfo> = Vec::new();

    for (provider_name, provider) in &config.providers {
        // Validate API key is non-empty
        if provider.api_key.trim().is_empty() {
            return Err(AppError::Config(format!(
                "Provider '{}': apiKey is empty. Please set your API key.",
                provider_name
            )));
        }

        // Validate at least one model is configured
        if provider.models.is_empty() {
            return Err(AppError::Config(format!(
                "Provider '{}': no models configured. Please add at least one model.",
                provider_name
            )));
        }

        // Validate each model
        for (model_name, model_config) in &provider.models {
            validate_model_config(provider_name, model_name, model_config)?;

            let service_type = model_config
                .service_type
                .unwrap_or_else(|| infer_service_type(model_name));

            model_infos.push(ModelInfo {
                name: model_name.clone(),
                provider: provider_name.clone(),
                service_type,
            });
        }
    }

    Ok((config, model_infos))
}

/// Validate a single model configuration entry.
fn validate_model_config(
    provider_name: &str,
    model_name: &str,
    mc: &ModelConfig,
) -> Result<(), AppError> {
    let ctx = format!("Provider '{}', model '{}'", provider_name, model_name);

    // URL must be non-empty and start with http(s)
    if mc.url.trim().is_empty() {
        return Err(AppError::Config(format!(
            "{}: URL is empty. Please set the API endpoint URL.",
            ctx
        )));
    }
    if !mc.url.starts_with("http://") && !mc.url.starts_with("https://") {
        return Err(AppError::Config(format!(
            "{}: URL must start with http:// or https://. Got: {}",
            ctx, mc.url
        )));
    }

    // If mode is async_poll, async_poll config must be present and valid
    if mc.mode == ApiMode::AsyncPoll {
        let poll_cfg = mc.async_poll.as_ref().ok_or_else(|| {
            AppError::Config(format!(
                "{}: mode is 'async_poll' but 'async_poll' configuration is missing.",
                ctx
            ))
        })?;

        if !poll_cfg.poll_url.contains("{task_id}") {
            return Err(AppError::Config(format!(
                "{}: async_poll.poll_url must contain '{{task_id}}' placeholder. Got: {}",
                ctx, poll_cfg.poll_url
            )));
        }

        if poll_cfg.poll_interval_secs < 1 {
            return Err(AppError::Config(format!(
                "{}: async_poll.poll_interval_secs must be >= 1. Got: {}",
                ctx, poll_cfg.poll_interval_secs
            )));
        }

        if poll_cfg.timeout_secs < poll_cfg.poll_interval_secs {
            return Err(AppError::Config(format!(
                "{}: async_poll.timeout_secs ({}) must be >= poll_interval_secs ({}).",
                ctx, poll_cfg.timeout_secs, poll_cfg.poll_interval_secs
            )));
        }
    }

    // If request_template is present, it must be a JSON Object
    if let Some(ref tmpl) = mc.request_template {
        if !tmpl.is_object() {
            return Err(AppError::Config(format!(
                "{}: request_template must be a JSON object, got: {}",
                ctx,
                match tmpl {
                    serde_json::Value::Array(_) => "array",
                    serde_json::Value::String(_) => "string",
                    serde_json::Value::Number(_) => "number",
                    serde_json::Value::Bool(_) => "boolean",
                    serde_json::Value::Null => "null",
                    _ => "unknown",
                }
            )));
        }
    }

    Ok(())
}

// ── Model Lookup Helpers ──

/// Find a model configuration by its exact name across all providers.
///
/// Returns `(api_key, model_config)` on success.
pub fn find_model_config<'a>(
    config: &'a Configuration,
    model_name: &str,
) -> Result<(&'a str, &'a ModelConfig), AppError> {
    for provider in config.providers.values() {
        if let Some(mc) = provider.models.get(model_name) {
            return Ok((&provider.api_key, mc));
        }
    }
    Err(AppError::Config(format!(
        "Model '{}' not found in configuration. Available models: {}",
        model_name,
        config
            .providers
            .values()
            .flat_map(|p| p.models.keys())
            .cloned()
            .collect::<Vec<_>>()
            .join(", ")
    )))
}

/// Find the first model matching a given service type across all providers.
///
/// Uses explicit `service_type` field when present, falls back to name heuristics.
/// Returns `(api_key, model_name, model_config)` on success.
pub fn find_model_by_service_type(
    config: &Configuration,
    target: ServiceType,
) -> Result<(&str, String, &ModelConfig), AppError> {
    // First pass: look for explicit service_type match
    for provider in config.providers.values() {
        for (name, mc) in &provider.models {
            if mc.service_type == Some(target) {
                return Ok((&provider.api_key, name.clone(), mc));
            }
        }
    }

    // Second pass: fall back to name heuristic inference
    for provider in config.providers.values() {
        for (name, mc) in &provider.models {
            if mc.service_type.is_none() && infer_service_type(name) == target {
                return Ok((&provider.api_key, name.clone(), mc));
            }
        }
    }

    let type_name = match target {
        ServiceType::Text2img => "text2img",
        ServiceType::Img2img => "img2img",
        ServiceType::Translate => "translate",
    };
    Err(AppError::Config(format!(
        "No model found for service type '{}'. Please configure a model with service_type: \"{}\" in your setting.json.",
        type_name, type_name
    )))
}
