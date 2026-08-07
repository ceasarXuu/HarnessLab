# Subagent VS Review: Job resume 产品化修复（权限/锁/代理/凭证）

- Created: 2026-08-08T02:38:00+0800
- Updated: 2026-08-08T03:20:00+0800
- Report schema: adversarial-v2
- Task: 让被中断的 Harbor Job 可以一键恢复（产品级根治：root 残留权限、陈旧锁、代理中继、敏感凭证四个根因）
- Report path: `vs_review/2026-08-08-resume-productization-review.md`
- Review mode: fresh internal subagents
- Source session policy: no inherited main-agent context
- Status: open（待用户决策 Round 3 或接受）
- Control outcome: user-decision-required
- Automatic round budget: 2
- Completed rounds: 2
- Last known-good checkpoint: `27ab94d`

## Review Control Contract

### Frozen Objective
被中断的 Harbor Docker Job 在点击"恢复"后能自动完成：① 容器清理前 chown bind-mount；② 陈旧 lock.json 在 Job 确证死亡时自动清除；③ resume 前重建代理中继并回写 config/trial config；④ 从 Agent 档案恢复被脱敏的敏感 env 并注入子进程；⑤ 失败时透出真实错误。所有修复对安装 ornnlab 的用户通用。

### Acceptance Criteria
- 中断 Job resume 不再因权限/锁/代理/凭证失败（每项有单测或实机证据）。
- 清锁只在 Job 确证死亡时发生（守卫不误伤活 Job，也不被死 Job 卡死）。
- 敏感值不落盘明文（恢复后由 Harbor 模板化，运行时解析）。
- 全量门禁通过。

### Explicit Non-goals
- 不修改 Harbor 第三方代码；不新增常驻依赖；不改公开 API 契约；不做 Windows 容器属主处理。

### Frozen Target Locations
- `ornnlab/services/webui_job_resume.py`、`webui_job_service.py`（resume_job/_resume_harbor_job）、`docker_orphan_service.py`（_chown_container_bind_mounts/_parse_mounts）、相关测试。

### Allowed Change Categories
- 修复性代码、测试、日志。

### Approval-required Changes
- 新顶层模块、新依赖、公开 API 变更、持久化数据变更、目标位置外改动。

### Authoritative Sources
| Authority | Source | What It Controls |
|---|---|---|
| E0 | 用户指令（"都产品化"） | 目标与范围 |
| E1 | 工程计划/PRD | 项目约束 |
| E2 | 实机验证、测试、代码链对照 | 实际行为 |
| E3 | Harbor 源码（templatize_sensitive_env 等） | 外部机制事实 |
| E4 | reviewer/main-agent 推理 | 假设 |

### Baseline And Rollback
- Baseline revision: `c7f9a92`（Round 1 起点）→ `bca943f`（审查修复后 HEAD）
- Rollback checkpoint: `27ab94d`
- Expected benefit: 中断 Job 恢复零手工干预；未来不再产生 root 残留
- Acceptable side effects: 清锁仅在确证死亡时发生；chown/代理恢复为 best-effort
- Automatic round budget: 2

## Round 1: 初始对抗性审查

### Round Control
- Round type: initial；Round number: 1；Completed automatic rounds before launch: 0
- User approval: n/a；Closure finding IDs: n/a；Target scope delta: none

### Review Input
见会话内导航包（objective/acceptance/target/risk focus/assumptions/adversarial lenses 同上方契约；round type=initial；reviewer=implementation-adversary；timeout policy=complex 20min+10min；只读、fresh session、无主上下文继承、引用行号、标注 E0-E4）。

### Reviewer Timeout Policy
| Complexity | Initial Wait | Extension | Max Attempts Per Role | Blocking Closure Behavior |
|---|---|---:|---:|---:|---|
| complex | 20 分钟 | +10 分钟 | 2 | 无法通过则不得标记 passed |

### Reviewer Selection
| Reviewer | Reason Selected | Risk Area |
|---|---|---|
| implementation-adversary | 多层状态/并发/失败/安全边界代码，正确性风险最高 | 守卫绕过、敏感值路径、docker 子进程边界 |

### Reviewer Launch Records
| Reviewer | Internal Mechanism | Session / Job ID | Trace Source | Context Forked | Input Packet | Context Explicitly Excluded | Read-only |
|---|---|---|---|---|---|---|---|
| implementation-adversary (Round 1) | Task tool (fresh subagent, general) | ses_0227abfedffehmQSIlzQOFV4sT | Task tool 调用 | fork_context=false | Round 1 Review Input | 主 agent 历史/推理/结论/完整 diff | yes |
| implementation-adversary (Round 2) | Task tool (fresh subagent, general) | ses_0226aa66bffegVpbE84VoiYYO4 | Task tool 调用 | fork_context=false | Round 2 Closure Input | 主 agent 历史/推理/结论 | yes |

