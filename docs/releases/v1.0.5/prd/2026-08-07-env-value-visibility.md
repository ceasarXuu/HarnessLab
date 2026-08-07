# PRD: Agent 详情页环境变量可见性开关

- Status: Ready for implementation
- Created: 2026-08-07
- Updated: 2026-08-07
- Owner / requester: 用户走查反馈（W1 系列新增需求）
- Source request: 在 Agent 详情页环境变量的 value 栏尾部增加可见/隐藏按钮，默认可见，点击隐藏/再点击显示，状态持久化且重启服务后保持，用于隐藏 API Key 等隐私信息。

## Requester Review Summary

- Key decisions:
  - 记忆存储在后端数据库（不随清浏览器数据/换浏览器失效）。
  - 不新增接口：在现有 Agent 环境变量配置数据接口（Agent DTO）中增加参数承载隐藏状态。
  - 隐藏为纯展示层脱敏（值仍可从页面源码/API 取回，点击可再查看）。
  - 隐藏状态按键名全局生效（隐藏 `OPENAI_API_KEY` 后，所有 Agent 中同名的该行都隐藏）。
  - 默认可见；不做"全部隐藏/全部显示"总开关。
  - 遮盖样式：完全 `••••••`（value 输入框在隐藏态以 `type=password` 显示为点号）。
  - 切换写入：复用现有 Agent 更新接口（`PATCH /agents/{id}`），携带当前 Agent 数据 + 新增 `hiddenEnvKeys` 参数。
  - 隐藏集合中的孤儿键名（无任何 Agent 引用）保留不清理。
- Important exceptions: 无
- Must-confirm before implementation: 无（已全部确认）
- Status reason: 产品决策与实现路径已确认，进入实现。

## 1. Background And Product Intent

Agent 配置中的环境变量常包含 API Key、Token 等敏感信息。当前 Agent 详情页以明文展示全部
`KEY=VALUE`，任何人经过屏幕都可能看到密钥。需要一个轻量的"隐藏/显示"开关，让用户能主动
遮盖敏感值，同时开关状态要可靠记忆，避免每次重启服务或换页面后需要重新设置。

## 2. Goals And Success Criteria

- 用户在 Agent 详情页能对每个环境变量 value 一键隐藏/显示。
- 隐藏状态下敏感值不可见（视觉上被遮盖），再次点击可恢复明文。
- 开关状态持久化在后端，重启 dev 服务、清浏览器数据或换浏览器后依然保持。
- 按键名全局生效，一处设置处处生效。

## 3. Users And Usage Context

- 本地 OrnnLab WebUI 用户（单机、单用户为主）。
- 场景：录入 API Key 后离开工位、或向他人展示页面时，不希望敏感值明文暴露。
- 用户能力：普通 Web 用户，无运维背景。

## 4. Scope

### In Scope

- Agent 详情页（只读视图）环境变量列表：每个 value 行尾增加可见/隐藏切换按钮。
- 隐藏/显示状态的后端持久化（按键名全局）。
- 现有 Agent 环境变量配置接口承载隐藏状态（不新增接口）。
- mock/MSW/Storybook 与真实 API 行为一致。

### Out Of Scope

- Agent 编辑（KeyValueControl）表单内的隐藏。
- 后端级值脱敏（值仍由 API 返回明文，隐藏仅作用于展示层）。
- 全局"全部隐藏/全部显示"总开关。
- 密钥加密存储、访问鉴权等安全加固。

## 5. Core User Journey

1. 打开 Agents 页，选中一个 Agent，进入详情抽屉。
2. 看到环境变量列表，每行 value 默认明文显示，行尾有"隐藏"按钮。
3. 点击"隐藏"→ 该行 value 变为遮盖文本，按钮变为"显示"。
4. 再次点击"显示"→ 恢复明文。
5. 重启 dev 服务或刷新浏览器后，之前设置为"隐藏"的行仍保持隐藏。

## 6. Interaction And Information Design

- 位置：Agent 详情抽屉（编辑态）环境变量列表每行的 value 单元格尾部（`KeyValueControl` 行内，删除按钮前）。
- 控件：眼睛图标按钮（lucide `Eye`/`EyeOff`），aria-label 为「隐藏/显示 <键名>」，点击即时切换，无需确认。
- 遮盖样式：value 输入框在隐藏态切换为 `type=password`（显示为 `••••••`），键名保持明文。
- 行为：点击隐藏 → 该键名加入全局隐藏集合 → 该行 value 变为点号，按钮变为「显示」；再次点击恢复。
- 隐藏集合按键名全局生效；任何 Agent 内切换都更新全局集合。
- value 为空或继承（无 `=`）的行不展示隐藏按钮。

## 7. Product Rules And State Logic

