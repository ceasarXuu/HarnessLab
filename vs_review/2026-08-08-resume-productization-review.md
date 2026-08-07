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
- User decision: **批准追加一轮聚焦复审**（Round 3，仅限 BLK-1/BLK-2 修复 `bca943f` 及其直接回归）
- Approval evidence: 用户消息"批准追加一轮"（2026-08-08）
- Authorized next scope: Round 3 仅审查 `_cmdline_targets_job` / `_config_targets_job` 的实现与测试，及其直接回归（bca943f diff 范围内）

## Round 3: BLK-1/BLK-2 修复聚焦复审（用户批准）

### Round Control
- Round type: user-approved-extra
- Round number: 3
- Completed automatic rounds before launch: 2
- User approval for this round: 用户消息"批准追加一轮"（2026-08-08，记录于 User Decision 节）
- Closure finding IDs: BLK-1、BLK-2
- Permitted closure relation: original-blocker-open | fix-regression | direct-adjacent-objective-failure
- Target scope delta allowed: 仅 bca943f diff（_cmdline_targets_job/_config_targets_job 及测试）

### Review Input
- Objective: 验证 BLK-1（代理模板化场景原始 run 进程识别）与 BLK-2（共享 jobsDir 跨 job 误伤）在 `bca943f` 的修复中真正闭环，且修复未引入直接回归。
- Target locations: `ornnlab/services/webui_job_resume.py`（_live_harbor_process_for/_cmdline_targets_job/_config_targets_job）、`tests/python/test_webui_job_resume.py`（test_live_harbor_process_* 6 用例）。
- Baseline: `1322d67`（Round 2 审查基线）；修复提交：`bca943f`。
- Change introduction: 进程探测从"cmdline 含 job 路径 / --config 前缀匹配"改为：resume 形态按 `--job-path` 精确路径匹配；run 形态读取 `--config` 文件内容，按 `job_name == job_path.name` 且 `jobs_dir == str(job_path.parent)` 精确归属。
- Risk focus（挑战）：
  - 精确匹配是否仍存在假阴性（原始 run 进程 cmdline 的其他形态？`harbor run` 无 `--config` 标志的形态？进程名不含 "harbor" 子串的形态，如 venv 路径含 "harbor" 但可执行名不同？）
  - `_config_targets_job` 读共享 `harbor.config.json` 时：该文件被后续 job 覆盖后，旧孤儿进程归属误判为 False（fail-open）的风险与概率
  - `Path(args[index+1]) == job_path` 的等价性（符号链接/相对路径/尾部斜杠）
  - 新增测试是否真正覆盖 BLK-1/BLK-2 的反例，而非仅验证实现细节
  - 性能：每个 resume 全量 psutil 扫描 + 读 config 文件，是否引入可感知延迟
- Closure relation 判定要求：每条发现标注 original-blocker-open / fix-regression / direct-adjacent-objective-failure / unrelated-existing-risk。
- Reviewer instructions: fresh session、只读、引用行号、标注 E0-E4、聚焦闭环节点。

### Reviewer Timeout Policy
| Complexity | Initial Wait | Extension | Max Attempts Per Role | Blocking Closure Behavior |
|---|---|---:|---:|---:|---|
| normal | 12 分钟 | +6 分钟 | 2 | 无法通过则不得标记 passed |

### Reviewer Selection
| Reviewer | Reason Selected | Risk Area |
|---|---|---|
| implementation-adversary | 与 Round 1/2 同主题（进程探测归属正确性），同角色一致性 | 假阴性/假阳性/回归 |

### Reviewer Launch Records
| Reviewer | Internal Mechanism | Session / Job ID | Trace Source | Context Forked | Input Packet | Context Explicitly Excluded | Read-only |
|---|---|---|---|---|---|---|---|
| implementation-adversary (Round 3) | Task tool (fresh subagent, general) | 待启动后填写 | Task tool 调用 | fork_context=false | Round 3 Review Input | 主 agent 历史/推理/结论 | yes |

### Reviewer Timeout Records
| Reviewer Output Key | Reviewer Role | Attempt | Session / Job ID | Waited | Status | Reason | Action |
|---|---:|---|---:|---|---:|---|
| implementation-adversary-r3 | implementation-adversary | 1 | ses_0224f8175ffePJvRu6gkk2t67j | <12min | completed | n/a | completed |

### Reviewer Outputs

#### implementation-adversary-r3（聚焦复审）

##### Summary
修复方向正确（内容归属 + resume 精确路径），BLK-1 主场景（SIGKILL 后 temp config 残留）与 BLK-2 主场景（resume `/a/b` vs `/a/b2`）已闭环。但内容启发在两类结构性场景产生 **fail-open 漏检**：① 共享 config 文件 last-writer-wins（后启动兄弟 job 覆写后，孤儿进程归属误判 False）；② legacy 布局（job_path == jobs_dir）下 `jobs_dir == str(job_path.parent)` 恒不成立，探测结构性失效。两处均为"活 Job 被清锁 → 双写"的安全方向失败。

