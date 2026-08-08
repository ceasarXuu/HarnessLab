# 工程计划：Job / Task 状态机重梳理与落地

- 状态：Active（立项）
- 更新：2026-08-09
- 来源：v1.0.5 走查定义冲突（见 `../first-round-walkthrough/issues.md` 相关记录）
- 方法：se-good-plan（最小闭环工作单元；证据优先于计划）

## 1. 问题定义

### 当前行为（事实）

| 位置 | 现状 |
|---|---|
| `ornnlab/services/harbor_subprocess.py:287-311` `_status_from_result_payload` | `n_errored_trials>0 → "failed"`；`completed>=total → "completed"`。即"跑完但有异常 trial"被映射为 failed |
| `ornnlab/services/webui_job_service.py:421` `_job_dto` | `status = runs.status`（DB），直接透出 |
| `ornnlab/services/recovery_service.py:16` | `TERMINAL_STATUSES = {completed, failed, cancelled, interrupted}` |
| `ornnlab/services/webui_job_progress.py` `job_trial_progress` | 明细 `total/completed/passed/notPassed/errored`（结果轴，已正确） |
| `ornnlab/services/harbor_results.py` `trial_dto` | `exception_info → status="failed"`（与明细的 `errored` 口径不一致） |
| `ornnlab/services/harbor_results.py` `has_resumable_trials` | 已对齐 Harbor resume 语义（无 result.json 或 CancelledError 才可恢复） |
| Harbor `models/job/result.py` `JobResult` | 无 job 级状态字段；只有 `finished_at` + trial 统计 |

### 差距与目标

- **G1**：job 状态改为两轴——执行状态（`completed` 仅看执行完成）+ 结果明细；`failed` 保留给执行级失败。
- **G2**：trial 状态统一——`errored` 取代 DTO 的 `failed`；`pending/running/passed/notPassed/errored/cancelled/interrupted`。
- **G3**：完成态 job 提供"重跑失败任务"（task 级，Harbor `--filter-error-type`）独立能力，与"续跑（resume）"分离。
- **G4**：存量数据兼容（旧 failed 行的读时派生）与文档/测试闭环。

### 非目标

- 不修改 Harbor 源码；不引入持久化状态机库；不改 DB schema（读时派生优先）；不做多 worker 竞态处理。

## 2. 关键假设与验证门

| ID | 关键假设 | 解锁的决策 | 最廉价可信方法 | 足够证据 / 不证明什么 | 预算与隔离 | 停止/清理 | 状态 |
|---|---|---|---|---|---|---|---|
| V1 | 读时派生（不改库）足以表达两轴模型 | 是否需 DB 迁移 | Static Evidence：梳理 `runs.status` 全部消费者（DTO/recovery/worker 判定） | 足够：所有消费者列表且无"状态必须持久化"的需求；不证明：历史数据需要回填 | 预算：代码阅读 ≤1h；允许：只读；禁止：改库 | 发现消费者依赖持久化语义则改为迁移方案 | planned |
| V2 | `harbor job resume --filter-error-type <T>` 只重跑匹配类型失败 trial、保留其余结果并合并 | W5 重跑失败任务的设计与是否投入 | Sandbox Evidence：用一次性沙箱 job（临时 jobs_dir + 2 个成功 1 个失败 trial）执行 filter 重跑，检查 result.json 合并与未匹配 trial 保留 | 足够：重跑后仅失败 trial 的新结果、成功 trial 结果原样保留、计数正确；不证明：真实 Harbor 上传/分享路径 | 预算：≤30min、临时目录；允许：临时 job 目录创建/删除；禁止：改动生产 job 目录 | 结论不符则 W5 改为手动删除 trial 目录方案 | planned |
| V3 | 存量 failed 行按新口径读时显示 completed+明细不破坏展示与恢复 | W6 兼容策略 | Static + 测试：用现有 job（`run-807c1fcbd081`，10/10 完成 1 errored）验证新派生 | 足够：该 job 显示 completed、明细 10/5/4/1、canResume=False；不证明：极端历史形态 | 预算：现有数据 + 单测 | 回归失败则回退派生改动 | planned |
| V4 | "重跑失败任务"入口形态（错误类型集合 vs 单 task） | W5 UI 设计 | 产品决策（E0/E1）：用户已倾向 task 级语义 | 足够：用户确认入口形态；不证明：无需技术验证 | n/a | 无 | planned |

## 3. 工作单元

