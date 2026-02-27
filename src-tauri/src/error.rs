use serde::Serialize;

/// Unified error type for the application.
/// Serialized as `{ kind, message }` for frontend consumption.
#[derive(Debug, thiserror::Error)]
pub enum AppError {
    #[error("Network error: {0}")]
    Network(String),

    #[error("API error: {0}")]
    Api(String),

    #[error("Request timed out: {0}")]
    Timeout(String),

    #[error("Configuration error: {0}")]
    Config(String),

    #[error("Unauthorized: {0}")]
    Unauthorized(String),

    #[error("Rate limited: {0}")]
    RateLimited(String),

    #[error("IO error: {0}")]
    Io(String),

    #[error("Not found: {0}")]
    NotFound(String),

    #[error("Validation error: {0}")]
    Validation(String),
}

impl AppError {
    /// Returns the error kind string used by the frontend to determine display strategy.
    fn kind(&self) -> &'static str {
        match self {
            AppError::Network(_) => "network",
            AppError::Api(_) => "api",
            AppError::Timeout(_) => "timeout",
            AppError::Config(_) => "config",
            AppError::Unauthorized(_) => "unauthorized",
            AppError::RateLimited(_) => "rateLimited",
            AppError::Io(_) => "io",
            AppError::NotFound(_) => "notFound",
            AppError::Validation(_) => "validation",
        }
    }
}

/// Serialized form sent to the frontend: `{ kind: string, message: string }`.
impl Serialize for AppError {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut state = serializer.serialize_struct("AppError", 2)?;
        state.serialize_field("kind", self.kind())?;
        state.serialize_field("message", &self.to_string())?;
        state.end()
    }
}

// Convenience conversions

impl From<std::io::Error> for AppError {
    fn from(err: std::io::Error) -> Self {
        AppError::Io(err.to_string())
    }
}

impl From<serde_json::Error> for AppError {
    fn from(err: serde_json::Error) -> Self {
        AppError::Config(format!("JSON parse error: {}", err))
    }
}

impl From<rusqlite::Error> for AppError {
    fn from(err: rusqlite::Error) -> Self {
        AppError::Io(format!("Database error: {}", err))
    }
}

impl From<reqwest::Error> for AppError {
    fn from(err: reqwest::Error) -> Self {
        if err.is_timeout() {
            AppError::Timeout(err.to_string())
        } else {
            AppError::Network(err.to_string())
        }
    }
}
