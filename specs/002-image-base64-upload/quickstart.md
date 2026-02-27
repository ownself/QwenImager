# Quickstart: Image Base64 Upload

**Feature Branch**: `002-image-base64-upload`  
**Date**: 2026-02-27

## Prerequisites

- Existing QwenImager project fully built (Phase 1-8 of `001-qwen-imager-app` complete)
- Rust stable toolchain installed
- `~/.qwenimage/setting.json` configured with valid DashScope API key

## New Dependency

Add the `base64` crate to `src-tauri/Cargo.toml`:

```toml
[dependencies]
base64 = "0.22"
```

## Implementation Steps

### 1. Add `base64` dependency

```bash
cd src-tauri
cargo add base64
```

### 2. Create `image_utils.rs` service module

Create `src-tauri/src/services/image_utils.rs` with:
- `encode_image_to_data_uri(path: &str) -> Result<String, AppError>` function
- MIME type detection from file extension
- File reading + Base64 encoding + data URI formatting

Register in `src-tauri/src/services/mod.rs`.

### 3. Update `edit_image` command

In `src-tauri/src/commands/generation.rs`:
- Replace `file://` URL construction with `encode_image_to_data_uri()` call
- Keep HTTP/HTTPS URL passthrough unchanged

### 4. Update `translate_image` command

In `src-tauri/src/commands/translation.rs`:
- Replace `file://` URL construction with `encode_image_to_data_uri()` call
- Keep HTTP/HTTPS URL passthrough unchanged

## Verification

```bash
# Rust compilation check
cd src-tauri && cargo check

# Lint check
cargo clippy

# Frontend (no changes, but verify no regressions)
npx tsc --noEmit
npx vite build
```

## Testing

1. Start dev server: `pnpm tauri dev`
2. Switch to "Image to Image" mode
3. Select a local image file, enter an editing prompt, submit
4. Verify the API returns a result (no URL error)
5. Switch to "Translate" mode
6. Select a local image, choose languages, click "Translate Image"
7. Verify the API returns a translated image
