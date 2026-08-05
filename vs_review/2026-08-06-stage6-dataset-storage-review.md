# Subagent VS Review: Stage 6 Dataset Storage Location Management

- Created: 2026-08-06T09:30:00+08:00
- Updated: 2026-08-06T09:30:00+08:00
- Report schema: adversarial-v1
- Task: 完成 v1.0.5 Stage 6（Dataset 存储位置管理）的独立对抗性审查，确认 S6-01~S6-06 无阻断项后允许将 Stage 6 标记为 Done
- Report path: `vs_review/2026-08-06-stage6-dataset-storage-review.md`
- Review mode: fresh internal subagents
- Source session policy: no inherited main-agent context
- Status: open

## Round 1: Stage 6 实现完整性、文件边界与失败路径对抗审查

### Review Input

#### Objective

用户/产品目标：让用户选择任意本地父目录下载 Harbor registry Dataset，由 OrnnLab 持久化管理唯一副本及其可用性，并保证导入、移动、重定位、删除的边界安全。本轮审查用于支撑 v1.0.5 工程计划 Stage 6（S6-01~S6-05 声称 Done，S6-06 要求独立对抗性审查）的完成判定。

#### Review Target

Stage 6 后端与前端实现：路径契约与持久化、registry 下载与取消、迁移/重定位/删除边界、本地导入边界、前端与 mock 对等。

#### Target Locations

- `ornnlab/services/webui_dataset_service.py`
- `ornnlab/services/dataset_download_state.py`
- `ornnlab/services/dataset_environment.py`
- `ornnlab/services/dataset_task_catalog.py`
- `ornnlab/services/container_image_platforms.py`
- `ornnlab/models/webui.py`（Dataset 相关输入模型）
- `ornnlab/api/webui.py`（`/datasets` 路由）
- `ornnlab/storage/migrations/005_dataset_storage_locations.sql`
- `tests/python/test_webui_dataset_storage.py`
- `tests/python/test_webui_api.py`
- `frontend/src/api/mockOperations.ts`、`mockQueries.ts`、`mockMappers.ts`
- `frontend/src/screens/DatasetsPage.tsx`
- `frontend/src/ui/components/DatasetDetail.tsx`、`DirectoryListControl.tsx`、`FolderPathInput.tsx`
- `frontend/src/ui/datasetSelectOptions.ts`
- `frontend/src/mocks/mswHandlers.ts`
- `frontend/src/**/*.stories.tsx`（Dataset 相关 Storybook）
- `docs/releases/v1.0.5/technical-design.md`（4.1 数据与契约边界）
- `docs/releases/v1.0.5/engineering-plan.md`（Stage 6 验收矩阵）

#### Change Introduction

v1.0.5 Stage 6 实现了 Dataset 存储位置管理：registry Dataset 可下载到用户选择的任意本地父目录下的唯一标记子目录；OrnnLab 持久化 `storage_kind`（managed/external）、父目录偏好、路径可用状态与下载记录；managed 支持异步移动、路径丢失后可重新定位或移除登记、存在目录只能删除；external 本地导入仅登记与加载，不得删除用户原始目录，删除请求被 API 拒绝；前端 API/mock/MSW 使用同一 DTO、Operation 与错误语义，路径控件经本机原生目录选择器回填只读路径，Storybook 覆盖 managed/external/path-unavailable 状态。

#### Risk Focus

- 用户选择父目录含恶意/异常路径时，标记子目录与删除/取消清理是否可能越界（路径穿越、符号链接、共享父目录误删）。
- 取消/失败清理是否只触碰 OrnnLab 标记的临时目录；同名冲突是否被正确拒绝。
- 移动/重定位/删除与并发 Operation（下载中、移动中）之间的竞态。
- external 导入、重定位、移除登记是否在任何分支下删除用户原始目录。
- `storage_kind` 迁移对既有 local 记录是否正确（005 迁移），旧数据升级后行为是否改变。
- 路径可用状态（path-unavailable）的判定与恢复路径是否真实可用，而非前端假状态。
- API/mock/MSW 三者的 Dataset 写操作语义是否对等（尤其删除拒绝、取消下载）。

#### User-Perspective Review Focus

- 用户能否在 UI 中理解并完成「选择父目录 → 下载 → 查看路径/大小 → 移动/重定位/删除」的完整流程。
- 路径控件回填只读、错误信息（同名冲突、路径不可用、外部目录不可删除）是否可理解。
- 下载进度（Operation 轮询）在刷新/重挂载后是否可恢复显示。

#### Implementation Completeness Focus

- S6-01~S6-05 每个验收项：生产代码路径、集成入口、测试证据、运行时/日志证据、是否存在 mock-only / 测试-only 布线。
- 前端路径选择是否真正调用本机原生目录选择器并回填，还是只有展示层假交互。

#### Target Benefit Focus

- 声称收益：「持久化管理唯一副本及其可用性」。请检查是否有基线、目标、测量方法与对比证据；缺失即记录为非阻断 warning，除非同一证据同时证明正确性/安全/数据/可靠性/运维失败。

#### Assumptions To Attack