##### Blocking Findings
- **B1**（fix-regression + original-blocker-open 残留，E2）：共享 jobsDir 下 `harbor.config.json` 内容 = 最后一次启动的 job（harbor_engine.py:95-98 原子覆写）；孤儿进程 A + B 已启动（文件=B）→ 探测 A 失配 → fail-open 清锁 → 双写。触发链：共享目录 + 孤儿存活期间兄弟启动 + 容器守卫已被启动清理移除。
- **B2**（fix-regression，E2）：legacy 布局 `resolve_harbor_job_path` 回退 job_path==jobs_dir（harbor_paths.py:24-27）→ `jobs_dir == str(job_path.parent)` 恒 False → run 进程探测结构性 miss。
- **B3**（original-blocker-open 残留，E2，低概率）：config 不可读/解析失败返回 False（fail-open），与"确证死亡才清锁"契约方向相反；temp config 在干净退出被清理后孤儿存活即触发。

##### Non-blocking Risks
- N1（配置依赖）：`ORNNLAB_HARBOR_CLI` 自定义为不含 "harbor" 的二进制名 → 前置过滤静默失效（fail-open）。
- N2：自定义 subprocess command 自带 `--config` 时 `args.index("--config")` 读到用户 command 自己的文件。
- N7（pre-existing）：resume 双击 TOCTOU（active_resume_operation 检查与 submit 非原子）——与本提交无关。
- N8：修复后常见路径 fail-closed → 孤儿存活时 resume 永久阻塞，系统无孤儿 harbor 进程清理机制。

##### 建议修复（未执行）
1. 根因：进程 PID 为身份信号（spawn 时持久化 pid+start_time，探测校验存活），精确且 fail-safe；文件内容启发在共享目录下不可判定。
2. 短期：统一 fail-closed（进程存在但不可读/失配 → 保留锁 + 日志）；修正 legacy 分支比对目标；配套孤儿进程清理机制。

### Main Agent Response（Round 3）

| Reviewer | Finding | 反例 | Severity | Decision | Authority | Closure Relation | Evidence / Reason | Scope Effect | Side Effects | Action Taken | Follow-up |
|---|---|---|---|---|---|---|---|---|---|---|---|
| r3 | B1 共享文件 last-writer-wins fail-open | 孤儿 A + 文件=B → 误清锁 | blocking | accept | E2 | fix-regression + original-blocker-open | 内容归属在共享目录不可判定 | 无 | 无 | 同目录 config 失配/不可读统一 fail-closed（保留锁 + `resume_lock_probe_inconclusive` 日志） | PID 信号为根治项（见下） |
| r3 | B2 legacy 布局结构性失配 | jobs_dir 恒 ≠ parent | blocking | accept | E2 | fix-regression | 比对目标修正 | 无 | 无 | `_config_matches_job` 接受 `jobs_dir in {job_path, job_path.parent}` | 无 |
| r3 | B3 不可读 fail-open | 信号不可得放行清锁 | blocking | accept | E2 | original-blocker-open | 统一 fail-closed | 无 | 无 | 同目录/ornnlab-harbor-runtime 前缀不可读 → True（保留锁） | 无 |
| r3 | N1 二进制名绕过 | 自定义名无 "harbor" | non-blocking | defer | E4 | unrelated-existing-risk | 配置依赖边缘；记录 | 无 | 无 | 记录 | PID 根治项一并处理 |
| r3 | N2 自定义 --config | index 读到 command 自身文件 | non-blocking | defer | E4 | unrelated-existing-risk | 自定义 command 边缘；同目录判定仍 fail-closed | 无 | 无 | 记录 | 同上 |
| r3 | N7 双击 TOCTOU | 检查与提交非原子 | non-blocking | defer | E4 | unrelated-existing-risk | pre-existing，单 worker 下窗口极小 | 无 | 无 | 记录 | 若多 worker 再处理 |
| r3 | N8 孤儿阻塞 | fail-closed 后 resume 卡死 | non-blocking | defer | E4 | unrelated-existing-risk | 有 reason=live_harbor_process 日志与 Harbor 锁错误提示 | 无 | 无 | 记录 | 孤儿进程清理为独立产品项（reconcile_startup 终止孤儿），需另行立项 |

**PID 根治项（未实施，需批准）**：以进程 PID + start_time 为身份信号（spawn 时持久化 sidecar，探测校验存活）替代文件内容启发——精确、无歧义、无共享目录歧义。需修改 `harbor_subprocess.py`（冻结目标外）并新增 sidecar 文件（持久数据变更），按控制契约需 E0 批准后另行实施。

