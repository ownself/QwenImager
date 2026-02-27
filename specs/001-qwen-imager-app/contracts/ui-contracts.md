# UI Contracts: Component Interface Specifications

**Feature Branch**: `001-qwen-imager-app`  
**Date**: 2026-02-27

## Layout Structure

```
┌────────────────────────────────────────────────────────┐
│                    Window Title Bar                     │
├──────────────┬─────────────────────────────────────────┤
│              │                                         │
│   Sidebar    │           Chat Area                     │
│  (History)   │     (Conversation Display)              │
│              │                                         │
│  - Conv 1    │   ┌─────────────────────────────┐      │
│  - Conv 2    │   │ [User] Prompt text...       │      │
│  - Conv 3    │   │ [Asst] 🖼️ Generated Image   │      │
│  ...         │   │ [User] Another prompt...    │      │
│              │   │ [Asst] 🖼️ Another Image     │      │
│              │   └─────────────────────────────┘      │
│              │                                         │
│  [Collapse]  ├─────────────────────────────────────────┤
│              │  [📎 Uploaded Images: 1️⃣ 2️⃣ 3️⃣]        │
│              │  ┌─────────────────────────┐ [Submit]   │
│              │  │ Enter your prompt...    │ [Mode ▼]   │
│              │  └─────────────────────────┘            │
├──────────────┴─────────────────────────────────────────┤
│                    Status Bar (optional)                │
└────────────────────────────────────────────────────────┘
```

## Component Contracts

### Sidebar (History Panel)

**职责**: 展示对话历史列表，支持折叠/展开，点击切换对话。

| Property | Type | Description |
|----------|------|-------------|
| conversations | ConversationSummary[] | 对话摘要列表 |
| activeConversationId | string \| null | 当前活跃的对话 ID |
| collapsed | boolean | 是否折叠状态 |

| Event | Payload | Description |
|-------|---------|-------------|
| onSelectConversation | conversationId: string | 用户点击某条对话 |
| onNewConversation | void | 用户点击新建对话 |
| onDeleteConversation | conversationId: string | 用户删除某条对话 |
| onToggleCollapse | void | 用户点击折叠/展开按钮 |

**UI 行为**:
- 对话列表按 `updatedAt` 降序排列
- 每条对话显示 `title`（截断至一行）和时间
- 活跃对话高亮显示
- 折叠后仅显示折叠按钮图标，宽度缩至最小
- 超过可视区域时支持滚动

---

### ChatArea (Conversation Display)

**职责**: 以对话气泡形式展示当前对话的全部消息。

| Property | Type | Description |
|----------|------|-------------|
| messages | MessageDetail[] | 当前对话的消息列表 |
| loading | boolean | 是否正在加载对话内容 |
| generatingStatus | GenerationStatus \| null | 当前生成任务的状态 |

**GenerationStatus**:
```typescript
type GenerationStatus = {
  taskId: string;
  status: "submitted" | "polling" | "processing";
  message?: string;
}
```

| Event | Payload | Description |
|-------|---------|-------------|
| onCopyImage | resultId: string | 用户复制图片到剪贴板 |
| onDownloadImage | resultId: string | 用户下载图片 |

**UI 行为**:
- 用户消息靠右，系统响应靠左
- 图片以内联方式展示，点击可放大预览
- 图片上方显示右键菜单或悬浮操作按钮（复制/下载）
- 生成中时底部显示加载动画和状态文字
- 新消息自动滚动到底部
- 超过可视区域时支持虚拟滚动

---

### PromptInput (Text Input)

**职责**: 用户输入文字 Prompt 和提交。

| Property | Type | Description |
|----------|------|-------------|
| disabled | boolean | 是否禁用（配置缺失或正在生成时） |
| submitting | boolean | 是否正在提交 |

| Event | Payload | Description |
|-------|---------|-------------|
| onSubmit | prompt: string | 用户提交 Prompt |

**UI 行为**:
- 多行文本框，支持 Enter 提交、Shift+Enter 换行
- 提交按钮在 `disabled` 或 `submitting` 或输入为空时禁用
- 提交后自动清空输入框

---

### ImageUpload (Multi-image Upload with Ordering)

**职责**: 管理上传图片列表，支持多图上传、排序、删除、剪贴板粘贴。

| Property | Type | Description |
|----------|------|-------------|
| images | UploadedImage[] | 当前上传的图片列表 |
| disabled | boolean | 是否禁用 |
| maxCount | number | 最大上传数量（默认 10） |

**UploadedImage**:
```typescript
type UploadedImage = {
  id: string;
  filePath: string;
  thumbnailUrl: string;
  displayOrder: number;
  fileSize: number;
  source: "upload" | "clipboard";
}
```

| Event | Payload | Description |
|-------|---------|-------------|
| onAddImages | filePaths: string[] | 用户通过文件选择器添加图片 |
| onPasteImage | imageData: Blob | 用户通过 Ctrl+V 粘贴图片 |
| onReorder | fromIndex: number, toIndex: number | 用户拖拽调整图片顺序 |
| onRemoveImage | imageId: string | 用户移除某张图片 |

**UI 行为**:
- 图片以缩略图网格形式展示，带序号标记（1, 2, 3...）
- 支持拖拽排序（使用 @dnd-kit）
- 每张图片有删除按钮
- 超出 `maxCount` 时上传按钮禁用
- 文件选择器仅允许图片格式（png, jpg, webp, gif）
- 上传超过 10MB 的图片时提示错误

---

### ModeSelector (Operation Mode Toggle)

**职责**: 切换操作模式：文生图 / 图生图 / 图片翻译。

| Property | Type | Description |
|----------|------|-------------|
| currentMode | "text2img" \| "img2img" \| "translate" | 当前模式 |
| disabled | boolean | 是否禁用 |

| Event | Payload | Description |
|-------|---------|-------------|
| onModeChange | mode: string | 用户切换模式 |

**UI 行为**:
- 以分段按钮或下拉菜单形式展示
- 切换模式时，输入区域相应调整：
  - `text2img`: 仅显示文本输入框
  - `img2img`: 显示文本输入框 + 图片上传区域
  - `translate`: 显示图片上传（单张）+ 语言选择器

---

### LanguageSelector (Translation Language Picker)

**职责**: 选择图片翻译的源语言和目标语言。仅在 `translate` 模式下显示。

| Property | Type | Description |
|----------|------|-------------|
| sourceLang | string | 当前源语言 |
| targetLang | string | 当前目标语言 |

| Event | Payload | Description |
|-------|---------|-------------|
| onSourceLangChange | lang: string | 用户切换源语言 |
| onTargetLangChange | lang: string | 用户切换目标语言 |

**支持的语言**:
- zh (中文), en (英文), ja (日文), ko (韩文), fr (法文), de (德文), es (西班牙文), ru (俄文)

**UI 行为**:
- 两个下拉选择框 + 中间的交换按钮
- 源语言和目标语言不能相同
- 默认值: 源语言 "zh", 目标语言 "en"
