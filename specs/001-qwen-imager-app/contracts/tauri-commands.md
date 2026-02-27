# Tauri Commands Contract: Rust ↔ Frontend IPC

**Feature Branch**: `001-qwen-imager-app`  
**Date**: 2026-02-27

## Overview

所有 Tauri commands 定义了 Rust 后端与 React 前端之间的 IPC 接口。前端通过 `invoke()` 调用命令，通过 `Channel` 接收异步事件流。

## Commands

### 1. Configuration

#### `load_config`

加载用户的 API 配置文件。

```
Direction: Frontend → Rust
Trigger: App startup
```

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| (none) | - | - | 自动读取 ~/.qwenimage/setting.json |

| Return | Type | Description |
|--------|------|-------------|
| Success | `ConfigStatus` | 配置加载状态和内容 |
| Error | `AppError` | 配置缺失或格式错误 |

**ConfigStatus**:
```typescript
type ConfigStatus = {
  loaded: boolean;
  available_models: string[];   // e.g., ["qwen-image-max", "qwen-image-edit-max", "qwen-mt-image"]
  error_message?: string;       // Only if loaded is false
}
```

---

### 2. Image Generation

#### `generate_image`

文生图：提交 Prompt 生成图片（异步，通过 Channel 推送进度）。

```
Direction: Frontend → Rust (with Channel for progress)
Trigger: User submits text prompt in text-to-image mode
```

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| conversation_id | string | Yes | 对话 ID |
| prompt | string | Yes | 用户输入的文字描述 |
| params | `GenerationParams` | No | 生成参数 |
| on_event | Channel | Yes | 事件回调通道 |

**GenerationParams**:
```typescript
type GenerationParams = {
  size?: string;          // e.g., "1024*1024", default "1024*1024"
  n?: number;             // Number of images, default 1
  negative_prompt?: string;
  prompt_extend?: boolean; // default true
  watermark?: boolean;     // default false
}
```

**Channel Events** (`GenerationEvent`):
```typescript
type GenerationEvent =
  | { event: "submitted"; data: { taskId: string } }
  | { event: "polling"; data: { taskId: string; status: string } }
  | { event: "succeeded"; data: { taskId: string; messageId: string; imageUrls: string[] } }
  | { event: "failed"; data: { taskId: string; error: string } };
```

| Return | Type | Description |
|--------|------|-------------|
| Success | `void` | 完成（结果通过 Channel 推送） |
| Error | `AppError` | 提交失败时立即返回 |

---

#### `edit_image`

图生图：上传参考图片并生成新图片（同步请求）。

```
Direction: Frontend → Rust
Trigger: User submits images + prompt in image-to-image mode
```

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| conversation_id | string | Yes | 对话 ID |
| image_paths | string[] | Yes | 参考图片本地路径列表（按顺序） |
| prompt | string | Yes | 文字描述 |
| params | `GenerationParams` | No | 生成参数 |

| Return | Type | Description |
|--------|------|-------------|
| Success | `EditResult` | 包含生成的图片 URL 和消息 ID |
| Error | `AppError` | API 错误或参数错误 |

**EditResult**:
```typescript
type EditResult = {
  messageId: string;
  imageUrls: string[];
}
```

---

#### `translate_image`

图片文字翻译（异步，通过 Channel 推送进度）。

```
Direction: Frontend → Rust (with Channel for progress)
Trigger: User submits image in translate mode
```

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| conversation_id | string | Yes | 对话 ID |
| image_path | string | Yes | 待翻译图片的本地路径 |
| source_lang | string | Yes | 源语言代码 (e.g., "zh") |
| target_lang | string | Yes | 目标语言代码 (e.g., "en") |
| on_event | Channel | Yes | 事件回调通道 |

**Channel Events**: 同 `GenerationEvent`

| Return | Type | Description |
|--------|------|-------------|
| Success | `void` | 完成（结果通过 Channel 推送） |
| Error | `AppError` | 提交失败时立即返回 |

---

### 3. Conversation Management

#### `get_conversations`

