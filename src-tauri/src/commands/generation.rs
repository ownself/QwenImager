use std::collections::HashMap;
use std::sync::Arc;

use serde_json::{json, Value};
use tauri::ipc::Channel;
use tauri::State;

use crate::error::AppError;
use crate::models::api::{GenerationEvent, GenerationParams};
use crate::models::config::{ApiMode, AsyncPollConfig, ModelConfig, ResponseFormat, ServiceType};
use crate::services::config_loader;
use crate::services::template_engine::{
    default_img2img_template, default_text2img_template, extract_base64_images, extract_strings,
    infer_service_type, render_template,
};
use crate::AppState;

// ── Helpers ──

/// Default `AsyncPollConfig` for DashScope async endpoints.
fn default_dashscope_poll_config() -> AsyncPollConfig {
    AsyncPollConfig {
        submit_headers: {
            let mut h = HashMap::new();
            h.insert("X-DashScope-Async".to_string(), "enable".to_string());
            h
        },
        poll_url: "https://dashscope.aliyuncs.com/api/v1/tasks/{task_id}".to_string(),
        poll_interval_secs: 3,
        timeout_secs: 180,
    }
}

/// Resolve model config for a given service type.
///
/// If `model_name` is provided, looks up by exact name; otherwise falls back
/// to service-type heuristic search.
///
/// Returns `(api_key, resolved_model_name, model_config)`.
fn resolve_model(
    model_name: &Option<String>,
    target_type: ServiceType,
) -> Result<(String, String, ModelConfig), AppError> {
    let (config, _models) = config_loader::load_config_inner()?;

    if let Some(name) = model_name {
        let (api_key, mc) = config_loader::find_model_config(&config, name)?;
        Ok((api_key.to_string(), name.clone(), mc.clone()))
    } else {
        let (api_key, name, mc) =
            config_loader::find_model_by_service_type(&config, target_type)?;
        Ok((api_key.to_string(), name, mc.clone()))
    }
}

/// Default response image path for a service type.
fn default_response_path(mc: &ModelConfig, model_name: &str) -> String {
    if let Some(ref path) = mc.response_image_path {
        return path.clone();
    }
    let st = mc.service_type.unwrap_or_else(|| infer_service_type(model_name));
    match st {
        ServiceType::Img2img => "output.choices[*].message.content[*].image".to_string(),
        _ => "output.results[*].url".to_string(),
    }
}

/// Extract images from API response based on the configured response format.
///
/// - `ResponseFormat::Url`: Extract URL strings from the response path
/// - `ResponseFormat::Base64`: Extract base64 data and convert to data: URIs
fn extract_images(resp: &Value, mc: &ModelConfig, model_name: &str) -> Vec<String> {
    let response_path = default_response_path(mc, model_name);

    match mc.response_format {
        ResponseFormat::Url => extract_strings(resp, &response_path),
        ResponseFormat::Base64 => extract_base64_images(resp, &response_path),
    }
}

/// Parse a data URI into (mimeType, base64Data).
///
/// Example: "data:image/png;base64,iVBORw..." -> ("image/png", "iVBORw...")
fn parse_data_uri(data_uri: &str) -> Option<(String, String)> {
    if !data_uri.starts_with("data:") {
        return None;
    }
    // Format: data:mime/type;base64,BASE64DATA
    let without_prefix = &data_uri[5..]; // Remove "data:"
    let semicolon_pos = without_prefix.find(';')?;
    let mime = &without_prefix[..semicolon_pos];
    
    let rest = &without_prefix[semicolon_pos + 1..];
    if !rest.starts_with("base64,") {
        return None;
    }
    let base64_data = &rest[7..]; // Remove "base64,"
    
    Some((mime.to_string(), base64_data.to_string()))
}

