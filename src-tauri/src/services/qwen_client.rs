use reqwest::Client;
use serde::Serialize;

use crate::error::AppError;
use crate::models::api::{AsyncTaskResponse, SyncGenerationResponse, TaskQueryResponse};

const TASK_POLL_URL: &str = "https://dashscope.aliyuncs.com/api/v1/tasks";

/// HTTP client for interacting with the DashScope (Qwen) API.
pub struct QwenClient {
    client: Client,
}

impl QwenClient {
    pub fn new() -> Self {
        let client = Client::builder()
            .timeout(std::time::Duration::from_secs(60))
            .build()
            .expect("Failed to build reqwest client");
        QwenClient { client }
    }

    /// Submit an async task (text-to-image, translation).
    ///
    /// Sends a POST request with `X-DashScope-Async: enable` header.
    /// Returns the task_id on success. Retries up to 3 times on 429.
    pub async fn submit_async_task<T: Serialize>(
        &self,
        url: &str,
        api_key: &str,
        body: &T,
    ) -> Result<AsyncTaskResponse, AppError> {
        let max_retries = 3;
        for attempt in 0..=max_retries {
            let resp = self
                .client
                .post(url)
                .header("Authorization", format!("Bearer {}", api_key))
                .header("Content-Type", "application/json")
                .header("X-DashScope-Async", "enable")
                .json(body)
                .send()
                .await?;

            let status = resp.status();

            if status.as_u16() == 429 && attempt < max_retries {
                let retry_after = resp
                    .headers()
                    .get("retry-after")
                    .and_then(|v| v.to_str().ok())
                    .and_then(|v| v.parse::<u64>().ok())
                    .unwrap_or(3);
                tokio::time::sleep(std::time::Duration::from_secs(retry_after)).await;
                continue;
            }

            if !status.is_success() {
                return Err(map_http_error(status.as_u16(), &resp.text().await.unwrap_or_default()));
            }

            let result: AsyncTaskResponse = resp.json().await.map_err(|e| {
                AppError::Api(format!("Failed to parse async task response: {}", e))
            })?;

            return Ok(result);
        }

        Err(AppError::RateLimited(
            "Rate limited after 3 retries. Please try again later.".to_string(),
        ))
    }

    /// Poll the status of an async task by task_id.
    ///
    /// GET `https://dashscope.aliyuncs.com/api/v1/tasks/{task_id}`
    pub async fn poll_task(
        &self,
        task_id: &str,
        api_key: &str,
    ) -> Result<TaskQueryResponse, AppError> {
        let url = format!("{}/{}", TASK_POLL_URL, task_id);
        let resp = self
            .client
            .get(&url)
            .header("Authorization", format!("Bearer {}", api_key))
            .send()
            .await?;

        let status = resp.status();
        if !status.is_success() {
            return Err(map_http_error(status.as_u16(), &resp.text().await.unwrap_or_default()));
        }

        let result: TaskQueryResponse = resp.json().await.map_err(|e| {
            AppError::Api(format!("Failed to parse task query response: {}", e))
        })?;

        Ok(result)
    }

    /// Send a synchronous request (image-to-image edit).
    ///
    /// No `X-DashScope-Async` header — waits for the full response.
    /// Retries up to 3 times on 429.
    pub async fn send_sync_request<T: Serialize>(
        &self,
        url: &str,
        api_key: &str,
        body: &T,
    ) -> Result<SyncGenerationResponse, AppError> {
        let max_retries = 3;
        for attempt in 0..=max_retries {
            let resp = self
                .client
                .post(url)
                .header("Authorization", format!("Bearer {}", api_key))
                .header("Content-Type", "application/json")
                .json(body)
                .send()
                .await?;

            let status = resp.status();

            if status.as_u16() == 429 && attempt < max_retries {
                let retry_after = resp
                    .headers()
                    .get("retry-after")
                    .and_then(|v| v.to_str().ok())
                    .and_then(|v| v.parse::<u64>().ok())
                    .unwrap_or(3);
                tokio::time::sleep(std::time::Duration::from_secs(retry_after)).await;
                continue;
            }

            if !status.is_success() {
                return Err(map_http_error(status.as_u16(), &resp.text().await.unwrap_or_default()));
            }

            let result: SyncGenerationResponse = resp.json().await.map_err(|e| {
                AppError::Api(format!("Failed to parse sync generation response: {}", e))
            })?;

            return Ok(result);
        }

        Err(AppError::RateLimited(
            "Rate limited after 3 retries. Please try again later.".to_string(),
        ))
    }
}

/// Map HTTP status codes to semantic `AppError` variants.
fn map_http_error(status: u16, body: &str) -> AppError {
    match status {
        400 => AppError::Validation(format!("Bad request: {}", body)),
        401 => AppError::Unauthorized(
            "Invalid API key. Please check your apiKey in ~/.qwenimage/setting.json".to_string(),
        ),
        429 => AppError::RateLimited(format!(
            "Rate limited by the API. Please try again later. {}",
            body
        )),
        500..=599 => AppError::Api(format!("Server error ({}): {}", status, body)),
        _ => AppError::Api(format!("HTTP error ({}): {}", status, body)),
    }
}
