# Feature Specification: Config-Driven API Service Integration

**Feature Branch**: `003-config-driven-api`  
**Created**: 2026-02-27  
**Status**: Draft  
**Input**: User description: "是否可以将目前已经支持的三个模型修改为以文件方式支持新的API服务，以应对未来更多模型的扩展"

## User Scenarios & Testing *(mandatory)*

### User Story 1 - 通过配置文件定义 API 服务 (Priority: P1)

作为 QwenImager 的用户，我希望能通过编辑配置文件来定义 API 服务的请求格式、认证方式和响应解析规则，这样当我想接入一个新的 AI 图像服务时，只需按照配置格式填写该服务的 API 信息（类似官方 cURL 示例中的字段），而无需修改应用程序源代码。

**Why this priority**: 这是整个功能的核心价值——让用户摆脱"每加一个 API 就改代码"的束缚。如果只实现一个 User Story，这个就足以交付 MVP。

**Independent Test**: 用户在配置文件中新增一个 API 服务定义（如使用 DashScope 的另一个模型变体），启动应用后能在界面上看到并使用该服务，无需重新编译。

**Acceptance Scenarios**:

1. **Given** 用户已安装 QwenImager 应用，**When** 用户在配置文件中按照文档格式定义一个新的文生图 API 服务（包含 URL、认证信息、请求模板、响应映射），**Then** 应用启动后能识别该服务并允许用户在文生图模式下选择和使用它
2. **Given** 配置文件中定义了多个 API 服务，**When** 用户启动应用，**Then** 所有已定义的服务都出现在可选列表中，用户可以自由切换
3. **Given** 用户在配置文件中填写了错误的 API 定义（如缺少必填字段），**When** 应用启动或用户尝试使用该服务时，**Then** 系统显示清晰的错误提示，指出具体的配置问题

---

### User Story 2 - 现有 DashScope 三种模式迁移到配置驱动 (Priority: P1)

作为 QwenImager 的现有用户，我希望应用升级到配置驱动架构后，我目前使用的文生图、图生图、图片翻译三种功能仍然完全正常工作，无需任何额外操作。

**Why this priority**: 与 US1 同为 P1，因为不能在引入新架构时破坏现有功能。现有用户的 setting.json 必须保持向后兼容。

**Independent Test**: 使用现有的 setting.json 配置文件（无需修改），应用的三种模式（文生图、图生图、翻译）均正常工作。

**Acceptance Scenarios**:

1. **Given** 用户使用现有格式的 setting.json，**When** 用户升级到新版本并启动应用，**Then** 文生图功能正常工作
2. **Given** 用户使用现有格式的 setting.json，**When** 用户使用图生图功能选择本地图片并输入编辑 Prompt，**Then** 图生图功能正常工作（Base64 编码仍然生效）
3. **Given** 用户使用现有格式的 setting.json，**When** 用户使用翻译功能，**Then** 翻译功能正常工作
4. **Given** 用户配置文件中同时包含旧格式和新格式的 API 定义，**When** 应用启动，**Then** 系统同时识别两种格式定义的服务

---

### User Story 3 - 支持不同 API 调用模式 (Priority: P2)

作为高级用户，我希望配置系统能支持不同的 API 调用模式（同步请求、异步提交+轮询），这样我可以对接不同风格的 AI 图像服务。

**Why this priority**: 这是扩展性的关键。DashScope 使用异步提交+轮询模式，而很多其他服务（如 OpenAI DALL-E）使用同步请求模式。不支持这一点将限制可对接的服务范围。

**Independent Test**: 用户分别配置一个同步 API 服务和一个异步 API 服务，两者都能正确工作。

**Acceptance Scenarios**:

1. **Given** 用户在配置文件中定义了一个同步调用模式的 API 服务，**When** 用户使用该服务生成图片，**Then** 系统发送请求后等待完整响应返回，直接展示结果
2. **Given** 用户在配置文件中定义了一个异步调用模式的 API 服务（提交任务+轮询），**When** 用户使用该服务，**Then** 系统提交任务后自动轮询，并显示进度状态直到完成

---