| ID | 目标 | 变更轴 | 位置 | 目标对象 | 具体动作 | 结果行为 | 收益 | 副作用 | 验证 | 安全停止/回滚 | 状态 |
|---|---|---|---|---|---|---|---|---|---|---|---|
| W1 | 状态模型定义落地 | 文档 | `docs/releases/v1.0.5/job-task-state-machine/` | README + 本计划 | 建立两轴状态图（执行状态 × 结果明细；task 状态表）并登记文档清单 | 定义成为后续实现唯一口径 | 消除定义分歧，实现有依据 | 复杂度：文档 2 文件；成本：清单维护 | 文档校验（rebrand 脚本 + 测试） | 可随时调整文档 | planned |
| W2 | job 执行状态派生修正 | internal | `harbor_subprocess.py` `_status_from_result_payload`、`webui_job_service.py` `_job_dto` | 状态派生函数 | 执行完成（`finished_at`/`completed>=total` 且 running=0）→ `completed`，不再因 `errored>0` 返回 failed；`failed` 仅保留给执行级失败（completed<total 且非 interrupted/cancelled 的异常出口） | 完成态 job（含异常/未通过 trial）显示 completed，结果在明细 | job 状态不再误导用户；与 Harbor 语义一致 | 复杂度：派生逻辑调整；成本：存量显示变化 | 单测 + 现有 job 实测（V3） | 回滚派生函数即可 | planned |
| W3 | trial 状态统一 | API + internal + client | `harbor_results.py` `trial_dto`、`contract.ts`、`domain/harbor.ts`、`viewModels.ts`、`DetailRail.tsx`、i18n、mock/stories | TrialStatus 联合类型与映射 | `exception_info → status="errored"`；`TrialStatus` 增 `errored`、去 `failed`（若有引用同步清理） | trial 状态与明细口径一致；失败原因字段（已有）继续展示 | 消除自相矛盾；用户理解"异常 vs 不得分" | 复杂度：类型/映射/夹具同步；成本：前端渲染分支 | 类型检查 + 前后端测试 + Storybook | 类型回退 | planned |
| W4 | 前端状态展示语义 | client | `JobStatusBadge.tsx`、`AppShell`/列表、i18n | 状态文案与颜色 | `completed` 语义文案（如"已完成·5/10 通过"）随明细联动；`failed` 仅执行级 | 用户一眼区分"跑完了但成绩差"与"没跑完" | 走查痛点（看不到结果含义）缓解 | 复杂度：展示组件调整；成本：i18n 两语言 | 前端测试 + 实机预览 | 组件回退 | planned |
| W5 | 重跑失败任务 | API + client | `webui_job_service.py` `_resume_harbor_job`/`resume_job`、`webui_job_resume.py`、前端入口 | resume 命令与 UI 动作 | 依据 V4 形态：后端 `_resume_harbor_job` 支持 `filter_error_types` 参数（透传 `--filter-error-type`）；UI 提供"重跑失败任务"入口（自动收集失败 trial 错误类型集合） | 完成态 job 可只重跑失败任务，其余结果保留合并 | 失败任务有出路；不浪费已通过任务的 token/时间 | 复杂度：resume 参数面 + UI 动作；成本：新入口维护 | V2 沙箱验证 + 全量门禁 + 实机 | 入口隐藏/后端参数回退 | planned |
| W6 | 存量兼容 | internal | `webui_job_service.py` `_job_dto` | 读时派生 | 新派生对存量 failed 行生效（读时），回归确认既有展示与恢复行为 | 历史数据不迁移也按新口径展示 | 免迁移风险 | 复杂度：无新结构；成本：回归面 | V3 实测 + 回归测试 | 无 | planned |
| W7 | 测试与门禁闭环 | 测试 | `tests/python/*`、前端测试 | 状态派生/展示/重跑相关用例 | 补齐 W2-W5 的单元/API/前端测试并跑全量门禁 | 新语义有回归保护 | 定义变更可持续演进 | 复杂度：测试用例新增；成本：门禁时长 | 全量门禁 | n/a | planned |
| W8 | 日志与可观测性 | observability | `webui_job_service.py`/`webui_job_resume.py` | resume/重跑操作日志 | 记录重跑动作（类型集合、trial 数）与状态派生事件 | 重跑/状态变更可审计 | 排障与审计 | 复杂度：日志行；成本：无 | 日志断言测试 | 无 | planned |

## 4. 阶段

### Phase 1：定义与验证门

- 入口条件：无（立项即开始）
- 工作单元：W1；并行执行 V1、V2、V3（V4 需用户决策）
- 阶段内证据：文档登记校验通过；V1 消费者清单；V2 沙箱重跑证据；V3 现有 job 实测
- 下一阶段条件：V1/V2/V3 结论支持读时派生与 W5 方向；V4 用户确认入口形态

### Phase 2：后端两轴语义

- 入口条件：Phase 1 结论
- 工作单元：W2、W3、W6
- 阶段内证据：单测 + 现有 job 实测（completed + 明细正确）
- 下一阶段条件：后端派生与 trial 状态统一通过门禁

### Phase 3：前端语义与入口

- 入口条件：Phase 2 通过
- 工作单元：W4、W5（含 V2 沙箱结论）
- 阶段内证据：前端测试 + 实机预览 + 重跑实机
- 下一阶段条件：全量门禁绿

### Phase 4：收尾

- 工作单元：W7、W8；更新工程计划总表与走查台账
- 阶段内证据：全量门禁、文档同步

## 5. 执行追踪

| 阶段 | 新证据 | 受影响的假设/旧结论 | 结论更新 | 下游计划变更 | 计划有效性 | 下一步 |
|---|---|---|---|---|---|---|
| （待开始） | | | | | | |

## 6. 开放问题

- V4：重跑失败任务入口形态——按错误类型集合（Harbor 原生）还是按单个 task（OrnnLab 删目录+resume）？需用户决策。
- "全异常" job（如 10/10 errored）是否也是 completed（执行完成）？当前定义是；如用户不接受需修订 W2 边界。
- 存量 failed 行读时派生的显示变化（如走查期间的历史 job）是否需要走查记录提示。
