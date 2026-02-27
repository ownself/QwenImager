use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Top-level configuration read from `~/.qwenimage/setting.json`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Configuration {
    pub providers: HashMap<String, Provider>,
}

/// A single AI provider's configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Provider {
    #[serde(alias = "apiKey")]
    pub api_key: String,
    pub models: HashMap<String, ModelConfig>,
}

/// Configuration for a single model endpoint.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelConfig {
    pub url: String,
}

/// Status returned to the frontend after attempting to load configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConfigStatus {
    pub loaded: bool,
    pub available_models: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error_message: Option<String>,
}
