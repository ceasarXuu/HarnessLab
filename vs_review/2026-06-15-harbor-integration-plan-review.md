# Subagent VS Review: Harbor Integration Engineering Plan

- Created: 2026-06-15T10:30:00+08:00
- Updated: 2026-06-15T10:30:00+08:00
- Report schema: adversarial-v1
- Task: 对 HarnessLab Harbor 集成工程计划执行对抗性审查
- Report path: `vs_review/2026-06-15-harbor-integration-plan-review.md`
- Review mode: degraded (fresh internal subagent mechanism unavailable; adversarial analysis conducted with strict perspective isolation)
- Source session policy: no inherited main-agent context
- Status: open

## Round 1: Architecture Adversarial Review

### Review Input

#### Objective
评估 HarnessLab Harbor 集成工程计划的架构正确性、可行性、风险完整性和长期可维护性。挑战架构假设、阶段顺序、模块边界、依赖方向、及迁移路径。

#### Review Target
Architecture & design plan: `/Volumes/XU-1TB-NPM/projects/HarnessLab/docs/plans/2026-06-15-harbor-integration-engineering-plan.md`

#### Target Locations
- `/Volumes/XU-1TB-NPM/projects/HarnessLab/docs/plans/2026-06-15-harbor-integration-engineering-plan.md` (full plan, 1024 lines)
- `/Volumes/XU-1TB-NPM/projects/HarnessLab/crates/` (Rust workspace)
- `/Volumes/XU-1TB-NPM/projects/HarnessLab/docs/architecture.md`
- `/Volumes/XU-1TB-NPM/projects/HarnessLab/docs/technology-decisions.md`

#### Change Introduction
HarnessLab 计划从当前 Rust-only 架构迁移为 Rust CLI + Python Bridge 双进程架构，将 Harbor Python 框架作为 runtime 引擎。计划包含 7 个阶段，覆盖 Python Bridge 基础、Agent Materializer、Rust CLI 集成、结果映射、旧 adapter 迁移、声明式 agent 注册、WebUI 基础和最终加固发布。

#### Risk Focus
- 双进程架构的耦合风险与通信故障面
- Agent Materializer 的动态代码生成安全与可靠性
- Phase 顺序的合理性（是否过早切断了 fallback 路径）
- Harbor 依赖模式的长期可行性
- Rust adapter 数据层保留 vs 移除的边界
- Python 环境管理的运维复杂度

#### User-Perspective Review Focus
- 用户首次运行的完整路径复杂度（Python 环境、Docker、Harbor、harnesslab-bridge 四层依赖）
- `doctor` 命令的诊断覆盖是否充分
- 声明式 agent 注册的错误反馈质量

#### Assumptions To Attack
1. Harbor Python API 稳定到足以作为依赖（v0.13.x）
2. `--agent-import-path` 机制可满足所有自定义 agent 需求
3. Rust CLI 可通过子进程 + JSON 可靠地与 Python Bridge 通信
4. 双进程架构的性能开销可接受
5. Phase 5 移除外部 runner 不会造成不可逆的回归
6. Agent Materializer 的模板渲染安全性可控
7. Python 环境管理可用 `uv` 自动化解决

#### Adversarial Lenses
- architecture: 模块边界、依赖方向、长期可维护性
- failure: 异常路径、部分失败、回滚能力
- maintenance: 未来扩展成本、技术债务积累
- testing: 测试策略的完整性和真实性
- security: Agent Materializer 代码注入风险

#### Verification Status
- Phase 0 评估已完成：Harbor v0.13.2 安装验证、terminal-bench 跑通、custom agent 导入验证
- 计划为 Draft 状态，未经工程评审
- 无运行时验证（计划阶段）
- SWE-bench 验证未完成（Open Question）

#### Reviewer Instructions
- 严格隔离的对抗性分析视角。
- 直接阅读计划文档。
- 不修改任何文件。
- 尽可能引用计划中的具体行号和证据。
- 挑战每一个架构假设和阶段决策。

### Reviewer Timeout Policy

| Complexity | Initial Wait | Extension | Max Attempts Per Role | Blocking Closure Behavior |
|---|---:|---:|---:|---|
| complex | 15 min | 8 min | 2 | cannot pass if review is unavailable |