### Edge Cases

- 配置文件中 API 定义的 URL 不可达时，系统应在请求超时后给出明确的网络错误提示
- 当 API 返回的响应结构与配置中定义的响应映射不匹配时，系统应提示"响应格式不匹配"而非崩溃
- 当用户配置了使用异步模式的服务但未提供轮询 URL 时，系统应在配置验证阶段就给出错误
- 当多个 API 服务的名称重复时，系统应提示冲突
- 配置文件格式错误（如 JSON 语法错误）时，系统应保持现有的错误处理行为

## Requirements *(mandatory)*

### Functional Requirements

- **FR-001**: 系统 MUST 支持通过配置文件定义 API 服务，每个服务定义包含：服务名称、功能模式（文生图/图生图/翻译）、API 端点 URL、认证方式、请求模板、响应结果提取规则
- **FR-002**: 系统 MUST 支持在请求模板中使用占位符变量（如 `{prompt}`、`{image_url}`、`{size}` 等），在实际请求时替换为用户输入
- **FR-003**: 系统 MUST 支持从 API 响应中通过可配置的路径规则提取结果图片 URL
- **FR-004**: 系统 MUST 支持至少两种 API 调用模式：同步请求（发送后等待完整响应）和异步提交+轮询（提交后定时查询状态）
- **FR-005**: 系统 MUST 在启动时验证所有 API 服务定义的完整性和格式正确性，对无效配置给出具体的错误提示
- **FR-006**: 系统 MUST 向后兼容现有的 setting.json 配置格式，现有用户无需修改配置即可正常使用
- **FR-007**: 系统 MUST 允许用户在界面上选择当前使用的 API 服务（当同一功能模式下有多个服务可用时）
- **FR-008**: 系统 MUST 支持至少两种认证方式：Bearer Token（`Authorization: Bearer {key}`）和自定义 Header（用户指定 Header 名称和值格式）
- **FR-009**: 系统 MUST 在 API 请求失败时提供包含服务名称的错误信息，帮助用户识别是哪个服务出了问题
- **FR-010**: 系统 MUST 保持现有的图片 Base64 编码能力，配置驱动的服务同样支持本地图片自动编码

### Key Entities

- **API 服务定义 (API Service Definition)**: 描述一个 AI 图像 API 的完整调用规则，包括名称、功能模式、端点、认证、请求模板和响应映射。一个 Provider 下可有多个服务定义。
- **请求模板 (Request Template)**: 定义发送给 API 的 JSON 请求体结构，含占位符变量。系统在运行时将占位符替换为实际值。
- **响应映射 (Response Mapping)**: 定义如何从 API 响应的 JSON 中提取所需数据（如图片 URL），使用路径表达式指向目标字段。
- **Provider**: 一个 API 服务提供商，拥有共享的认证信息（API Key）和一组 API 服务定义。

## Assumptions

- 所有目标 API 服务均使用 JSON 格式的 HTTP REST 接口（不支持 gRPC、WebSocket、GraphQL 等协议）
- API 的认证信息（如 API Key）以明文存储在本地配置文件中，安全性由用户的文件系统权限保证（与现有行为一致）
- 配置文件仍使用 JSON 格式（setting.json），不引入 YAML 或 TOML 等新格式
- 异步轮询模式的轮询间隔和超时时间使用合理的默认值（如 3 秒间隔、180 秒超时），但可在配置中覆盖
- 前端界面的三种功能模式（文生图、图生图、翻译）保持不变，配置驱动的变化主要在后端

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: 用户可以在 10 分钟内通过编辑配置文件新增一个 API 服务定义，并成功使用该服务生成图片
- **SC-002**: 现有 DashScope 三种功能在升级后 100% 保持可用，无需用户修改配置文件
- **SC-003**: 系统可同时管理至少 5 个不同的 API 服务定义，用户可在 3 次点击内切换当前使用的服务
- **SC-004**: 配置错误的诊断信息足够清晰，用户能在 2 分钟内定位并修复配置问题
- **SC-005**: 添加新的 API 服务不需要修改任何应用程序源代码，仅通过配置文件完成
