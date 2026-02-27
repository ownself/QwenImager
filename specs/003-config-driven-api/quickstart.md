# Quickstart: Config-Driven API Service Integration

**Feature Branch**: `003-config-driven-api`  
**Date**: 2026-02-27

## 前置条件

- QwenImager 项目已完成 feature 001 和 002（所有功能正常运行）
- Rust stable toolchain
- Node.js + pnpm
- 现有 `~/.qwenimage/setting.json` 配置正常

## 无新依赖

本功能不添加新的 Rust crate 或 npm 包。所有实现基于：
- `serde_json::Value`（已有）用于模板渲染和响应提取
- `reqwest`（已有）用于 HTTP 请求
- `serde`（已有）用于配置反序列化

## 实现步骤概览

### 1. 扩展配置模型 (`models/config.rs`)

扩展 `ModelConfig` 结构，新增：
- `service_type: Option<ServiceType>` — 功能类型（text2img/img2img/translate）
- `mode: ApiMode` — 调用模式（sync/async_poll），默认 sync
- `request_template: Option<Value>` — JSON 请求模板
- `response_image_path: Option<String>` — 响应图片提取路径
- `headers: Option<HashMap<String, String>>` — 额外请求 Header
- `async_poll: Option<AsyncPollConfig>` — 异步轮询配置

新增 `ServiceType`、`ApiMode`、`AsyncPollConfig`、`ModelInfo` 类型。

### 2. 创建模板引擎 (`services/template_engine.rs`)

实现两个核心函数：
- `render_template(template: &Value, vars: &HashMap<&str, String>) -> Option<Value>` — 递归遍历 JSON 树替换 `{var}` 占位符
- `extract_strings(json: &Value, path: &str) -> Vec<String>` — 点分路径 + `[*]` 通配符提取值

### 3. 重构 HTTP 客户端 (`services/qwen_client.rs`)

将三个独立方法统一为配置驱动的执行方法：
- 读取 `ModelConfig.mode` 决定同步或异步调用
- 读取 `ModelConfig.headers` 和 `async_poll.submit_headers` 构建请求 Header
- 使用 `render_template` 构建请求体
- 使用 `extract_strings` 提取响应中的图片 URL
- 保留 429 重试逻辑

### 4. 更新命令层 (`commands/generation.rs`, `commands/translation.rs`)

- 新增可选 `model_name` 参数
- 使用 `model_name` 直接查找或保留启发式查找
- 调用统一的客户端执行方法
- 其余逻辑（DB 保存、Channel 事件）不变

### 5. 更新配置验证 (`services/config_loader.rs`)

- 移除硬编码的 `providers.get("qwen")`，改为遍历所有 provider
- 验证 `async_poll` 配置完整性
- 推断缺省的 `service_type`
- 返回 `Vec<ModelInfo>` 而非 `Vec<String>`

### 6. 前端适配

- 更新 `configStore`：`availableModels` 改为 `ModelInfo[]`
- 新增 `ModelSelector` 组件
- `conversationStore` 添加 `selectedModel` 状态
- 调用命令时传入 `modelName` 参数

## 验证

```bash
# Rust 编译
cd src-tauri && cargo check

# Lint
cargo clippy

# 前端类型检查
npx tsc --noEmit

# 前端构建
npx vite build
```

## 测试场景

1. **向后兼容**：使用现有 setting.json（无新字段），三种功能正常工作
2. **显式配置**：在 setting.json 中为现有模型添加 `service_type`、`mode`、`async_poll` 等字段，功能正常
3. **多模型**：配置同一 service_type 下的多个模型，界面出现选择器，可切换使用
4. **错误诊断**：故意写错配置（如 async_poll 缺少 poll_url），应用给出具体错误提示

## 配置文件示例（完整版）

```json
{
  "providers": {
    "qwen": {
      "apiKey": "sk-your-key",
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
        },
        "qwen-image-edit-max": {
          "url": "https://dashscope.aliyuncs.com/api/v1/services/aigc/multimodal-generation/generation",
          "service_type": "img2img",
          "mode": "sync",
          "response_image_path": "output.choices[*].message.content[*].image"
        },
        "qwen-mt-image": {
          "url": "https://dashscope.aliyuncs.com/api/v1/services/aigc/image2image/image-synthesis",
          "service_type": "translate",
          "mode": "async_poll",
          "async_poll": {
            "submit_headers": { "X-DashScope-Async": "enable" },
            "poll_url": "https://dashscope.aliyuncs.com/api/v1/tasks/{task_id}"
          }
        }
      }
    }
  }
}
```
