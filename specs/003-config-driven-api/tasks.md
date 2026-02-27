# Tasks: Config-Driven API Service Integration

**Input**: Design documents from `/specs/003-config-driven-api/`
**Prerequisites**: plan.md, spec.md, research.md, data-model.md, contracts/

**Organization**: Tasks are grouped by user story to enable independent implementation and testing of each story.

## Format: `[ID] [P?] [Story?] Description`

- **[P]**: Can run in parallel (different files, no dependencies)
- **[Story]**: Which user story this task belongs to (e.g., US1, US2)
- Include exact file paths in descriptions

---

## Phase 1: Setup (Shared Infrastructure)

**Purpose**: 创建新的服务模块和扩展配置数据模型，为所有 User Story 提供基础设施

- [x] T001 在 src-tauri/src/models/config.rs 中新增 `ServiceType` 枚举（`text2img`, `img2img`, `translate`），使用 `#[serde(rename_all = "snake_case")]`，并实现 `Default` 返回 `text2img`
- [x] T002 在 src-tauri/src/models/config.rs 中新增 `ApiMode` 枚举（`Sync`, `AsyncPoll`），使用 `#[serde(rename_all = "snake_case")]`，并实现 `Default` 返回 `Sync`
- [x] T003 在 src-tauri/src/models/config.rs 中新增 `AsyncPollConfig` 结构体，包含 `submit_headers: HashMap<String, String>`（默认空）、`poll_url: String`、`poll_interval_secs: u64`（默认 3）、`timeout_secs: u64`（默认 180）
- [x] T004 在 src-tauri/src/models/config.rs 中扩展现有 `ModelConfig` 结构体，新增字段：`service_type: Option<ServiceType>`（serde default）、`mode: ApiMode`（serde default）、`request_template: Option<serde_json::Value>`（serde default）、`response_image_path: Option<String>`（serde default）、`headers: Option<HashMap<String, String>>`（serde default）、`async_poll: Option<AsyncPollConfig>`（serde default）；所有新字段使用 `#[serde(default)]` 确保向后兼容
- [x] T005 在 src-tauri/src/models/config.rs 中新增 `ModelInfo` 结构体（`name: String`, `provider: String`, `service_type: ServiceType`），使用 `#[derive(Serialize)]`，`service_type` 使用 `#[serde(rename = "serviceType")]`；将 `ConfigStatus.available_models` 类型从 `Vec<String>` 改为 `Vec<ModelInfo>`
- [x] T006 运行 `cargo check` 确认配置模型扩展编译通过

**Checkpoint**: 配置数据模型就绪 — 所有新类型定义完成，向后兼容的反序列化可用

---

## Phase 2: Foundational (Blocking Prerequisites)

**Purpose**: 创建模板引擎和更新配置加载器，这些是所有 User Story 的共享基础

**⚠️ CRITICAL**: 所有 User Story 都依赖此阶段完成