获取对话列表。

```
Direction: Frontend → Rust
Trigger: App startup, sidebar render
```

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| limit | number | No | 返回数量限制，默认 50 |
| offset | number | No | 偏移量，默认 0 |

| Return | Type | Description |
|--------|------|-------------|
| Success | `ConversationSummary[]` | 对话摘要列表 |
| Error | `AppError` | 数据库错误 |

**ConversationSummary**:
```typescript
type ConversationSummary = {
  id: string;
  title: string;
  createdAt: number;    // Unix timestamp
  updatedAt: number;
  messageCount: number;
}
```

---

#### `get_conversation_messages`

获取某个对话的全部消息。

```
Direction: Frontend → Rust
Trigger: User clicks conversation in sidebar
```

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| conversation_id | string | Yes | 对话 ID |

| Return | Type | Description |
|--------|------|-------------|
| Success | `MessageDetail[]` | 消息详情列表（含附件和结果） |
| Error | `AppError` | 对话不存在或数据库错误 |

**MessageDetail**:
```typescript
type MessageDetail = {
  id: string;
  role: "user" | "assistant";
  textContent?: string;
  mode: "text2img" | "img2img" | "translate";
  attachments: AttachmentInfo[];
  results: GenerationResultInfo[];
  extraParams?: Record<string, unknown>;
  createdAt: number;
}

type AttachmentInfo = {
  id: string;
  filePath: string;
  displayOrder: number;
  fileSize: number;
  mimeType: string;
  source: "upload" | "clipboard";
}

type GenerationResultInfo = {
  id: string;
  resourceUrl?: string;
  localPath?: string;
  resourceType: "image" | "video";
  modelUsed: string;
  createdAt: number;
}
```

---

#### `create_conversation`

创建新对话。

```
Direction: Frontend → Rust
Trigger: User starts a new conversation
```

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| title | string | No | 对话标题，默认 "新对话" |

| Return | Type | Description |
|--------|------|-------------|
| Success | `string` | 新对话的 ID |
| Error | `AppError` | 数据库错误 |

---

#### `delete_conversation`

删除对话及其所有消息和附件。

```
Direction: Frontend → Rust
Trigger: User deletes a conversation from sidebar
```

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| conversation_id | string | Yes | 对话 ID |

| Return | Type | Description |
|--------|------|-------------|
| Success | `void` | 删除成功 |
| Error | `AppError` | 对话不存在或数据库错误 |

---

### 4. File Operations

#### `save_image`

将图片保存到用户指定位置或默认下载文件夹。

```
Direction: Frontend → Rust
Trigger: User clicks "download" on a generated image
```

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| source_url | string | No | 远程图片 URL |
| source_path | string | No | 本地图片路径 |
| save_path | string | No | 保存路径，为空则弹出保存对话框 |

| Return | Type | Description |
|--------|------|-------------|
| Success | `string` | 保存后的文件路径 |
| Error | `AppError` | 下载或保存失败 |

---

#### `save_clipboard_image`

将剪贴板中的图片数据保存为临时文件。

```
Direction: Frontend → Rust
Trigger: User presses Ctrl+V in the input area
```

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| image_data | number[] | Yes | 图片二进制数据（从前端读取） |
| mime_type | string | Yes | MIME 类型 |

| Return | Type | Description |
|--------|------|-------------|
| Success | `string` | 临时文件路径 |
| Error | `AppError` | 保存失败 |

---

## Error Contract

所有 commands 的错误统一使用 `AppError` 结构：

```typescript
type AppError = {
  kind: "network" | "api" | "timeout" | "config" | "unauthorized" | "rateLimited" | "io" | "notFound" | "validation";
  message: string;
}
```

前端根据 `kind` 决定展示策略：
- `unauthorized`: 提示用户检查 API 密钥配置
- `network`: 提示网络连接问题
- `rateLimited`: 提示稍后重试
- `timeout`: 提示生成超时，建议重试
- `config`: 提示配置文件问题
- `validation`: 提示参数错误
- 其他: 通用错误提示
