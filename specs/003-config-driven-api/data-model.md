# Data Model: Config-Driven API Service Integration

**Feature Branch**: `003-config-driven-api`  
**Date**: 2026-02-27

## 概述

本功能不新增 SQLite 数据库表或修改现有 schema。所有变更集中在配置文件模型（`models/config.rs`）的扩展。

## 配置模型变更

### 不变的实体

| 实体 | 文件 | 变更 |
|------|------|------|
| `Configuration` | `models/config.rs` | 不变 — `providers: HashMap<String, Provider>` |
| `Provider` | `models/config.rs` | 不变 — `api_key`, `models` |
| `ConfigStatus` | `models/config.rs` | 扩展（见下文）|
| `ModelConfig` | `models/config.rs` | 大幅扩展（见下文）|

### 扩展后的 `ModelConfig`

```
ModelConfig
├── url: String                          # API 端点 URL（必填，已有）
├── service_type: Option<ServiceType>    # 新增：服务类型（text2img/img2img/translate）
├── mode: ApiMode                        # 新增：调用模式（sync/async_poll），默认 sync
├── request_template: Option<Value>      # 新增：请求体 JSON 模板，含 {占位符}
├── response_image_path: Option<String>  # 新增：响应中图片 URL 的提取路径
├── headers: Option<Map<String,String>>  # 新增：额外请求 Header
└── async_poll: Option<AsyncPollConfig>  # 新增：异步轮询配置
```

### 新增实体：`AsyncPollConfig`

```
AsyncPollConfig
├── submit_headers: Map<String, String>  # 提交请求额外 Header（如 X-DashScope-Async）
├── poll_url: String                     # 轮询 URL 模板（含 {task_id}）
├── poll_interval_secs: u64              # 轮询间隔（默认 3）
└── timeout_secs: u64                    # 总超时（默认 180）
```

### 新增枚举：`ServiceType`

```
ServiceType = text2img | img2img | translate
```

- 显式指定模型的功能类型
- 缺省时由现有名称启发式推断（`contains("edit")` → img2img, `contains("mt")` → translate, 其他 → text2img）

### 新增枚举：`ApiMode`

```
ApiMode = sync | async_poll
```

- `sync`：POST 请求后等待完整响应（默认值）
- `async_poll`：POST 提交任务 → 获取 task_id → 轮询直到完成

### 扩展后的 `ConfigStatus`

```
ConfigStatus
├── loaded: bool                         # 已有
├── available_models: Vec<ModelInfo>     # 变更：从 Vec<String> 改为 Vec<ModelInfo>
└── error_message: Option<String>        # 已有
```

### 新增实体：`ModelInfo`

```
ModelInfo
├── name: String                         # 模型配置名称（如 "qwen-image-plus"）
├── provider: String                     # Provider 名称（如 "qwen"）
└── service_type: ServiceType            # 服务类型
```

前端据此按 `service_type` 分组显示可选模型。

## 向后兼容规则

所有新增字段使用 `#[serde(default)]`，缺省值如下：

| 字段 | 缺省值 | 推断逻辑 |
|------|--------|---------|
| `service_type` | `None` | 由模型名称启发式推断 |
| `mode` | `sync` | 与现有 img2img 行为一致 |
| `request_template` | `None` | 使用对应 service_type 的内置默认模板 |
| `response_image_path` | `None` | 使用对应 service_type 的内置默认路径 |
| `headers` | `None` | 无额外 Header |
| `async_poll` | `None` | mode 为 async_poll 时必须提供 |

## 验证规则

1. 如果 `mode == async_poll`，则 `async_poll` 字段必须存在
2. 如果 `async_poll` 存在，则 `poll_url` 必须包含 `{task_id}` 子串
3. `poll_interval_secs` 必须 >= 1
4. `timeout_secs` 必须 >= `poll_interval_secs`
5. `request_template` 如果存在，必须是有效的 JSON 对象（`Value::Object`）
6. `response_image_path` 如果存在，必须是有效的点分路径格式
7. `url` 必须以 `http://` 或 `https://` 开头（已有验证）

## 配置文件示例

### 最小配置（向后兼容现有格式）

```json
{
  "providers": {
    "qwen": {
      "apiKey": "sk-xxx",
      "models": {
        "qwen-image-plus": {
          "url": "https://dashscope.aliyuncs.com/api/v1/services/aigc/text2image/image-synthesis"
        }
      }
    }
  }
}
```

### 完整配置（显式指定所有字段）

```json
{
  "providers": {
    "qwen": {
      "apiKey": "sk-xxx",
      "models": {
        "qwen-image-plus": {
          "url": "https://dashscope.aliyuncs.com/api/v1/services/aigc/text2image/image-synthesis",
          "service_type": "text2img",
          "mode": "async_poll",
          "request_template": {
            "model": "{model}",
            "input": { "prompt": "{prompt}" },
            "parameters": { "size": "{size}", "prompt_extend": true, "watermark": false }
          },
          "response_image_path": "output.results[*].url",
          "async_poll": {
            "submit_headers": { "X-DashScope-Async": "enable" },
            "poll_url": "https://dashscope.aliyuncs.com/api/v1/tasks/{task_id}",
            "poll_interval_secs": 3,
            "timeout_secs": 180
          }
        }
      }
    }
  }
}
```

## 数据库影响

**无影响**。SQLite schema（conversations, messages, attachments, generation_results 表）不需要任何修改。生成结果仍以 image URL 存储在 `generation_results.result_url` 中，与 API 调用方式无关。
