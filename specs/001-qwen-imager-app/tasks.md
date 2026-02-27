# Tasks: Qwen AI Image Generation Desktop App

**Input**: Design documents from `/specs/001-qwen-imager-app/`
**Prerequisites**: plan.md, spec.md, data-model.md, contracts/, research.md, quickstart.md

**Organization**: Tasks are grouped by user story to enable independent implementation and testing of each story.

## Format: `[ID] [P?] [Story?] Description`

- **[P]**: Can run in parallel (different files, no dependencies)
- **[Story]**: Which user story this task belongs to (e.g., US1, US2, US3)
- Include exact file paths in descriptions

---

## Phase 1: Setup (Shared Infrastructure)

**Purpose**: Tauri v2 项目初始化，安装所有依赖，建立项目骨架

- [x] T001 使用 `pnpm create tauri-app . --template react-ts --manager pnpm` 初始化 Tauri v2 项目，生成 src-tauri/ 和 src/ 基本结构
- [x] T002 安装前端依赖：运行 `pnpm add tailwindcss @tailwindcss/vite zustand @dnd-kit/core @dnd-kit/sortable @dnd-kit/utilities @tanstack/react-virtual lucide-react` 和 `npx shadcn@latest init`
- [x] T003 配置 Rust 依赖：在 src-tauri/Cargo.toml 中添加 tauri v2、tauri-plugin-store、tauri-plugin-dialog、tauri-plugin-fs、tauri-plugin-clipboard-manager、reqwest (json, rustls-tls)、tokio (full)、serde (derive)、serde_json、thiserror、rusqlite (bundled)、uuid (v4)、dirs
- [x] T004 [P] 配置 Tailwind CSS：在 vite.config.ts 中添加 @tailwindcss/vite 插件，在 src/styles/globals.css 中导入 tailwindcss
- [x] T005 [P] 按 plan.md 中的项目结构创建前端目录骨架：src/components/layout/、src/components/chat/、src/components/input/、src/components/common/、src/stores/、src/hooks/、src/lib/
- [x] T006 [P] 按 plan.md 中的项目结构创建 Rust 后端目录骨架：src-tauri/src/commands/、src-tauri/src/models/、src-tauri/src/services/，并创建各目录的 mod.rs 文件
- [x] T007 验证项目可以成功编译运行：执行 `pnpm tauri dev` 确认应用窗口正常打开

---

## Phase 2: Foundational (Blocking Prerequisites)

**Purpose**: 核心基础设施，所有用户故事均依赖此阶段完成

**⚠️ CRITICAL**: 所有用户故事的工作必须在此阶段完成后才能开始