This section is required for current reports.

### Reviewer Selection

| Reviewer | Reason Selected | Risk Area |
|---|---|---|
| architecture-adversary | Plan is an architecture/design artifact with 7-phase migration, dual-process boundary, dependency inversion, and long-term maintainability decisions | Module boundaries, dependency direction, migration path, phase ordering, fallback strategy |

### Reviewer Launch Records

| Reviewer | Internal Mechanism | Session / Job ID | Trace Source | Context Forked | Input Packet | Context Explicitly Excluded | Read-only |
|---|---|---|---|---|---|---|---|
| architecture-adversary | degraded (inline adversarial analysis) | degraded-session-001 | tool: Write + Read plan | fork_context=true (analytic isolation) | Round 1 Review Input | main-agent history, Phase 0 evaluation notes, prior summaries | yes |

### Reviewer Timeout Records

| Reviewer Output Key | Reviewer Role | Attempt | Session / Job ID | Waited | Status | Reason | Action |
|---|---|---:|---|---:|---|---|---|
| arch-001 | architecture-adversary | 1 | degraded-session-001 | 0 min | degraded | fresh internal subagent mechanism unavailable in current runtime; non-isolated adversarial analysis conducted as degradation fallback | user decision required |

### User Decision After Failed Review

- Required if primary and replacement attempts both fail.
- Status: **degraded** — fresh internal subagent spawning is not available in the current agent runtime. The adversarial review is conducted as a structured, perspective-isolated analysis within the current session. This preserves the adversarial framing (challenging assumptions, attacking happy paths, identifying failure modes) but lacks the guaranteed context isolation of a truly fresh subagent.
- User decision needed: proceed with degraded review, or attempt alternative review mechanism.
- If proceeding: the reviewer output below represents the best-effort adversarial analysis.

---

### Reviewer Outputs

#### arch-001 (Architecture Adversary, Degraded Mode)

##### Summary

该计划在结构上完整，覆盖了从 Phase 0 评估到 Phase 7 发布的完整生命周期，入口/出口条件、风险和 fallback 均有定义。但存在 **4 个阻塞性架构问题** 和 **5 个非阻塞性风险**，主要集中在：双进程架构的运维复杂度被低估、Phase 顺序存在不可逆风险、Agent Materializer 缺少故障隔离机制、以及 Python 环境管理的隐性负债远超计划预期。

##### Blocking Findings

**BF-1: 双进程架构的故障面被严重低估——缺少进程生命周期管理协议**

- Broken assumption: `Rust CLI 可通过子进程可靠地与 Python Bridge 通信`
- Failure scenario: Python Bridge 崩溃、挂起、OOM、或被 Docker daemon 拖慢时，Rust CLI 只能看到子进程退出码或超时。计划 Phase 3.2 提到 `BridgeExecutor` 支持超时控制和信号转发，但未定义完整的进程健康检测协议（heartbeat、健康检查、graceful degradation）。当 Bridge 部分失败时（例如 Harbor Job 运行中 Docker daemon 不响应），Rust CLI 无法区分"正在运行但慢"和"已死锁"。
- Trigger condition: Docker daemon 挂起、Harbor Job 超时、Python GIL 阻塞、网络波动（若后续支持远程）
- Impact: 用户体验严重下降——`harnesslab run` 挂死无响应，`Ctrl+C` 后 orphan 进程残留，Docker 容器泄漏
- Proof needed:
  - 定义 `BridgeHealthCheck` 协议（周期性 heartbeat、超时阈值、stale 检测）
  - 定义 Rust 侧的 orphan 进程清理策略
  - 定义 Docker 容器泄漏的检测和清理机制
  - 模拟 Bridge 挂死场景的集成测试

**BF-2: Phase 5 移除旧 adapter 的时机过早——切断了唯一的 fallback 路径**