/// Truncate base64 data in JSON for debug logging.
fn truncate_base64_in_json(value: &Value) -> Value {
    match value {
        Value::String(s) => {
            // If it looks like base64 data (long string), truncate it
            if s.len() > 100 && !s.contains(' ') {
                Value::String(format!("{}...[truncated {} chars]", &s[..50], s.len() - 50))
            } else {
                Value::String(s.clone())
            }
        }
        Value::Array(arr) => {
            Value::Array(arr.iter().map(truncate_base64_in_json).collect())
        }
        Value::Object(map) => {
            Value::Object(
                map.iter()
                    .map(|(k, v)| (k.clone(), truncate_base64_in_json(v)))
                    .collect(),
            )
        }
        other => other.clone(),
    }
}

/// Resolve effective API mode for a model config.
///
/// For backward compatibility: if `mode` is `Sync` but the model's
/// inferred service_type is text2img or translate (which were historically
/// async_poll for DashScope), and there's no explicit `service_type` or
/// `mode` set, upgrade to async_poll with default DashScope config.
fn resolve_mode_and_poll(
    mc: &ModelConfig,
    model_name: &str,
) -> (ApiMode, Option<AsyncPollConfig>) {
    match mc.mode {
        ApiMode::AsyncPoll => {
            let poll = mc
                .async_poll
                .clone()
                .unwrap_or_else(default_dashscope_poll_config);
            (ApiMode::AsyncPoll, Some(poll))
        }
        ApiMode::Sync => {
            // Backward compat: old configs have mode=sync (default), but text2img
            // and translate were historically async on DashScope.
            // If service_type is NOT explicitly set AND mode is default (sync),
            // infer the correct mode from the model name.
            if mc.service_type.is_none() && mc.async_poll.is_none() {
                let inferred = mc
                    .service_type
                    .unwrap_or_else(|| infer_service_type(model_name));
                match inferred {
                    ServiceType::Text2img | ServiceType::Translate => {
                        // These were historically async_poll on DashScope
                        (ApiMode::AsyncPoll, Some(default_dashscope_poll_config()))
                    }
                    ServiceType::Img2img => (ApiMode::Sync, None),
                }
            } else {
                (ApiMode::Sync, None)
            }
        }
    }
}

// ── Commands ──