- [x] T008 在 src-tauri/src/error.rs 中定义 AppError 枚举类型（含 Http、Api、Timeout、Config、Unauthorized、RateLimited、Io、NotFound、Validation 变体），使用 thiserror 派生 Error，实现 Serialize 以 `{ kind, message }` 格式序列化传递给前端
- [x] T009 在 src-tauri/src/models/config.rs 中定义 Configuration、Provider、ModelConfig 数据结构（对应 data-model.md 中的配置实体），使用 Serde 支持 JSON 反序列化，字段包括 providers Map、api_key、models Map（含 url 字段）
- [x] T010 在 src-tauri/src/services/config_loader.rs 中实现配置文件加载服务：从 `~/.qwenimage/setting.json` 读取并解析为 Configuration 结构体，处理文件不存在、格式错误等情况并返回对应的 AppError
- [x] T011 在 src-tauri/src/commands/config.rs 中实现 `load_config` Tauri command（按 contracts/tauri-commands.md 定义的接口），返回 ConfigStatus { loaded, available_models, error_message }
- [x] T012 在 src-tauri/src/models/conversation.rs 中定义 Conversation 数据结构（id: UUID, title: String, created_at: i64, updated_at: i64），在 src-tauri/src/models/message.rs 中定义 Message 数据结构（id, conversation_id, role, text_content, mode, extra_params, created_at），两者均派生 Serialize/Deserialize
- [x] T013 在 src-tauri/src/models/api.rs 中定义 DashScope API 请求/响应类型：TextToImageRequest、ImageEditRequest、TranslationRequest、AsyncTaskResponse（含 task_id、task_status）、TaskQueryResponse（含 results 数组和 task_metrics）、GenerationParams（size、n、negative_prompt、prompt_extend、watermark）
- [x] T014 创建 SQLite 数据库初始化迁移文件 src-tauri/migrations/001_initial.sql，包含 conversations、messages、attachments、generation_results 四张表的 CREATE TABLE 语句和索引（按 data-model.md 中的 SQLite Schema 部分）
- [x] T015 在 src-tauri/src/services/db.rs 中实现数据库服务：初始化 SQLite 连接（数据库文件路径为 `{app_data_dir}/qwenimager.db`）、执行迁移脚本、提供 connection pool 或单例访问，并在 Tauri 启动时通过 manage() 注入为 AppState
- [x] T016 在 src-tauri/src/services/qwen_client.rs 中实现 QwenClient 结构体：使用 reqwest::Client 实例，提供 submit_async_task()（发送 POST 请求带 X-DashScope-Async 头，返回 task_id）、poll_task()（GET /api/v1/tasks/{task_id}，返回任务状态和结果）、send_sync_request()（图生图同步请求）三个核心方法，处理 401/429/500 等 HTTP 错误码映射到 AppError
- [x] T017 在 src-tauri/src/lib.rs 中注册 Tauri 插件（store、dialog、fs、clipboard-manager）和所有 Tauri commands，配置 AppState（含 reqwest::Client、数据库连接、配置信息），设置应用启动时自动加载配置和初始化数据库
- [x] T018 在 src/stores/configStore.ts 中创建 Zustand store 管理配置状态：loaded、availableModels、errorMessage 字段，提供 loadConfig action（调用 invoke('load_config')）
- [x] T019 在 src/lib/tauri.ts 中创建 Tauri IPC 封装工具：统一的 invoke 包装函数（含错误处理和 AppError 类型解析）、Channel 创建辅助函数、TypeScript 类型定义（GenerationEvent、ConfigStatus、AppError、ConversationSummary、MessageDetail 等，按 contracts/tauri-commands.md 中的类型定义）
- [x] T020 在 src/components/common/ErrorDisplay.tsx 中实现错误展示组件：根据 AppError.kind 字段渲染不同样式和提示文案（unauthorized → 检查 API 密钥、network → 检查网络连接、config → 检查配置文件等）
- [x] T021 在 src/components/common/LoadingState.tsx 中实现加载状态组件：支持显示文字提示（如"正在生成..."、"正在轮询..."）和加载动画

**Checkpoint**: 基础设施就绪 — 数据库已初始化、配置可加载、API 客户端可用、错误处理已建立，用户故事实现可以开始

---

## Phase 3: User Story 1 - 文生图：输入文字生成图片 (Priority: P1) 🎯 MVP

**Goal**: 用户输入文字 Prompt，系统调用 Qwen 文生图 API 生成图片，以对话形式展示结果，支持复制和下载

**Independent Test**: 输入一段文字描述并提交，验证图片是否成功生成并展示在对话区域中，可以复制和下载

### Implementation for User Story 1

