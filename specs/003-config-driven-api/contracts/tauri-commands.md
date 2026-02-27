# Tauri Commands Contract: Config-Driven API

**Feature Branch**: `003-config-driven-api`  
**Date**: 2026-02-27

## 概述

本功能修改现有 Tauri 命令的签名和行为，并扩展配置返回类型。

## 修改的命令

### `load_config` — 扩展返回值

**当前签名**:
```
load_config() -> ConfigStatus { loaded, available_models: Vec<String>, error_message }
```

**新签名**:
```
load_config() -> ConfigStatus { loaded, available_models: Vec<ModelInfo>, error_message }
```

**变更**:
- `available_models` 从 `Vec<String>`（模型名称列表）变为 `Vec<ModelInfo>`（含 provider、service_type 的结构化信息）
- 前端据此按 `service_type` 分组显示可选模型

**`ModelInfo` 结构**:
```typescript
interface ModelInfo {
  name: string;           // 模型配置名称，如 "qwen-image-plus"
  provider: string;       // Provider 名称，如 "qwen"
  serviceType: string;    // "text2img" | "img2img" | "translate"
}
```

---

### `generate_image` — 新增 `model_name` 参数

**当前签名**:
```
generate_image(conversation_id, prompt, params?, on_event) -> ()
```

**新签名**:
```
generate_image(conversation_id, prompt, model_name?, params?, on_event) -> ()
```

**变更**:
- 新增可选 `model_name: Option<String>` 参数
- 传入时：直接查找该模型配置，跳过启发式匹配
- 缺省时：保留现有 `contains("image") && !contains("edit") && !contains("mt")` 逻辑
- 内部行为变更：根据模型的 `mode` 配置决定使用同步还是异步调用模式（不再硬编码为异步）
- `on_event` Channel 行为不变

---

### `edit_image` — 新增 `model_name` 参数

**当前签名**:
```
edit_image(conversation_id, image_paths, prompt, params?) -> Value
```

**新签名**:
```
edit_image(conversation_id, image_paths, prompt, model_name?, params?) -> Value
```

**变更**:
- 新增可选 `model_name: Option<String>` 参数
- 传入时：直接查找该模型配置
- 缺省时：保留现有 `contains("edit")` 逻辑
- 返回值格式不变：`{ messageId, imageUrls }`

---

### `translate_image` — 新增 `model_name` 参数

**当前签名**:
```
translate_image(conversation_id, image_path, source_lang, target_lang, on_event) -> ()
```

**新签名**:
```
translate_image(conversation_id, image_path, source_lang, target_lang, model_name?, on_event) -> ()
```

**变更**:
- 新增可选 `model_name: Option<String>` 参数
- 传入时：直接查找该模型配置
- 缺省时：保留现有 `contains("mt")` 逻辑

---

## 不变的命令

| 命令 | 说明 |
|------|------|
| `save_image` | 文件保存，与 API 无关 |
| `save_clipboard_image` | 剪贴板保存，与 API 无关 |
| `get_conversations` | 数据库查询 |
| `get_messages` | 数据库查询 |
| `create_conversation` | 数据库操作 |
| `delete_conversation` | 数据库操作 |
| `cleanup_temp_files` | 文件清理 |

## 前端调用变更

### TypeScript 接口更新

```typescript
// 原有
interface ConfigStatus {
  loaded: boolean;
  availableModels: string[];
  errorMessage?: string;
}

// 新增
interface ConfigStatus {
  loaded: boolean;
  availableModels: ModelInfo[];
  errorMessage?: string;
}

interface ModelInfo {
  name: string;
  provider: string;
  serviceType: "text2img" | "img2img" | "translate";
}
```

### 命令调用变更

```typescript
// 原有（无 model_name）
await invoke("generate_image", { conversationId, prompt, params, onEvent });

// 新增（可选传入 model_name）
await invoke("generate_image", { conversationId, prompt, modelName, params, onEvent });
```

## 错误处理

- 当 `model_name` 指定但在配置中找不到时，返回 `AppError::Config("Model '{name}' not found in configuration")`
- 当模型配置的 `mode` 与实际需求不匹配时（如 async_poll 缺少 async_poll 配置），返回 `AppError::Config` 并提示具体问题
- 错误信息包含服务名称，便于用户定位问题（FR-009）