/// Text-to-image generation command.
///
/// Builds a config-driven request, sends it via the appropriate mode
/// (sync or async_poll), and sends progress events via a Tauri Channel.
#[tauri::command]
pub async fn generate_image(
    state: State<'_, Arc<AppState>>,
    conversation_id: String,
    prompt: String,
    model_name: Option<String>,
    params: Option<GenerationParams>,
    on_event: Channel<GenerationEvent>,
) -> Result<(), AppError> {
    let (api_key, resolved_name, mc) = resolve_model(&model_name, ServiceType::Text2img)?;

    // Create user message in DB
    let _user_msg = state.db.add_message(
        &conversation_id,
        "user",
        Some(&prompt),
        "text2img",
        None,
    )?;

    // Build template variables
    let gen_params = params.unwrap_or_default();

    let mut vars: HashMap<&str, Value> = HashMap::new();
    vars.insert("model", Value::String(resolved_name.clone()));
    vars.insert("prompt", Value::String(prompt.clone()));

    // Only inject {size} when the model declares supports_size = true
    if mc.supports_size {
        let size = gen_params
            .size
            .clone()
            .unwrap_or_else(|| "1024*1024".to_string());
        vars.insert("size", Value::String(size));
    }

    // Use configured template or default
    let template = mc
        .request_template
        .clone()
        .unwrap_or_else(default_text2img_template);
    let body = render_template(&template, &vars)
        .ok_or_else(|| AppError::Api("Failed to render request template".to_string()))?;

    let extra_headers = mc.headers.clone().unwrap_or_default();

    let (mode, poll_config) = resolve_mode_and_poll(&mc, &resolved_name);

    match mode {
        ApiMode::AsyncPoll => {
            let poll_cfg = poll_config.unwrap_or_else(default_dashscope_poll_config);

            let result = state
                .qwen_client
                .execute_async_poll(
                    &mc.url,
                    &api_key,
                    &body,
                    &poll_cfg,
                    &extra_headers,
                    |task_id, status| {
                        match status {
                            "SUBMITTED" => {
                                let _ = on_event.send(GenerationEvent::Submitted {
                                    task_id: task_id.to_string(),
                                });
                            }
                            _ => {
                                let _ = on_event.send(GenerationEvent::Polling {
                                    task_id: task_id.to_string(),
                                    status: status.to_string(),
                                });
                            }
                        }
                    },
                )
                .await;

            let final_resp = match result {
                Ok(resp) => resp,
                Err(e) => {
                    let _ = on_event.send(GenerationEvent::Failed {
                        task_id: String::new(),
                        error: e.to_string(),
                    });
                    return Err(e);
                }
            };

            let task_id = final_resp
                .get("output")
                .and_then(|o| o.get("task_id"))
                .and_then(|t| t.as_str())
                .unwrap_or("unknown")
                .to_string();

            let image_urls = extract_images(&final_resp, &mc, &resolved_name);

            // Create assistant message and save generation results
            let assistant_msg = state.db.add_message(
                &conversation_id,
                "assistant",
                None,
                "text2img",
                None,
            )?;

            for url in &image_urls {
                state.db.add_generation_result(
                    &assistant_msg.id,
                    Some(url),
                    None,
                    "image",
                    &resolved_name,
                    None,
                )?;
            }

            let _ = on_event.send(GenerationEvent::Succeeded {
                task_id,
                message_id: assistant_msg.id,
                image_urls,
            });
        }
        ApiMode::Sync => {
            let resp = state
                .qwen_client
                .execute_sync(&mc.url, &api_key, &body, &extra_headers)
                .await?;

            let image_urls = extract_images(&resp, &mc, &resolved_name);

            if image_urls.is_empty() {
                // Debug: log the response structure for troubleshooting
                let resp_debug = truncate_base64_in_json(&resp);
                eprintln!("[DEBUG] No images extracted. Response: {}", 
                    serde_json::to_string_pretty(&resp_debug).unwrap_or_default());
                eprintln!("[DEBUG] response_image_path: {:?}", mc.response_image_path);
                eprintln!("[DEBUG] response_format: {:?}", mc.response_format);
                
                return Err(AppError::Api(format!(
                    "No images returned from API response. Check response_image_path configuration. Response keys: {:?}",
                    resp.as_object().map(|o| o.keys().collect::<Vec<_>>()).unwrap_or_default()
                )));
            }

            let assistant_msg = state.db.add_message(
                &conversation_id,
                "assistant",
                None,
                "text2img",
                None,
            )?;

            for url in &image_urls {
                state.db.add_generation_result(
                    &assistant_msg.id,
                    Some(url),
                    None,
                    "image",
                    &resolved_name,
                    None,
                )?;
            }

            let _ = on_event.send(GenerationEvent::Succeeded {
                task_id: "sync".to_string(),
                message_id: assistant_msg.id,
                image_urls,
            });
        }
    }

    Ok(())
}