- [x] T022 [US1] 在 src-tauri/src/services/db.rs 中实现对话和消息的 CRUD 操作：create_conversation()、add_message()（含 user 和 assistant 角色）、add_generation_result()（保存 API 返回的图片 URL 和本地路径）、get_conversation_messages()（JOIN 查询消息及其关联的 generation_results），update_conversation_title()（首次提交时用 Prompt 前 30 字符更新标题）
- [x] T023 [US1] 在 src-tauri/src/commands/generation.rs 中实现 `generate_image` Tauri command（按 contracts/tauri-commands.md 定义）：接收 conversation_id、prompt、params、on_event Channel 参数，调用 db 创建 user message，调用 QwenClient.submit_async_task() 提交文生图请求，通过 Channel 发送 Submitted 事件，循环调用 poll_task() 每 3 秒轮询一次（最长 180 秒），通过 Channel 发送 Polling/Succeeded/Failed 事件，成功时将结果保存到 generation_results 表并创建 assistant message
- [x] T024 [US1] 在 src-tauri/src/commands/generation.rs 中实现 `save_image` Tauri command（按 contracts/tauri-commands.md 定义）：支持从 URL 下载或从本地路径复制图片到用户指定位置（使用 tauri-plugin-dialog 弹出保存对话框），返回保存后的文件路径
- [x] T025 [US1] 在 src/stores/conversationStore.ts 中创建 Zustand store 管理对话状态：currentConversationId、messages 数组、generatingStatus（submitted/polling/succeeded/failed）字段，提供 createConversation、sendPrompt（调用 invoke 并监听 Channel 事件更新 generatingStatus）、addMessage 等 actions
- [x] T026 [US1] 在 src/hooks/useGeneration.ts 中实现文生图 hook：封装 Channel 创建和事件监听逻辑，管理生成状态（isGenerating、taskId、status），调用 conversationStore 的 actions 更新消息列表，处理错误并触发 ErrorDisplay
- [x] T027 [US1] 在 src/components/input/ModeSelector.tsx 中实现模式选择组件（按 contracts/ui-contracts.md 定义）：使用 shadcn/ui 的 ToggleGroup 或 Tabs 组件实现三个模式切换按钮（文生图/图生图/翻译），当前先仅启用 text2img 模式
- [x] T028 [US1] 在 src/components/input/PromptInput.tsx 中实现 Prompt 输入组件（按 contracts/ui-contracts.md 定义）：使用 shadcn/ui 的 Textarea 实现多行输入框，支持 Enter 提交 / Shift+Enter 换行，提交按钮在 disabled、submitting 或输入为空时禁用，提交后清空输入框
- [x] T029 [US1] 在 src/components/chat/MessageBubble.tsx 中实现消息气泡组件：根据 role 字段区分用户消息（靠右）和助手消息（靠左），用户消息显示文字内容，助手消息显示关联的生成结果图片
- [x] T030 [US1] 在 src/components/chat/ImageResult.tsx 中实现生成图片展示组件：显示图片（支持点击放大预览），提供悬浮操作栏包含"复制"按钮（调用剪贴板 API 复制图片）和"下载"按钮（调用 invoke('save_image')）
- [x] T031 [US1] 在 src/components/chat/ChatArea.tsx 中实现对话展示区域组件（按 contracts/ui-contracts.md 定义）：渲染 MessageBubble 列表，底部显示 LoadingState（当 generatingStatus 非空时），新消息自动滚动到底部，使用 @tanstack/react-virtual 支持虚拟滚动
- [x] T032 [US1] 在 src/components/layout/MainPanel.tsx 中组装主面板布局：上方 ChatArea 占据剩余空间，下方固定区域包含 ModeSelector + PromptInput，当配置未加载时整个面板显示 ErrorDisplay 提示配置缺失
- [x] T033 [US1] 在 src/App.tsx 中组装根布局：启动时调用 configStore.loadConfig() 加载配置，渲染 MainPanel（暂不含 Sidebar，留给 US4），将 conversationStore 连接到 MainPanel

**Checkpoint**: 文生图功能完整可用 — 用户可以输入 Prompt 生成图片、查看结果、复制和下载图片。这是一个完整的 MVP。

---

## Phase 4: User Story 2 - 图生图：上传参考图片生成新图片 (Priority: P2)

**Goal**: 用户上传多张参考图片并结合文字描述，调用图生图 API 生成新图片，支持拖拽排序和剪贴板粘贴

**Independent Test**: 上传一张参考图片并输入描述，验证系统能正确组合图片和文字发送请求并展示结果

### Implementation for User Story 2

