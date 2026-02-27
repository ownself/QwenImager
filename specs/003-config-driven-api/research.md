# Research: Config-Driven API Service Integration

**Feature Branch**: `003-config-driven-api`  
**Date**: 2026-02-27

## 1. JSON 请求模板渲染方案

### 决策：自定义 `serde_json::Value` 树遍历替换（零依赖）

### 理由

需要将配置文件中的 JSON 请求模板（含 `{prompt}`、`{image_url}` 等占位符）在运行时替换为实际值。评估了三种方案：

| 方案 | 新依赖 | 代码量 | Base64 安全 | 可选字段省略 |
|------|--------|--------|-------------|------------|
| A. 自定义 Value 遍历 | 无 | ~40 行 | 安全 | 支持 |
| B. 模板引擎 (minijinja/tera) | 180-500KB | ~20 行+配置 | **不安全** | 不支持 |
| C. Trait Resolver | 无 | ~70 行 | 安全 | 支持 |

### 考虑的替代方案

- **模板引擎** (minijinja, handlebars, tera)：致命缺陷——需要将 JSON 序列化为字符串再模板渲染再反序列化。当变量包含 `"`、`\` 或换行符（Base64 数据 URI 和用户 Prompt 经常包含）时，渲染后的字符串是**无效 JSON**。必须手动 JSON 转义每个变量，本末倒置。依赖体积 180-500KB+。

- **Trait Resolver**：过早抽象。当前需求明确（`{key}` 替换），单一 HashMap 解析器足矣。如果未来需要多种解析策略，从方案 A 重构为方案 C 只需 15 分钟。

### 实现要点

```rust
fn render_template(template: &Value, vars: &HashMap<&str, String>) -> Option<Value> {
    match template {
        Value::String(s) => {
            // 整个值是单个占位符：如果变量不存在则省略该字段
            if s.starts_with('{') && s.ends_with('}') && s.matches('{').count() == 1 {
                let key = &s[1..s.len()-1];
                return vars.get(key).map(|v| Value::String(v.clone()));
            }
            // 嵌入式占位符：替换所有 {var}
            let mut result = s.clone();
            for (key, val) in vars {
                result = result.replace(&format!("{{{}}}", key), val);
            }
            Some(Value::String(result))
        }
        Value::Object(map) => { /* 递归遍历，None 值的 key 被省略 */ }
        Value::Array(arr) => { /* 递归遍历 */ }
        other => Some(other.clone()),
    }
}
```

---

## 2. JSON 响应值提取方案

### 决策：自定义点分路径解析器 + `[*]` 通配符（零依赖）

### 理由

需要从 API 响应中通过可配置的路径表达式提取图片 URL。评估了三种方案：

| 方案 | 新依赖 | 语法示例 | 通配符支持 |
|------|--------|---------|-----------|
| A. serde_json_path (RFC 9535) | ~6 个传递依赖 (nom, regex 等) | `$.output.results[*].url` | 完整 JSONPath |
| B. 自定义点分路径 | 无 | `output.results[*].url` | `[*]` 数组遍历 |
| C. JSON Pointer (内置) | 无 | `/output/results/0/url` | **不支持** |

### 考虑的替代方案

- **serde_json_path**：功能完整的 RFC 9535 实现，但引入 6 个传递依赖（nom, inventory, regex 等），对桌面应用来说过重。当前需求仅需点分访问 + 数组遍历，不需要过滤器、切片、后代运算符。

- **JSON Pointer (RFC 6901)**：`serde_json` 内置 `.pointer()` 方法，但**无法表达通配符**——必须指定精确数组索引 `/results/0/url`，而我们需要遍历未知长度的数组。直接排除。

### 支持的路径格式

```
output.results[*].url                    → DashScope 异步响应
output.choices[*].message.content[*].image → DashScope 同步响应  
data[*].url                              → OpenAI DALL-E 响应
```

### 实现要点

~50 行代码，两种 Segment 类型：`Key(String)` 和 `Wildcard`。解析 `output.results[*].url` 为 `[Key("output"), Key("results"), Wildcard, Key("url")]`，然后逐段遍历 JSON 树。

---

## 3. 同步/异步 API 调用模式配置

### 决策：`mode` 字段区分 + `async_poll` 嵌套配置对象

### 理由

当前两种调用模式（同步、异步提交+轮询）的所有参数都硬编码在 Rust 代码中。需要让用户通过配置文件指定。

### 配置结构

```json
{
  "mode": "async_poll",
  "async_poll": {
    "submit_headers": { "X-DashScope-Async": "enable" },
    "poll_url": "https://dashscope.aliyuncs.com/api/v1/tasks/{task_id}",
    "poll_interval_secs": 3,
    "timeout_secs": 180
  }
}
```

| 字段 | 默认值 | 说明 |
|------|--------|------|
| `mode` | `"sync"` | 缺省时默认同步，保证向后兼容 |
| `async_poll.submit_headers` | `{}` | 提交请求的额外 Header |
| `async_poll.poll_url` | (必填) | 轮询 URL 模板，含 `{task_id}` |
| `async_poll.poll_interval_secs` | `3` | 轮询间隔秒数 |
| `async_poll.timeout_secs` | `180` | 总超时秒数 |

### 关键设计决策

**不引入通用 JSON 提取路径用于异步响应**。原因：
1. 当前三个 DashScope 端点共享相同的响应格式（`output.task_id`, `output.task_status`, `output.results`），无实际变化需要配置
2. 同步响应有两种格式（`choices` vs `results`），当前代码已用 fallback 逻辑处理，JSONPath 无法简洁表达"先试 A，再试 B"
3. 如未来确实需要，可增加 `response_format` 枚举变体（如 `"dashscope_v1"`, `"openai_images"`）选择硬编码提取策略

**认证 Header 不放在配置中**。`Authorization: Bearer {api_key}` 由代码从 `provider.api_key` 自动注入，避免 API Key 重复。

---

## 4. 向后兼容性策略

### 决策：`#[serde(default)]` + 缺省值推断