### Review Governor (Round 3)
- Completed rounds before decision: 3（预算 2 + 用户批准 1）
- Unresolved blockers after round: 3（B1/B2/B3），随后由主代理在冻结目标内修复并经测试验证（233 passed、门禁绿）
- Blockers closed: B1/B2/B3 全部接受并以 fail-closed + legacy 修正闭环；测试从 6 扩至 9 个用例（兄弟同目录 fail-closed、异目录忽略、同目录不可读 fail-closed、legacy 命中）
- New blocker classes: none（同主题 fail-open 方向）
- Repeated failure class: yes - 进程探测归属（B2 → BLK-1/BLK-2 → B1/B2/B3，最终以 fail-closed 收敛）
- Scope expansion proposed: no（PID 根治项作为需批准的后续项记录，未实施）
- Governor decision: user-decision-required（Round 4 需用户批准；当前 fail-closed 修复已消除全部已证 fail-open 路径）
- Decision reason: 自动预算 2 轮 + 用户批准 1 轮均已使用；B1/B2/B3 已在冻结目标内修复并有测试证据；是否追加 Round 4 或接受当前状态需用户决定

### Convergence Reflection
- Completed rounds versus budget: 3/3（2 自动 + 1 用户批准）
- Findings closed: B1-B4（R1）、BLK-1/BLK-2（R2）、B1/B2/B3（R3）；全部以测试 + 门禁闭环
- Findings repeated: 进程探测归属主题经历 3 轮（前缀→内容→fail-closed），最终收敛为"无法确证排除即保留锁"的安全方向
- Benefits achieved: resume 全自动（权限/锁/代理/凭证四层实机 + 测试验证）
- Risk direction: decreasing（fail-open 路径全部转为 fail-closed）
- Last known-good checkpoint: `27ab94d`；Rollback: `git revert` 最近提交
- Recommended bounded choices: ① 追加 Round 4（仅审 fail-closed 修复）；② 接受当前状态（B1/B2/B3 修复有 9 个用例 + 门禁绿）；③ 批准 PID 根治项另行实施；④ 回退

### User Decision (Round 3)
- Decision requested: 追加 Round 4 / 接受当前状态 / 批准 PID 根治项 / 回退
- User decision: **批准 PID 根治项 + 追加 Round 4**（用户消息"批准1、3"，2026-08-08）
- Authorized next scope: ①实施 PID sidecar 根治（修改 harbor_subprocess.py + 新增 sidecar 文件）；②Round 4 聚焦复审（e3271b3 fail-closed 修复 + PID 实现及其直接回归）

## Round 4: PID 根治项 + fail-closed 修复聚焦复审（用户批准）

### Round Control
- Round type: user-approved-extra；Round number: 4
- Completed automatic rounds before launch: 3（2 自动 + 1 用户批准）
- User approval: 用户消息"批准1、3"（2026-08-08）
- Closure finding IDs: e3271b3（fail-closed）+ 2eb2ed6（PID sidecar）的全部已接受阻断项
- Permitted closure relation: original-blocker-open | fix-regression | direct-adjacent-objective-failure
- Target scope delta allowed: e3271b3 + 2eb2ed6 提交范围

### Review Input
- Objective: 验证 fail-closed 修复（同目录失配/不可读保留锁）与 PID sidecar 根治（spawn 持久化 pid+start_time、探测精确判定）真正闭环，无直接回归，且无残余 fail-open 路径。
- Target locations: `ornnlab/services/harbor_subprocess.py`（_write/_remove_job_pid_sidecar、run 调用点）、`ornnlab/services/webui_job_resume.py`（_live_harbor_process_for/_sidecar_process_alive/_cmdline_targets_job/_config_belongs_to_job）、相关测试。
- Risk focus: sidecar 生命周期竞态、start_time 阈值、路径净化注入、resume 进程无 sidecar 的并发保护、job_name 映射一致性、fail-closed 残留的兄弟阻塞、测试真实性、TOCTOU/权限/多实例/并发写。
- Reviewer instructions: fresh session、只读、引用行号、标注 E0-E4、闭环节点关系。

### Reviewer Timeout Records
| Reviewer Output Key | Reviewer Role | Attempt | Session / Job ID | Waited | Status | Reason | Action |
|---|---:|---|---:|---|---:|---|
| implementation-adversary-r4 | implementation-adversary | 1 | ses_0222fa17dffeJTyEb6HThiEqEI | <12min | lost（空结果） | reviewer 返回空输出 | replacement spawned |
| implementation-adversary-r4 | implementation-adversary | 2 | ses_0222a5f65ffeJNqjWFVq16zmX2 | <12min | completed | n/a | completed |

### Reviewer Outputs

#### implementation-adversary-r4（替换尝试）