- [x] T034 [P] [US2] 在 src-tauri/src/models/message.rs 中添加 Attachment 数据结构（id, message_id, file_path, display_order, file_size, mime_type, source），在 src-tauri/src/services/db.rs 中添加 add_attachment()、get_attachments_by_message() 方法
- [x] T035 [P] [US2] 在 src-tauri/src/commands/generation.rs 中实现 `save_clipboard_image` Tauri command（按 contracts/tauri-commands.md 定义）：接收前端传来的图片二进制数据和 mime_type，保存到 `{temp_dir}/qwenimager-clipboard/` 目录下生成唯一文件名，返回临时文件路径
- [x] T036 [US2] 在 src-tauri/src/commands/generation.rs 中实现 `edit_image` Tauri command（按 contracts/tauri-commands.md 定义）：接收 conversation_id、image_paths 数组（按顺序）、prompt、params 参数，构建图生图 API 请求体（messages 格式，图片以 image 字段、文字以 text 字段按顺序排列），发送同步请求（无异步头），解析响应保存 generation_results 和 assistant message，将图片路径保存为 attachments 记录
- [x] T037 [US2] 在 src/hooks/useClipboard.ts 中实现剪贴板粘贴 hook：监听 paste 事件，从 clipboardData.items 中提取图片 Blob，转换为 Uint8Array 后调用 invoke('save_clipboard_image') 保存临时文件，返回文件路径用于添加到上传列表
- [x] T038 [US2] 在 src/components/input/ImageUpload.tsx 中实现多图上传组件（按 contracts/ui-contracts.md 定义）：使用 tauri-plugin-dialog 的 open() 打开文件选择器（过滤 png/jpg/webp/gif），已选图片以缩略图网格展示带序号标记（1,2,3...），每张图片有删除按钮，使用 @dnd-kit/sortable 实现拖拽排序，超过 10 张或单张超过 10MB 时提示错误，集成 useClipboard hook 支持 Ctrl+V 粘贴
- [x] T039 [US2] 更新 src/stores/conversationStore.ts：添加 uploadedImages 数组字段和 addImages、removeImage、reorderImages、clearImages actions，在 sendPrompt 中根据当前 mode 判断调用 generate_image 还是 edit_image
- [x] T040 [US2] 更新 src/components/input/ModeSelector.tsx：启用 img2img 模式按钮
- [x] T041 [US2] 更新 src/components/layout/MainPanel.tsx：当 mode 为 img2img 时在 PromptInput 上方显示 ImageUpload 组件，将 uploadedImages 和相关事件连接到 conversationStore
- [x] T042 [US2] 更新 src/components/chat/MessageBubble.tsx：当用户消息包含 attachments 时，在文字内容下方以缩略图网格形式展示上传的参考图片（带序号）

**Checkpoint**: 图生图功能完整可用 — 用户可以上传多张图片、拖拽排序、从剪贴板粘贴，结合文字生成新图片。文生图功能（US1）仍然独立可用。

---

## Phase 5: User Story 3 - 图片文字翻译 (Priority: P3)

**Goal**: 用户上传含文字的图片，选择源语言和目标语言，调用翻译 API 生成翻译后的图片

**Independent Test**: 上传一张包含中文的图片，选择翻译为英文，验证翻译后的图片正确展示

### Implementation for User Story 3

- [x] T043 [P] [US3] 在 src-tauri/src/commands/translation.rs 中实现 `translate_image` Tauri command（按 contracts/tauri-commands.md 定义）：接收 conversation_id、image_path、source_lang、target_lang、on_event Channel 参数，构建翻译 API 请求体（含 image_url、source_lang、target_lang、ext.config），通过异步提交-轮询模式处理（同 generate_image 的轮询逻辑），将翻译结果保存到 generation_results 表
- [x] T044 [P] [US3] 在 src/components/input/LanguageSelector.tsx 中实现语言选择组件（按 contracts/ui-contracts.md 定义）：使用 shadcn/ui 的 Select 组件实现两个语言下拉框（源语言/目标语言），中间放置交换按钮，支持 zh/en/ja/ko/fr/de/es/ru 八种语言，默认源语言 zh、目标语言 en，验证源语言和目标语言不能相同
- [x] T045 [US3] 更新 src/stores/conversationStore.ts：添加 sourceLang、targetLang 字段和 setSourceLang、setTargetLang actions，在 sendPrompt 中当 mode 为 translate 时调用 translate_image command 并传入语言参数
- [x] T046 [US3] 更新 src/components/input/ModeSelector.tsx：启用 translate 模式按钮
- [x] T047 [US3] 更新 src/components/layout/MainPanel.tsx：当 mode 为 translate 时显示 ImageUpload（限制单张上传）+ LanguageSelector 组件，隐藏 PromptInput（翻译模式不需要文字输入）

