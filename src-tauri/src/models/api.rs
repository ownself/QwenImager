use serde::{Deserialize, Serialize};

// ── Generation parameters (shared across text2img and img2img) ──

/// Parameters for image generation requests.
///
/// Passed from the frontend to control generation settings.
/// When using config-driven templates, `size` is injected into the
/// request template; other fields are reserved for future use.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct GenerationParams {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub size: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub n: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub negative_prompt: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub prompt_extend: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub watermark: Option<bool>,
}

// ── Channel event types (sent to frontend) ──

/// Events pushed to the frontend via Tauri Channel during async generation.
#[derive(Debug, Clone, Serialize)]
#[serde(tag = "event", content = "data")]
pub enum GenerationEvent {
    #[serde(rename = "submitted")]
    Submitted {
        #[serde(rename = "taskId")]
        task_id: String,
    },
    #[serde(rename = "polling")]
    Polling {
        #[serde(rename = "taskId")]
        task_id: String,
        status: String,
    },
    #[serde(rename = "succeeded")]
    Succeeded {
        #[serde(rename = "taskId")]
        task_id: String,
        #[serde(rename = "messageId")]
        message_id: String,
        #[serde(rename = "imageUrls")]
        image_urls: Vec<String>,
    },
    #[serde(rename = "failed")]
    Failed {
        #[serde(rename = "taskId")]
        task_id: String,
        error: String,
    },
}
