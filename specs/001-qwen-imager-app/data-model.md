# Data Model: Qwen AI Image Generation Desktop App

**Feature Branch**: `001-qwen-imager-app`  
**Date**: 2026-02-27  
**Source**: [spec.md](./spec.md) Key Entities section

## Entity Relationship Overview

```text
Configuration (1)
    │
    ├── providers (1:N)
    │       └── models (1:N)
    │
Conversation (1)
    │
    └── Message (1:N)
            │
            ├── Attachment (0:N)  [user messages only]
            │
            └── GenerationResult (0:N)  [assistant messages only]
```

## Entities

### Configuration

用户的 API 配置信息，从 `~/.qwenimage/setting.json` 读取。

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| providers | Map<string, Provider> | Yes | 提供商配置映射，当前仅支持 "qwen" |

### Provider

单个 AI 提供商的配置。

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| api_key | string | Yes | API 认证密钥 |
| models | Map<string, ModelConfig> | Yes | 模型配置映射 |

### ModelConfig

单个模型的配置。

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| url | string (URL) | Yes | 模型的 API 端点 URL |

### Conversation

代表一次完整的用户交互会话。

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| id | string (UUID) | Yes | 唯一标识符，自动生成 |
| title | string | Yes | 对话标题，默认取首条 Prompt 的前 30 个字符 |
| created_at | timestamp | Yes | 创建时间 |
| updated_at | timestamp | Yes | 最后更新时间（用于排序） |

**验证规则**:
- `title` 非空，最长 100 个字符
- `created_at` 不可修改
- `updated_at` 在每次添加消息时自动更新

**状态转换**: 无（对话无状态机，创建后持续存在直到被删除）

### Message

对话中的单条记录。

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| id | string (UUID) | Yes | 唯一标识符，自动生成 |
| conversation_id | string (UUID) | Yes | 所属对话的 ID |
| role | enum: "user" \| "assistant" | Yes | 消息角色 |
| text_content | string | No | 文字内容（用户的 Prompt 或系统提示） |
| mode | enum: "text2img" \| "img2img" \| "translate" | Yes | 操作模式 |
| created_at | timestamp | Yes | 创建时间 |

**验证规则**:
- `role` 仅允许 "user" 或 "assistant"
- `mode` 仅允许 "text2img"、"img2img" 或 "translate"
- 用户消息至少包含 `text_content` 或关联的 `Attachment`
- `conversation_id` 必须引用已存在的 Conversation

### Attachment

用户上传或从剪贴板粘贴的图片文件。仅关联到 `role: "user"` 的消息。

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| id | string (UUID) | Yes | 唯一标识符 |
| message_id | string (UUID) | Yes | 所属消息的 ID |
| file_path | string | Yes | 图片文件在本地的存储路径 |
| display_order | integer | Yes | 在消息中的显示顺序（从 0 开始） |
| file_size | integer | Yes | 文件大小（字节） |
| mime_type | string | Yes | MIME 类型（image/png, image/jpeg 等） |
| source | enum: "upload" \| "clipboard" | Yes | 来源方式 |

**验证规则**:
- `file_size` 不超过 10MB (10,485,760 bytes)
- `mime_type` 仅允许图片格式：image/png, image/jpeg, image/webp, image/gif
- `display_order` >= 0，同一消息内唯一
- `file_path` 必须指向存在的文件

### GenerationResult

API 返回的图片或视频资源。仅关联到 `role: "assistant"` 的消息。

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| id | string (UUID) | Yes | 唯一标识符 |
| message_id | string (UUID) | Yes | 所属消息的 ID |
| resource_url | string (URL) | No | API 返回的远程资源 URL |
| local_path | string | No | 缓存到本地的文件路径 |
| resource_type | enum: "image" \| "video" | Yes | 资源类型 |
| model_used | string | Yes | 使用的模型名称 |
| generation_params | JSON | No | 生成时的参数快照（size, n, prompt_extend 等） |
| created_at | timestamp | Yes | 生成时间 |

**验证规则**:
- `resource_url` 和 `local_path` 至少有一个非空
- `resource_type` 仅允许 "image" 或 "video"
- `model_used` 必须是已知的模型标识符

### TranslationParams（嵌入式，翻译模式特有）

图片翻译的额外参数，存储在 Message 的扩展字段中。

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| source_lang | string | Yes | 源语言代码（如 "zh", "en", "ja"） |
| target_lang | string | Yes | 目标语言代码 |

**验证规则**:
- `source_lang` 和 `target_lang` 不能相同
- 语言代码使用 ISO 639-1 标准

## SQLite Schema

```sql
CREATE TABLE conversations (
    id          TEXT PRIMARY KEY,
    title       TEXT NOT NULL CHECK(length(title) <= 100),
    created_at  INTEGER NOT NULL,
    updated_at  INTEGER NOT NULL
);

CREATE INDEX idx_conversations_updated ON conversations(updated_at DESC);

CREATE TABLE messages (
    id              TEXT PRIMARY KEY,
    conversation_id TEXT NOT NULL REFERENCES conversations(id) ON DELETE CASCADE,
    role            TEXT NOT NULL CHECK(role IN ('user', 'assistant')),
    text_content    TEXT,
    mode            TEXT NOT NULL CHECK(mode IN ('text2img', 'img2img', 'translate')),
    extra_params    TEXT,  -- JSON, e.g. translation source/target lang
    created_at      INTEGER NOT NULL
);

CREATE INDEX idx_messages_conv ON messages(conversation_id, created_at);

CREATE TABLE attachments (
    id            TEXT PRIMARY KEY,
    message_id    TEXT NOT NULL REFERENCES messages(id) ON DELETE CASCADE,
    file_path     TEXT NOT NULL,
    display_order INTEGER NOT NULL DEFAULT 0,
    file_size     INTEGER NOT NULL CHECK(file_size <= 10485760),
    mime_type     TEXT NOT NULL CHECK(mime_type IN ('image/png','image/jpeg','image/webp','image/gif')),
    source        TEXT NOT NULL CHECK(source IN ('upload', 'clipboard'))
);

CREATE INDEX idx_attachments_msg ON attachments(message_id, display_order);

CREATE TABLE generation_results (
    id              TEXT PRIMARY KEY,
    message_id      TEXT NOT NULL REFERENCES messages(id) ON DELETE CASCADE,
    resource_url    TEXT,
    local_path      TEXT,
    resource_type   TEXT NOT NULL CHECK(resource_type IN ('image', 'video')),
    model_used      TEXT NOT NULL,
    generation_params TEXT,  -- JSON blob
    created_at      INTEGER NOT NULL,
    CHECK(resource_url IS NOT NULL OR local_path IS NOT NULL)
);

CREATE INDEX idx_results_msg ON generation_results(message_id);
```

## Storage Locations

| Data | Storage | Location |
|------|---------|----------|
| 对话历史 + 消息 | SQLite | `{app_data_dir}/qwenimager.db` |
| 用户偏好 (UI 状态) | tauri-plugin-store | `{app_data_dir}/preferences.json` |
| API 配置 | 文件读取 (只读) | `~/.qwenimage/setting.json` |
| 上传的图片 | 文件系统 | `{app_data_dir}/attachments/{conversation_id}/` |
| 剪贴板临时图片 | 文件系统 | `{temp_dir}/qwenimager-clipboard/` |
| 缓存的生成结果 | 文件系统 | `{app_data_dir}/cache/{conversation_id}/` |
