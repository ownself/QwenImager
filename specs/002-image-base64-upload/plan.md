# Implementation Plan: Image Base64 Upload for API Requests

**Branch**: `002-image-base64-upload` | **Date**: 2026-02-27 | **Spec**: [spec.md](./spec.md)
**Input**: Feature specification from `/specs/002-image-base64-upload/spec.md`

## Summary

修复图生图和图片翻译功能中本地图片无法被 API 接受的问题。当前实现使用 `file://` URL 传递本地文件路径，但 DashScope API 不支持此格式。需要将本地图片文件读取为二进制数据，进行 Base64 编码，并按 `data:{mime_type};base64,{base64_data}` 格式拼接后作为图片参数传递给 API。此变更仅影响 Rust 后端的 API 请求构建逻辑，不涉及前端或用户交互变化。

## Technical Context

**Language/Version**: Rust (stable, latest) + TypeScript 5.x  
**Primary Dependencies**: Tauri v2, React 19, Vite, reqwest, rusqlite, shadcn/ui, zustand, @dnd-kit  
**Storage**: SQLite (rusqlite with bundled feature)  
**Testing**: cargo test; cargo clippy  
**Target Platform**: Windows (WebView2), macOS (WebKit), Linux (WebKitGTK)  
**Project Type**: desktop-app (Tauri v2)  
**Performance Goals**: Base64 编码处理时间不可感知（10MB 文件编码 < 100ms）  
**Constraints**: 安装包 < 50MB，单张图片 < 10MB（已有前端校验）  
**Scale/Scope**: 仅修改 2 个 Rust 命令文件（generation.rs、translation.rs），新增 1 个工具函数

## Constitution Check

*GATE: Must pass before Phase 0 research. Re-check after Phase 1 design.*

项目章程为模板状态（未定制），无具体门控条件。按默认最佳实践执行：
- 简洁性：仅修改必要代码，不引入新依赖（Rust 标准库已包含 Base64 所需能力，但需要 `base64` crate）
- 向后兼容：HTTP/HTTPS URL 继续按原路径传递，仅本地文件路径转换为 Base64
- 无用户交互变更：前端代码无需修改

**Gate Status**: PASS

## Project Structure

### Documentation (this feature)

```text
specs/002-image-base64-upload/
├── plan.md              # This file
├── spec.md              # Feature specification
├── research.md          # Phase 0: Base64 encoding approach
├── data-model.md        # Phase 1: N/A (no data model changes)
├── quickstart.md        # Phase 1: Implementation guide
├── contracts/           # Phase 1: Updated contracts
│   └── tauri-commands.md    # Updated image path handling docs
├── checklists/
│   └── requirements.md     # Spec quality checklist
└── tasks.md             # Phase 2 output
```

### Source Code (affected files)

```text
src-tauri/
├── Cargo.toml                          # Add base64 crate dependency
└── src/
    ├── commands/
    │   ├── generation.rs               # Modify edit_image: Base64 encode local images
    │   └── translation.rs              # Modify translate_image: Base64 encode local image
    └── services/
        └── image_utils.rs              # NEW: shared Base64 encoding utility
```

**Structure Decision**: 新增 `image_utils.rs` 服务模块，提供 `encode_image_to_data_uri()` 共享函数，被 `generation.rs` 和 `translation.rs` 共同调用，避免代码重复。