/// Save an image to a user-specified location.
///
/// Downloads from URL or copies from local path. If `save_path` is not provided,
/// opens a save dialog.
#[tauri::command]
pub async fn save_image(
    app: tauri::AppHandle,
    source_url: Option<String>,
    source_path: Option<String>,
    save_path: Option<String>,
) -> Result<String, AppError> {
    use tauri_plugin_dialog::DialogExt;

    // Determine the final save path
    let final_path = if let Some(path) = save_path {
        std::path::PathBuf::from(path)
    } else {
        // Show save dialog
        let dialog_result = app
            .dialog()
            .file()
            .set_title("Save Image")
            .add_filter("Images", &["png", "jpg", "jpeg", "webp", "gif"])
            .set_file_name("generated_image.png")
            .blocking_save_file();

        match dialog_result {
            Some(file_path) => file_path.as_path().unwrap().to_path_buf(),
            None => return Err(AppError::Io("Save cancelled by user".to_string())),
        }
    };

    // Download from URL or copy from local path
    if let Some(url) = source_url {
        let response = reqwest::get(&url).await?;
        let bytes = response.bytes().await?;
        std::fs::write(&final_path, &bytes)?;
    } else if let Some(local) = source_path {
        std::fs::copy(&local, &final_path)?;
    } else {
        return Err(AppError::Validation(
            "Either source_url or source_path must be provided".to_string(),
        ));
    }

    Ok(final_path.to_string_lossy().to_string())
}

/// Save clipboard image data to a temporary file.
///
/// Receives raw image bytes from the frontend, writes to
/// `{temp_dir}/qwenimager-clipboard/{uuid}.{ext}`, returns the path.
#[tauri::command]
pub fn save_clipboard_image(image_data: Vec<u8>, mime_type: String) -> Result<String, AppError> {
    let ext = match mime_type.as_str() {
        "image/png" => "png",
        "image/jpeg" => "jpg",
        "image/webp" => "webp",
        "image/gif" => "gif",
        _ => "png",
    };

    let temp_dir = std::env::temp_dir().join("qwenimager-clipboard");
    std::fs::create_dir_all(&temp_dir)?;

    let filename = format!("{}.{}", uuid::Uuid::new_v4(), ext);
    let file_path = temp_dir.join(&filename);

    std::fs::write(&file_path, &image_data)?;

    Ok(file_path.to_string_lossy().to_string())
}