##### Summary
设计闭环方向正确：PID sidecar 精确身份信号消除共享 jobsDir last-writer-wins 歧义，start_time 校验为"权威死亡"判定提供不可伪造身份，cmdline 内容启发退居 fail-closed 回退位。**未能推翻其安全方向（fail-open）——无阻断性发现。** 全部 8 个挑战点静态分析 + 分支实证（19/19 测试）验证。

##### Blocking Findings
无。

##### Non-blocking Risks
- N1：`_write_job_pid_sidecar` 在 try/finally 外，写失败（ENOSPC/EACCES）冒泡 → 运行标 failed 而 harbor 子进程孤儿继续 → 建议 try/except + warn。
- N2：`--job-path` 缺值返回 False（fail-open）与 `--config` 缺值返回 True 不对称 → 建议对齐 True。
- N3：sidecar 权威死亡路径无专属日志 → 建议 `resume_lock_probe_sidecar_dead`。
- N4：sidecar 命名函数两模块重复 → 建议共享。
- N5：Round-3 已记录的 N7/N8 仍开放（已有日志兜底与文档挂账）。

##### Evidence
- 挑战点结论：快速退出/写入顺序安全；1s 阈值合理（自身时钟不漂移，失配仅来自 PID 复用，<1s 即保守阻塞）；路径净化无逃逸（`.ornnlab-` 前缀 + `/`→`_`）；resume 进程由 `--job-path` 精确匹配 + 操作表双层覆盖；job_name 映射同源（experiment_service.py:326）；兄弟阻塞仅残留于无 sidecar 场景（已文档化）；测试走真实分支。

### Main Agent Response（Round 4）

| Reviewer | Finding | 反例 | Severity | Decision | Authority | Closure Relation | Evidence / Reason | Action Taken |
|---|---|---|---|---|---|---|---|---|
| r4 | 无阻断性发现 | 全部挑战点在 fail-open 方向无法构造反例 | blocking | - | E2 实证 | n/a | 19/19 测试 + 静态推演 | 接受通过 |
| r4 | N1 sidecar 写失败冒泡 | ENOSPC/EACCES | non-blocking | accept | E4 | n/a | 写失败不应失败整个 run | 已加 try/except + `sidecar_write_failed` warn |
| r4 | N2 缺值不对称 | `--job-path` 缺值 False | non-blocking | accept | E4 | n/a | 对齐 fail-closed True | 已修 |
| r4 | N3 sidecar 死亡无日志 | 无法区分权威死亡与无进程 | non-blocking | accept | E4 | n/a | 排查依据 | 已加 `resume_lock_probe_sidecar_dead` |
| r4 | N4 命名函数重复 | 净化规则漂移 | non-blocking | accept | E4 | n/a | 共享 | webui_job_resume 改导入 harbor_subprocess 实现 |
| r4 | N5 N7/N8 遗留 | 双击 TOCTOU、孤儿清理 | non-blocking | defer | E4 | unrelated-existing-risk | 已有日志兜底 | 记录 |

### Review Governor (Round 4)
- Completed rounds before decision: 4（2 自动 + 2 用户批准）
- Unresolved blockers after round: 0
- Blockers closed: 全部 9 个阻断项（R1: B1-B4；R2: BLK-1/BLK-2；R3: B1/B2/B3）闭环
- New blocker classes: none；Repeated failure class: 进程探测历经 4 轮收敛为"PID 权威判定 + fail-closed 回退"
- Scope expansion: 用户批准的 PID 根治项（harbor_subprocess 修改 + sidecar 文件）已实施
- Governor decision: pass
- Decision reason: Round 4 无阻断；N1-N4 已硬化；全量门禁绿（pytest 238/4）

### Closure Status
- Blocking findings found: yes（R1: 4；R2: 2；R3: 3）
- Accepted blocking findings fixed: yes（全部）
- Blocking re-review completed: yes（R2/R3/R4）
- Blocking re-review passed: yes（R4 无阻断）
- Automatic round budget respected: yes（2/2 自动；R3/R4 用户批准）
- Third-or-later round explicitly user-approved before launch: yes（R3、R4）
- Scope drift detected: no（PID 根治为用户批准项）
- Control outcome: none
- Allowed to proceed: yes

## Final Conclusion

**PASS**。三轮自动+批准审查共发现并闭环 9 个阻断项（权限 chown、锁守卫、代理重建、凭证恢复、env 合并、进程探测 4 轮收敛到 PID 权威判定 + fail-closed 回退），Round 4 复审无阻断性发现，N1-N4 硬化项已实施。全量门禁绿（pytest 238/4、前端 34 files、smoke、launcher 27/27）。遗留非阻断项（N7 双击 TOCTOU、N8 孤儿进程清理机制）已记录待后续维护。审查闭环，任务可继续。