### 理由

现有 `setting.json` 只有 `{ "url": "..." }`，新增的 `mode`、`service_type`、`request_template`、`response_image_path` 等字段全部使用 `#[serde(default)]`，缺省时按以下规则推断：

| 字段 | 缺省行为 |
|------|---------|
| `mode` | 默认 `"sync"` |
| `service_type` | 如果缺省，由命令层的模型查找逻辑推断（保留现有 `contains("edit")` / `contains("mt")` 启发式）|
| `request_template` | 如果缺省，使用对应 `service_type` 的内置默认模板 |
| `response_image_path` | 如果缺省，使用对应 `service_type` 的内置默认路径 |
| `async_poll` | 如果 `mode` 为 `"async_poll"` 但缺省此字段，使用 DashScope 默认值 |

这确保旧配置文件无需任何修改即可正常工作。

---

## 5. 服务类型 (service_type) 设计

### 决策：显式 `service_type` 字段替代名称启发式

### 理由

当前通过模型名称子串匹配（`contains("edit")`, `contains("mt")`）来确定模型功能类型，这在用户自定义服务名称时会失效。引入显式 `service_type` 字段：

```json
{
  "qwen-image-plus": {
    "url": "...",
    "service_type": "text2img"
  }
}
```

支持的类型：`text2img`、`img2img`、`translate`

**向后兼容**：当 `service_type` 缺省时，保留现有的名称启发式推断逻辑。

---

## 6. 前端模型选择器设计

### 决策：基于 `service_type` 分组的下拉选择器

### 发现

- 后端 `ConfigStatus.available_models` 已返回模型名称列表给前端，但**前端从未使用**
- 前端无任何模型选择状态或 UI
- 后端命令不接受模型参数，自动选择模型

### 方案

1. 扩展 `ConfigStatus` 返回每个模型的 `service_type`，前端按 `service_type` 分组显示
2. 在 `conversationStore` 添加 `selectedModel` 状态
3. 后端命令新增可选 `model_name` 参数，传入时直接使用该模型，不再做启发式查找
4. 当同一 `service_type` 只有一个模型时，自动选择不显示选择器

---

## 7. 请求模板默认值

### 决策：为三种 DashScope 服务提供内置默认模板

当 `request_template` 缺省时，系统使用以下默认模板：

**text2img 默认模板**:
```json
{
  "model": "{model}",
  "input": { "prompt": "{prompt}" },
  "parameters": {
    "size": "{size}",
    "prompt_extend": true,
    "watermark": false
  }
}
```

**img2img 默认模板**:
```json
{
  "model": "{model}",
  "input": {
    "messages": [{
      "role": "user",
      "content": "{content}"
    }]
  },
  "parameters": {
    "prompt_extend": true,
    "watermark": false
  }
}
```

**translate 默认模板**:
```json
{
  "model": "{model}",
  "input": {
    "image_url": "{image_url}",
    "source_lang": "{source_lang}",
    "target_lang": "{target_lang}"
  }
}
```

> 注意：img2img 的 `{content}` 是一个特殊变量，在运行时会被替换为包含图片和文本的 JSON 数组，而非简单字符串替换。这需要模板引擎支持非字符串值替换。
