# Feature Specification: Image Base64 Upload for API Requests

**Feature Branch**: `002-image-base64-upload`  
**Created**: 2026-02-27  
**Status**: Draft  
**Input**: User description: "Qwen模型的图生图功能所上传的图片是需要将数据以Base64编码的字符串方式上传的，请为我们的图生图以及图片文字翻译等功能实现该机制"

## User Scenarios & Testing *(mandatory)*

### User Story 1 - Image-to-Image with Local Files (Priority: P1)

A user selects one or more local image files and enters an editing prompt in image-to-image mode. The system reads each local image file, converts it to Base64-encoded data in the format `data:{mime_type};base64,{base64_data}`, and sends the encoded data to the Qwen image editing API. The API processes the request successfully and returns the edited image(s).

Previously, the system attempted to pass local file paths as `file://` URLs, which the API does not accept. With this change, local images are transmitted as inline Base64 data that the API can process directly.

**Why this priority**: This is the core bug fix. Without Base64 encoding, image-to-image editing with local files does not work at all. This is the primary use case that must function correctly.

**Independent Test**: Upload a local image in image-to-image mode, enter an editing prompt, and verify the API returns a successfully edited image without URL errors.

**Acceptance Scenarios**:

1. **Given** a user has selected a local PNG image in image-to-image mode, **When** the user submits an editing prompt, **Then** the system converts the image to Base64, sends it to the API, and displays the generated result image.
2. **Given** a user has selected multiple local images (JPG, PNG, WebP) in image-to-image mode, **When** the user submits an editing prompt, **Then** all images are correctly Base64-encoded with their respective MIME types and the API processes the request successfully.
3. **Given** a user has pasted an image from the clipboard (saved as a temporary local file), **When** the user submits in image-to-image mode, **Then** the clipboard image is Base64-encoded and processed by the API just like any other local image.

---

### User Story 2 - Image Translation with Local Files (Priority: P1)

A user selects a local image containing text in translate mode, chooses source and target languages, and submits. The system converts the local image to Base64-encoded data in the format `data:{mime_type};base64,{base64_data}` and sends it to the Qwen translation API. The API processes the translation and returns the translated image.

**Why this priority**: Same fundamental issue as image-to-image — the translation API also requires either a public URL or Base64 data for image input. Local `file://` URLs are not supported.

**Independent Test**: Upload a local image with visible text in translate mode, select languages, and verify the API returns a translated image without URL errors.

**Acceptance Scenarios**:

1. **Given** a user has selected a local image with Chinese text in translate mode, **When** the user selects zh→en translation and submits, **Then** the system converts the image to Base64, sends it to the translation API, and displays the translated image.
2. **Given** a user has pasted an image from the clipboard in translate mode, **When** the user submits a translation, **Then** the clipboard image is Base64-encoded and processed correctly.

---

### Edge Cases

- What happens when a user selects an image file that is very large (close to the 10MB limit)? The Base64 encoding increases data size by approximately 33%. The system must handle this gracefully and report an appropriate error if the encoded payload exceeds API limits.
- What happens when an image file cannot be read (e.g., file was deleted after selection)? The system must display a clear error message indicating the file is not accessible.
- What happens when an image file has an unsupported or ambiguous extension? The system must determine the correct MIME type based on the file extension and fall back to a safe default (e.g., `image/png`).
- What happens with images that already have HTTP/HTTPS URLs (e.g., from a previous API response)? The system must continue to pass URL-based images as URLs without Base64 encoding, since the API accepts both formats.

## Requirements *(mandatory)*

### Functional Requirements

- **FR-001**: System MUST read local image files and convert them to Base64-encoded strings in the format `data:{mime_type};base64,{base64_data}` before sending to the Qwen API.
- **FR-002**: System MUST correctly determine the MIME type based on the image file extension (JPEG → `image/jpeg`, PNG → `image/png`, WebP → `image/webp`, GIF → `image/gif`, BMP → `image/bmp`, TIFF → `image/tiff`).
- **FR-003**: System MUST apply Base64 encoding for the image-to-image editing API requests when the image source is a local file path.
- **FR-004**: System MUST apply Base64 encoding for the image translation API requests when the image source is a local file path.
- **FR-005**: System MUST continue to pass HTTP/HTTPS URLs as-is (without Base64 encoding) when the image source is already a remote URL.
- **FR-006**: System MUST report a clear error if a local image file cannot be read or does not exist at the time of submission.
- **FR-007**: System MUST NOT change any user-facing behavior — the upload, preview, and submission workflow remains identical from the user's perspective.

### Assumptions

- The DashScope API accepts Base64-encoded image data in the `data:{mime_type};base64,{base64_data}` format as documented in the official Qwen Image Edit guide.
- The 10MB file size limit (already enforced in the upload component) is sufficient to prevent API payload issues, even after the ~33% increase from Base64 encoding.
- The same Base64 encoding mechanism applies to both the image-to-image API (`multimodal-generation/generation`) and the translation API (`image2image/image-synthesis`).

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: Users can successfully complete image-to-image editing with local files — 100% of submissions with valid local images are accepted by the API without URL-related errors.
- **SC-002**: Users can successfully complete image translation with local files — 100% of submissions with valid local images are accepted by the API without URL-related errors.
- **SC-003**: Images pasted from clipboard work identically to images selected via file picker in both image-to-image and translate modes.
- **SC-004**: The image conversion process adds no perceptible delay to the user experience — the encoding completes within the existing submission flow without additional loading indicators.
- **SC-005**: Existing functionality (text-to-image, URL-based images) remains unaffected — no regression in any previously working feature.