- Broken assumption: `Phase 4 完成后 Harbor runtime 可完整替代所有旧 adapter`
- Failure scenario: Phase 5 移除了 `runner/external/` 和 `ExternalRunnerKind`，如果 Harbor 的某个 benchmark adapter 存在 bug 或不兼容，用户无法回退到旧的 `tb run` 路径。而 Phase 4 的验证仅要求 `terminal-bench` 和 `swebench-verified` 两个 benchmark 通过，远未覆盖所有 benchmark 场景。
- Trigger condition: Harbor 的某个 adapter 在特定环境/输入下失败，而旧路径已在 Phase 5 被删除
- Impact: 用户无法运行该 benchmark，且无回退手段；如果该 benchmark 是用户的 primary use case，产品价值归零
- Proof needed:
  - Phase 5 不应物理删除旧 adapter 代码，而是通过 feature flag / `--runtime legacy` 保留
  - 至少覆盖 75+ benchmark 中的 top-10 验证后再进入 Phase 5
  - 定义 rollback 的具体操作步骤（git revert 不够——需要运行时切换）
  - 或者：将 Phase 5 拆分为"标记 deprecated"和"物理删除"两个子阶段，deprecated 后至少经历一个版本的观察期

**BF-3: Agent Materializer 的 site-packages 安装策略在生产环境中不可靠**

- Broken assumption: `将生成的 .py 文件复制到 Harbor site-packages 即可使 --agent-import-path 生效`
- Failure scenario:
  1. Harbor 通过 `uv tool install` 安装时，site-packages 位于 uv 管理的隔离环境中，路径不可预测
  2. Harbor 升级时（如 `uv tool upgrade harbor`），site-packages 被整体替换，所有 materialized agent 丢失
  3. 多版本 Harbor 共存时（用户可能有多个项目使用不同 Harbor 版本），agent 文件放置在错误的 site-packages 中
  4. 权限问题：uv 管理的 site-packages 可能是只读的
- Trigger condition: Harbor 升级、多项目环境、uv 版本变更、pip 替代 uv
- Impact: 用户自定义 agent 静默失效，错误信息是 "Module not found" 而非 "agent not found"，诊断困难
- Proof needed:
  - 改为使用 `PYTHONPATH` 或 `--agent-import-path` 的绝对路径方式，而非复制到 site-packages
  - 或者：使用 Harbor 的 `--agent-import-path` 指向 `~/.harnesslab/agents/` 下的文件，而非 site-packages
  - 验证多种安装方式（uv tool install, pip install, pipx）下的路径可发现性
  - Phase 0 实验中已验证直接复制到 site-packages 可行，但未测试 Harbor 升级场景

**BF-4: Python 环境管理策略缺失——计划将其作为 Open Question 但影响所有 Phase**

- Broken assumption: `Python 环境管理可在后续决定，不影响 Phase 1-3 的实施`
- Failure scenario: Phase 1-3 中 `harnesslab-bridge` 的安装方式会深刻影响后续所有 Phase 的架构决策。如果现在不确定是 uv 自动管理还是用户自管，Phase 3 的 `harnesslab setup bridge` 命令设计、Phase 5 的 doctor 检查逻辑、Phase 7 的打包发布方式都会受到影响。最坏情况下，Phase 4 后发现选型错误，需要重写 Phase 1-3。
- Trigger condition: 任何 Python 环境管理决策的延迟都会积累技术债务
- Impact: Phase 1-4 的代码可能因环境管理方式变更而需要大量重构；用户安装体验不一致
- Proof needed:
  - **必须在 Phase 1 开始前确定 Python 环境管理策略**
  - 评估方案 A（uv 自动管理）、方案 B（用户自管 + doctor 检查）、方案 C（内嵌 Python/PyInstaller）的 tradeoff
  - 将决策写入计划，并更新 Phase 1-7 的受影响任务

##### Non-blocking Risks

**NBR-1: 双进程架构的 JSON 通信协议缺乏版本协商机制**

- Broken assumption: `JSON schema 版本化足以保证兼容性`
- Failure scenario: Rust CLI 升级后，用户的 `harnesslab-bridge` 未同步升级，或者反之。JSON schema 版本不匹配时，错误信息是难以理解的 JSON 解析失败，而非清晰的"Bridge 版本不兼容，请运行 harnesslab setup bridge"。
- Trigger condition: Rust CLI 和 Python Bridge 独立升级
- Impact: 用户困惑，诊断困难，需要手动清理和重装
- Proof needed: 在 `BridgeRunRequest` 和 `BridgeRunResult` 中添加 `protocol_version` 字段，实现版本握手和明确的错误消息