- 用户选择的父目录可能位于其他卷、只读、含空格/Unicode、已存在同名 Dataset 目录、是符号链接或挂载点。
- Dataset ref 可能包含特殊字符；`dataset_ref:path` 路由可携带 `/`。
- 下载取消可能与完成、移动、删除并发发生。
- 旧库升级前存在 `source='local'` 记录，迁移后的 `storage_kind` 假设可能不成立。
- 前端刷新期间 Operation 状态可能丢失。

#### Adversarial Lenses

requirements | state | input | concurrency | failure | data | security | usability | implementation-completeness | testing | observability

#### Verification Status

- 已执行：`uv run pytest tests/python -q` → 187 passed / 4 skipped；前端 `npm run test` → 32 files / 117 tests；typecheck、lint 通过。
- 已知缺口：S6-06 独立对抗性审查未执行；本次即该审查。

#### Reviewer Instructions

- 全新内部 subagent 会话，不继承主 agent 上下文。
- 直接读取目标文件与文档，不要修改任何文件（只读审查）。
- 以对抗姿态尝试推翻 S6-01~S6-05 的 Done 结论与文件/数据边界安全假设。
- 输出按 `references/review-report-template.md` 的结构（Summary / Blocking Findings / Non-blocking Risks / User-Perspective Checks / Implementation Completeness Checks / Target Benefit Checks / Required Fixes / Missing Tests / Missing Logs / Evidence），并尽量给 `path:line` 证据。
- Blocking 结论必须给出可复现反例（broken assumption / failure scenario / trigger / impact / proof needed）。

### Internal Subagent Unavailable Fallback

- 内部 subagent 不可用原因：n/a
- 本地 CLI 发现命令：
  - `command -v claude`
  - `command -v claude-code`
  - `command -v codex`
  - `command -v codex-cli`
  - `command -v opencode`
  - `command -v pi`
- 发现候选：n/a（内部 subagent 可用）
- 用户推荐替代 agent：n/a
- 用户批准命令：n/a
- Fallback outcome：n/a

### Reviewer Timeout Policy

| Complexity | Initial Wait | Extension | Max Attempts Per Role | Blocking Closure Behavior |
|---|---:|---:|---:|---|
| complex | 15 min | 1 次，约 10 min | 2 | 审查不可用时不得判通过 |

### Review Budget Policy（用户约束，2026-08-06）

- 审查轮次上限：2 轮（Round 1 初始审查；如产生被接受的阻断项，Round 2 为闭环复审）。
- 超过 2 轮仍未收敛（Round 2 仍有阻断项、审查不可用或需要继续迭代）时：视为可能存在严重设计缺陷或约束问题；停止推进，总结情况与反思，等待用户指示；不得把 Stage 6 标记为 Done。
- Round 1 无阻断项时视为收敛，无需 Round 2。

### Reviewer Selection

| Reviewer | Reason Selected | Risk Area |
|---|---|---|
| implementation-adversary | Stage 6 完成判定取决于文件/状态机正确性、失败路径与并发竞态；需对抗验证 S6-01~S6-05 生产路径与边界安全 | 文件系统边界、并发、部分成功、数据一致性、mock/API 对等 |

### Reviewer Launch Records

| Reviewer | Internal Mechanism | Session / Job ID | Trace Source | Context Forked | Input Packet | Context Explicitly Excluded | Read-only |
|---|---|---|---|---|---|---|---|
| implementation-adversary | spawn_agent（fresh） | /root/stage6_review | 本轮 spawn 调用（2026-08-06T09:31+08:00） | fork_turns=none | Round 1 Review Input | 主 agent 历史、推理、结论、完整 diff | yes |
| implementation-adversary（替补） | spawn_agent（fresh） | /root/stage6_review/stage6_review_r2 | 本轮 spawn 调用（2026-08-06T03:35+00:00） | fork_turns=none | Round 1 Review Input | 主 agent 历史、推理、结论、完整 diff | yes |

### Reviewer Timeout Records

| Reviewer Output Key | Reviewer Role | Attempt | Session / Job ID | Waited | Status | Reason | Action |
|---|---:|---|---:|---|---:|---|
| round1-impl | implementation-adversary | 1 | /root/stage6_review | 15 min（首轮）+ 8 min（恢复后二次等待） | lost | 首轮 15 min 无输出；恢复会话后追加等待 8 min 并对状态询问无响应，视为失联 | replacement spawned |
| round1-impl-r2 | implementation-adversary | 2 | /root/stage6_review/stage6_review_r2 | 15 min + 10 min（延长） | lost | 首发 15 min 无输出；延长 10 min 后仍无输出，状态询问无响应，视为失联 | user decision required |

### User Decision After Failed Review

- 主审查与替补审查均失败（无输出、状态询问无响应），按技能硬规则不得以「无发现」通过，也不得伪造独立审查完成。
- Decision: 待用户选择 retry / narrow scope / change reviewer type / accept risk（含主 agent 独立对抗审查作为备选）

### Reviewer Outputs

Round 1 无 reviewer 输出：首发 `/root/stage6_review` 与替补 `/root/stage6_review/stage6_review_r2` 均未返回审查结论（详见 Timeout Records）。

