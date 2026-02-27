use base64::prelude::*;

use crate::error::AppError;

/// Reads a local image file and encodes it as a Base64 data URI.
///
/// Returns a string in the format `data:{mime_type};base64,{base64_data}`,
/// which is accepted by the DashScope API for image inputs.
///
/// # MIME Type Detection
///
/// The MIME type is determined from the file extension:
/// - `jpg`, `jpeg` → `image/jpeg`
/// - `png` → `image/png`
/// - `webp` → `image/webp`
/// - `gif` → `image/gif`
/// - `bmp` → `image/bmp`
/// - `tiff`, `tif` → `image/tiff`
/// - anything else → `image/png` (safe default)
pub fn encode_image_to_data_uri(path: &str) -> Result<String, AppError> {
    // Read file bytes
    let bytes = std::fs::read(path)
        .map_err(|e| AppError::Io(format!("Failed to read image file '{}': {}", path, e)))?;

    // Detect MIME type from file extension
    let mime_type = match std::path::Path::new(path)
        .extension()
        .and_then(|e| e.to_str())
        .unwrap_or("png")
        .to_lowercase()
        .as_str()
    {
        "jpg" | "jpeg" => "image/jpeg",
        "png" => "image/png",
        "webp" => "image/webp",
        "gif" => "image/gif",
        "bmp" => "image/bmp",
        "tiff" | "tif" => "image/tiff",
        _ => "image/png",
    };

    // Encode to Base64
    let base64_data = BASE64_STANDARD.encode(&bytes);

    // Format as data URI
    Ok(format!("data:{};base64,{}", mime_type, base64_data))
}
