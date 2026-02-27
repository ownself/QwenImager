use serde::{Deserialize, Serialize};

/// A single message within a conversation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Message {
    pub id: String,
    pub conversation_id: String,
    pub role: String, // "user" | "assistant"
    pub text_content: Option<String>,
    pub mode: String,                 // "text2img" | "img2img" | "translate"
    pub extra_params: Option<String>, // JSON string
    pub created_at: i64,
}

/// Full message detail returned to the frontend, including attachments and results.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MessageDetail {
    pub id: String,
    pub role: String,
    #[serde(rename = "textContent")]
    pub text_content: Option<String>,
    pub mode: String,
    pub attachments: Vec<AttachmentInfo>,
    pub results: Vec<GenerationResultInfo>,
    #[serde(rename = "extraParams")]
    pub extra_params: Option<serde_json::Value>,
    #[serde(rename = "createdAt")]
    pub created_at: i64,
}

/// Information about an uploaded/pasted image attachment.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Attachment {
    pub id: String,
    pub message_id: String,
    pub file_path: String,
    pub display_order: i32,
    pub file_size: i64,
    pub mime_type: String,
    pub source: String, // "upload" | "clipboard"
}

/// Attachment info for frontend display.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AttachmentInfo {
    pub id: String,
    #[serde(rename = "filePath")]
    pub file_path: String,
    #[serde(rename = "displayOrder")]
    pub display_order: i32,
    #[serde(rename = "fileSize")]
    pub file_size: i64,
    #[serde(rename = "mimeType")]
    pub mime_type: String,
    pub source: String,
}

/// A generated image/video result from an API call.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GenerationResult {
    pub id: String,
    pub message_id: String,
    pub resource_url: Option<String>,
    pub local_path: Option<String>,
    pub resource_type: String, // "image" | "video"
    pub model_used: String,
    pub generation_params: Option<String>, // JSON string
    pub created_at: i64,
}

/// Generation result info for frontend display.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GenerationResultInfo {
    pub id: String,
    #[serde(rename = "resourceUrl")]
    pub resource_url: Option<String>,
    #[serde(rename = "localPath")]
    pub local_path: Option<String>,
    #[serde(rename = "resourceType")]
    pub resource_type: String,
    #[serde(rename = "modelUsed")]
    pub model_used: String,
    #[serde(rename = "createdAt")]
    pub created_at: i64,
}
