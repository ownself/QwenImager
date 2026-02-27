use std::sync::Arc;

use tauri::ipc::Channel;
use tauri::State;

use crate::error::AppError;
use crate::models::api::{
    GenerationEvent, GenerationParams, ImageEditContent, ImageEditInput, ImageEditMessage,
    ImageEditRequest, TextToImageInput, TextToImageRequest,
};
use crate::services::config_loader;
use crate::AppState;

/// Text-to-image generation command.
///
/// Submits an async task to the DashScope API, polls for completion,
/// and sends progress events via a Tauri Channel.
#[tauri::command]
pub async fn generate_image(
    state: State<'_, Arc<AppState>>,
    conversation_id: String,
    prompt: String,
    params: Option<GenerationParams>,
    on_event: Channel<GenerationEvent>,
) -> Result<(), AppError> {
    // Load config to get API key and model URL
    let (config, _models) = config_loader::load_config_inner()?;
    let provider = config.providers.get("qwen").ok_or_else(|| {
        AppError::Config("Qwen provider not configured".to_string())
    })?;
    let api_key = &provider.api_key;

    // Find text2img model URL (try common names)
    let model_entry = provider
        .models
        .iter()
        .find(|(k, _)| k.contains("image") && !k.contains("edit") && !k.contains("mt"))
        .or_else(|| provider.models.iter().next())
        .ok_or_else(|| AppError::Config("No text2img model configured".to_string()))?;

    let model_name = model_entry.0.clone();
    let model_url = &model_entry.1.url;

    // Create user message in DB
    let _user_msg = state.db.add_message(
        &conversation_id,
        "user",
        Some(&prompt),
        "text2img",
        None,
    )?;

    // Build API request
    let mut gen_params = params.unwrap_or_default();
    if gen_params.size.is_none() {
        gen_params.size = Some("1024*1024".to_string());
    }
    if gen_params.prompt_extend.is_none() {
        gen_params.prompt_extend = Some(true);
    }
    if gen_params.watermark.is_none() {
        gen_params.watermark = Some(false);
    }

    let request = TextToImageRequest {
        model: model_name.clone(),
        input: TextToImageInput {
            prompt: prompt.clone(),
        },
        parameters: Some(gen_params),
    };

    // Submit async task
    let submit_response = state
        .qwen_client
        .submit_async_task(model_url, api_key, &request)
        .await?;

    let task_id = submit_response.output.task_id;

    // Send "submitted" event
    let _ = on_event.send(GenerationEvent::Submitted {
        task_id: task_id.clone(),
    });

    // Poll loop: every 3s, max 180s (60 iterations)
    let max_polls = 60;
    for _ in 0..max_polls {
        tokio::time::sleep(std::time::Duration::from_secs(3)).await;

        let poll_response = state.qwen_client.poll_task(&task_id, api_key).await?;
        let status = &poll_response.output.task_status;

        match status.as_str() {
            "SUCCEEDED" => {
                // Extract image URLs from results
                let image_urls: Vec<String> = poll_response
                    .output
                    .results
                    .unwrap_or_default()
                    .iter()
                    .filter_map(|r| r.url.clone())
                    .collect();

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
                        &model_name,
                        None,
                    )?;
                }

                let _ = on_event.send(GenerationEvent::Succeeded {
                    task_id: task_id.clone(),
                    message_id: assistant_msg.id,
                    image_urls,
                });

                return Ok(());
            }
            "FAILED" | "CANCELED" => {
                let error_msg = poll_response
                    .output
                    .message
                    .unwrap_or_else(|| format!("Task {}", status));

                let _ = on_event.send(GenerationEvent::Failed {
                    task_id: task_id.clone(),
                    error: error_msg.clone(),
                });

                return Err(AppError::Api(error_msg));
            }
            _ => {
                // PENDING or RUNNING — send polling event and continue
                let _ = on_event.send(GenerationEvent::Polling {
                    task_id: task_id.clone(),
                    status: status.clone(),
                });
            }
        }
    }

    // Timeout after max polls
    let _ = on_event.send(GenerationEvent::Failed {
        task_id: task_id.clone(),
        error: "Generation timed out after 180 seconds".to_string(),
    });
    Err(AppError::Timeout(
        "Image generation timed out after 180 seconds".to_string(),
    ))
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
pub fn save_clipboard_image(
    image_data: Vec<u8>,
    mime_type: String,
) -> Result<String, AppError> {
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

/// Image-to-image editing command (synchronous API).
///
/// Builds a messages-format request with interleaved images and text,
/// sends a sync request, and saves the results.
#[tauri::command]
pub async fn edit_image(
    state: State<'_, Arc<AppState>>,
    conversation_id: String,
    image_paths: Vec<String>,
    prompt: String,
    params: Option<GenerationParams>,
) -> Result<serde_json::Value, AppError> {
    // Load config
    let (config, _models) = config_loader::load_config_inner()?;
    let provider = config.providers.get("qwen").ok_or_else(|| {
        AppError::Config("Qwen provider not configured".to_string())
    })?;
    let api_key = &provider.api_key;

    // Find img2img model URL (contains "edit")
    let model_entry = provider
        .models
        .iter()
        .find(|(k, _)| k.contains("edit"))
        .or_else(|| provider.models.iter().next())
        .ok_or_else(|| AppError::Config("No img2img model configured".to_string()))?;

    let model_name = model_entry.0.clone();
    let model_url = &model_entry.1.url;

    // Validate inputs
    if image_paths.is_empty() {
        return Err(AppError::Validation("At least one image is required".to_string()));
    }
    if prompt.trim().is_empty() {
        return Err(AppError::Validation("Prompt text is required".to_string()));
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

        // Detect MIME type from extension
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

        // Determine source based on path
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

    // Build content array: images first (in order), then text
    let mut content: Vec<ImageEditContent> = Vec::new();
    for path in &image_paths {
        // For the API: HTTP URLs pass through, local files are Base64-encoded
        let image_url = if path.starts_with("http") {
            path.clone()
        } else {
            crate::services::image_utils::encode_image_to_data_uri(path)?
        };
        content.push(ImageEditContent::Image { image: image_url });
    }
    content.push(ImageEditContent::Text { text: prompt.clone() });

    // Build generation params
    let mut gen_params = params.unwrap_or_default();
    if gen_params.prompt_extend.is_none() {
        gen_params.prompt_extend = Some(true);
    }
    if gen_params.watermark.is_none() {
        gen_params.watermark = Some(false);
    }

    let request = ImageEditRequest {
        model: model_name.clone(),
        input: ImageEditInput {
            messages: vec![ImageEditMessage {
                role: "user".to_string(),
                content,
            }],
        },
        parameters: Some(gen_params),
    };

    // Send synchronous request (no X-DashScope-Async header)
    let response = state
        .qwen_client
        .send_sync_request(model_url, api_key, &request)
        .await?;

    // Extract image URLs from the response
    let mut image_urls: Vec<String> = Vec::new();

    // Try choices format first (multimodal-generation response)
    if let Some(choices) = &response.output.choices {
        for choice in choices {
            if let Some(msg) = &choice.message {
                if let Some(contents) = &msg.content {
                    for c in contents {
                        if let crate::models::api::SyncChoiceContent::Image { image } = c {
                            image_urls.push(image.clone());
                        }
                    }
                }
            }
        }
    }

    // Fallback to results format
    if image_urls.is_empty() {
        if let Some(results) = &response.output.results {
            for r in results {
                if let Some(url) = &r.url {
                    image_urls.push(url.clone());
                }
            }
        }
    }

    if image_urls.is_empty() {
        return Err(AppError::Api("No images returned from edit API".to_string()));
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
            &model_name,
            None,
        )?;
    }

    // Return EditResult format
    Ok(serde_json::json!({
        "messageId": assistant_msg.id,
        "imageUrls": image_urls,
    }))
}
