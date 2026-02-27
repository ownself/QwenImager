# Tasks: Image Base64 Upload for API Requests

**Input**: Design documents from `/specs/002-image-base64-upload/`
**Prerequisites**: plan.md, spec.md, research.md, data-model.md, contracts/

**Organization**: Tasks are grouped by user story to enable independent implementation and testing of each story.

## Format: `[ID] [P?] [Story?] Description`

- **[P]**: Can run in parallel (different files, no dependencies)
- **[Story]**: Which user story this task belongs to (e.g., US1, US2)
- Include exact file paths in descriptions

---

## Phase 1: Setup (Shared Infrastructure)

**Purpose**: 添加新依赖并创建共享的 Base64 编码工具模块

- [x] T001 在 src-tauri/Cargo.toml 中添加 `base64 = "0.22"` 依赖，运行 `cargo check` 确认依赖解析成功
- [x] T002 在 src-tauri/src/services/image_utils.rs 中创建 `encode_image_to_data_uri(path: &str) -> Result<String, AppError>` 函数：读取文件内容为 `Vec<u8>`，根据文件扩展名确定 MIME 类型（jpg/jpeg → image/jpeg, png → image/png, webp → image/webp, gif → image/gif, bmp → image/bmp, tiff/tif → image/tiff, 其他 → image/png），使用 `base64::prelude::BASE64_STANDARD.encode()` 编码，返回 `data:{mime_type};base64,{base64_data}` 格式字符串；文件读取失败时返回 `AppError::Io`
- [x] T003 在 src-tauri/src/services/mod.rs 中添加 `pub mod image_utils;` 注册新模块

**Checkpoint**: 基础设施就绪 — `encode_image_to_data_uri()` 函数可用，`cargo check` 通过

---

## Phase 2: User Story 1 - 图生图 Base64 编码 (Priority: P1) 🎯 MVP

**Goal**: 图生图（img2img）模式下，本地图片通过 Base64 编码传递给 DashScope API，替代不被支持的 `file://` URL

**Independent Test**: 在 img2img 模式下选择本地图片，输入编辑 Prompt 并提交，验证 API 成功返回编辑后的图片（无 URL 错误）

### Implementation for User Story 1

- [x] T004 [US1] 更新 src-tauri/src/commands/generation.rs 中的 `edit_image` 函数：在构建 `ImageEditContent::Image` 时，将本地文件路径的处理从 `format!("file://{}", path.replace('\\', "/"))` 替换为调用 `crate::services::image_utils::encode_image_to_data_uri(path)?`，保持 HTTP/HTTPS URL 的直接传递不变（`if path.starts_with("http")` 分支保持原样）
- [x] T005 [US1] 运行 `cargo check` 和 `cargo clippy` 验证 `edit_image` 修改无编译错误和 lint 警告

**Checkpoint**: 图生图功能修复完成 — 本地图片可通过 Base64 方式被 API 接受

---

## Phase 3: User Story 2 - 图片翻译 Base64 编码 (Priority: P1)

**Goal**: 翻译模式下，本地图片通过 Base64 编码传递给 DashScope 翻译 API，替代不被支持的 `file://` URL

**Independent Test**: 在 translate 模式下选择本地图片，选择源语言和目标语言并提交，验证 API 成功返回翻译后的图片（无 URL 错误）

### Implementation for User Story 2

- [x] T006 [US2] 更新 src-tauri/src/commands/translation.rs 中的 `translate_image` 函数：将本地文件路径的处理从 `format!("file://{}", image_path.replace('\\', "/"))` 替换为调用 `crate::services::image_utils::encode_image_to_data_uri(&image_path)?`，保持 HTTP/HTTPS URL 的直接传递不变（`if image_path.starts_with("http")` 分支保持原样）
- [x] T007 [US2] 运行 `cargo check` 和 `cargo clippy` 验证 `translate_image` 修改无编译错误和 lint 警告

**Checkpoint**: 图片翻译功能修复完成 — 本地图片可通过 Base64 方式被 API 接受

---

## Phase 4: Polish & Cross-Cutting Concerns

**Purpose**: 验证无回归，确保全部功能正常

- [x] T008 运行 `npx tsc --noEmit` 确认前端无类型错误（预期无变化，防止回归）
- [x] T009 运行 `npx vite build` 确认前端生产构建成功（预期无变化，防止回归）
- [x] T010 运行 `cargo clippy` 确认 Rust 代码无 lint 警告（含新增的 image_utils.rs）

---

## Dependencies & Execution Order

### Phase Dependencies

- **Setup (Phase 1)**: 无依赖 — 可立即开始
- **User Story 1 (Phase 2)**: 依赖 Phase 1 完成（需要 `encode_image_to_data_uri` 函数）
- **User Story 2 (Phase 3)**: 依赖 Phase 1 完成，但与 Phase 2 无依赖关系（可并行）
- **Polish (Phase 4)**: 依赖 Phase 2 和 Phase 3 全部完成

### User Story Dependencies

- **User Story 1 (P1)**: Phase 1 完成后可开始 — 不依赖 User Story 2
- **User Story 2 (P1)**: Phase 1 完成后可开始 — 不依赖 User Story 1
- 两个 User Story 修改不同文件（generation.rs vs translation.rs），可完全并行

### Within Each Phase

- Phase 1: T001 → T002 → T003（顺序执行，依赖关系明确）
- Phase 2: T004 → T005（顺序执行）
- Phase 3: T006 → T007（顺序执行）
- Phase 4: T008、T009、T010 全部可并行

### Parallel Opportunities

- Phase 2 和 Phase 3 可并行执行（修改不同文件）
- Phase 4 的三个验证任务可并行执行

---

## Parallel Example: User Story 1 + User Story 2

```bash
# Phase 1 完成后，两个 User Story 可并行执行：
Task: "T004 - 更新 edit_image 使用 Base64 编码 (src-tauri/src/commands/generation.rs)"
Task: "T006 - 更新 translate_image 使用 Base64 编码 (src-tauri/src/commands/translation.rs)"

# Phase 4 验证任务可并行执行：
Task: "T008 - npx tsc --noEmit"
Task: "T009 - npx vite build"
Task: "T010 - cargo clippy"
```

---

## Implementation Strategy

### MVP First (User Story 1 Only)

1. Complete Phase 1: Setup（添加依赖 + 创建工具函数）
2. Complete Phase 2: User Story 1（修复图生图）
3. **STOP and VALIDATE**: 在 img2img 模式下测试本地图片编辑
4. 此时图生图功能已修复，可以使用

### Incremental Delivery

1. Complete Setup → 基础设施就绪
2. Add User Story 1 → 图生图修复 → **MVP!**
3. Add User Story 2 → 图片翻译修复 → 增量发布
4. Polish → 全面验证无回归 → 正式发布

### Recommended Execution Order (Single Developer)

Phase 1 → Phase 2 (US1) → Phase 3 (US2) → Phase 4 (Polish)

---

## Notes

- [P] tasks = different files, no dependencies
- [Story] label maps task to specific user story for traceability
- 此功能不涉及前端修改，所有变更在 Rust 后端
- 前端验证（T008、T009）仅用于确认无回归
- 每个任务完成后应运行 `cargo check` 确认编译通过