- [x] T007 在 src-tauri/src/services/template_engine.rs 中创建新模块，实现 `render_template(template: &serde_json::Value, vars: &HashMap<&str, serde_json::Value>) -> Option<serde_json::Value>` 函数：递归遍历 JSON 树，对 `Value::String` 检查是否为单个占位符（`{var}`），如果变量存在则用对应 Value 替换（支持非字符串替换），不存在则返回 None（省略该字段）；对嵌入式占位符做字符串替换；对 `Value::Object` 递归遍历并省略 None 值的 key；对 `Value::Array` 递归遍历；其他类型原样返回
- [x] T008 在 src-tauri/src/services/template_engine.rs 中实现 `extract_strings(json: &serde_json::Value, path: &str) -> Vec<String>` 函数：解析点分路径为 `Segment::Key(String)` 和 `Segment::Wildcard` 段，逐段遍历 JSON 树，Key 段做对象属性访问，Wildcard 段展开数组所有元素，最终收集所有叶节点的字符串值；支持路径格式如 `output.results[*].url`、`output.choices[*].message.content[*].image`、`data[*].url`
- [x] T009 在 src-tauri/src/services/template_engine.rs 中实现 `infer_service_type(model_name: &str) -> ServiceType` 函数：如果名称包含 `"edit"` 返回 `img2img`，如果包含 `"mt"` 返回 `translate`，否则返回 `text2img`（保留现有启发式逻辑作为默认推断）
- [x] T010 在 src-tauri/src/services/template_engine.rs 中实现三个默认模板函数：`default_text2img_template() -> Value`、`default_img2img_template() -> Value`、`default_translate_template() -> Value`，分别返回 research.md 第 7 节定义的 DashScope 默认请求模板 JSON
- [x] T011 在 src-tauri/src/services/mod.rs 中添加 `pub mod template_engine;` 注册新模块
- [x] T012 更新 src-tauri/src/services/config_loader.rs 中的 `load_config_inner()` 函数：移除硬编码的 `config.providers.get("qwen")` 逻辑，改为遍历所有 providers；为每个 provider 下的每个 model 构建 `ModelInfo`（如果 `service_type` 为 None 则调用 `infer_service_type` 推断）；返回 `Vec<ModelInfo>` 而非 `Vec<String>`
- [x] T013 更新 src-tauri/src/services/config_loader.rs 的验证逻辑：新增验证规则——如果 `mode == AsyncPoll` 则 `async_poll` 字段必须存在且 `poll_url` 必须包含 `{task_id}`、`poll_interval_secs >= 1`、`timeout_secs >= poll_interval_secs`；如果 `request_template` 存在则必须是 JSON Object；对每个 provider 的每个 model 的 URL 仍验证 http/https 前缀
- [x] T014 新增 src-tauri/src/services/config_loader.rs 中的辅助函数 `find_model_config(config: &Configuration, model_name: &str) -> Result<(&str, &str, &ModelConfig), AppError>` 用于根据名称在所有 providers 中查找模型，返回 `(provider_name, api_key, &ModelConfig)` 三元组；以及 `find_model_by_service_type(config: &Configuration, service_type: ServiceType) -> Result<(String, String, &ModelConfig), AppError>` 用于按 service_type 查找第一个匹配模型（保留向后兼容的启发式查找作为 fallback）
- [x] T015 运行 `cargo check` 确认基础设施编译通过

**Checkpoint**: 基础设施就绪 — 模板引擎、响应提取器、配置加载器全部可用，User Story 实现可以开始

---

## Phase 3: User Story 1 + User Story 2 - 配置驱动 API 调用 + 向后兼容 (Priority: P1) 🎯 MVP

**Goal**: 重构 HTTP 客户端和命令层，使 API 调用完全由配置驱动；同时确保现有 DashScope 三种模式在旧配置文件下 100% 正常工作

**Independent Test**: 
- US1: 在 setting.json 中新增一个带有完整配置（service_type, mode, request_template, response_image_path, async_poll）的模型定义，应用能识别并使用
- US2: 使用现有 setting.json（仅含 url 字段），三种功能全部正常工作

> 注意：US1 和 US2 紧密耦合（同一文件的同一函数必须同时支持新配置和旧配置），因此合并为一个 Phase 实现

### Implementation

