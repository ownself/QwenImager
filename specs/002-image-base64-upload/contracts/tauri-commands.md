# Contract Update: Image Path Handling in Tauri Commands

**Feature Branch**: `002-image-base64-upload`  
**Date**: 2026-02-27

## Overview

No changes to the Tauri command signatures or frontend IPC interface. The modification is internal to the Rust backend — how local file paths are transformed before being sent to the DashScope API.

## Affected Commands

### `edit_image`

**No signature change.** The `image_paths: Vec<String>` parameter continues to accept local file paths from the frontend.

**Internal behavior change:**

| Step | Before | After |
|------|--------|-------|
| Build `ImageEditContent::Image` | `file://{path}` URL | `data:{mime};base64,{data}` data URI |
| HTTP URL detection | Passthrough if starts with `http` | Passthrough if starts with `http` (unchanged) |

### `translate_image`

**No signature change.** The `image_path: String` parameter continues to accept a local file path from the frontend.

**Internal behavior change:**

| Step | Before | After |
|------|--------|-------|
| Build `TranslationInput.image_url` | `file://{path}` URL | `data:{mime};base64,{data}` data URI |
| HTTP URL detection | Passthrough if starts with `http` | Passthrough if starts with `http` (unchanged) |

## New Internal Utility

### `encode_image_to_data_uri(path: &str) -> Result<String, AppError>`

A shared utility function (not a Tauri command) that:
1. Reads the file at `path` into memory
2. Determines MIME type from file extension
3. Base64-encodes the file contents
4. Returns formatted string: `data:{mime_type};base64,{base64_data}`
5. Returns `AppError::Io` if the file cannot be read

This function is called by both `edit_image` and `translate_image` commands, replacing the previous `file://` URL construction.