- 状态：`visible`（默认）↔ `hidden`，按 env key 名全局记录。
- 状态源：后端持久化的隐藏键名集合；前端在 Agent 详情加载时随环境变量配置数据一并取得。
- 切换写入：复用现有 Agent 环境变量配置接口（Agent 更新接口），携带新增的隐藏状态参数，
  后端更新全局隐藏集合。
- 任何 Agent 里对键名 X 的切换都会更新全局集合；所有 Agent 中键名为 X 的行立即同步生效
  （前端在下次加载 Agent 详情时应用）。
- 键名大小写敏感，与 env key 原文一致。

## 8. Edge Cases, Errors, And Recovery

- value 为空或为继承占位（`none`/`-`）的行：无敏感信息，不显示隐藏按钮或按钮置灰。
- 隐藏集合包含某个已不再存在于任何 Agent 的键名：保留（无副作用），也可在后续清理。
- 切换写入失败（后端不可用）：前端保留当前交互状态并提示，不静默失败。
- 新创建/新导入的 Agent 中出现同名键：继承全局隐藏状态，自动隐藏。
- 同一浏览器多页面同时打开：以最后一次切换为准（由后端全局状态保证一致）。

## 9. Content And Terminology

- 按钮文案（中/英）：`隐藏` / `Hidden`，`显示` / `Show`（或等效图标 + aria-label）。
- 遮盖字符：统一用 `••••••••` 或等价占位（待确认）。
- 不对 key 做任何遮盖。

## 10. Acceptance Criteria

- Given Agent 详情页有多个环境变量，when 用户点击某行 value 的"隐藏"，then 该行 value 变为遮盖文本且按钮变为"显示"。
- Given 某行处于隐藏，when 用户点击"显示"，then 该行恢复明文 value。
- Given 用户在 Agent A 隐藏了键名 X，when 打开含键名 X 的 Agent B 详情，then X 的行同样处于隐藏。
- Given 用户隐藏了某键名，when 重启 dev 服务/刷新页面，then 该键名相关行仍保持隐藏。
- Given 隐藏集合在数据库持久化，when 清浏览器数据或换浏览器访问，then 隐藏状态不变。
- Given value 为空/继承占位的行，when 渲染列表，then 不展示隐藏按钮或按钮置灰。
- mock 与真实 API 的隐藏状态读写语义一致，Storybook 覆盖 hidden/visible 状态。

## 11. Review Checklist And Sign-off Questions

已全部签署：1A（完全遮盖）、2A（复用 Agent 更新接口承载）、3A（孤儿键名保留）。

## 12. Clarification Decision Log

| Topic | Decision | Rationale | Source Round |
|---|---|---|---|
| 存储位置 | 后端数据库（不随清浏览器/换浏览器失效） | 用户明确要求持久记忆，本地数据目录为准 | Round 1 |
| 接口方式 | 不新增接口，在现有环境变量配置数据接口增加参数 | 用户明确约束，控制范围 | Round 1 |
| 隐藏强度 | 纯展示层脱敏（值仍可取回，点击再查看） | 目标是防被看到，非防抓取 | Round 1 |
| 隐藏粒度 | 按键名全局记忆 | 一次设置处处生效，语义一致 | Round 1 |
| 总开关 | 不需要 | 最小范围优先 | Round 1 |
| 默认状态 | 可见 | 用户原始需求 | Round 1 |
| 遮盖样式 | 完全 `••••••`（value 输入框 type=password） | 用户选择 1A | Round 2 |
| 写入路径 | 复用 Agent 更新接口，携带当前 Agent 数据 + `hiddenEnvKeys` | 用户选择 2A | Round 2 |
| 孤儿键名 | 保留不清理 | 用户选择 3A | Round 2 |

## 13. Open Questions And Risks

- 无待确认项。后续观察项：孤儿键名是否随时间积累（当前低风险）。

## 14. Implementation Notes

- 产品驱动约束：状态必须存储在后端（用户明确）；不新增 API 路由，扩展现有 Agent 环境变量
  配置 DTO；隐藏为前端展示层行为；状态按键名全局生效。
- 数据：复用现有 `webui_system_preferences`（key=`hidden_env_keys`，JSON 数组）全局存储隐藏键名，
  无需新增表与迁移。
- 接口：`AgentInput` 增加可选 `hiddenEnvKeys`（默认 None，未携带时不清空全局集合）；`AgentDto`
  始终返回当前全局隐藏键名集合；`_agent_dto` 写入 agent 配置前剥离 `hiddenEnvKeys`，避免污染
  `config_json`（隐藏状态是全局偏好，不属于单个 Agent 配置）。
- 前端：Agent 详情抽屉（编辑态 `KeyValueControl`）value 单元格尾部加眼睛按钮；隐藏态 value 输入框
  切换 `type=password`；`hiddenEnvKeys` 变更经既有 debounce autosave 随 `agentRowToDto` 一并提交。
- 需要覆盖 i18n（中/英）、mock/MSW、Storybook（hidden/visible 状态）、前后端回归测试。