替补会话在父代理声明失联后仍长时间运行（无输出），主 agent 于 2026-08-06T10:20+08:00 将其终止；如后续出现任何迟到输出，一律按 late result 处理，不得覆盖本轮的失联记录。

### Main Agent Response

无 reviewer 输出可供分级（没有发现项，也没有 PASS/FAIL 结论）。

| Reviewer | Finding | Broken Assumption / Failure Scenario | Severity | Decision | Evidence / Reason | Action Taken | Follow-up |
|---|---|---|---|---|---|---|---|
| implementation-adversary（首发） | 无输出（失联） | 审查基础设施未交付结论 | n/a | n/a | Timeout Records：15 min + 8 min 无输出，状态询问无响应 | 启动替补；按技能规则不判通过 | 记录 user decision required |
| implementation-adversary（替补） | 无输出（失联） | 审查基础设施未交付结论 | n/a | n/a | Timeout Records：15 min + 10 min 延长仍无输出 | 终止会话；不判通过 | 记录 user decision required |

主 agent 结论：Round 1 未产生审查结论。依据用户 2026-08-06 审查预算约束（最多 2 轮，未收敛即停止并等待指示）与 subagent-vs-review 硬规则（审查不可用 ≠ 通过、≠ 无发现），不得把 Stage 6 标记为 Done，S6-06 保持 In progress。

旁证（非独立审查，主 agent 侧抽查确认，供用户决策参考）：

- `ornnlab/services/webui_dataset_service.py:454` `_managed_directory_name`、`:465` `_assert_managed_directory`、`:27` 标记文件 `.ornnlab-dataset.json`。
- `ornnlab/services/webui_dataset_service.py:246` external 删除被 API 拒绝（"external Dataset files cannot be deleted by OrnnLab"）。
- `ornnlab/storage/migrations/005_dataset_storage_locations.sql` 将 `source='local'` 迁移为 `storage_kind='external'`。
- 原生目录选择器属实：`ornnlab/services/webui_system_service.py:148,232,241,256-257`（osascript / PowerShell / zenity / kdialog）；`frontend/src/api/webUiClient.ts:100` 经 `/system/directory-picker` 调用。

### Closure Status

- Blocking findings found: 未完成（两轮 fresh subagent 审查均失联，无 reviewer 输出）
- Accepted blocking findings fixed: n/a
- Blocking re-review completed: n/a
- Blocking re-review passed: n/a
- Rejected findings backed by evidence: n/a
- Deferred findings documented: n/a
- Implementation completeness gaps resolved or accepted by user: n/a（无审查结论）
- Target benefit warnings recorded: n/a
- Blocked reason: 首发与替补 implementation-adversary 均无输出且对状态询问无响应；按 subagent-vs-review 硬规则不得判通过
- Allowed to proceed: no（待用户决策）

## Final Conclusion

Stage 6 的 S6-06 独立对抗性审查在预算内未收敛：两个 fresh internal subagent 会话（/root/stage6_review 与 /root/stage6_review/stage6_review_r2）均未返回审查输出，且对状态询问无响应。依据 subagent-vs-review 技能硬规则（审查不可用 ≠ 通过、≠ 无发现）与用户 2 轮预算约束，审查未收敛即停止推进：S6-06 保持 In progress，Stage 6 不得标记 Done，等待用户决策（重试 / 缩小范围 / 更换审查类型 / 主 agent 内非 fresh 对抗审查（如实标注） / 显式接受风险）。

## Appendix A: Subagent 可用性探针（2026-08-06）

用户要求先确认「能否正确创建可用的 subagent」。主 agent 做了三组最小探针：

| 探针 | fork_turns | 任务 | 结果 |
|---|---|---|---|
| /root/subagent_probe | none | 执行 pwd / git log -1 / python --version | 未执行，返回「我准备好了，请告诉我任务」；follow-up 补发任务后仍无响应 |
| /root/subagent_probe2 | none | 同上（措辞改为 USER TASK） | 未执行，返回同类就绪问候 |
| /root/subagent_probe3 | 1 | 同上 | 完整执行：probe3 → probe_v2 → selfcheck 三级链路均收到任务、执行命令并回传结果 |

结论：

- subagent 创建与通信机制本身可用，但**fresh 派生（fork_turns=none）下初始任务消息投递不稳定**：连续两次探针均未识别任务，只返回就绪问候；带少量上下文（fork_turns=1）的探针完整跑通。
- 这与 Stage 6 审查失败的现象一致：审查者（/root/stage6_review）收到任务但长时间无输出，其 fresh 替补（stage6_review_r2）失联。
- 对独立审查的影响：subagent-vs-review 要求 fresh 会话以避免上下文污染，但当前运行时 fresh 会话的任务投递不可靠，导致「独立 fresh 审查」这一前提难以稳定满足；重试同方式预期会复现失联。
- 后续若继续 subagent 审查，需要用户决策：接受带上下文派生（弱化 fresh 隔离并如实标注）、改为 spawn 后显式补发任务、或改用主 agent 内非 fresh 对抗审查 / 显式接受风险。
