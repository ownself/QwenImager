use serde::{Deserialize, Serialize};

// ── Generation parameters (shared across text2img and img2img) ──

/// Parameters for image generation requests.
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

// ── Text-to-Image (async) ──

/// Request body for text-to-image API.
/// ```json
/// { "model": "...", "input": { "prompt": "..." }, "parameters": { ... } }
/// ```
#[derive(Debug, Serialize)]
pub struct TextToImageRequest {
    pub model: String,
    pub input: TextToImageInput,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub parameters: Option<GenerationParams>,
}

#[derive(Debug, Serialize)]
pub struct TextToImageInput {
    pub prompt: String,
}

// ── Image-to-Image / Edit (sync) ──

/// Request body for image editing API.
/// ```json
/// { "model": "...", "input": { "messages": [...] }, "parameters": { ... } }
/// ```
#[derive(Debug, Serialize)]
pub struct ImageEditRequest {
    pub model: String,
    pub input: ImageEditInput,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub parameters: Option<GenerationParams>,
}

#[derive(Debug, Serialize)]
pub struct ImageEditInput {
    pub messages: Vec<ImageEditMessage>,
}

#[derive(Debug, Serialize)]
pub struct ImageEditMessage {
    pub role: String,
    pub content: Vec<ImageEditContent>,
}

/// A content item in an image-edit message: either an image URL or text.
#[derive(Debug, Serialize)]
#[serde(untagged)]
pub enum ImageEditContent {
    Image { image: String },
    Text { text: String },
}

// ── Translation (async) ──

/// Request body for image translation API.
/// ```json
/// { "model": "...", "input": { "image_url": "...", "source_lang": "...", "target_lang": "...", "ext": { ... } } }
/// ```
#[derive(Debug, Serialize)]
pub struct TranslationRequest {
    pub model: String,
    pub input: TranslationInput,
}

#[derive(Debug, Serialize)]
pub struct TranslationInput {
    pub image_url: String,
    pub source_lang: String,
    pub target_lang: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ext: Option<TranslationExt>,
}

#[derive(Debug, Serialize)]
pub struct TranslationExt {
    pub config: TranslationExtConfig,
}

#[derive(Debug, Serialize)]
pub struct TranslationExtConfig {
    #[serde(rename = "imageSegment")]
    pub image_segment: bool,
}

// ── Async task responses ──

/// Response from submitting an async task (text2img, translation).
/// Contains `output.task_id` and `output.task_status`.
#[derive(Debug, Deserialize)]
pub struct AsyncTaskResponse {
    pub output: AsyncTaskOutput,
    pub request_id: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct AsyncTaskOutput {
    pub task_id: String,
    pub task_status: String,
}

/// Response from polling a task's status via GET /api/v1/tasks/{task_id}.
#[derive(Debug, Deserialize)]
pub struct TaskQueryResponse {
    pub output: TaskQueryOutput,
    pub request_id: Option<String>,
    pub usage: Option<serde_json::Value>,
}

#[derive(Debug, Deserialize)]
pub struct TaskQueryOutput {
    pub task_id: String,
    pub task_status: String,
    #[serde(default)]
    pub results: Option<Vec<TaskResult>>,
    pub task_metrics: Option<TaskMetrics>,
    pub code: Option<String>,
    pub message: Option<String>,
}

/// A single result item from a completed task.
#[derive(Debug, Deserialize)]
pub struct TaskResult {
    pub url: Option<String>,
    #[serde(rename = "orig_url")]
    pub orig_url: Option<String>,
}

/// Metrics about task processing (e.g. how many images generated).
#[derive(Debug, Deserialize)]
pub struct TaskMetrics {
    #[serde(rename = "TOTAL")]
    pub total: Option<u32>,
    #[serde(rename = "SUCCEEDED")]
    pub succeeded: Option<u32>,
    #[serde(rename = "FAILED")]
    pub failed: Option<u32>,
}

// ── Sync response (image edit) ──

/// Response from the synchronous image-edit API.
#[derive(Debug, Deserialize)]
pub struct SyncGenerationResponse {
    pub output: SyncGenerationOutput,
    pub request_id: Option<String>,
    pub usage: Option<serde_json::Value>,
}

#[derive(Debug, Deserialize)]
pub struct SyncGenerationOutput {
    pub choices: Option<Vec<SyncChoice>>,
    // Fallback fields for different response formats
    pub results: Option<Vec<TaskResult>>,
}

#[derive(Debug, Deserialize)]
pub struct SyncChoice {
    pub message: Option<SyncChoiceMessage>,
}

#[derive(Debug, Deserialize)]
pub struct SyncChoiceMessage {
    pub content: Option<Vec<SyncChoiceContent>>,
}

#[derive(Debug, Deserialize)]
#[serde(untagged)]
pub enum SyncChoiceContent {
    Image { image: String },
    Text { text: String },
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
