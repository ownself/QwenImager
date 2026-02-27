use std::collections::HashMap;

use reqwest::Client;
use serde_json::Value;

use crate::error::AppError;
use crate::models::config::AsyncPollConfig;

/// HTTP client for interacting with AI image generation APIs.
///
/// Provides two unified execution modes:
/// - `execute_sync` — single POST request, returns response JSON
/// - `execute_async_poll` — POST submit + GET poll loop, returns final response JSON
///
/// Both methods accept generic `serde_json::Value` request bodies and return
/// `serde_json::Value` responses, driven entirely by configuration.
pub struct QwenClient {
    client: Client,
}

impl QwenClient {
    pub fn new() -> Self {
        let client = Client::builder()
            .timeout(std::time::Duration::from_secs(120))
            .build()
            .expect("Failed to build reqwest client");
        QwenClient { client }
    }

    /// Execute a synchronous API call.
    ///
    /// Sends a POST request with the given body and headers, returns the
    /// parsed JSON response. Retries up to 3 times on HTTP 429.
    pub async fn execute_sync(
        &self,
        url: &str,
        api_key: &str,
        body: &Value,
        extra_headers: &HashMap<String, String>,
    ) -> Result<Value, AppError> {
        let max_retries = 3;
        for attempt in 0..=max_retries {
            let mut req = self
                .client
                .post(url)
                .header("Authorization", format!("Bearer {}", api_key))
                .header("Content-Type", "application/json");

            for (k, v) in extra_headers {
                req = req.header(k.as_str(), v.as_str());
            }

            let resp = req.json(body).send().await?;
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
                return Err(map_http_error(
                    status.as_u16(),
                    &resp.text().await.unwrap_or_default(),
                ));
            }

            let result: Value = resp.json().await.map_err(|e| {
                AppError::Api(format!("Failed to parse API response: {}", e))
            })?;

            return Ok(result);
        }

        Err(AppError::RateLimited(
            "Rate limited after 3 retries. Please try again later.".to_string(),
        ))
    }

    /// Execute an async submit+poll API call.
    ///
    /// 1. Sends a POST request with `poll_config.submit_headers` merged into
    ///    `extra_headers`.
    /// 2. Extracts `output.task_id` from the submit response.
    /// 3. Polls `poll_config.poll_url` (with `{task_id}` replaced) at the
    ///    configured interval until success, failure, or timeout.
    /// 4. Calls `on_poll(task_id, status)` on each poll iteration.
    /// 5. Returns the final successful response JSON.
    pub async fn execute_async_poll(
        &self,
        url: &str,
        api_key: &str,
        body: &Value,
        poll_config: &AsyncPollConfig,
        extra_headers: &HashMap<String, String>,
        on_poll: impl Fn(&str, &str),
    ) -> Result<Value, AppError> {
        // Merge submit_headers with extra_headers (submit_headers take precedence)
        let mut merged_headers = extra_headers.clone();
        for (k, v) in &poll_config.submit_headers {
            merged_headers.insert(k.clone(), v.clone());
        }

        // Submit the task
        let submit_resp = self
            .execute_sync(url, api_key, body, &merged_headers)
            .await?;

        // Extract task_id from response
        let task_id = submit_resp
            .get("output")
            .and_then(|o| o.get("task_id"))
            .and_then(|t| t.as_str())
            .ok_or_else(|| {
                AppError::Api(format!(
                    "Async submit response missing output.task_id. Response: {}",
                    serde_json::to_string(&submit_resp).unwrap_or_default()
                ))
            })?
            .to_string();

        // Notify caller of successful submission
        on_poll(&task_id, "SUBMITTED");

        // Build poll URL
        let poll_url = poll_config.poll_url.replace("{task_id}", &task_id);

        // Poll loop
        let interval = std::time::Duration::from_secs(poll_config.poll_interval_secs);
        let max_polls = poll_config.timeout_secs / poll_config.poll_interval_secs;

        for _ in 0..max_polls {
            tokio::time::sleep(interval).await;

            let resp = self
                .client
                .get(&poll_url)
                .header("Authorization", format!("Bearer {}", api_key))
                .send()
                .await?;

            let status_code = resp.status();
            if !status_code.is_success() {
                return Err(map_http_error(
                    status_code.as_u16(),
                    &resp.text().await.unwrap_or_default(),
                ));
            }

            let poll_resp: Value = resp.json().await.map_err(|e| {
                AppError::Api(format!("Failed to parse poll response: {}", e))
            })?;

            let task_status = poll_resp
                .get("output")
                .and_then(|o| o.get("task_status"))
                .and_then(|s| s.as_str())
                .unwrap_or("UNKNOWN");

            match task_status {
                "SUCCEEDED" => return Ok(poll_resp),
                "FAILED" | "CANCELED" => {
                    let error_msg = poll_resp
                        .get("output")
                        .and_then(|o| o.get("message"))
                        .and_then(|m| m.as_str())
                        .unwrap_or("Task failed")
                        .to_string();
                    return Err(AppError::Api(error_msg));
                }
                _ => {
                    // PENDING or RUNNING — notify caller and continue
                    on_poll(&task_id, task_status);
                }
            }
        }

        Err(AppError::Timeout(format!(
            "Async task '{}' timed out after {} seconds",
            task_id, poll_config.timeout_secs
        )))
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