**Checkpoint**: 图片翻译功能完整可用 — 用户可以上传图片并翻译其中的文字。文生图（US1）和图生图（US2）功能仍然独立可用。

---

## Phase 6: User Story 4 - 对话历史管理 (Priority: P4)

**Goal**: 左侧可折叠历史栏展示对话列表，点击恢复完整对话内容，支持新建和删除对话

**Independent Test**: 完成几次生成操作后，在历史栏中点击不同记录，验证对话能完整恢复

### Implementation for User Story 4

- [x] T048 [P] [US4] 在 src-tauri/src/commands/conversation.rs 中实现 `get_conversations` Tauri command（按 contracts/tauri-commands.md 定义）：查询 conversations 表并 LEFT JOIN messages 获取消息数量，支持 limit 和 offset 参数，按 updated_at 降序返回 ConversationSummary 数组
- [x] T049 [P] [US4] 在 src-tauri/src/commands/conversation.rs 中实现 `get_conversation_messages` Tauri command：根据 conversation_id 查询 messages 表并 LEFT JOIN attachments 和 generation_results，按 created_at 升序返回 MessageDetail 数组
- [x] T050 [P] [US4] 在 src-tauri/src/commands/conversation.rs 中实现 `create_conversation` 和 `delete_conversation` Tauri commands：create 生成 UUID 插入 conversations 表，delete 按 CASCADE 删除对话及其所有消息、附件和生成结果，同时清理 attachments 和 cache 目录下的关联文件
- [x] T051 [US4] 更新 src/stores/conversationStore.ts：添加 conversations 数组字段（ConversationSummary[]），提供 loadConversations（调用 get_conversations）、switchConversation（调用 get_conversation_messages 并更新 messages）、deleteConversation actions，在 sendPrompt 成功后自动刷新 conversations 列表
- [x] T052 [US4] 在 src/stores/uiStore.ts 中创建 Zustand store 管理 UI 状态：sidebarCollapsed 字段，提供 toggleSidebar action，使用 tauri-plugin-store 持久化侧边栏折叠状态
- [x] T053 [US4] 在 src/components/layout/Sidebar.tsx 中实现历史侧边栏组件（按 contracts/ui-contracts.md 定义）：渲染对话列表（每条显示 title 截断至一行 + 时间戳），当前对话高亮，顶部放置"新建对话"按钮，每条对话提供删除按钮（通过右键菜单或悬浮显示），折叠状态下仅显示展开按钮图标，使用 @tanstack/react-virtual 支持虚拟滚动（100+ 对话不卡顿）
- [x] T054 [US4] 更新 src/App.tsx：将布局改为左侧 Sidebar + 右侧 MainPanel 的 flex 布局，启动时调用 conversationStore.loadConversations() 加载历史列表，连接 Sidebar 的事件到 conversationStore（选择/新建/删除对话），连接 uiStore 控制侧边栏折叠
- [x] T055 [US4] 更新 src/components/layout/MainPanel.tsx：当没有选中对话时显示欢迎页面（提示用户新建对话），折叠动画过渡效果

**Checkpoint**: 对话历史管理完整可用 — 用户可以查看、切换、新建、删除对话，侧边栏可折叠。所有之前的功能仍然独立可用。

---

## Phase 7: User Story 5 - 配置加载与状态提示 (Priority: P5)

**Goal**: 优化启动体验，在配置缺失或无效时显示友好的引导提示，支持配置格式错误的具体错误信息

