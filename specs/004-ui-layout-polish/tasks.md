# Tasks: UI Layout Polish

**Input**: Design documents from `/specs/004-ui-layout-polish/`  
**Prerequisites**: plan.md, spec.md, research.md, contracts/ui-contracts.md, quickstart.md

## Format: `[ID] [P?] [Story] Description`

- **[P]**: Can run in parallel (different files, no dependencies)
- **[Story]**: Which user story this task belongs to (US1, US2, US3, US4)
- Exact file paths included in descriptions

---

## Phase 1: Setup (Shared Infrastructure)

**Purpose**: Install shadcn/ui components and configure the foundation for all UI polish work

- [x] T001 运行 `npx shadcn add tooltip scroll-area` 安装 shadcn/ui 组件到 src/components/ui/，验证生成 src/components/ui/tooltip.tsx 和 src/components/ui/scroll-area.tsx 两个文件
- [x] T002 在 src/App.tsx 中导入 `TooltipProvider` 并在主布局外层包裹 `<TooltipProvider>`，确保所有子组件可使用 Tooltip

**Checkpoint**: shadcn/ui 组件就位，TooltipProvider 生效，`npx tsc --noEmit` 0 错误

---

## Phase 2: User Story 1 — Polished Chat Interface (Priority: P1) MVP

**Goal**: 聊天区域的消息气泡、图片结果、输入区域具备平衡的间距和比例，界面感觉经过精心设计而非原型状态

**Independent Test**: 打开应用，发送文字 prompt 并接收图片结果，验证消息气泡、图片、输入区域间距一致、视觉平衡，无拥挤或过于松散的区域

### Implementation for User Story 1

- [x] T003 [P] [US1] 更新 src/components/chat/ChatArea.tsx：将滚动容器的 `px-4 py-4` 改为 `px-4 py-6`；将消息间距 `gap-4` 改为 `gap-6`；将 loading 气泡的 `rounded-lg` 改为 `rounded-2xl`；用 `cn()` 替换所有模板字符串拼接的条件类名
- [x] T004 [P] [US1] 更新 src/components/chat/MessageBubble.tsx：将气泡 padding 从 `px-3 py-2` 改为 `px-4 py-3`；将气泡圆角从 `rounded-lg` 改为 `rounded-2xl`；将内容最大宽度从 `max-w-[75%]` 改为 `max-w-[80%]`；为头像添加 `shadow-sm`；将附件缩略图从 `h-16 w-16` 改为 `h-20 w-20`；将附件间距从 `gap-1.5` 改为 `gap-2`；将序号标记字体从 `text-[8px]` 改为 `text-xs`，尺寸从 `h-4 w-4` 改为 `h-5 w-5`；将结果间距从 `gap-2` 改为 `gap-3`；用 `cn()` 替换所有模板字符串拼接的条件类名
- [x] T005 [P] [US1] 更新 src/components/chat/ImageResult.tsx：将图片容器圆角从 `rounded-lg` 改为 `rounded-xl`；将最大高度从 `max-h-80` 改为 `max-h-96`；将 hover 工具栏位置从 `right-2 top-2` 改为 `right-3 top-3`，间距从 `gap-1` 改为 `gap-1.5`；将工具栏按钮圆角改为 `rounded-lg`；用 `cn()` 替换所有模板字符串拼接的条件类名
- [x] T006 [P] [US1] 更新 src/components/input/PromptInput.tsx：将容器间距从 `gap-2` 改为 `gap-3`；将 textarea 圆角从 `rounded-lg` 改为 `rounded-xl`；将 textarea padding 从 `px-3 py-2` 改为 `px-4 py-3`；将发送按钮圆角改为 `rounded-xl`；将最小高度从 40px 改为 44px（`min-h-11` 替代 inline style `minHeight: "40px"`）；用 `cn()` 替换所有模板字符串拼接的条件类名
- [x] T007 [P] [US1] 更新 src/components/input/ModeSelector.tsx：将容器圆角从 `rounded-lg` 改为 `rounded-xl`；将 tab 按钮圆角改为 `rounded-lg`；为活跃 tab 添加 `shadow-sm`；用 `cn()` 替换所有模板字符串拼接的条件类名
- [x] T008 [P] [US1] 更新 src/components/input/ModelSelector.tsx：将圆角从 `rounded-md` 改为 `rounded-lg`；用 `cn()` 替换所有模板字符串拼接的条件类名
- [x] T009 [P] [US1] 更新 src/components/input/ImageUpload.tsx：将缩略图尺寸从 `h-20 w-20` 改为 `h-24 w-24`；将缩略图圆角从 `rounded-lg` 改为 `rounded-xl`；将序号标记字体从 `text-[10px]` 改为 `text-xs`；将添加按钮尺寸改为 `h-24 w-24 rounded-xl`；将网格间距从 `gap-2` 改为 `gap-2.5`；用 `cn()` 替换所有模板字符串拼接的条件类名
- [x] T010 [P] [US1] 更新 src/components/input/LanguageSelector.tsx：将间距从 `gap-2` 改为 `gap-3`；将 select 圆角从 `rounded-md` 改为 `rounded-lg`；将 swap 按钮圆角改为 `rounded-lg`；用 `cn()` 替换所有模板字符串拼接的条件类名
- [x] T011 [US1] 更新 src/components/layout/MainPanel.tsx 的输入区域部分：将容器 padding 从 `py-3` 改为 `py-4`；将内部间距从 `space-y-2` 改为 `space-y-3`；将翻译按钮圆角改为 `rounded-xl`，高度改为 `py-2.5`；用 `cn()` 替换所有模板字符串拼接的条件类名

