//! Config-driven template engine for API request building and response extraction.
//!
//! Provides:
//! - `render_template()` — Recursive JSON tree walk replacing `{var}` placeholders
//! - `extract_strings()` — Dot-notation path with `[*]` wildcard for extracting values
//! - `infer_service_type()` — Model name heuristic for backward compatibility
//! - Default DashScope request templates for text2img, img2img, translate

use std::collections::HashMap;

use serde_json::{json, Value};

use crate::models::config::ServiceType;

// ── Template Rendering ──

/// Render a JSON template by replacing `{placeholder}` variables with values.
///
/// - If a string value is a single placeholder `{var}` and the var exists,
///   it is replaced by the var's Value (supporting non-string replacement).
/// - If the var does not exist, the field is omitted (returns None).
/// - Embedded placeholders in strings are replaced as string interpolation.
/// - Objects are recursed; keys whose value renders to None are omitted.
/// - Arrays are recursed; items that render to None are dropped.
/// - Non-string primitives (numbers, booleans, null) pass through unchanged.
pub fn render_template(template: &Value, vars: &HashMap<&str, Value>) -> Option<Value> {
    match template {
        Value::String(s) => {
            // Check if the entire value is a single placeholder: "{var_name}"
            if s.starts_with('{') && s.ends_with('}') && s.matches('{').count() == 1 {
                let key = &s[1..s.len() - 1];
                // Return the variable's Value directly (supports non-string types)
                // If the variable is missing, return None to omit this field
                return vars.get(key).cloned();
            }
            // Embedded placeholders: replace all {var} occurrences as strings
            let mut result = s.clone();
            for (key, val) in vars {
                if let Value::String(val_str) = val {
                    result = result.replace(&format!("{{{}}}", key), val_str);
                }
            }
            Some(Value::String(result))
        }
        Value::Object(map) => {
            let mut out = serde_json::Map::new();
            for (k, v) in map {
                if let Some(rendered) = render_template(v, vars) {
                    out.insert(k.clone(), rendered);
                }
                // If None, key is omitted (optional variable not provided)
            }
            Some(Value::Object(out))
        }
        Value::Array(arr) => {
            let rendered: Vec<Value> = arr
                .iter()
                .filter_map(|v| render_template(v, vars))
                .collect();
            Some(Value::Array(rendered))
        }
        // Numbers, booleans, null pass through unchanged
        other => Some(other.clone()),
    }
}

// ── Response Value Extraction ──

/// Segment of a parsed dot-notation path.
enum Segment {
    /// Object key access: `"output"`, `"results"`, `"url"`
    Key(String),
    /// Array wildcard: `[*]` — iterate all elements
    Wildcard,
}

/// Parse a dot-notation path into segments.
///
/// `"output.results[*].url"` → `[Key("output"), Key("results"), Wildcard, Key("url")]`
fn parse_path(path: &str) -> Vec<Segment> {
    let mut segments = Vec::new();
    for part in path.split('.') {
        if part.is_empty() {
            continue;
        }
        if let Some(key) = part.strip_suffix("[*]") {
            if !key.is_empty() {
                segments.push(Segment::Key(key.to_string()));
            }
            segments.push(Segment::Wildcard);
        } else {
            segments.push(Segment::Key(part.to_string()));
        }
    }
    segments
}

/// Extract string values from a JSON value using a simple dot-notation path.
///
/// Supports object access (`a.b.c`) and array wildcards (`[*]`).
///
/// # Examples
///
/// ```text
/// "output.results[*].url"                       → DashScope async response
/// "output.choices[*].message.content[*].image"  → DashScope sync response
/// "data[*].url"                                 → OpenAI DALL-E response
/// ```
pub fn extract_strings(json: &Value, path: &str) -> Vec<String> {
    let segments = parse_path(path);
    let mut current: Vec<&Value> = vec![json];

    for seg in &segments {
        let mut next: Vec<&Value> = Vec::new();
        for val in &current {
            match seg {
                Segment::Key(key) => {
                    if let Some(child) = val.get(key.as_str()) {
                        next.push(child);
                    }
                }
                Segment::Wildcard => {
                    if let Some(arr) = val.as_array() {
                        next.extend(arr.iter());
                    }
                }
            }
        }
        current = next;
    }

    current
        .into_iter()
        .filter_map(|v| v.as_str().map(|s| s.to_string()))
        .collect()
}

// ── Service Type Inference ──

/// Infer the service type from a model name using heuristics.
///
/// This preserves backward compatibility with the existing model lookup logic:
/// - Name contains `"edit"` → `Img2img`
/// - Name contains `"mt"` → `Translate`
/// - Otherwise → `Text2img`
pub fn infer_service_type(model_name: &str) -> ServiceType {
    if model_name.contains("edit") {
        ServiceType::Img2img
    } else if model_name.contains("mt") {
        ServiceType::Translate
    } else {
        ServiceType::Text2img
    }
}

// ── Default DashScope Request Templates ──

/// Default request template for text-to-image (DashScope format).
///
/// ```json
/// {
///   "model": "{model}",
///   "input": { "prompt": "{prompt}" },
///   "parameters": { "size": "{size}", "prompt_extend": true, "watermark": false }
/// }
/// ```
pub fn default_text2img_template() -> Value {
    json!({
        "model": "{model}",
        "input": {
            "prompt": "{prompt}"
        },
        "parameters": {
            "size": "{size}",
            "prompt_extend": true,
            "watermark": false
        }
    })
}

/// Default request template for image-to-image editing (DashScope format).
///
/// The `{content}` placeholder is replaced at runtime with a JSON array
/// containing interleaved image and text content items.
///
/// ```json
/// {
///   "model": "{model}",
///   "input": { "messages": [{ "role": "user", "content": "{content}" }] },
///   "parameters": { "prompt_extend": true, "watermark": false }
/// }
/// ```
pub fn default_img2img_template() -> Value {
    json!({
        "model": "{model}",
        "input": {
            "messages": [{
                "role": "user",
                "content": "{content}"
            }]
        },
        "parameters": {
            "prompt_extend": true,
            "watermark": false
        }
    })
}

/// Default request template for image translation (DashScope format).
///
/// ```json
/// {
///   "model": "{model}",
///   "input": { "image_url": "{image_url}", "source_lang": "{source_lang}", "target_lang": "{target_lang}" }
/// }
/// ```
pub fn default_translate_template() -> Value {
    json!({
        "model": "{model}",
        "input": {
            "image_url": "{image_url}",
            "source_lang": "{source_lang}",
            "target_lang": "{target_lang}"
        }
    })
}