### Reviewer Timeout Records
| Reviewer Output Key | Reviewer Role | Attempt | Session / Job ID | Waited | Status | Reason | Action |
|---|---:|---|---:|---|---:|---|
| implementation-adversary-r1 | implementation-adversary | 1 | ses_0227abfedffehmQSIlzQOFV4sT | <20min | completed | n/a | completed |
| implementation-adversary-r2 | implementation-adversary | 1 | ses_0226aa66bffegVpbE84VoiYYO4 | <20min | completed | n/a | completed |

### Reviewer Outputs

#### implementation-adversary-r1

##### Summary
四层修复编排链已接入生产路径、测试覆盖快乐路径。发现 1 个确定阻断（B1 env 替换而非合并）、1 个安全边界阻断（B2 守卫 2 对原始 harbor 进程失明）、1 个覆盖缺口（B3 environment/verifier 敏感 env 不恢复）、1 个死容器误伤（B4）。

##### Blocking Findings
- **B1**：`_resume_harbor_job` 用 `env=agent_env` 替换子进程环境 → PATH/HOME 丢失。E1（调用点对照 + Harbor preflight `shutil.which("docker")`）。后经主代理实证：Linux 上 `shutil.which` 无 PATH 时回退 `os.defpath=/bin:/usr/bin` 侥幸可用；macOS（/opt/homebrew）与继承 env 场景必炸。
- **B2**：`_live_harbor_process_for` 只匹配 cmdline 含 job 路径的进程；原始 run 进程 cmdline 为 `harbor run --config <jobs_dir>/harbor.config.json` 或 `/tmp/ornnlab-harbor-runtime-*`，均不含 `<jobs_dir>/<job_name>` → 孤儿进程漏检 → 活 Job 清锁 → 双执行。E1。
- **B3**：`restore_sensitive_env` 只恢复 `agents/agent` env；EnvironmentConfig/VerifierConfig 同样有 templatize 序列化器 → environment/verifier 敏感值 resume 后仍脱敏 → 静默 401。E1。
- **B4**：`_run_has_live_containers` 用 `docker ps -aq`（含 Exited）→ 死容器永久阻止清锁（尤其 keep_containers=retain 配置）→ resume 永远 lock mismatch。E1。

##### Non-blocking Risks
1. 事件循环冻结（docker ps timeout 10s + psutil 全量扫在同步段）；2. 多 worker 竞态（操作表无唯一约束）；3. resume 产物容器无生命周期管理；4. chown 失败不进 cleanup errors；5. 离线 alpine 拉取失败；6. resume 取消时 CancelledError 未捕获 → run 停留 running、harbor 孤儿；7. 宿主代理移除时 config 残留死 relay URL；8. 小项（非原子写/`****` 子串误恢复/守卫 3 不过滤 instance_id/错误尾部 300 字符）。

##### User-Perspective Checks
- 可恢复性 pass（失败回 interrupted 可再点）；可诊断性差（守卫拒绝原因无日志无 UI 透出）；canResume 预检不足。

##### Implementation Completeness Checks
- 四层修复均在 resume_job 生产编排链；B1 合并语义为核心错；B2 守卫失明；B3 恢复范围不全；B4 死容器判活。

##### Evidence
- `webui_job_service.py:341-348`（env 替换）、`webui_job_resume.py:147-156`（marker 匹配）、`webui_job_resume.py:220-231`（恢复范围）、`webui_job_resume.py:159-178`（-aq）、`harbor_subprocess.py:56-66`（run cmdline）、site-packages/harbor/utils/env.py:55-58（脱敏格式）。

#### implementation-adversary-r2（闭环节点）

##### Summary
B1、B3、B4 及附加修复均已闭环且无直接回归。B2 未闭环：代理模板化场景（--config → /tmp/ornnlab-harbor-runtime-*）原始 run 进程依旧失明（BLK-1）；新目录前缀匹配在共享 jobsDir 布局下必然跨 job 误伤（BLK-2）。`or None` 确认为恒真死代码（BLK-3，非阻断）。

##### Blocking Findings
- **BLK-1**（original-blocker-open，E2）：`--config` 前缀匹配不覆盖 `/tmp/ornnlab-harbor-runtime-*` 临时 config 形态（代理模板化场景，正是本功能核心场景）→ 活 Job 下清锁仍可能发生。
- **BLK-2**（fix-regression，E2）：jobsDir 前缀匹配以目录为粒度；共享 jobsDir 下所有 job 的 config 是同一文件 `<jobsDir>/harbor.config.json` → 兄弟 job 运行中会阻塞本 job 清锁（fail-closed 但 resume 永远失败，仅 info 日志）。