- [x] T016 [US1][US2] 重构 src-tauri/src/services/qwen_client.rs：将 `QwenClient` 重命名为 `ApiClient`（或保留原名但更新文档注释）；新增统一方法 `execute_sync(&self, url: &str, api_key: &str, body: &serde_json::Value, extra_headers: &HashMap<String, String>) -> Result<serde_json::Value, AppError>` 发送 POST 请求并返回 `serde_json::Value`（而非特定类型），支持自定义 Headers，保留 429 重试逻辑
- [x] T017 [US1][US2] 在 src-tauri/src/services/qwen_client.rs 中新增 `execute_async_poll(&self, url: &str, api_key: &str, body: &serde_json::Value, poll_config: &AsyncPollConfig, extra_headers: &HashMap<String, String>, on_poll: impl Fn(&str, &str)) -> Result<serde_json::Value, AppError>` 方法：提交 POST 请求时合并 `poll_config.submit_headers` 和 `extra_headers`，从响应中提取 `output.task_id`，使用 `poll_config.poll_url` 模板（替换 `{task_id}`）轮询，根据 `poll_config.poll_interval_secs` 和 `poll_config.timeout_secs` 控制轮询间隔和超时，返回最终成功的完整响应 JSON
- [x] T018 [US1][US2] 在 src-tauri/src/services/qwen_client.rs 中保留原有的 `submit_async_task`、`poll_task`、`send_sync_request` 方法但标记为 `#[allow(dead_code)]` 或移除（如果所有调用方已迁移到新方法）；保留 `map_http_error` 辅助函数
- [x] T019 [US1][US2] 重构 src-tauri/src/commands/generation.rs 的 `generate_image` 函数：新增 `model_name: Option<String>` 参数；如果 `model_name` 有值则调用 `find_model_config` 查找，否则调用 `find_model_by_service_type(Text2Img)` 保留启发式查找；根据 `model_config.mode` 决定调用 `execute_sync` 或 `execute_async_poll`；使用 `render_template` 构建请求体（如果 `request_template` 存在则用它，否则用 `default_text2img_template()`）；使用 `extract_strings` 和 `response_image_path`（缺省使用 `"output.results[*].url"`）提取结果图片 URL；保留 DB 保存、Channel 事件发送逻辑不变
- [x] T020 [US1][US2] 重构 src-tauri/src/commands/generation.rs 的 `edit_image` 函数：新增 `model_name: Option<String>` 参数；如果 `model_name` 有值则调用 `find_model_config`，否则调用 `find_model_by_service_type(Img2Img)`；根据 `model_config.mode` 选择同步或异步调用；使用 `render_template` 构建请求体（缺省使用 `default_img2img_template()`，`{content}` 变量替换为包含图片和文本的 JSON 数组 Value）；使用 `extract_strings` 和 `response_image_path`（缺省使用 `"output.choices[*].message.content[*].image"` 并 fallback 到 `"output.results[*].url"`）提取结果；保留 DB 保存和返回格式不变
- [x] T021 [US1][US2] 重构 src-tauri/src/commands/translation.rs 的 `translate_image` 函数：新增 `model_name: Option<String>` 参数；如果 `model_name` 有值则调用 `find_model_config`，否则调用 `find_model_by_service_type(Translate)`；根据 `model_config.mode` 选择同步或异步调用；使用 `render_template` 构建请求体（缺省使用 `default_translate_template()`）；使用 `extract_strings` 和 `response_image_path`（缺省 `"output.results[*].url"`）提取结果；保留 DB 保存、Channel 事件逻辑不变
- [x] T022 [US1][US2] 更新 src-tauri/src/commands/config.rs 的 `load_config` 命令：适配 `config_loader::load_config_inner()` 新返回类型（`Vec<ModelInfo>` 而非 `Vec<String>`），确保 `ConfigStatus` 正确序列化带有 `serviceType` 字段的 `ModelInfo` 列表
- [x] T023 [US1][US2] 更新 src-tauri/src/lib.rs：如果 `QwenClient` 被重命名为 `ApiClient`，更新 `AppState` 结构体中的字段名和导入路径；确保所有 `.invoke_handler` 注册的命令签名与新参数匹配
- [x] T024 运行 `cargo check` 和 `cargo clippy` 确认 Rust 后端全部编译通过且无 lint 警告

**Checkpoint**: 后端配置驱动架构完成 — 使用旧配置文件的三种功能正常工作（US2 验证），新配置格式也被正确识别（US1 验证）

---