**Independent Test**: 分别在有配置、无配置、配置格式错误的情况下启动应用，验证应用行为符合预期

### Implementation for User Story 5

- [x] T056 [P] [US5] 在 src/components/common/ConfigGuide.tsx 中实现配置引导组件：当配置未加载时展示友好提示页面，包含配置文件路径说明（~/.qwenimage/setting.json）、JSON 格式示例、步骤指引，当配置格式错误时显示具体的错误信息（来自 ConfigStatus.error_message）
- [x] T057 [US5] 更新 src-tauri/src/services/config_loader.rs：增强配置验证逻辑，检查必要字段是否存在（providers.qwen.apiKey、providers.qwen.models），为每种缺失情况生成具体的错误描述（如"缺少 apiKey 字段"、"models 格式不正确"），在 API 密钥被服务端拒绝（401 错误）时更新配置状态为 unauthorized
- [x] T058 [US5] 更新 src/App.tsx：在应用根组件中根据 configStore 的状态决定渲染 ConfigGuide 还是主界面，当 loaded 为 false 时全屏展示 ConfigGuide，当 loaded 为 true 时正常渲染 Sidebar + MainPanel

**Checkpoint**: 配置体验优化完成 — 所有配置状态都有友好的用户引导。

---

## Phase 8: Polish & Cross-Cutting Concerns

**Purpose**: 跨功能的优化和边缘情况处理

- [x] T059 [P] 实现重复提交防护：在 src/stores/conversationStore.ts 中添加 isSubmitting 状态锁，在 src/components/input/PromptInput.tsx 中根据 isSubmitting 禁用提交按钮（FR-016）
- [x] T060 [P] 在 src-tauri/src/services/qwen_client.rs 中实现 429 限流重试逻辑：检测 Retry-After 头，最多重试 3 次，超出后返回 RateLimited 错误
- [x] T061 [P] 在 src/components/chat/ImageResult.tsx 中实现图片放大预览功能：点击图片弹出全屏预览对话框（使用 shadcn/ui 的 Dialog），支持原始尺寸查看
- [x] T062 [P] 实现文件大小验证：在 src/components/input/ImageUpload.tsx 中检查文件大小超过 10MB 时使用 shadcn/ui 的 Toast 提示错误，拒绝添加到列表
- [x] T063 [P] 实现文件类型验证：在 src/components/input/ImageUpload.tsx 中检查文件 MIME 类型，拒绝非图片格式文件并提示仅支持 png/jpg/webp/gif
- [x] T064 [P] 在 src-tauri/src/lib.rs 中注册应用退出事件处理，清理 `{temp_dir}/qwenimager-clipboard/` 临时目录中的剪贴板图片文件
- [x] T065 运行 `pnpm tauri build` 验证生产构建，检查安装包大小是否满足 50MB 以内的目标（SC-006）
- [x] T066 运行 `cargo clippy --manifest-path src-tauri/Cargo.toml` 检查 Rust 代码质量，修复所有 warning

---

## Dependencies & Execution Order

### Phase Dependencies

- **Setup (Phase 1)**: 无依赖 — 可立即开始
- **Foundational (Phase 2)**: 依赖 Phase 1 完成 — **阻塞所有用户故事**
- **User Stories (Phase 3-7)**: 全部依赖 Phase 2 完成
  - US1 (Phase 3): 可在 Phase 2 完成后立即开始
  - US2 (Phase 4): 可在 Phase 2 完成后独立开始，但建议在 US1 之后（复用 ChatArea、MessageBubble 等组件）
  - US3 (Phase 5): 可在 Phase 2 完成后独立开始，但建议在 US2 之后（复用 ImageUpload 组件）
  - US4 (Phase 6): 可在 Phase 2 完成后独立开始，不依赖其他用户故事
  - US5 (Phase 7): 可在 Phase 2 完成后独立开始，不依赖其他用户故事
- **Polish (Phase 8)**: 依赖所有用户故事完成

### User Story Dependencies

