# Implementation Plan: Qwen AI Image Generation Desktop App

**Branch**: `001-qwen-imager-app` | **Date**: 2026-02-27 | **Spec**: [spec.md](./spec.md)
**Input**: Feature specification from `/specs/001-qwen-imager-app/spec.md`

## Summary

构建一款基于 Tauri v2 的跨平台桌面应用，通过与 Qwen/DashScope API 交互实现文生图、图生图和图片文字翻译功能。应用采用 Rust 后端处理 API 通信和数据持久化，React + TypeScript 前端提供聊天式交互界面，包含可折叠的历史对话侧边栏、多图上传排序、剪贴板粘贴等交互能力。

## Technical Context

**Language/Version**: Rust (stable, latest) + TypeScript 5.x  
**Primary Dependencies**: Tauri v2, React 19, Vite, reqwest, rusqlite, shadcn/ui, zustand, @dnd-kit  
**Storage**: SQLite (rusqlite with bundled feature) for conversation history; tauri-plugin-store for user preferences  
**Testing**: cargo test (Rust), Vitest (frontend)  
**Target Platform**: Windows (WebView2), macOS (WebKit), Linux (WebKitGTK)  
**Project Type**: desktop-app (Tauri v2)  
**Performance Goals**: 冷启动 < 5s, 操作响应 < 60s (不含 API 处理时间), 历史恢复 < 3s  
**Constraints**: 安装包 < 50MB, 对话历史 100+ 条保持流畅  
**Scale/Scope**: 单用户桌面应用, 3 种 API 交互模式, 5 个主要用户故事

## Constitution Check

*GATE: Must pass before Phase 0 research. Re-check after Phase 1 design.*

项目章程尚未填写（模板状态），无具体门控条件需要验证。按照默认最佳实践执行：
- 简洁性原则：YAGNI，从最小可行方案开始
- 测试优先：关键路径覆盖测试
- 可观察性：结构化错误处理和用户反馈

**Gate Status**: PASS（无违规项）

## Project Structure

### Documentation (this feature)

```text
specs/001-qwen-imager-app/
├── plan.md              # This file
├── spec.md              # Feature specification
├── research.md          # Phase 0: Technology research decisions
├── data-model.md        # Phase 1: Entity model and relationships
├── quickstart.md        # Phase 1: Developer setup guide
├── contracts/           # Phase 1: Interface contracts
│   ├── tauri-commands.md    # Rust ↔ Frontend IPC contracts
│   └── ui-contracts.md      # UI component contracts
├── checklists/
│   └── requirements.md     # Spec quality checklist
└── tasks.md             # Phase 2 output (/speckit.tasks)
```

### Source Code (repository root)

```text
src-tauri/
├── Cargo.toml
├── tauri.conf.json
├── src/
│   ├── main.rs              # Tauri app entry point
│   ├── lib.rs               # Module declarations
│   ├── commands/             # Tauri command handlers
│   │   ├── mod.rs
│   │   ├── generation.rs    # Text-to-image, image-to-image commands
│   │   ├── translation.rs   # Image translation commands
│   │   ├── conversation.rs  # Conversation CRUD commands
│   │   └── config.rs        # Configuration loading commands
│   ├── models/               # Data structures
│   │   ├── mod.rs
│   │   ├── conversation.rs
│   │   ├── message.rs
│   │   ├── config.rs
│   │   └── api.rs           # API request/response types
│   ├── services/             # Business logic
│   │   ├── mod.rs
│   │   ├── qwen_client.rs   # DashScope API client (reqwest)
│   │   ├── db.rs            # SQLite database operations
│   │   └── config_loader.rs # Configuration file reader
│   └── error.rs             # Centralized error types
├── migrations/              # SQLite schema migrations
│   └── 001_initial.sql
└── icons/                   # App icons

src/                          # Frontend (React + TypeScript)
├── main.tsx                  # React entry point
├── App.tsx                   # Root component with layout
├── components/
│   ├── layout/
│   │   ├── Sidebar.tsx       # Collapsible history sidebar
│   │   └── MainPanel.tsx     # Right-side interaction panel
│   ├── chat/
│   │   ├── ChatArea.tsx      # Conversation display area
│   │   ├── MessageBubble.tsx # Single message (text/image/video)
│   │   └── ImageResult.tsx   # Generated image with copy/download actions
│   ├── input/
│   │   ├── PromptInput.tsx   # Text input + submit button
│   │   ├── ImageUpload.tsx   # Multi-image upload with ordering
│   │   └── ModeSelector.tsx  # Text-to-image / Image-to-image / Translate toggle
│   └── common/
│       ├── LoadingState.tsx   # Loading/processing indicator
│       └── ErrorDisplay.tsx   # Error message display
├── stores/
│   ├── conversationStore.ts  # Zustand store for conversations
│   ├── uiStore.ts            # Zustand store for UI state
│   └── configStore.ts        # Zustand store for config state
├── hooks/
│   ├── useGeneration.ts      # Hook for image generation flow
│   ├── useClipboard.ts       # Hook for clipboard paste
│   └── useConversation.ts    # Hook for conversation management
├── lib/
│   ├── tauri.ts              # Tauri invoke/channel wrappers
│   └── utils.ts              # Utility functions
└── styles/
    └── globals.css            # Tailwind + global styles

index.html                    # Vite entry HTML
vite.config.ts                # Vite configuration
tailwind.config.ts            # Tailwind configuration
tsconfig.json                 # TypeScript configuration
package.json                  # Frontend dependencies
```

**Structure Decision**: 采用 Tauri v2 标准项目结构。`src-tauri/` 存放 Rust 后端代码（API 通信、数据库、配置管理），`src/` 存放 React 前端代码（UI 组件、状态管理、用户交互）。Rust 后端按职责分为 commands（IPC 接口）、models（数据结构）、services（业务逻辑）三层。前端按组件功能分为 layout、chat、input、common 四个区域。

## Complexity Tracking

无章程违规需要记录。