## Phase 4: User Story 1 续 + User Story 3 - 前端模型选择器 + 多调用模式 (Priority: P1/P2)

**Goal**: 前端适配新的 ConfigStatus 结构，添加模型选择器 UI，支持用户在界面上选择不同 API 服务

**Independent Test**: 
- US1 续: 配置多个同 service_type 的模型，界面出现下拉选择器，选择后使用对应模型
- US3: 分别配置同步和异步两种模式的服务，两者都能正常工作

### Implementation

- [x] T025 [P] [US1] 更新 src/stores/configStore.ts：将 `availableModels` 类型从 `string[]` 改为 `ModelInfo[]`（新增 `ModelInfo` 接口：`name: string, provider: string, serviceType: "text2img" | "img2img" | "translate"`）；新增 `getModelsByType(type: string): ModelInfo[]` 方法用于按 serviceType 筛选模型
- [x] T026 [P] [US1] 更新 src/stores/conversationStore.ts：新增 `selectedModel: string | null` 状态和 `setSelectedModel(name: string | null): void` action；在 `setMode` action 中自动将 `selectedModel` 重置为 `null`；在 `sendPrompt`/`sendEditPrompt`/`sendTranslatePrompt` 中将 `selectedModel` 作为 `modelName` 参数传递给对应 Tauri 命令的 `invoke` 调用
- [x] T027 [US1] 在 src/components/input/ModelSelector.tsx 中创建新组件：接收 `serviceType` prop；从 `configStore.getModelsByType(serviceType)` 获取可用模型列表；如果只有一个模型则不渲染任何内容（自动选择）；如果有多个模型，渲染 shadcn/ui `<Select>` 下拉框，选项格式为 `"{provider} / {name}"`；选中时调用 `conversationStore.setSelectedModel(name)`
- [x] T028 [US1] 更新 src/components/layout/MainPanel.tsx：在 Prompt 输入区域上方（ModeSelector 下方）嵌入 `<ModelSelector serviceType={mode} />`，当前 mode 映射为 `text2img`→`"text2img"`, `img2img`→`"img2img"`, `translate`→`"translate"`
- [x] T029 运行 `npx tsc --noEmit` 确认 TypeScript 无类型错误

**Checkpoint**: 前端适配完成 — 模型选择器工作正常，同步/异步模式均由配置控制

---

## Phase 5: Polish & Cross-Cutting Concerns

**Purpose**: 验证无回归，确保全部功能正常

- [x] T030 [P] 运行 `npx tsc --noEmit` 确认前端无类型错误（最终验证）
- [x] T031 [P] 运行 `npx vite build` 确认前端生产构建成功
- [x] T032 [P] 运行 `cargo clippy` 确认 Rust 代码无 lint 警告（含新增的 template_engine.rs）
- [x] T033 清理 src-tauri/src/models/api.rs 中不再需要的类型（如果所有硬编码的 `TextToImageRequest`、`ImageEditRequest`、`TranslationRequest` 等类型已被 `serde_json::Value` 模板替代，则可移除或保留为参考）；如果 `QwenClient` 的旧方法已全部迁移，移除 `#[allow(dead_code)]` 标记和废弃代码
- [x] T034 更新 src-tauri/src/services/config_loader.rs 中的错误信息：将所有 `"DashScope"` 或 `"qwen"` 相关的硬编码错误消息改为通用表述（如 `"Please set your API key in ~/.qwenimage/setting.json"` 而非 `"Please set your DashScope API key"`），确保错误信息包含具体的 provider 和 model 名称（FR-009）

---

## Dependencies & Execution Order

### Phase Dependencies

- **Setup (Phase 1)**: 无依赖 — 可立即开始
- **Foundational (Phase 2)**: 依赖 Phase 1 完成（需要新的配置类型）— **阻塞所有 User Story**
- **User Story 1+2 (Phase 3)**: 依赖 Phase 2 完成（需要模板引擎和配置加载器）
- **User Story 1 续 + 3 (Phase 4)**: 依赖 Phase 3 完成（需要后端命令新签名）
- **Polish (Phase 5)**: 依赖 Phase 3 和 Phase 4 完成

