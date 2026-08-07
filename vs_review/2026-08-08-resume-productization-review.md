# Subagent VS Review: Job resume 产品化修复（权限/锁/代理/凭证）

- Created: 2026-08-08T02:38:00+0800
- Updated: 2026-08-08T02:38:00+0800
- Report schema: adversarial-v2
- Task: 让被中断的 Harbor Job 可以一键恢复（产品级根治：root 残留权限、陈旧锁、代理中继、敏感凭证四个根因）
- Report path: `vs_review/2026-08-08-resume-productization-review.md`
- Review mode: fresh internal subagents
- Source session policy: no inherited main-agent context
- Status: open
- Control outcome: none
- Automatic round budget: 2
- Completed rounds: 0
- Last known-good checkpoint: `27ab94d`（resume 修复链开始前）

## Review Control Contract

### Frozen Objective
被中断的 Harbor Docker Job 在点击"恢复"后能自动完成：① 容器清理前 chown bind-mount（杜绝 root 残留）；② 陈旧 lock.json 在 Job 确证死亡时自动清除；③ resume 前重建代理中继并回写 config/trial config；④ 从 Agent 档案恢复被 Harbor 序列化脱敏的敏感 env 并注入子进程；⑤ 失败时透出真实错误。所有修复对安装 ornnlab 的用户通用（非本机补丁）。

### Acceptance Criteria
- 中断 Job resume 不再因权限/锁/代理/凭证失败（每项有单测或实机证据）。
- 清锁只在 Job 确证死亡时发生（三重守卫：无活动操作、无 harbor 进程、无归属容器）。
- 敏感值不落盘明文（恢复后由 Harbor 模板化，运行时解析）。
- 全量门禁通过。

### Explicit Non-goals
- 不修改 Harbor 第三方代码。
- 不新增常驻依赖；不改变公开 API 契约。
- 不做 Windows 容器属主处理（Docker Desktop VM 自动处理）。

### Frozen Target Locations
- `ornnlab/services/webui_job_resume.py`
- `ornnlab/services/webui_job_service.py`（resume_job / _resume_harbor_job）
- `ornnlab/services/docker_orphan_service.py`（_chown_container_bind_mounts / _parse_mounts）
- `tests/python/test_webui_job_resume.py`、`tests/python/test_docker_orphan_service.py`、`tests/python/test_webui_api.py`

### Allowed Change Categories
- 修复性代码、测试、日志。

### Approval-required Changes
- 新顶层模块、新依赖、公开 API 变更、持久化数据变更、目标位置外改动。

### Authoritative Sources
| Authority | Source | What It Controls |
|---|---|---|
| E0 | 用户指令（"都产品化"） | 目标与范围 |
| E1 | 工程计划/PRD（anti-fabrication、事件脱敏） | 项目约束 |
| E2 | 实机验证（chown/锁/代理/token 恢复）、测试 | 实际行为 |
| E3 | Harbor `templatize_sensitive_env` 源码 | 外部机制事实 |
| E4 | reviewer/main-agent 推理 | 假设 |

### Baseline And Rollback
- Baseline revision: `c7f9a92`
- Rollback checkpoint: `27ab94d`
- Expected benefit: 中断 Job 恢复零手工干预；未来不再产生 root 残留
- Acceptable side effects: 清锁仅在确证死亡时发生；chown/代理恢复为 best-effort
- Automatic round budget: 2

## Round 1: 初始对抗性审查

### Round Control
- Round type: initial
- Round number: 1
- Completed automatic rounds before launch: 0
- User approval for this round: n/a
- Closure finding IDs: n/a
- Permitted closure relation: n/a
- Target scope delta allowed: none

### Review Input

#### Objective
见 Frozen Objective。

#### Acceptance Criteria
见 Acceptance Criteria。

#### Explicit Non-goals
见 Explicit Non-goals。

#### Review Target
代码实现（A/B/C/D 四层修复 + 失败透出）。

#### Target Locations
- `ornnlab/services/webui_job_resume.py`（prepare_resume_proxy / cleanup_resume_leftovers / clear_stale_job_lock / restore_sensitive_env / agent_env / 守卫辅助）
- `ornnlab/services/webui_job_service.py`（resume_job 编排 / _resume_harbor_job env 注入）
- `ornnlab/services/docker_orphan_service.py`（_chown_container_bind_mounts stopped 容器 start 重试 / _parse_mounts）
- `tests/python/test_webui_job_resume.py` 等

#### Baseline And Rollback Checkpoint
- Baseline: `c7f9a92`；Rollback: `27ab94d`

#### Change Introduction
为"中断 Job 一键恢复"实现四层产品级修复：A) resume 前用 docker root 容器 chown root 残留；B) 陈旧 lock.json 在三重死亡守卫下备份清除；C) docker 清理时对 stopped 容器先 start 再 chown bind-mount；D) resume 前从 Agent 档案恢复 config 中被 Harbor 序列化脱敏（****）的敏感 env，并把 Agent env 注入 harbor 子进程。另含 resume 失败 stderr 透出、并发 resume 防重入、代理 relay 重建与 config/trial config 回写。