**Checkpoint**: 聊天区域所有组件已更新，消息气泡、图片、输入区域视觉一致且间距自然。运行 `npx tsc --noEmit` 0 错误。

---

## Phase 3: User Story 2 — Professional Sidebar Navigation (Priority: P2)

**Goal**: 侧边栏具备平滑的展开/收起动画、清晰的视觉层级、舒适的点击目标，并使用 shadcn/ui 的 Tooltip 和 ScrollArea 提升体验

**Independent Test**: 展开/收起侧边栏验证平滑过渡动画；创建新会话并切换，验证 hover/active 状态清晰；鼠标悬停在图标按钮上验证 Tooltip 显示；滚动会话列表验证自定义滚动条

- [x] T012 [US2] 重构 src/components/layout/Sidebar.tsx：将当前两个条件分支（collapsed/expanded 各自独立 `<div>` 树）合并为单个持久 `<div>` 容器，使用 `cn()` 动态切换 `w-12` / `w-64` 类名。外层容器添加 `transition-[width] duration-300 ease-in-out overflow-hidden`。收起状态下文字元素使用 `opacity-0 w-0 overflow-hidden` 隐藏；展开状态下文字使用 `animate-in fade-in duration-200` 渐入
- [x] T013 [US2] 在 src/components/layout/Sidebar.tsx 中将所有 `title="..."` 属性替换为 shadcn `<Tooltip>` 组件（从 `@/components/ui/tooltip` 导入 `Tooltip, TooltipTrigger, TooltipContent`）：包括展开/收起按钮、新建会话按钮、删除按钮的 tooltip
- [x] T014 [US2] 在 src/components/layout/Sidebar.tsx 中将会话列表的 `overflow-y-auto` 替换为 `<ScrollArea>` 组件（从 `@/components/ui/scroll-area` 导入）：用 `<ScrollArea className="flex-1">` 包裹会话列表
- [x] T015 [US2] 在 src/components/layout/Sidebar.tsx 中更新间距和样式：header padding 改为 `px-4 py-3`；会话项 padding 改为 `px-3 py-2.5 mx-2`，圆角改为 `rounded-lg`；添加 `truncate` 类到会话标题以处理长标题截断；确保删除按钮的 `transition-opacity duration-150` hover 效果正常

**Checkpoint**: 侧边栏展开/收起有平滑动画，Tooltip 和 ScrollArea 正常工作。运行 `npx tsc --noEmit` 0 错误。

---

## Phase 4: User Story 3 — Cohesive Welcome Screen (Priority: P3)

**Goal**: 欢迎页面的功能卡片视觉上吸引人、文字可读、间距平衡，给人精心设计的感觉

**Independent Test**: 打开应用（无活跃会话），验证欢迎页功能卡片间距平衡、文字可读、整体视觉和谐

- [x] T016 [US3] 更新 src/components/layout/MainPanel.tsx 的欢迎页面部分：将容器间距从 `gap-4` 改为 `gap-6`；将副标题 `mt-2` 改为 `mt-3`；将卡片网格从 `max-w-lg grid-cols-3 gap-3` 改为 `max-w-xl grid-cols-3 gap-5`；将卡片 padding 从 `p-3` 改为 `p-5`；将卡片圆角从 `rounded-lg` 改为 `rounded-xl`；将卡片标题字体从 `text-xs` 改为 `text-sm font-medium`；将卡片描述字体从 `text-[10px]` 改为 `text-xs text-muted-foreground mt-1.5`

**Checkpoint**: 欢迎页面卡片视觉清晰、文字可读。运行 `npx tsc --noEmit` 0 错误。

---

## Phase 5: User Story 4 — Consistent Error and Status Feedback (Priority: P3)

**Goal**: 错误提示使用设计系统的 destructive 颜色令牌，而非硬编码红色，确保与界面其他部分视觉一致

**Independent Test**: 触发错误条件（如无效 API key），验证错误显示使用与界面一致的颜色方案

