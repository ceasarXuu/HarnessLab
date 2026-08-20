# Case: Agent Skills 路径未完整显示、未展示解析出的 skills

## Problem P-001

- 症状：用户在 Agent 配置 Skills 标签"选择文件夹"后，详情页（AgentDetail →
  AgentProfileEditor skills 标签）只显示文件夹名（basename），不显示完整绝对
  路径；且从不展示"解析出来的 skills"（文件夹内 SKILL.md 列表）。
- 期望：选择文件夹后保存并回到详情页，应能看到完整本地路径；并希望能看到该
  路径下被识别出的 skill 列表。
- 影响：Agent 配置 customer-visible 工作流；当前经"选择文件夹"登记到的 skill
  源只有文件夹名（如 `my-skills`），不含绝对前缀，Harbor 运行时无法据此定位
  本地 skill 目录。
- 复现路径：Agent 编辑页 → Skills 标签 → 点击 "Choose folder" → pick 本地
  目录 → 保存 → 返回/查看详情页。
- 已知事实：
  - `DirectoryListControl` 用 `webKitDirectory` 文件 input 选目录
    （AgentProfileEditor.tsx:362-394）。
  - `getSelectedDirectoryPath`（AgentProfileEditor.tsx:413-419）依赖
    `file.path`（非标准，仅 Electron/Node 有，浏览器为 undefined）与
    `webkitRelativePath`。
  - 测试 `AgentProfileEditor.test.tsx:55` 明确断言 Skills 源"只能"用文件夹选择
    添加（`showAddAction={false}`，不允许手动文本新增/编辑）。
  - 后端只存路径字符串：`agent_harbor_config` 把
    `agent["skillSources"]` 原样放进 Harbor `AgentConfig.skills`
    （webui_profile_service.py:119；harbor_engine.py:282/287）。
  - Harbor `AgentConfig.skills` 字段语义：list[str|Path]，支持本地路径 / git
    URL / org/name[@ref]，运行时由 Harbor 解析（0.21 已确认）。
  - 全仓无任何"读取文件夹内容 / 枚举 SKILL.md / 展示解析 skill 列表"的代码。
- 待排除方向：详情页 read-only 渲染是否单独截断路径（已排除：详情页同样用
  AgentProfileEditor，非 readOnly，渲染 EditableStringList 原样显示 value）。

## Hypothesis H-001

- 声明：经浏览器文件选择器选目录时，只能拿到 `webkitRelativePath`（相对路径，
  首段即文件夹名），拿不到绝对路径（`file.path` 在浏览器为 undefined）。
  `getSelectedDirectoryPath` 因此走 `return folderName`，只存文件夹名而非绝对
  路径 → 详情页"路径不完整"。
- 预测：浏览器环境（Vite preview / Codex Web Preview）下，`file.path` 为
  undefined，`fullPath.endsWith(relativePath)` 为 false，函数返回 basename。
- 预测（反证）：若浏览器能拿到绝对路径，则 `fullPath.endsWith(relativePath)`
  为 true，函数返回完整路径。但浏览器安全模型禁止暴露绝对路径，故不成立。
- 诊断证据计划：用 Node 复现 `getSelectedDirectoryPath`，构造仅含
  `webkitRelativePath`、无 `path` 的伪 File，断言返回值为文件夹名而非绝对路径。

## Hypothesis H-002

- 声明："展示解析出来的 skills"是一项从未实现的能力：OrnnLab 只把路径字符串
  传给 Harbor，从不读取/枚举所选文件夹内的 SKILL.md。
- 预测：前端无任何解析 skill 目录的组件/请求；后端无对应 API。
- 诊断证据计划：全仓 grep `SKILL.md` / skill parse / list skills，确认无相关
  实现（已确认无）。
- 结论：这不是"解析失败"，而是"没有做解析"——与用户提问"是不是没有做这个
  解析功能"一致。

## Evidence

- E-001（代码路径，支持 H-001/H-002）：AgentProfileEditor.tsx:362-394
  `DirectoryListControl` 使用 `webKitDirectory` input；:413-419
  `getSelectedDirectoryPath` 依赖 `file.path`。
- E-002（测试，支持 H-001 设计意图）：AgentProfileEditor.test.tsx:55-62 断言
  Skills 源仅能通过"Choose folder"添加，隐藏 "Add" 按钮 → 用户无法手动输入
  绝对路径。
- E-003（数据流，支持 H-002）：后端不读文件夹内容，原样传路径给 Harbor
  （webui_profile_service.py:107-147；harbor_engine.py:282-288）。
- E-004（复现，支持 H-001）：Node 复现 `getSelectedDirectoryPath`——
  浏览器式 File（无 path）返回 `"my-skills"`（basename）；Node 式 File（带
  path）返回完整绝对路径。`frontend` 运行于浏览器，故只拿 basename。
- E-005（全仓 grep，支持 H-002）：无 `SKILL.md` 解析 / list-skills 实现。

## 状态更新

- H-001：`confirmed`（E-001 + E-002 + E-004）。浏览器安全模型禁止文件 input
  暴露绝对路径，`getSelectedDirectoryPath` 回退到文件夹名；且 `showAddAction=
  false` 禁止手动输入，用户无法补绝对路径。
- H-002：`confirmed`（E-003 + E-005）。"展示解析出的 skills"从未实现，OrnnLab
  仅把路径字符串传给 Harbor。
- P-001：根因已确认，待用户确认修复方向后进入 Repair Design。