##### Non-blocking Risks
- B3 部分恢复时告警语义不完整（elif 短路）；B4 legacy 无 instance 标签容器 fail-open；B4 status=running 不含 paused/restarting；B4 fake docker 测试不校验 filter；`str(CancelledError)` 为空 → failure_summary 为空（外观）。

##### Closure Relation
| 修复项 | 结论 | 关系 |
|---|---|---|
| B1 env 合并 | 闭环 | original-blocker-closed |
| B2 进程探测 | 未闭环 | original-blocker-open（BLK-1）+ fix-regression（BLK-2） |
| B3 恢复范围 | 闭环 | original-blocker-closed |
| B4 容器过滤 | 闭环 | original-blocker-closed |
| 附加修复 | 无回归 | n/a |

##### 建议方向（未执行）
B2 应读 config 内容校验 job_name/jobs_dir 归属，而非目录前缀。

### Main Agent Response

| Reviewer | Finding | 反例 | Severity | Decision | Authority | Closure Relation | Evidence / Reason | Scope Effect | Side Effects | Action Taken | Follow-up |
|---|---|---|---|---|---|---|---|---|---|---|---|
| r1 | B1 env 替换 | PATH 丢失 | blocking | accept | E1+E2 实证 | original-blocker | shutil.which 回退 defpath 掩盖；macOS 必炸 | env 合并 | 无 | 改 `{**os.environ, **run_agent_env}`；移除 `or None` 死代码（BLK-3） | 无 |
| r1 | B2 守卫失明 | 原始 run cmdline 无 job 路径 | blocking | accept | E1 | original-blocker | Round 1 修复为前缀匹配，Round 2 判定不完整 → 二次修复 | 见 Round 2 | 见 Round 2 | 见 Round 2 修复 | 无 |
| r1 | B3 恢复范围 | environment/verifier 仍脱敏 | blocking | accept | E1 | original-blocker | 扩展 environment.env（环境预设档案）+ verifier 告警 | 小 | 无 | 已实现 + 测试 | 无 |
| r1 | B4 死容器 | -aq 含 Exited | blocking | accept | E1 | original-blocker | 加 status=running + instance_id 过滤 | 小 | legacy 容器 fail-open（非阻断） | 已实现 + 测试 | 无 |
| r1 | 取消路径 | CancelledError 未捕获 | non-blocking | accept | E1 | n/a | 补 catch + _mark_resume_failed | 小 | 无 | 已实现 | 无 |
| r1 | 守卫拒绝无日志 | 无法诊断 | non-blocking | accept | E4 | n/a | 补 reason= 日志 | 小 | 无 | 已实现 | 无 |
| r1 | 非原子写 | 崩溃截断 | non-blocking | accept | E4 | n/a | 用 atomic_write_text | 小 | 无 | 已实现 | 无 |
| r1 | docker ps 超时 10s | 事件循环冻结 | non-blocking | accept | E4 | n/a | 5s；resume 为低频操作 | 小 | 无 | 已实现 | 仍同步执行，如需彻底避免需 to_thread |
| r1 | 多 worker 竞态 | 双 resume | non-blocking | reject | E1 | unrelated | 单 worker 架构（app.py 单 uvicorn） | 无 | 无 | 记录 | 若未来多 worker 再处理 |
| r1 | resume 容器生命周期 | 残留至下次启动 | non-blocking | defer | E4 | unrelated | 启动清理兜底 | 无 | 无 | 记录为后续维护项 | 待跟踪 |
| r1 | chown 失败不上报 | 用户误信成功 | non-blocking | defer | E4 | unrelated | 保留 warning 日志；resume 路径有 A 兜底 | 无 | 无 | 记录 | 待跟踪 |
| r1 | 离线 alpine | 失败可诊断性 | non-blocking | defer | E4 | unrelated | 错误尾部透出 PermissionError | 无 | 无 | 记录 | 待跟踪 |
| r1 | 代理失效静默 | 死 relay URL | non-blocking | defer | E4 | unrelated | 宿主代理移除属配置变更 | 无 | 无 | 记录 | 待跟踪 |
| r1 | `****` 子串误恢复 | 病理值 | non-blocking | reject | E4 | n/a | 档案为权威源，替换方向正确 | 无 | 无 | 拒绝 | 无 |
| r1 | 错误尾部 300 字符 | 截断 | non-blocking | reject | E4 | n/a | 尾部为最相关信息 | 无 | 无 | 拒绝 | 无 |
| r2 | BLK-1 模板化场景失明 | temp config 形态 | blocking | accept | E2 | original-blocker-open | 改为读 --config 文件内容按 job_name+jobs_dir 归属 | 中 | 无 | 已实现（_config_targets_job）+ 测试 | 无 |
| r2 | BLK-2 共享 jobsDir 误伤 | 目录前缀粒度 | blocking | accept | E2 | fix-regression | 移除前缀启发；resume 按 --job-path 精确匹配；run 按 config 内容归属 | 中 | 无 | 已实现 + 测试（异 job/兄弟路径/不可读） | 无 |
| r2 | BLK-3 or None 死代码 | 恒真 | non-blocking | accept | E4 | n/a | 移除 | 无 | 无 | 已实现 | 无 |