- [x] T017 [US4] 更新 src/components/common/ErrorDisplay.tsx：将 `border-red-200 bg-red-50 text-red-800` 替换为 `border-destructive/20 bg-destructive/5 text-foreground`；将 `dark:border-red-800 dark:bg-red-950 dark:text-red-200` 全部移除（design tokens 自动处理暗黑模式）；将错误详情文本 `text-red-600 dark:text-red-400` 替换为 `text-destructive/80`；将图标颜色统一使用 `text-destructive`；将圆角从 `rounded-lg` 改为 `rounded-xl`；用 `cn()` 替换所有模板字符串拼接的条件类名

**Checkpoint**: 错误显示使用 destructive 令牌，暗黑模式自动适配。运行 `npx tsc --noEmit` 0 错误。

---

## Phase 6: Polish & Cross-Cutting Concerns

**Purpose**: 最终验证和收尾

- [x] T018 运行 `npx tsc --noEmit` 确认 TypeScript 编译 0 错误
- [x] T019 运行 `npx vite build` 确认前端构建成功
- [ ] T020 在 index.html 的 `<html>` 标签临时添加 `class="dark"` 后运行 `cargo tauri dev`，验证所有修改在暗黑模式下视觉正确，然后移除 `class="dark"`
- [x] T021 运行 `cargo clippy` 确认 Rust 代码无警告（虽然本功能不修改 Rust 代码，但确保不影响编译）

---

## Dependencies & Execution Order

### Phase Dependencies

- **Phase 1 (Setup)**: 无依赖 — 立即开始
- **Phase 2 (US1)**: 依赖 Phase 1 完成（需要 TooltipProvider 就位，但 US1 不直接使用 Tooltip）
- **Phase 3 (US2)**: 依赖 Phase 1 完成（需要 Tooltip 和 ScrollArea 组件）
- **Phase 4 (US3)**: 依赖 Phase 1 完成
- **Phase 5 (US4)**: 依赖 Phase 1 完成
- **Phase 6 (Polish)**: 依赖所有前面阶段完成

### User Story Dependencies

- **US1 (P1)**: Phase 1 完成后即可开始，不依赖其他故事
- **US2 (P2)**: Phase 1 完成后即可开始，不依赖 US1（但建议按顺序）
- **US3 (P3)**: Phase 1 完成后即可开始，不依赖 US1/US2
- **US4 (P3)**: Phase 1 完成后即可开始，不依赖其他故事

### Parallel Opportunities

- Phase 2 中 T003-T010 均标记 [P]，可全部并行执行（不同文件）
- Phase 2 的 T011 与 Phase 4 的 T016 修改同一文件 (MainPanel.tsx)，但改不同区域，需顺序执行
- Phase 3 (US2) 和 Phase 4 (US3) 可并行执行（不同文件）
- Phase 5 (US4) 可与 Phase 3/4 并行执行

---

## Parallel Example: User Story 1

```bash
# 以下 8 个任务可同时启动（全部标记 [P]，修改不同文件）:
Task: "T003 更新 ChatArea.tsx"
Task: "T004 更新 MessageBubble.tsx"
Task: "T005 更新 ImageResult.tsx"
Task: "T006 更新 PromptInput.tsx"
Task: "T007 更新 ModeSelector.tsx"
Task: "T008 更新 ModelSelector.tsx"
Task: "T009 更新 ImageUpload.tsx"
Task: "T010 更新 LanguageSelector.tsx"

# T011 修改 MainPanel.tsx 输入区域，与上述不冲突但与 T016 冲突（同文件），需顺序执行
```

---

## Implementation Strategy

### MVP First (User Story 1 Only)

1. Complete Phase 1: Setup (T001-T002)
2. Complete Phase 2: User Story 1 (T003-T011)
3. **STOP and VALIDATE**: 运行 `cargo tauri dev`，验证聊天区域视觉已显著改善
4. 如果 MVP 满意，可部署

### Incremental Delivery

1. Phase 1 (Setup) → shadcn/ui 就位
2. Phase 2 (US1) → 聊天区域已 polish → 验证 → 最核心的改善已完成
3. Phase 3 (US2) → 侧边栏已 polish → 验证 → 导航体验提升
4. Phase 4 + 5 (US3 + US4) → 欢迎页 + 错误提示已 polish → 验证 → 全面完善
5. Phase 6 (Polish) → 最终验证 → 完成

---

## Notes

- 本功能 **不修改任何 Rust 后端代码**，所有改动仅限前端 TypeScript/TSX 文件
- 所有组件的 `cn()` 迁移应在该组件的任务中同步完成，不作为独立任务
- 每个任务完成后建议运行 `npx tsc --noEmit` 快速验证无类型错误
- 所有 Tailwind 类名变更均参考 `contracts/ui-contracts.md` 中定义的目标值
