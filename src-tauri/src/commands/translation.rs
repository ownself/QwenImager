use std::collections::HashMap;
use std::sync::Arc;

use serde_json::Value;
use tauri::ipc::Channel;
use tauri::State;

use crate::error::AppError;
use crate::models::api::GenerationEvent;
use crate::models::config::{ApiMode, AsyncPollConfig, ServiceType};
use crate::services::config_loader;
use crate::services::template_engine::{
    default_translate_template, extract_strings, infer_service_type, render_template,
};
use crate::AppState;

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

/// Image translation command.
///
/// Builds a config-driven request, sends it via the appropriate mode
/// (sync or async_poll), and sends progress events via a Tauri Channel.
#[tauri::command]
pub async fn translate_image(
    state: State<'_, Arc<AppState>>,
    conversation_id: String,
    image_path: String,
    source_lang: String,
    target_lang: String,
    model_name: Option<String>,
    on_event: Channel<GenerationEvent>,
) -> Result<(), AppError> {
    // Validate languages are different
    if source_lang == target_lang {
        return Err(AppError::Validation(
            "Source and target languages must be different".to_string(),
        ));
    }

    // Resolve model config
    let (config, _models) = config_loader::load_config_inner()?;

    let (api_key, resolved_name, mc) = if let Some(ref name) = model_name {
        let (key, mc) = config_loader::find_model_config(&config, name)?;
        (key.to_string(), name.clone(), mc.clone())
    } else {
        let (key, name, mc) =
            config_loader::find_model_by_service_type(&config, ServiceType::Translate)?;
        (key.to_string(), name, mc.clone())
    };

    // Build image URL: HTTP URLs pass through, local files are Base64-encoded
    let image_url = if image_path.starts_with("http") {
        image_path.clone()
    } else {
        crate::services::image_utils::encode_image_to_data_uri(&image_path)?
    };

    // Create user message in DB
    let user_msg = state.db.add_message(
        &conversation_id,
        "user",
        Some(&format!("{} → {}", source_lang, target_lang)),
        "translate",
        None,
    )?;

    // Save the image as an attachment on the user message
    let metadata = std::fs::metadata(&image_path)
        .map_err(|e| AppError::Io(format!("Cannot read image file {}: {}", image_path, e)))?;
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

    state
        .db
        .add_attachment(&user_msg.id, &image_path, 0, file_size, mime, source)?;

    // Build template variables
    let mut vars: HashMap<&str, Value> = HashMap::new();
    vars.insert("model", Value::String(resolved_name.clone()));
    vars.insert("image_url", Value::String(image_url));
    vars.insert("source_lang", Value::String(source_lang.clone()));
    vars.insert("target_lang", Value::String(target_lang.clone()));

    // Use configured template or default
    let template = mc
        .request_template
        .clone()
        .unwrap_or_else(default_translate_template);
    let body = render_template(&template, &vars)
        .ok_or_else(|| AppError::Api("Failed to render request template".to_string()))?;

    let extra_headers = mc.headers.clone().unwrap_or_default();
    let response_path = mc
        .response_image_path
        .clone()
        .unwrap_or_else(|| "output.image_url".to_string());

    // Resolve effective mode
    let (mode, poll_config) = match mc.mode {
        ApiMode::AsyncPoll => {
            let poll = mc
                .async_poll
                .clone()
                .unwrap_or_else(default_dashscope_poll_config);
            (ApiMode::AsyncPoll, Some(poll))
        }
        ApiMode::Sync => {
            // Backward compat: old configs have mode=sync (default), but translate
            // was historically async on DashScope
            if mc.service_type.is_none() && mc.async_poll.is_none() {
                let inferred = mc
                    .service_type
                    .unwrap_or_else(|| infer_service_type(&resolved_name));
                if inferred == ServiceType::Translate {
                    (ApiMode::AsyncPoll, Some(default_dashscope_poll_config()))
                } else {
                    (ApiMode::Sync, None)
                }
            } else {
                (ApiMode::Sync, None)
            }
        }
    };

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
                    |task_id, status| match status {
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

            let image_urls = extract_strings(&final_resp, &response_path);

            // Create assistant message and save generation results
            let assistant_msg =
                state
                    .db
                    .add_message(&conversation_id, "assistant", None, "translate", None)?;

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

            let image_urls = extract_strings(&resp, &response_path);

            if image_urls.is_empty() {
                return Err(AppError::Api(
                    "No images returned from translation API".to_string(),
                ));
            }

            let assistant_msg =
                state
                    .db
                    .add_message(&conversation_id, "assistant", None, "translate", None)?;

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