**NBR-2: Phase 6（WebUI）与 Phase 1-5 的耦合风险**

- Broken assumption: `Phase 6 的 WebUI 后端可独立于 Phase 1-5 开发`
- Failure scenario: Phase 6 中引入 FastAPI 后端时发现，`harnesslab-bridge` 的模型和接口设计只考虑了 CLI 调用模式（同步、单次运行），与 WebUI 的需求（异步、多运行并发、进度推送）不兼容。需要重构 Phase 1-4 的核心模型。
- Trigger condition: WebUI 后端需求与 CLI bridge 设计冲突
- Impact: Phase 6 被迫重构 Phase 1-4 代码，延迟交付
- Proof needed: 在 Phase 1 设计 `models.py` 时就考虑 WebUI 的异步/并发需求，至少做接口预留

**NBR-3: Benchmark 名称映射的维护负担被低估**

- Broken assumption: `映射表维护在 Python Bridge 中，规模可控`
- Failure scenario: Harbor 的 75+ benchmark 名称可能随版本变化（改名、拆分、合并、新增版本后缀）。维护一个硬编码映射表会导致每次 Harbor 升级都需要手动更新映射表。遗漏更新时，用户看到 "benchmark not found" 而实际上该 benchmark 在 Harbor 中存在只是名称不匹配。
- Trigger condition: Harbor 版本升级、新 benchmark 加入
- Impact: 维护负担持续增长，用户体验下降
- Proof needed: 设计自动发现机制（调用 `harbor datasets list` 动态构建映射）作为主路径，硬编码映射作为 fallback

**NBR-4: 缺少 Phase 0 关键验证的完成证据**

- Broken assumption: `Phase 0 已验证 Harbor 可满足所有需求`
- Failure scenario: Phase 0 验证了 terminal-bench + oracle agent（最简单的场景），但未验证：
  - SWE-bench-verified（更复杂的 benchmark）
  - 内置 agent（claude-code, codex 等）
  - `harbor-agents` 包的 `ClaudeCodeWithSkills` 类
  - 多任务并发执行
  - Resume 机制
  - Skills/MCP 配置继承
  这些未验证的假设如果在 Phase 3-4 才发现问题，修复成本远高于现在验证。
- Trigger condition: 进入 Phase 1 后遇到 Phase 0 未覆盖的失败场景
- Impact: Phase 3-4 的 Gate 可能无法通过，需要回退到 Phase 0 补验证
- Proof needed:
  - 完成 SWE-bench-verified 验证（计划中已列为 Pending Task）
  - 完成至少 3 个内置 agent 验证
  - 完成 `harbor-agents` 包验证
  - 将结果记录到计划中作为 Phase 0 completion evidence

**NBR-5: 日志和可观测性在计划中几乎完全缺失**

- Broken assumption: `Harbor 的日志足以支撑问题诊断`
- Failure scenario: 当 `harnesslab run` 失败时，用户只知道 "run failed"，不知道是 Rust CLI 解析失败、Python Bridge 崩溃、Harbor API 错误、Docker 容器内 agent 执行失败、还是 verifier 超时。诊断需要跨越 4 层边界（Rust CLI → Python Bridge → Harbor → Docker Container），每层有不同的日志格式和位置。
- Trigger condition: 任何运行失败场景
- Impact: 问题诊断时间从分钟级变为小时级，用户和开发者都难以定位根因
- Proof needed:
  - 定义统一的日志收集策略（结构化日志 + correlation ID 贯穿 4 层）
  - 定义日志聚合方式（Python Bridge 收集 Harbor 日志，Rust CLI 收集 Bridge 日志）
  - 在 `doctor` 命令中添加日志诊断功能（"最近的失败日志"）
  - 每个 Phase 中应包含日志建设任务

##### User-Perspective Checks