/// Image-to-image editing command.
///
/// Builds a config-driven request with interleaved images and text,
/// sends via the appropriate mode, and saves the results.
#[tauri::command]
pub async fn edit_image(
    state: State<'_, Arc<AppState>>,
    conversation_id: String,
    image_paths: Vec<String>,
    prompt: String,
    model_name: Option<String>,
    params: Option<GenerationParams>,
) -> Result<Value, AppError> {
    let (api_key, resolved_name, mc) = resolve_model(&model_name, ServiceType::Img2img)?;

    // Validate inputs
    if image_paths.is_empty() {
        return Err(AppError::Validation(
            "At least one image is required".to_string(),
        ));
    }
    if prompt.trim().is_empty() {
        return Err(AppError::Validation(
            "Prompt text is required".to_string(),
        ));
    }

    // Create user message in DB
    let user_msg = state.db.add_message(
        &conversation_id,
        "user",
        Some(&prompt),
        "img2img",
        None,
    )?;

    // Save attachments for the user message
    for (i, path) in image_paths.iter().enumerate() {
        let metadata = std::fs::metadata(path).map_err(|e| {
            AppError::Io(format!("Cannot read image file {}: {}", path, e))
        })?;
        let file_size = metadata.len() as i64;

        let mime = match std::path::Path::new(path)
            .extension()
            .and_then(|e| e.to_str())
            .unwrap_or("png")
            .to_lowercase()
            .as_str()
        {
            "jpg" | "jpeg" => "image/jpeg",
            "webp" => "image/webp",
            "gif" => "image/gif",
            _ => "image/png",
        };

        let source = if path.contains("qwenimager-clipboard") {
            "clipboard"
        } else {
            "upload"
        };

        state.db.add_attachment(
            &user_msg.id,
            path,
            i as i32,
            file_size,
            mime,
            source,
        )?;
    }

    // Build content arrays in different formats for various API providers
    let mut dashscope_content: Vec<Value> = Vec::new();
    let mut openai_content: Vec<Value> = Vec::new();
    let mut gemini_parts: Vec<Value> = Vec::new();

    for path in &image_paths {
        let image_data_uri = if path.starts_with("http") {
            path.clone()
        } else {
            crate::services::image_utils::encode_image_to_data_uri(path)?
        };

        // DashScope format: { "image": "data:..." }
        dashscope_content.push(json!({ "image": image_data_uri.clone() }));

        // OpenAI/LiteLLM format: { "type": "image_url", "image_url": { "url": "data:..." } }
        openai_content.push(json!({
            "type": "image_url",
            "image_url": { "url": image_data_uri.clone() }
        }));

        // Gemini format: { "inlineData": { "mimeType": "...", "data": "..." } }
        // Extract mime and base64 from data URI
        if let Some((mime, base64_data)) = parse_data_uri(&image_data_uri) {
            gemini_parts.push(json!({
                "inlineData": {
                    "mimeType": mime,
                    "data": base64_data
                }
            }));
        }
    }

    // Add text content
    dashscope_content.push(json!({ "text": prompt.clone() }));
    openai_content.push(json!({ "type": "text", "text": prompt.clone() }));
    gemini_parts.push(json!({ "text": prompt.clone() }));

    // Build template variables
    let gen_params = params.unwrap_or_default();

    let mut vars: HashMap<&str, Value> = HashMap::new();
    vars.insert("model", Value::String(resolved_name.clone()));
    vars.insert("prompt", Value::String(prompt.clone()));

    // Only inject {size} when the model declares supports_size = true
    if mc.supports_size {
        let size = gen_params
            .size
            .clone()
            .unwrap_or_else(|| "1280*1280".to_string());
        vars.insert("size", Value::String(size));
    }
    // {content} for DashScope format (backward compatible)
    vars.insert("content", Value::Array(dashscope_content));
    // {openai_content} for OpenAI/LiteLLM format
    vars.insert("openai_content", Value::Array(openai_content));
    // {gemini_parts} for direct Gemini Vertex AI format
    vars.insert("gemini_parts", Value::Array(gemini_parts));

    // Use configured template or default
    let template = mc
        .request_template
        .clone()
        .unwrap_or_else(default_img2img_template);
    let body = render_template(&template, &vars)
        .ok_or_else(|| AppError::Api("Failed to render request template".to_string()))?;

    let extra_headers = mc.headers.clone().unwrap_or_default();

    let (mode, poll_config) = resolve_mode_and_poll(&mc, &resolved_name);

    let resp = match mode {
        ApiMode::AsyncPoll => {
            let poll_cfg = poll_config.unwrap_or_else(default_dashscope_poll_config);
            state
                .qwen_client
                .execute_async_poll(
                    &mc.url,
                    &api_key,
                    &body,
                    &poll_cfg,
                    &extra_headers,
                    |_task_id, _status| {},
                )
                .await?
        }
        ApiMode::Sync => {
            state
                .qwen_client
                .execute_sync(&mc.url, &api_key, &body, &extra_headers)
                .await?
        }
    };

    // Extract images based on response format
    let mut image_urls = extract_images(&resp, &mc, &resolved_name);
    
    // Fallback for img2img: try results format if choices format returned empty
    if image_urls.is_empty() && mc.response_format == ResponseFormat::Url {
        let response_path = default_response_path(&mc, &resolved_name);
        if response_path.contains("choices") {
            image_urls = extract_strings(&resp, "output.results[*].url");
        }
    }

    if image_urls.is_empty() {
        return Err(AppError::Api(
            "No images returned from edit API".to_string(),
        ));
    }

    // Create assistant message and save generation results
    let assistant_msg = state.db.add_message(
        &conversation_id,
        "assistant",
        None,
        "img2img",
        None,
    )?;

    for url in &image_urls {
        state.db.add_generation_result(
            &assistant_msg.id,
            Some(url),
            None,
            "image",
            &resolved_name,
            None,
        )?;
    }

    Ok(json!({
        "messageId": assistant_msg.id,
        "imageUrls": image_urls,
    }))
}