### Review Governor (Round 1)
- Completed rounds before decision: 1；Unresolved blockers after round: 4（B1-B4）
- Governor decision: start-closure-round（4 个阻断均接受且有 E1/E2 证据；修复在冻结目标位置内；无范围扩张）
- Decision reason: 阻断项均与冻结目标直接相关，证据充分，修复边界清晰

### Review Governor (Round 2)
- Completed rounds before decision: 2（预算已用尽）
- Unresolved blockers after round: 2（BLK-1、BLK-2，随后由主代理修复并经测试验证）
- Blockers closed: B1、B3、B4、BLK-3；BLK-1/BLK-2 由主代理在 Round 2 后修复（bca943f），测试覆盖（_cmdline_targets_job/_config_targets_job 6 个用例）
- New blocker classes: none（BLK-2 为 B2 修复的回归，同类）
- Repeated failure class: yes - 进程探测（B2 → BLK-1/BLK-2 同主题）
- Scope expansion proposed: no；New modules/deps/API/data changes: none
- Governor decision: user-decision-required（自动预算 2 轮已用尽；Round 2 闭环节点曾发现原阻断未闭环，修复后是否追加一轮聚焦复审需用户决定）
- Decision reason: 预算规则——Round 2 后不自动启动 Round 3

### Convergence Reflection
- Original objective / acceptance criteria / non-goals: 见契约
- Completed rounds versus budget: 2/2
- Findings closed: B1、B3、B4、BLK-3 及 5 项非阻断；BLK-1/BLK-2 已修复（bca943f）未复审
- Findings repeated: 进程探测主题在 B2/BLK-1/BLK-2 重复出现，最终以"读 config 内容精确归属"收敛
- Evidence inventory: E0（用户授权产品化）、E1（代码/计划）、E2（实机验证 + 测试 231 passed + 门禁绿）、E3（Harbor 源码）、E4（reviewer 推理）
- Newly touched: webui_job_resume.py、webui_job_service.py、docker_orphan_service.py、测试 3 文件
- Cumulative growth: webui_job_resume 357 行（新模块）、webui_job_service 499 行（限内）
- Benefits achieved: 中断 Job resume 全自动（权限/锁/代理/凭证四层实机验证）
- Side effects: 无已知回归（全量门禁绿）
- Risk direction: decreasing
- Last known-good checkpoint: `27ab94d`；Rollback options: `git revert bca943f`（或回退 27ab94d）
- Recommended bounded choices: ① 追加一轮聚焦复审（BLK-1/BLK-2 修复，预算外需批准）；② 基于测试证据接受修复；③ 回退

### User Decision
- Decision requested: 追加一轮聚焦复审 / 接受修复 / 回退
- Options and consequences:
  - 追加 Round 3（仅审 BLK-1/BLK-2 修复与其直接回归）：证据最充分，成本一次 fresh reviewer
  - 接受修复（依据 6 个新测试 + 全量门禁 + 代码链对照）：成本零，风险为进程探测仍可能有未覆盖形态
  - 回退到 27ab94d：放弃本轮 resume 产品化
- User decision: pending

### Closure Status
- Blocking findings found: yes（Round 1: B1-B4；Round 2: BLK-1/BLK-2）
- Accepted blocking findings fixed: yes（bca943f 已含全部修复）
- Blocking re-review completed: no（Round 2 闭环节点发现原阻断未闭环；修复后未再复审）
- Blocking re-review passed: no
- Automatic round budget respected: yes（2/2）
- Third-or-later round explicitly user-approved before launch: n/a（未启动）
- Scope drift detected: no
- Evidence sufficient for scope-expanding actions: yes（E1/E2）
- Control outcome: user-decision-required
- Allowed to proceed: 待用户决策

## Final Conclusion

Round 1（初始）发现 4 阻断（B1-B4）已全部接受并修复；Round 2（闭环节点）发现 B2 修复不完整（BLK-1 模板化场景失明、BLK-2 共享目录误伤），主代理已按 reviewer 建议改为"读 config 内容精确归属"并修复（`bca943f`），6 个新测试 + 全量门禁绿。自动审查预算 2 轮已用尽，按规则不自动启动 Round 3。任务状态：**待用户决策**——追加一轮聚焦复审 / 接受当前修复 / 回退。
