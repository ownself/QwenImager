# Implementation Plan: Config-Driven API Service Integration

**Branch**: `003-config-driven-api` | **Date**: 2026-02-27 | **Spec**: [spec.md](spec.md)
**Input**: Feature specification from `/specs/003-config-driven-api/spec.md`

## Summary

将 QwenImager 当前硬编码的三种 DashScope API 调用（text2img、img2img、translate）重构为配置文件驱动的架构。用户通过编辑 `~/.qwenimage/setting.json` 定义 API 服务的请求模板、响应映射和调用模式，无需修改源代码即可接入新的 AI 图像服务。

技术方案：自定义 `serde_json::Value` 树遍历实现模板渲染和响应提取（零新依赖），配合 `mode` + `async_poll` 配置结构支持同步和异步两种调用模式。

## Technical Context

**Language/Version**: Rust (stable, latest) + TypeScript 5.x  
**Primary Dependencies**: Tauri v2, React 19, reqwest, serde_json, rusqlite, shadcn/ui, zustand  
**Storage**: SQLite (rusqlite with bundled feature) — 本功能不修改 schema  
**Testing**: cargo clippy, cargo check, npx tsc --noEmit, npx vite build  
**Target Platform**: Windows / macOS / Linux (desktop)  
**Project Type**: Desktop App (Tauri v2)  
**Performance Goals**: 模板渲染 <1ms，配置加载 <100ms  
**Constraints**: 零新依赖（仅使用已有的 serde_json 和 reqwest）  
**Scale/Scope**: 支持 5+ API 服务定义

## Constitution Check

*GATE: Must pass before Phase 0 research. Re-check after Phase 1 design.*

项目 constitution.md 为空白模板，无项目特定约束。门控通过。

**Post-design re-assessment**: 通过。设计决策符合最小依赖原则（零新 crate），保持向后兼容，不过度抽象。

## Project Structure

### Documentation (this feature)

```text
specs/003-config-driven-api/
├── plan.md              # 本文件
├── research.md          # 技术调研（模板引擎、JSON 路径、异步模式）
├── data-model.md        # 配置数据模型设计
├── quickstart.md        # 实现快速指南
├── contracts/
│   ├── tauri-commands.md  # 命令签名变更
│   └── ui-contracts.md    # 前端组件契约
└── tasks.md             # 任务清单（/speckit.tasks 生成）
```

### Source Code (repository root)

```text
src-tauri/src/
├── models/
│   ├── config.rs          # 修改：扩展 ModelConfig、新增 ServiceType/ApiMode/AsyncPollConfig/ModelInfo
│   └── api.rs             # 保留：DashScope 类型保留作为默认模板的类型支持
├── services/
│   ├── config_loader.rs   # 修改：移除硬编码 "qwen"、新增验证逻辑、返回 ModelInfo
│   ├── qwen_client.rs     # 重构：统一执行方法，根据 ModelConfig.mode 分发
│   ├── template_engine.rs # 新增：render_template + extract_strings
│   ├── image_utils.rs     # 不变
│   └── db.rs              # 不变
├── commands/
│   ├── generation.rs      # 修改：新增 model_name 参数、使用配置驱动调用
│   ├── translation.rs     # 修改：新增 model_name 参数、使用配置驱动调用
│   ├── config.rs          # 修改：适配新 ConfigStatus 返回值
│   └── conversation.rs    # 不变
├── error.rs               # 不变
└── lib.rs                 # 不变（AppState 无结构变更）

src/
├── stores/
│   ├── configStore.ts     # 修改：availableModels 改为 ModelInfo[]、新增 getModelsByType
│   └── conversationStore.ts # 修改：新增 selectedModel 状态、sendPrompt 传 modelName
├── components/
│   ├── input/
│   │   └── ModelSelector.tsx  # 新增：模型选择器下拉组件
│   └── layout/
│       └── MainPanel.tsx      # 修改：嵌入 ModelSelector
└── types/                     # 新增或扩展：ModelInfo 接口
```

**Structure Decision**: 保持现有 Tauri v2 前后端结构不变。新增 `template_engine.rs` 作为独立服务模块，`ModelSelector.tsx` 作为输入组件。变更集中在配置模型层和命令层。