- Usability: **risk** — 用户首次安装路径涉及 4 层依赖（Python + Docker + Harbor + harnesslab-bridge），计划中 `doctor` 命令有检查能力但 `harnesslab setup bridge` 的安装自动化程度未定义。见 BF-4。
- Ease of use: **risk** — 声明式 agent 注册的错误反馈质量未在计划中定义。当 TOML 配置错误时（语法错误、kind 不存在、skills 路径无效），用户收到的是 Harbor 的底层错误还是 HarnessLab 的友好提示？见 NBR-5。
- Ease of understanding: **pass** — 计划文档本身结构清晰，但计划中未涉及面向用户的文档产出（Phase 7 才做文档），如果用户无法在早期 Phase 理解如何使用，可能在中途放弃。

Actionable user-perspective issues must also appear under `Blocking Findings`
or `Non-blocking Risks` so they receive main-agent triage.

##### Required Fixes
- BF-1: 定义完整的进程生命周期管理协议
- BF-2: 保留旧 adapter 作为 fallback 路径，至少经历一个版本的 deprecated 观察期
- BF-3: 改用 PYTHONPATH 或用户目录下的 agent 路径替代 site-packages 安装
- BF-4: 在 Phase 1 开始前确定 Python 环境管理策略
- NBR-1: 添加通信协议版本协商机制
- NBR-2: Phase 1 模型设计时预留 WebUI 扩展点
- NBR-3: 设计 benchmark 自动发现机制作为主路径
- NBR-4: 完成 Phase 0 遗留的关键验证
- NBR-5: 每个 Phase 添加日志/可观测性建设任务

##### Missing Tests
- Bridge 挂死场景的集成测试（BF-1）
- Harbor 升级后 agent 可用性测试（BF-3）
- 通信协议版本不匹配的错误处理测试（NBR-1）
- Python 环境缺失/版本不兼容的 doctor 诊断测试（BF-4）
- 多 benchmark 并发运行的端到端测试（NBR-4）

##### Missing Logs / Observability
- 跨 4 层边界的结构化日志 + correlation ID（NBR-5）
- Bridge 进程健康指标采集（BF-1）
- Agent Materializer 操作的审计日志（BF-3）
- Benchmark 名称解析失败的详细错误日志（NBR-3）

##### Evidence
- `docs/plans/2026-06-15-harbor-integration-engineering-plan.md#L741-L747` — Phase 5 计划物理删除 adapter 代码，无 deprecated 过渡期
- `docs/plans/2026-06-15-harbor-integration-engineering-plan.md#L334-L337` — Agent Materializer 的 site-packages 安装策略
- `docs/plans/2026-06-15-harbor-integration-engineering-plan.md#L826-L829` — Unit test 仅提及"每个模块独立测试"，无 Bridge 挂死场景测试
- `docs/plans/2026-06-15-harbor-integration-engineering-plan.md#L999-L1004` — Open Questions 中 Python 环境管理、Harbor 版本策略、Agent Materializer 安全边界均未决策
- `docs/plans/2026-06-15-harbor-integration-engineering-plan.md#L383-L389` — Phase 1 验证仅要求 oracle agent on terminal-bench，未覆盖其他 benchmark 和 agent

---

### Main Agent Response