### User Story Dependencies

- **User Story 1 (P1)**: 核心功能，贯穿 Phase 3 和 Phase 4
- **User Story 2 (P1)**: 与 US1 在 Phase 3 中合并实现（同一函数的两种分支）
- **User Story 3 (P2)**: 后端部分在 Phase 3 中已完成（`mode` 配置驱动同步/异步选择），Phase 4 为前端适配

### Within Each Phase

- Phase 1: T001 → T002 → T003 → T004 → T005 → T006（顺序执行，类型依赖）
- Phase 2: T007 → T008 → T009 → T010 → T011（顺序）→ T012 → T013 → T014 → T015（顺序）
- Phase 3: T016 → T017 → T018（客户端重构，顺序）→ T019/T020/T021 可并行（不同命令文件）→ T022 → T023 → T024
- Phase 4: T025/T026 可并行（不同 store 文件）→ T027 → T028 → T029
- Phase 5: T030/T031/T032 全部可并行

### Parallel Opportunities

- Phase 3: T019（generation.rs）、T020（generation.rs 的 edit_image，同一文件但不同函数——建议顺序）、T021（translation.rs）可部分并行
- Phase 4: T025（configStore.ts）和 T026（conversationStore.ts）可完全并行
- Phase 5: 三个验证任务可完全并行

---

## Parallel Example: Phase 3 命令层重构

```bash
# 客户端重构完成后，三个命令可并行重构（不同文件或不同函数）：
Task: "T019 - 重构 generate_image (src-tauri/src/commands/generation.rs)"
Task: "T021 - 重构 translate_image (src-tauri/src/commands/translation.rs)"

# Phase 4 Store 更新可并行：
Task: "T025 - 更新 configStore (src/stores/configStore.ts)"
Task: "T026 - 更新 conversationStore (src/stores/conversationStore.ts)"

# Phase 5 验证任务可并行：
Task: "T030 - npx tsc --noEmit"
Task: "T031 - npx vite build"
Task: "T032 - cargo clippy"
```

---

## Implementation Strategy

### MVP First (Phase 1 + 2 + 3)

1. Complete Phase 1: Setup（扩展配置模型）
2. Complete Phase 2: Foundational（模板引擎 + 配置加载器）
3. Complete Phase 3: US1+US2 后端（配置驱动 API 调用 + 向后兼容）
4. **STOP and VALIDATE**: 使用旧配置文件验证三种功能正常，使用新配置格式验证新服务可识别
5. 此时后端 MVP 完成

### Incremental Delivery

1. Phase 1 + 2 → 基础设施就绪
2. Phase 3 → 后端完成 → **MVP!**（旧配置正常 + 新配置可用）
3. Phase 4 → 前端适配 → 完整体验（模型选择器 + 多模式支持）
4. Phase 5 → 验证无回归 → 正式发布

### Recommended Execution Order (Single Developer)

Phase 1 → Phase 2 → Phase 3 → Phase 4 → Phase 5

---

## Notes

- [P] tasks = different files, no dependencies
- [Story] label maps task to specific user story for traceability
- US1 和 US2 在 Phase 3 合并实现，因为它们修改同一函数的同一逻辑（新配置 + 旧配置 fallback）
- US3（多调用模式）的后端支持在 Phase 3 中自然实现（`mode` 字段驱动同步/异步分发）
- 本功能不添加任何新的 Rust crate 或 npm 包
- 每个任务完成后应运行 `cargo check` 确认编译通过
- img2img 的 `{content}` 变量是特殊的 JSON Value 替换（非字符串），render_template 的 vars 参数类型使用 `HashMap<&str, serde_json::Value>` 而非 `HashMap<&str, String>`