#### Risk Focus
- 清锁守卫是否可被绕过（操作表状态、进程 cmdline 匹配、容器标签匹配的假阴性/假阳性）
- 敏感 env 恢复与注入的落盘/泄露路径（恢复后 Harbor 是否必然模板化；注入 env 是否会写日志）
- docker 子进程命令的注入面（`-v <path>:/work`、`--filter label=ornnlab.run_id=<id>` 等参数的边界）
- 并发/幂等：两次 resume、resume 与运行中清理、操作表中陈旧 running 行
- 失败路径：docker 不可用、psutil 异常、进程消失、文件被外部删除
- 对真实用户的可诊断性（错误信息是否指向可行动作）

#### User-Perspective Review Focus
- resume 失败时用户能否理解原因并行动（错误文案/日志）
- 无 docker 环境（纯 Harbor 无容器）的 Job resume 是否被新逻辑误伤

#### Implementation Completeness Focus
- A/B/C/D 是否都走生产路径（resume_job 编排链）而非测试专用
- 每层是否有失败兜底且不阻断主流程

#### Target Benefit Focus
- 声称"中断 Job 零手工干预"：是否有实机证据覆盖四层（有 E2 实机记录，需核对完整性）

#### Evidence Sources And Gaps
- E2: 实机 chown（running/stopped 容器）、锁自动清除、proxy relay 重建、token 恢复为模板且新 trial 0 认证错误
- E3: Harbor `harbor/utils/env.py` templatize_sensitive_env 源码
- E4: 各守卫假设需 reviewer 挑战

#### Assumptions To Attack
- "无活动操作 = 无进程在跑"（操作表陈旧行）
- "进程 cmdline 含 job 路径即可识别 harbor 进程"（初始 run 的 cmdline 是 `harbor run --config <tmp>`，不含 job 路径——孤儿进程探测可能漏）
- "`docker ps --filter label=ornnlab.run_id=X` 能识别活容器"（残留容器/清理前窗口）
- "restore 后 Harbor 必然模板化而非再次脱敏"（env 注入与 templatize 的匹配条件）
- "`docker exec chown -R` 对 bind mount 全目标安全"（mount 目标包含敏感路径？）
- "`****` 是脱敏唯一标记"（Harbor 对 ≤8 字符值脱敏为纯 `****`）

#### Adversarial Lenses
- implementation | concurrency | failure | data | security | observability | maintenance | testing

#### Verification Status
- pytest 223 passed / 4 skipped；全量门禁绿；实机验证记录见工程计划与本会话
- 已知缺口：无 Windows 实机；`configure-git-webserver` 一次瞬时 401 未复现

#### Reviewer Instructions
- Fresh internal subagent session（fork_context=false）。
- 不继承主 agent 上下文、推理、结论。
- 直接阅读目标文件，只读，不修改任何文件。
- 对抗性：尝试推翻至少一个假设/快乐路径/失败路径/安全边界。
- 每条阻断或范围扩张结论标注 E0-E4。
- 引用证据路径与行号。
- 输出：摘要、阻断项（含反例：破坏的假设、失败场景、触发条件、影响、所需证明、证据级别）、非阻断风险、用户视角检查、实现完整性检查、所需修复、缺失测试、缺失日志、证据清单。

### Reviewer Timeout Policy
| Complexity | Initial Wait | Extension | Max Attempts Per Role | Blocking Closure Behavior |
|---|---|---:|---:|---|
| complex | 20 分钟 | +10 分钟 | 2 | 无法通过则不得标记 passed |

### Reviewer Selection
| Reviewer | Reason Selected | Risk Area |
|---|---|---|
| implementation-adversary | 目标是多层状态/并发/失败/安全边界的代码实现，正确性风险最高 | 守卫绕过、敏感值路径、docker 子进程边界、并发与幂等 |

### Reviewer Launch Records
| Reviewer | Internal Mechanism | Session / Job ID | Trace Source | Context Forked | Input Packet | Context Explicitly Excluded | Read-only |
|---|---|---|---|---|---|---|---|
| implementation-adversary | Task tool (fresh subagent) | 待启动后填写 | Task tool 调用记录 | fork_context=false | Round 1 Review Input | 主 agent 历史、推理、草稿、结论、完整 diff | yes |

### Reviewer Timeout Records
| Reviewer Output Key | Reviewer Role | Attempt | Session / Job ID | Waited | Status | Reason | Action |
|---|---:|---|---:|---|---:|---|
| implementation-adversary-r1 | implementation-adversary | 1 | 待启动后填写 | 待定 | 待定 | 待定 | 待定 |

### Reviewer Outputs

（待 reviewer 返回后填写）

### Main Agent Response

（待 reviewer 返回后填写）

### Review Governor

（待 reviewer 返回后填写）

### Convergence Reflection

（如需要）

### User Decision

（如需要）

### Closure Status

（待定）

## Final Conclusion

（待定）
