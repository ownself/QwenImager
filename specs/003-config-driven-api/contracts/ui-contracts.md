# UI Contracts: Config-Driven API

**Feature Branch**: `003-config-driven-api`  
**Date**: 2026-02-27

## 概述

前端新增模型选择器组件，扩展 Store 状态管理，其余 UI 不变。

## 新增组件

### `ModelSelector`

**位置**: `src/components/input/ModelSelector.tsx`

**功能**: 当同一 `service_type` 下有多个可用模型时，显示下拉选择器让用户选择。

**Props**:
```typescript
interface ModelSelectorProps {
  serviceType: "text2img" | "img2img" | "translate";
}
```

**行为**:
- 从 `configStore.availableModels` 中筛选匹配当前 `serviceType` 的模型
- 如果只有一个模型，不显示选择器（自动选择）
- 如果有多个模型，显示 `<Select>` 下拉框，格式为 `"{provider} / {name}"`
- 选择变更时更新 `conversationStore.selectedModel`

**位置**: 嵌入 `MainPanel.tsx` 的 Prompt 输入区域上方

---

## Store 变更

### `configStore` 扩展

```typescript
interface ConfigState {
  // 已有
  loading: boolean;
  configLoaded: boolean;
  configError: string | null;
  
  // 变更：从 string[] 改为 ModelInfo[]
  availableModels: ModelInfo[];
  
  // 新增
  getModelsByType: (type: ServiceType) => ModelInfo[];
}
```

### `conversationStore` 扩展

```typescript
interface ConversationState {
  // 已有字段不变...
  
  // 新增
  selectedModel: string | null;        // 当前选中的模型名称
  setSelectedModel: (name: string | null) => void;
}
```

**行为**:
- `selectedModel` 切换 mode 时自动重置为 null（使用默认模型）
- `sendPrompt` 时将 `selectedModel` 传递给后端命令的 `model_name` 参数

---

## 不变的组件

| 组件 | 说明 |
|------|------|
| `ModeSelector` | text2img/img2img/translate 三模式切换，不变 |
| `PromptInput` | 文本输入，不变 |
| `ImageUpload` | 图片上传，不变 |
| `LanguageSelector` | 语言选择，不变 |
| `ChatArea` / `MessageBubble` / `ImageResult` | 聊天展示，不变 |
| `Sidebar` | 对话列表，不变 |
| `ConfigGuide` | 配置引导，不变 |
