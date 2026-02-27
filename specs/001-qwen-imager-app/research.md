# Research: Qwen AI Image Generation Desktop App

**Feature Branch**: `001-qwen-imager-app`  
**Date**: 2026-02-27

## 研究领域

### 1. 前端框架选型

**决策**: React + Vite + TypeScript

**理由**:
- UI 组件生态最成熟：Shadcn/ui + Radix 原语提供开箱即用的聊天界面、侧边栏、图片网格、右键菜单、对话框等组件
- 拖拽排序方案最佳：`@dnd-kit` 是目前最成熟的拖拽库，完美支持图片排序需求（FR-009）
- 最低风险：社区最大，WebView 兼容性问题、剪贴板边缘情况、虚拟滚动性能等均有现成解决方案
- Tauri 社区中 React 项目最多，调试 Tauri + 前端集成问题时参考资源最丰富
- 包体积差异可忽略：React (~50KB) vs Svelte (~10KB) 的差异仅 40KB，相对 50MB 安装包目标占比 0.08%

**考虑的替代方案**:

| 框架 | 拒绝原因 |
|------|---------|
| Vue 3 | 强力竞争者，中文社区优势明显。组件生态略逊于 React（Naive UI 优秀但 Shadcn/ui 的组件所有权模式更利于长期维护）。如开发者强烈偏好 Vue，这是完全可行的选择。 |
| Svelte 5 | 最佳 DX 和最小包体积。组件生态较薄，shadcn-svelte 落后于 React 版本，DnD 库不够成熟。 |
| Solid.js | 性能基准最佳。生态太小，Kobalte 组件库有缺口，不适合 18 条功能需求的生产级应用。 |

**关键依赖库**:
- `react` + `react-dom` - 框架核心
- `vite` - 构建工具
- `typescript` - 类型安全
- `@tauri-apps/api` - Tauri IPC、事件、文件系统、对话框
- `tailwindcss` - 原子化 CSS
- `shadcn/ui` - UI 组件原语
- `@dnd-kit/core` + `@dnd-kit/sortable` - 拖拽排序
- `@tanstack/react-virtual` - 虚拟滚动（100+ 对话历史）
- `zustand` - 轻量级状态管理（~1KB，无模板代码）
- `lucide-react` - 图标库

---

### 2. 本地数据存储方案

**决策**: SQLite（通过 Rust 后端的 `rusqlite` crate，使用 `bundled` feature）

**理由**:
- 完整 SQL 查询能力：支持 SELECT、JOIN、ORDER BY、LIMIT、WHERE 等，完美满足对话列表和消息检索需求
- 性能优异：对话列表和消息加载的索引扫描在亚毫秒级完成，远超 SC-005（3 秒历史恢复）的要求
- 轻量级：编译后约 1MB，`bundled` feature 静态链接，无外部依赖
- 跨平台：SQLite 是最可移植的数据库，`rusqlite` + `bundled` 在 Windows/macOS/Linux 上无需系统依赖即可编译
- 架构契合：Rust 后端是 Tauri v2 中数据管理的自然位置，前端保持精简
- 备份简单：单个 `.db` 文件即可完整备份

**考虑的替代方案**:

| 方案 | 拒绝原因 |
|------|---------|
| tauri-plugin-store | 无查询能力，仅适合键值存储（如设置/偏好）。获取排序后的对话列表需反序列化整个存储，不适合对话数据。 |
| JSON 文件 | 需手动维护索引文件，无原子操作（崩溃时可能损坏），实质是重新发明数据库。 |
| IndexedDB | 平台不一致性（Tauri 使用系统 WebView），数据生命周期不可靠，错误的架构层（数据应在 Rust 层管理）。 |
| sql.js (WASM) | 增加 ~1.5MB 前端包体积，整个数据库需加载到内存，架构层错误（Rust 端无法直接访问）。 |

**附加决策**: 使用 `tauri-plugin-store` 存储用户偏好和 UI 状态（如侧边栏折叠状态），与 SQLite 互补。

---

### 3. Qwen/DashScope API 异步模式

**决策**: 异步 API（文生图、图片翻译）使用提交-轮询模式，同步 API（图生图编辑）使用直接请求-响应模式

**API 异步性分类**:

| API | 模型 | 异步 | 说明 |
|-----|------|------|------|
| 文生图 | qwen-image-plus | 是 | `X-DashScope-Async: enable` |
| 图生图编辑 | qwen-image-edit-max | 否 | 同步请求 |
| 图片翻译 | qwen-mt-image | 是 | `X-DashScope-Async: enable` |

**异步流程**:
1. **提交任务**: POST 到对应端点，返回 `task_id` 和 `task_status: PENDING`
2. **轮询状态**: GET `https://dashscope.aliyuncs.com/api/v1/tasks/{task_id}`，检查 `task_status`
3. **获取结果**: 当 `task_status` 为 `SUCCEEDED` 时，从 `results[].url` 获取图片 URL

**轮询策略**:
- 初始延迟：3 秒
- 轮询间隔：3 秒
- 最大超时：180 秒（3 分钟）

**任务状态值**: PENDING → RUNNING → SUCCEEDED / FAILED / CANCELED

---

### 4. Rust HTTP 客户端与前后端通信

**决策**: 使用 `reqwest` + `rustls-tls`，通过 Tauri Channels 传递异步进度

**理由**:
- Tauri v2 内置 `tokio` 运行时，`reqwest` 基于 `tokio` 无缝集成
- `rustls-tls` 避免链接 OpenSSL，减小二进制大小并简化跨平台编译
- API 调用从 Rust 后端发起，API 密钥不暴露给 WebView（安全性）

**前后端通信机制**:

| 机制 | 用途 |
|------|------|
| Channels (`tauri::ipc::Channel<T>`) | 异步轮询流程的进度流（已提交/轮询中/成功/失败） |
| Commands（返回值） | 同步图生图 API（单次请求-响应） |
| Events | 全局通知（配置变更、应用级别警告） |

---

### 5. 错误处理策略

**决策**: 使用 `thiserror` 定义语义化错误类型，实现 `Serialize` 以传递给前端

**错误分类**:

| HTTP 状态 | DashScope 含义 | 处理策略 |
|-----------|---------------|---------|
| 200 | 成功 | 处理响应 |
| 400 | 请求错误/参数无效 | 显示参数验证错误 |
| 401 | API 密钥无效 | 提示检查 API 密钥 |
| 429 | 限流 | 等待并重试（尊重 Retry-After 头） |
| 500 | 服务器错误 | 退避重试，然后失败 |
| 超时 | 网络超时 | 重试一次，然后显示网络错误 |

**Rust 依赖**:

```toml
[dependencies]
tauri = { version = "2", features = [] }
reqwest = { version = "0.12", features = ["json", "rustls-tls"] }
tokio = { version = "1", features = ["full"] }
serde = { version = "1", features = ["derive"] }
serde_json = "1"
thiserror = "2"
rusqlite = { version = "0.31", features = ["bundled"] }
```
