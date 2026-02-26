---
description: 使用计划模板执行实现规划工作流，生成设计文档。
handoffs: 
  - label: 创建任务
    agent: speckit.tasks
    prompt: 将计划拆分为任务
    send: true
  - label: 创建检查清单
    agent: speckit.checklist
    prompt: 为以下领域创建检查清单...
---

## 用户输入

```text
$ARGUMENTS
```

在继续之前**必须**考虑用户输入（如果非空）。

## 大纲

1. **设置**: 从仓库根目录运行 `.specify/scripts/powershell/setup-plan.ps1 -Json` 并解析 JSON 获取 FEATURE_SPEC、IMPL_PLAN、SPECS_DIR、BRANCH。对于参数中的单引号（如 "I'm Groot"），请使用转义语法：例如 'I'\''m Groot'（或者如果可以的话使用双引号："I'm Groot"）。

2. **加载上下文**: 读取 FEATURE_SPEC 和 `.specify/memory/constitution.md`。加载 IMPL_PLAN 模板（已复制）。

3. **执行计划工作流**: 按照 IMPL_PLAN 模板的结构执行：
   - 填写技术上下文（将未知项标记为"需要澄清"）
   - 从章程填写章程检查部分
   - 评估门控条件（如果违规无正当理由则报错）
   - 阶段 0：生成 research.md（解决所有需要澄清的问题）
   - 阶段 1：生成 data-model.md、contracts/、quickstart.md
   - 阶段 1：通过运行代理脚本更新代理上下文
   - 设计后重新评估章程检查

4. **停止并报告**: 命令在阶段 2 规划后结束。报告分支、IMPL_PLAN 路径和生成的文档。

## 阶段

### 阶段 0：概要与研究

1. **从上述技术上下文中提取未知项**:
   - 对于每个需要澄清的项 → 研究任务
   - 对于每个依赖项 → 最佳实践任务
   - 对于每个集成 → 模式任务

2. **生成并分派研究代理**:

   ```text
   对于技术上下文中的每个未知项:
     任务: "为 {功能上下文} 研究 {未知项}"
   对于每个技术选择:
     任务: "在 {领域} 中查找 {技术} 的最佳实践"
   ```

3. **整合 findings** 到 `research.md`，使用以下格式：
   - 决策: [选择了什么]
   - 理由: [为什么选择]
   - 考虑的替代方案: [还评估了什么]

**输出**: research.md，包含所有已解决的需要澄清项

### 阶段 1：设计与契约

**前置条件:** `research.md` 完成

1. **从功能规范中提取实体** → `data-model.md`:
   - 实体名称、字段、关系
   - 来自需求的验证规则
   - 状态转换（如果适用）

2. **定义接口契约**（如果项目有外部接口）→ `/contracts/`:
   - 确定项目向用户或其他系统暴露的接口
   - 记录适合项目类型的契约格式
   - 示例：库的公共 API、CLI 工具的命令模式、Web 服务的端点、解析器的语法、应用程序的 UI 契约
   - 如果项目是纯内部的则跳过（构建脚本、一次性工具等）

3. **代理上下文更新**:
   - 运行 `.specify/scripts/powershell/update-agent-context.ps1 -AgentType opencode`
   - 这些脚本检测正在使用的 AI 代理
   - 更新相应的代理特定上下文文件
   - 仅添加当前计划中的新技术
   - 保留标记之间的手动添加内容

**输出**: data-model.md, /contracts/*, quickstart.md, 代理特定文件

## 关键规则

- 使用绝对路径
- 门控失败或未解决的澄清项时报错
