use std::sync::Arc;

use tauri::ipc::Channel;
use tauri::State;

use crate::error::AppError;
use crate::models::api::{
    GenerationEvent, TranslationExt, TranslationExtConfig, TranslationInput, TranslationRequest,
};
use crate::services::config_loader;
use crate::AppState;

/// Image translation command (async submit-poll pattern).
///
/// Submits a translation task to the DashScope API, polls for completion,
/// and sends progress events via a Tauri Channel.
#[tauri::command]
pub async fn translate_image(
    state: State<'_, Arc<AppState>>,
    conversation_id: String,
    image_path: String,
    source_lang: String,
    target_lang: String,
    on_event: Channel<GenerationEvent>,
) -> Result<(), AppError> {
    // Validate languages are different
    if source_lang == target_lang {
        return Err(AppError::Validation(
            "Source and target languages must be different".to_string(),
        ));
    }

    // Load config to get API key and model URL
    let (config, _models) = config_loader::load_config_inner()?;
    let provider = config.providers.get("qwen").ok_or_else(|| {
        AppError::Config("Qwen provider not configured".to_string())
    })?;
    let api_key = &provider.api_key;

    // Find translation model URL (contains "mt")
    let model_entry = provider
        .models
        .iter()
        .find(|(k, _)| k.contains("mt"))
        .or_else(|| provider.models.iter().next())
        .ok_or_else(|| AppError::Config("No translation model configured".to_string()))?;

    let model_name = model_entry.0.clone();
    let model_url = &model_entry.1.url;

    // Build image URL: HTTP URLs pass through, local files are Base64-encoded
    let image_url = if image_path.starts_with("http") {
        image_path.clone()
    } else {
        crate::services::image_utils::encode_image_to_data_uri(&image_path)?
    };

    // Create user message in DB (no text content for translation, image is the input)
    let user_msg = state.db.add_message(
        &conversation_id,
        "user",
        Some(&format!("{} → {}", source_lang, target_lang)),
        "translate",
        None,
    )?;

    // Save the image as an attachment on the user message
    let metadata = std::fs::metadata(&image_path).map_err(|e| {
        AppError::Io(format!("Cannot read image file {}: {}", image_path, e))
    })?;
    let file_size = metadata.len() as i64;

    let mime = match std::path::Path::new(&image_path)
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

    let source = if image_path.contains("qwenimager-clipboard") {
        "clipboard"
    } else {
        "upload"
    };

    state.db.add_attachment(
        &user_msg.id,
        &image_path,
        0,
        file_size,
        mime,
        source,
    )?;

    // Build translation API request
    let request = TranslationRequest {
        model: model_name.clone(),
        input: TranslationInput {
            image_url,
            source_lang: source_lang.clone(),
            target_lang: target_lang.clone(),
            ext: Some(TranslationExt {
                config: TranslationExtConfig {
                    image_segment: false,
                },
            }),
        },
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
                    "translate",
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
        error: "Translation timed out after 180 seconds".to_string(),
    });
    Err(AppError::Timeout(
        "Image translation timed out after 180 seconds".to_string(),
    ))
}