- **US1 (P1)**: Phase 2 完成后可开始 — 不依赖其他故事
- **US2 (P2)**: Phase 2 完成后可开始 — 建议在 US1 之后（复用 ChatArea、MessageBubble、conversationStore）
- **US3 (P3)**: Phase 2 完成后可开始 — 建议在 US2 之后（复用 ImageUpload 组件和 Attachment 模型）
- **US4 (P4)**: Phase 2 完成后可开始 — 独立于生成功能，但需要数据库中有数据来测试
- **US5 (P5)**: Phase 2 完成后可开始 — 独立于所有其他故事

### Within Each User Story

- Rust commands/models 先于前端组件
- 前端 stores/hooks 先于 UI 组件
- 基础组件先于组合组件

### Parallel Opportunities

- Phase 1: T004、T005、T006 可并行执行
- Phase 2: T008-T009 可并行，T012-T013 可并行，T018-T021 可并行
- Phase 4: T034、T035 可并行
- Phase 5: T043、T044 可并行
- Phase 6: T048、T049、T050 可并行
- Phase 8: T059-T064 全部可并行

---

## Parallel Example: User Story 1

```bash
# Rust 后端任务可先并行完成：
Task: "T022 - 实现对话和消息的 CRUD 操作 (src-tauri/src/services/db.rs)"
Task: "T023 - 实现 generate_image command (src-tauri/src/commands/generation.rs)"
Task: "T024 - 实现 save_image command (src-tauri/src/commands/generation.rs)"

# 前端 store 和 hook（依赖 Rust 后端类型定义）：
Task: "T025 - 创建 conversationStore (src/stores/conversationStore.ts)"
Task: "T026 - 实现 useGeneration hook (src/hooks/useGeneration.ts)"

# 前端 UI 组件可并行：
Task: "T027 - ModeSelector 组件 (src/components/input/ModeSelector.tsx)"
Task: "T028 - PromptInput 组件 (src/components/input/PromptInput.tsx)"
Task: "T029 - MessageBubble 组件 (src/components/chat/MessageBubble.tsx)"
Task: "T030 - ImageResult 组件 (src/components/chat/ImageResult.tsx)"
```

---

## Parallel Example: User Story 4

```bash
# 三个 Rust command 可完全并行：
Task: "T048 - get_conversations command (src-tauri/src/commands/conversation.rs)"
Task: "T049 - get_conversation_messages command (src-tauri/src/commands/conversation.rs)"
Task: "T050 - create/delete conversation commands (src-tauri/src/commands/conversation.rs)"
```

---

## Implementation Strategy

### MVP First (User Story 1 Only)

1. Complete Phase 1: Setup
2. Complete Phase 2: Foundational (CRITICAL - blocks all stories)
3. Complete Phase 3: User Story 1
4. **STOP and VALIDATE**: 测试文生图全流程 — 输入 Prompt → 生成图片 → 查看结果 → 复制/下载
5. 此时应用已具备核心价值，可以发布 MVP

### Incremental Delivery

1. Complete Setup + Foundational → 基础设施就绪
2. Add US1 → 文生图可用 → **MVP!**
3. Add US2 → 图生图可用 → 增量发布
4. Add US3 → 图片翻译可用 → 增量发布
5. Add US4 → 对话历史管理 → 增量发布
6. Add US5 → 配置体验优化 → 增量发布
7. Polish → 边缘情况处理和代码质量 → 正式发布

### Recommended Execution Order (Single Developer)

Phase 1 → Phase 2 → Phase 3 (US1) → Phase 4 (US2) → Phase 5 (US3) → Phase 6 (US4) → Phase 7 (US5) → Phase 8 (Polish)

---

## Notes

- [P] tasks = different files, no dependencies
- [Story] label maps task to specific user story for traceability
- Each user story should be independently completable and testable
- Commit after each task or logical group
- Stop at any checkpoint to validate story independently
- US5（配置优化）虽然优先级 P5，但可以提前实现以改善开发体验
- 所有 Rust 代码中涉及的 Tauri command 都需要在 src-tauri/src/lib.rs 的 invoke_handler 中注册