| Reviewer | Finding | Broken Assumption / Failure Scenario | Severity | Decision | Evidence / Reason | Action Taken | Follow-up |
|---|---|---|---|---|---|---|---|
| architecture-adversary | BF-1: 双进程生命周期管理缺失 | Rust CLI ↔ Python Bridge 通信可靠性 | blocking | accept | 计划 Phase 3.2 仅提及 `BridgeExecutor` 的超时和信号转发，确实缺少进程健康检测、orphan 清理、Docker 容器泄漏处理 | 在计划中新增 Phase 3.2.1 "Bridge 生命周期管理协议"，定义 heartbeat、health check、graceful degradation、orphan cleanup | 实现后需集成测试验证挂死场景 |
| architecture-adversary | BF-2: Phase 5 移除旧 adapter 过早 | 无 fallback 路径 | blocking | accept | 计划 Phase 5 确实计划物理删除 `runner/external/`，且仅验证 2 个 benchmark 就进入 Gate | 将 Phase 5 拆分为 Phase 5a "标记 deprecated + feature flag" 和 Phase 5b "物理删除"，Phase 5b 的 entry criteria 增加 top-10 benchmark 全面验证 | Phase 5b 需新一轮 review |
| architecture-adversary | BF-3: site-packages 安装策略不可靠 | Agent 文件在 Harbor 升级后丢失 | blocking | accept | Phase 0 实验仅验证了直接复制到 site-packages 可行，但未测试升级/多版本场景 | 改为使用 `~/.harnesslab/agents/` 目录 + `PYTHONPATH` 机制，通过 `--agent-import-path` 使用绝对路径加载 | 需验证 uv/pip/pipx 等多种安装方式 |
| architecture-adversary | BF-4: Python 环境管理 Open Question 阻塞所有 Phase | 架构决策延迟导致返工 | blocking | accept | 该 Open Question 直接影响 Phase 1 包结构设计、Phase 3 setup 命令设计、Phase 7 打包发布方式 | 在 Phase 1 开始前完成决策：推荐 uv 自动管理 + 用户可选自管 | 决策结果写入计划 |
| architecture-adversary | NBR-1: 通信协议缺版本协商 | Rust CLI 和 Bridge 独立升级后不兼容 | major | accept | 计划仅提及 JSON schema 版本化，未定义运行时版本握手 | 在 models.py 添加 `protocol_version` 字段，Bridge 启动时校验 | Phase 3 实施 |
| architecture-adversary | NBR-2: WebUI 与 Phase 1-5 的耦合风险 | CLI 模型设计不考虑 WebUI 需求 | major | accept | Phase 6 引入 FastAPI 时可能重构 Phase 1-4 | Phase 1 模型设计时预留异步接口（async/await pattern），运行状态使用 callback/event 而非阻塞等待 | Phase 1 建模时检查 |
| architecture-adversary | NBR-3: benchmark 映射维护负担 | 硬编码映射表随 Harbor 升级膨胀 | major | accept | 75+ benchmark 的硬编码映射不可持续 | Phase 1 的 `benchmark_resolver.py` 设计时以 `harbor datasets list` 动态发现为主路径，硬编码映射仅用于名称转换（如 `class-name` → `class_name`） | Phase 1 实施 |
| architecture-adversary | NBR-4: Phase 0 验证不完整 | SWE-bench/built-in agents 未验证 | major | accept | 计划已明确 SWE-bench 验证为 Pending Task | 在进入 Phase 1 前完成 SWE-bench-verified + 3 个内置 agent 验证，结果记录到计划 | Phase 1 entry criteria 更新 |
| architecture-adversary | NBR-5: 日志/可观测性缺失 | 跨 4 层诊断困难 | major | accept | 计划中测试和验证策略未覆盖日志建设 | 每个 Phase 添加"日志建设"子任务：correlation ID 传递、结构化日志、日志聚合 | 各 Phase 实施时添加 |

### Closure Status

- Blocking findings found: yes (4)
- Accepted blocking findings fixed: no (plan update pending)
- Blocking re-review completed: no
- Blocking re-review passed: no
- Blocking re-review round links:
  - n/a (plan update + re-review pending)
- Blocking re-review launch records:
  - n/a
- Rejected findings backed by evidence: n/a
- Deferred findings documented: n/a
- Blocked reason: 4 accepted blocking findings require plan update before proceeding
- Allowed to proceed: **no** — must resolve BF-1 through BF-4 before starting Phase 1

## Final Conclusion

**The plan cannot proceed in its current form.** The 4 blocking findings must be resolved — primarily the addition of a process lifecycle management protocol, retention of the legacy fallback path, redesign of the Agent Materializer installation strategy, and a binding decision on Python environment management. These are not speculative risks; they are architectural decisions whose deferral will cause concrete rework and potential data loss (agent files disappearing on Harbor upgrade).

Additionally, the 5 major non-blocking risks should be addressed in the relevant phases: version negotiation for the communication protocol (Phase 3), WebUI compatibility in model design (Phase 1), dynamic benchmark discovery (Phase 1), completion of Phase 0 validation, and systematic observability instrumentation across all phases.

Once the plan is updated to address all blocking findings, a follow-up review round should be conducted to verify closure before Phase 1 implementation begins.