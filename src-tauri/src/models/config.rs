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

// ── New types for config-driven API ──

/// The functional type of an API service.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ServiceType {
    #[default]
    Text2img,
    Img2img,
    Translate,
}

/// The format of image data in the API response.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ResponseFormat {
    /// Response contains image URLs (e.g., DashScope, DALL-E)
    #[default]
    Url,
    /// Response contains base64-encoded image data (e.g., Gemini)
    Base64,
}

/// The API calling mode.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ApiMode {
    #[default]
    Sync,
    AsyncPoll,
}

/// Configuration for async submit+poll API pattern.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AsyncPollConfig {
    /// Extra headers added to the submit POST request.
    /// Example: `{ "X-DashScope-Async": "enable" }`
    #[serde(default)]
    pub submit_headers: HashMap<String, String>,

    /// URL template for polling. Must contain `{task_id}` placeholder.
    /// Example: `"https://dashscope.aliyuncs.com/api/v1/tasks/{task_id}"`
    pub poll_url: String,

    /// Seconds between poll requests. Default: 3.
    #[serde(default = "default_poll_interval")]
    pub poll_interval_secs: u64,

    /// Total timeout in seconds before giving up. Default: 180.
    #[serde(default = "default_timeout")]
    pub timeout_secs: u64,
}

fn default_poll_interval() -> u64 {
    3
}

fn default_timeout() -> u64 {
    180
}

/// Configuration for a single model endpoint.
///
/// All new fields use `#[serde(default)]` for backward compatibility.
/// Existing configs with only `{ "url": "..." }` continue to work.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelConfig {
    /// The API endpoint URL (required, existing field).
    pub url: String,

    /// Explicit service type. If absent, inferred from model name heuristics.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub service_type: Option<ServiceType>,

    /// API calling mode: "sync" (default) or "async_poll".
    #[serde(default)]
    pub mode: ApiMode,

    /// JSON request body template with `{placeholder}` variables.
    /// If absent, uses built-in default template for the service type.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub request_template: Option<serde_json::Value>,

    /// Dot-notation path to extract image data from the API response.
    /// Example: `"output.results[*].url"` or `"candidates[*].content.parts[*].inlineData"`.
    /// If absent, uses built-in default path for the service type.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub response_image_path: Option<String>,

    /// Format of image data in the response: "url" (default) or "base64".
    /// - `url`: response_image_path points to URL strings
    /// - `base64`: response_image_path points to objects with `mimeType` and `data` fields
    #[serde(default)]
    pub response_format: ResponseFormat,

    /// Extra headers to include in API requests.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub headers: Option<HashMap<String, String>>,

    /// Configuration for async poll mode. Required when `mode == "async_poll"`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub async_poll: Option<AsyncPollConfig>,

    /// Whether this model supports the `size` parameter for output resolution.
    /// When `true`, the `{size}` template variable is injected and the prompt
    /// parser extracts resolution keywords. When `false` (default), the size
    /// variable is not injected (even if `{size}` appears in the template).
    #[serde(default)]
    pub supports_size: bool,
}

// ── Frontend-facing types ──

/// Information about a single available model, sent to the frontend.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelInfo {
    pub name: String,
    pub provider: String,
    #[serde(rename = "serviceType")]
    pub service_type: ServiceType,
    #[serde(rename = "supportsSize")]
    pub supports_size: bool,
}

/// Status returned to the frontend after attempting to load configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConfigStatus {
    pub loaded: bool,
    pub available_models: Vec<ModelInfo>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error_message: Option<String>,
}
