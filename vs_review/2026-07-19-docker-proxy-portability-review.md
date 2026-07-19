# Subagent VS Review: Docker proxy portability

- Created: 2026-07-19T09:10:00+08:00
- Updated: 2026-07-19T09:10:00+08:00
- Report schema: adversarial-v1
- Task: 审查 OrnnLab 自动继承宿主代理的实现是否适用于其他位置和设备，而非仅适配当前 Ubuntu + Clash。
- Report path: `vs_review/2026-07-19-docker-proxy-portability-review.md`
- Review mode: fresh internal subagents
- Source session policy: no inherited main-agent context
- Status: open

## Round 1: portability architecture challenge

### Review Input

#### Objective
验证自动宿主代理继承是否采用 capability-driven 的跨设备架构，并找出会在其他部署环境中失败或产生安全问题的假设。

#### Review Target
已实现的代理 runtime、Harbor 注入链、生命周期、测试与部署文档。

#### Target Locations
- `ornnlab/services/container_proxy_runtime.py`
- `ornnlab/services/harbor_engine.py`
- `ornnlab/services/harbor_subprocess.py`
- `ornnlab/services/experiment_service.py`
- `ornnlab/app.py`
- `tests/python/test_container_proxy_runtime.py`
- `docs/releases/v1.0.5/technical-design.md`

#### Change Introduction
实现会读取标准 proxy env；非回环地址通过子进程 env 继承；回环地址查询 Docker default bridge gateway、在该地址启动临时 TCP relay，并把模板作为 Agent env 默认值写入 Harbor config。

#### Risk Focus
- Docker Desktop、rootless Docker、远程 context、非默认 bridge、IPv6、不同 proxy scheme 与 Harbor engine 的错误假设。
- relay 暴露面、凭据、网络 allowlist 绕过、并发和生命周期。

#### User-Perspective Review Focus
- 用户是否能在新设备零手工配置使用；不支持时是否得到可理解、可恢复的错误和关闭方式。

#### Implementation Completeness Focus
- 生产执行路径是否真的按 daemon capability 选择策略，而非测试或文档声称支持。
- 测试是否覆盖平台分类、降级和真实 Docker 边界。

#### Target Benefit Focus
- 声称的“跨设备自动继承”是否有支持矩阵、平台对照证据和回归检查；当前基线仅为 Ubuntu rootful Docker。

#### Assumptions To Attack
- `docker network inspect bridge` 的 gateway 一定属于运行 OrnnLab 的宿主并可 bind。
- Docker daemon 一定是本地 rootful Linux。
- Agent-only env 足以等价于“容器继承代理”。
- subprocess-only 限制可被视为通用能力。
- proxy env 大小写、scheme、认证和 network policy 行为一致。

#### Adversarial Lenses
- architecture
- implementation-completeness
- security
- failure
- maintenance
- testing
- observability

#### Verification Status
- 当前 Ubuntu rootful Docker + Clash 自动 relay HTTP 200。
- Python、前端、launcher 和启停回归通过。
- 尚无 Docker Desktop、rootless、远程 daemon 的运行证据。

#### Reviewer Instructions
- Fresh internal subagent session.
- No inherited main-agent context.
- Read target files directly.
- Do not modify files.
- Cite evidence paths and line numbers when possible.
- Try to falsify portability and production-completeness claims; do not confirm the implementation by default.

### Internal Subagent Unavailable Fallback

- Internal subagent unavailable reason: n/a
- Local CLI discovery commands: n/a
- Discovered CLI candidates: n/a
- User-recommended alternative agent requested: n/a
- User-recommended agent command: n/a
- User-recommended agent verification: n/a
- User approval requested: n/a
- User-approved CLI command: n/a
- User decision: n/a
- Fallback outcome: n/a

### Reviewer Timeout Policy

| Complexity | Initial Wait | Extension | Max Attempts Per Role | Blocking Closure Behavior |
|---|---:|---:|---:|---|
| high-risk | 20 minutes | one 10 minute extension | 2 | cannot pass if review is unavailable |

### Reviewer Selection

| Reviewer | Reason Selected | Risk Area |
|---|---|---|
| architecture-adversary | 最高风险是把单机网络拓扑固化为产品架构，并对其他 daemon 类型作出错误承诺 | 跨平台边界、依赖方向、长期扩展与安全降级 |

### Reviewer Launch Records

| Reviewer | Internal Mechanism | Session / Job ID | Trace Source | Context Forked | Input Packet | Context Explicitly Excluded | Read-only |
|---|---|---|---|---|---|---|---|
| architecture-adversary | internal fresh subagent | `/root/proxy_portability_arch_review` | `spawn_agent` | fork_context=false | Round 1 Review Input | main-agent history, reasoning, drafts, conclusions, full diff | yes |

### Reviewer Timeout Records

| Reviewer Output Key | Reviewer Role | Attempt | Session / Job ID | Waited | Status | Reason | Action |
|---|---|---:|---|---:|---|---|---|
| round1-architecture | architecture-adversary | 1 | `/root/proxy_portability_arch_review` | < 20 minutes | completed | output received | triaged below |

### Reviewer Outputs

Verdict: **FAIL / BLOCK**。

Blocking findings:

1. daemon 端 `bridge` gateway 被错误当作 OrnnLab OS 可绑定地址；remote、Desktop、
   rootless 的网络边界不重合。
2. 显式 Agent/Environment proxy 在自动 policy 之后才参与合并，不能真正恢复失败。
3. relay 无认证且跨 Job 常驻，扩大同 daemon 容器的代理使用窗口。

Important findings:

4. 硬编码 default `bridge`，自定义网络与无 bridge 缺少证据。
5. 非回环 URL 未证明目标容器 DNS/路由可达。
6. bind `OSError` 和上游不可用未稳定分类。
7. IPv6 relay URL netloc 缺少方括号。
8. relay cache key 缺少 scheme。

Completeness/test/operations gaps: 未区分 Agent env 与 daemon build/pull/verifier；无平台
矩阵、真实 custom network、policy 回收和 target 结构化日志。

### Main Agent Response

| Finding | Disposition | Response / Evidence |
|---|---|---|
| 1 | accept-blocking | 新增 `docker_proxy_target.py`，遵循有效 Context/`DOCKER_HOST`，读取 daemon OS/security options，区分 local rootful、Desktop、rootless、remote/virtualized；仅 local rootful 候选可继续 bind capability check。 |
| 2 | accept-blocking | `ExperimentService` 先取得 Agent/Environment 显式 env 名，再传给 runtime 按大小写组裁剪；被覆盖组不解析、不探测 target、不建 relay。 |
| 3 | accept-blocking-partial | relay 改为每 Job 独立 policy，准备失败、取消、运行结束立即回收；接受资源窗口问题。拒绝将 Environment allow-host 解释为代理 ACL：Agent 获得 proxy env 本身就是明确出网授权，direct proxy 同样绕过目标级 host 限制；同 daemon 不可信 sibling 风险写入设计和部署文档，建议显式受控代理并关闭 auto。 |
| 4 | accept-important | 无 default gateway 或 gateway 不可 bind 时稳定失败；真实 user-defined bridge 容器经 host relay 请求 HTTP 200。未声称任意 network plugin 均支持，unsupported 使用显式容器可达 proxy。 |
| 5 | defer-with-warning | 非回环 URL 本身是独立寻址配置，跨 daemon 可能可达也可能不可达；OrnnLab 无权拉取探针镜像或假定业务目标，因此不做静默预检。文档明确 DNS/路由由目标容器网络保证。 |
| 6 | accept-important | target discovery、unsupported、bind 和 loopback upstream preflight 均包装为 `ProxyConfigurationError`；测试断言稳定分类。 |
| 7 | accept-important | IPv6 relay netloc 使用方括号；地址族的实际选择仍以 daemon host gateway 为准。 |
| 8 | accept-minor | policy 内 relay key 改为 `(scheme, host, port)` 并补不同 scheme 回归。 |

验证：Python `144 passed, 3 skipped`；前端 `108 passed`；pyright 0；scoped ruff
通过；生产 build 与 polling Storybook smoke 通过；default 和 user-defined Docker bridge
真实请求 Claude bootstrap 均 HTTP 200，临时网络已移除。

### Closure Status

- Blocking findings found: yes
- Accepted blocking findings fixed: yes
- Blocking re-review completed: yes, Round 4
- Blocking re-review passed: yes, PASS WITH FOLLOW-UPS
- Blocking re-review round links:
  - Round 2 below
- Blocking re-review launch records:
  - `/root/proxy_portability_closure_review`
  - `/root/proxy_portability_final_closure`
  - `/root/proxy_allowlist_final_review`
- Rejected findings backed by evidence: yes, proxy ACL/allow-host interpretation documented above
- Deferred findings documented: yes, non-loopback container reachability and non-Agent phases
- Implementation completeness gaps resolved or accepted by user: Agent-only scope documented; closure pending
- Target benefit warnings recorded: yes
- Blocked reason: none
- Allowed to proceed: yes

## Round 2: blocking closure review

### Review Input

#### Objective
验证 Round 1 接受的阻断问题是否真正闭环，并尝试证明新 capability 分类、显式优先和
policy 生命周期仍有跨设备或并发漏洞。

#### Review Target
当前未提交工作树中的代理 runtime、Docker target provider、Experiment 调用顺序、测试、
PRD/技术设计/playbook，以及 Round 1 triage。

#### Target Locations
- `ornnlab/services/docker_proxy_target.py`
- `ornnlab/services/container_proxy_runtime.py`
- `ornnlab/services/experiment_service.py`
- `tests/python/test_docker_proxy_target.py`
- `tests/python/test_container_proxy_runtime.py`
- `tests/python/test_experiment_service.py`
- `docs/releases/v1.0.5/prd.md`
- `docs/releases/v1.0.5/technical-design.md`
- `docs/playbooks/development-operations.md`
- 本报告 Round 1 Main Agent Response

#### Required Closure Checks
1. remote/Desktop/rootless 是否还可能进入宿主 bind 分支；Context/环境覆盖顺序是否正确。
2. 显式 Agent/Environment proxy 是否在所有失败点之前裁剪自动组。
3. 每个正常、异常、取消、pre-running return 是否都会关闭 policy；并发 Job 是否互相关闭。
4. bind/upstream/target discovery 是否都稳定归类且不泄露 proxy URL/凭据。
5. 文档是否诚实区分已支持能力、unsupported 路径与 Agent-only 范围。
6. Round 1 finding 3 的部分拒绝是否有足够证据，剩余风险是否仍应阻断。

#### Verification Evidence
- scoped ruff pass；pyright 0。
- Python `144 passed, 3 skipped`；frontend `108 passed`；build pass；Storybook smoke pass。
- current rootful Linux default bridge 和 disposable user-defined bridge HTTP 200。
- Desktop/rootless/remote 为纯能力矩阵测试，无对应设备实测；目标行为是明确拒绝 loopback relay。

#### Reviewer Instructions
- Fresh internal subagent session; no inherited main-agent context.
- Read files directly; do not modify files.
- Cite file:line evidence.
- Only pass accepted blocking closure if production paths, failure paths and tests support it.

### Reviewer Selection

| Reviewer | Reason Selected | Risk Area |
|---|---|---|
| architecture-adversary | blocking closure 必须由 fresh 同类 adversary 重新攻击，不能复用 Round 1 session | capability 边界、生命周期、拒绝风险 |

### Reviewer Launch Records

| Reviewer | Internal Mechanism | Session / Job ID | Trace Source | Context Forked | Input Packet | Context Explicitly Excluded | Read-only |
|---|---|---|---|---|---|---|---|
| architecture-adversary | internal fresh subagent | `/root/proxy_portability_closure_review` | `spawn_agent` | fork_context=false | Round 2 Review Input | main-agent history, reasoning, drafts, conclusions, full diff | yes |

### Reviewer Output

Verdict: **FAIL-BLOCK**。

Blocking:

1. `DOCKER_CONTEXT` 与 `DOCKER_HOST` 同时存在时分类器错误优先 `DOCKER_HOST`，但后续
   Docker CLI 按 Context 查询，可能混合两个 daemon 的 endpoint/info/gateway。
2. `prepare_policy()` 创建第一个 relay 后在第二个 await 被取消时，`CancelledError`
   不进入 `except Exception`，listener 泄漏；reviewer 已内联复现。
3. 默认 auto proxy 与 `extra_allowed_hosts` 白名单语义冲突；显式 proxy 的风险不能证明
   默认自动授权可接受。

Important: direct 非回环 proxy 不应因 Docker discovery 失败而失败；非法 gateway 和
`wait_closed` 异常分类仍有缺口。Required Closure Checks 结果为
`FAIL, PASS, FAIL, PASS-with-gaps, FAIL, FAIL`。

### Main Agent Closure Response

| Finding | Disposition | Closure repair / evidence |
|---|---|---|
| target 优先级混用 | accept-blocking | `DOCKER_CONTEXT` 明确覆盖 `DOCKER_HOST`；无 Context 时 Host 覆盖默认 Context。所有 info/network 命令显式带同一个 `--context` 或 `--host`，并新增冲突矩阵测试。 |
| preparation cancellation 泄漏 | accept-blocking | 单独捕获 `asyncio.CancelledError`，先取消 active connection tasks、再等待 listener close；复现测试断言 listener 集合归零且端口拒绝连接。 |
| allow-host 默认授权冲突 | accept-blocking | 撤回 Round 1 的部分拒绝。Agent 存在 `extra_allowed_hosts` 时整体跳过默认 auto，并记录 `docker_proxy_policy_skipped`；只有用户显式 Profile proxy 可与白名单共同存在。PRD/设计/playbook 同步。 |
| direct proxy discovery | accept-important | 非回环 URL 不再调用 Docker target resolver；测试使用会主动抛错的 resolver 证明独立路径。 |
| 非常规错误分类 | accept-important | gateway 在 bind 前解析并包装；preflight close 的 `OSError` 被安全抑制；非法 gateway 测试断言无 listener。 |
| 生命周期/并发测试缺口 | accept-important | 新增 active connection 强制结束、两个 concurrent policy 独立关闭、config build 失败 policy close 与正常运行 close 测试。 |

修后证据：targeted `33 passed`；全 Python `152 passed, 3 skipped`；真实 disposable
user-defined bridge HTTP 200 且临时网络已删除。所有代码文件仍少于 500 行。

## Round 3: second blocking closure review

### Review Input

#### Objective
对 Round 2 新发现的三个阻断问题做 fresh closure，重点验证 daemon 查询一致性、取消
事务清理和 allow-host 默认安全语义，不复用前两轮结论。

#### Review Target
- `ornnlab/services/docker_proxy_target.py`
- `ornnlab/services/container_proxy_runtime.py`
- `ornnlab/services/experiment_service.py`
- `tests/python/test_docker_proxy_target.py`
- `tests/python/test_container_proxy_runtime.py`
- `tests/python/test_experiment_service.py`
- `docs/releases/v1.0.5/prd.md`
- `docs/releases/v1.0.5/technical-design.md`
- `docs/playbooks/development-operations.md`
- 本报告 Round 2 reviewer output 与 closure response

#### Required Closure Checks
1. `DOCKER_CONTEXT`/`DOCKER_HOST` 优先级及 endpoint/info/network daemon identity 一致。
2. preparation cancellation 在任意 await 点不会留下 listener 或 active connection。
3. `extra_allowed_hosts` 存在时默认自动 proxy 确实不进入 artifact/child env，显式配置除外。
4. direct proxy 不依赖 Docker discovery；loopback unsupported 仍明确失败。
5. 正常、builder error、active connection、concurrent policy 的释放测试对应生产路径。
6. 日志、错误和文档不夸大 Desktop/rootless/remote 实测支持。

#### Verification Evidence
- scoped ruff/pyright pass；targeted 33 passed。
- full Python `152 passed, 3 skipped`。
- current local rootful user-defined bridge HTTP 200；临时网络已删除。
- Desktop/rootless/remote 目标为 fail-safe 模拟矩阵，无设备实测支持声明。

#### Reviewer Instructions
- Fresh internal subagent; fork_context=false; read-only.
- Read current files, not cached line numbers.
- Cite file:line evidence and attempt cancellation/precedence counterexamples.
- Return PASS only if all accepted blocking findings are closed.

### Reviewer Selection

| Reviewer | Reason Selected | Risk Area |
|---|---|---|
| architecture-adversary | Round 2 新增 blocking 必须由第三个 fresh session 独立闭环 | daemon identity、异步资源安全、网络策略 |

### Reviewer Launch Records

| Reviewer | Internal Mechanism | Session / Job ID | Trace Source | Context Forked | Input Packet | Context Explicitly Excluded | Read-only |
|---|---|---|---|---|---|---|---|
| architecture-adversary | internal fresh subagent | `/root/proxy_portability_final_closure` | `spawn_agent` | fork_context=false | Round 3 Review Input | main-agent history, reasoning, drafts, conclusions, full diff | yes |

### Reviewer Output

Verdict: **FAIL-BLOCK**。

Checks 1、2、4、5、6 PASS；check 3 FAIL。Environment Profile 的白名单实际编译到
`overrides.environment.extra_allowed_hosts`，而 Round 2 修复只检查 Agent config，导致
Environment 白名单下 `automatic_proxy_allowed=True`。Reviewer 使用生产路径 harness
复现自动 HTTPS proxy defaults 仍进入 builder。另指出日志 reason 与文档错误写成
Agent-only。

### Main Agent Closure Response

接受 blocking。新增 `_automatic_proxy_allowed(agent_config, overrides)` 统一检查 Agent 与
Environment 两个来源；`ExperimentService` 只调用该函数。新增 Environment override →
ExperimentService → runtime 的集成回归，捕获 `automatic_proxy_allowed=False`，以及两个
来源的函数矩阵测试。PRD、技术设计、playbook 与结构化日志 reason 均改为通用
network allowlist。

## Round 4: final allowlist closure review

### Review Input

#### Objective
只验证 Round 3 唯一 blocking：Agent/Environment 任一网络白名单是否都能在生产数据流中
阻止默认自动代理，同时不破坏显式 Profile proxy 和无白名单的 auto 行为。

#### Review Target
- `ornnlab/services/experiment_service.py`
- `ornnlab/services/container_proxy_runtime.py`
- `tests/python/test_experiment_service.py`
- `tests/python/test_container_proxy_runtime.py`
- `docs/releases/v1.0.5/prd.md`
- `docs/releases/v1.0.5/technical-design.md`
- `docs/playbooks/development-operations.md`
- 本报告 Round 3 output/response

#### Required Closure Checks
1. Agent `extra_allowed_hosts` 非空 → auto skipped。
2. Environment `extra_allowed_hosts` 非空 → auto skipped，生产调用链测试成立。
3. 两者为空 → auto 保持原行为；显式 Environment proxy 不被删除。
4. skip 日志、PRD、技术设计和 playbook 均使用 Agent/Environment 通用语义。
5. Round 3 已 PASS 的 daemon identity、cancel cleanup、direct path、policy isolation 未被改坏。

#### Verification Evidence
- Environment production-path integration 与 helper matrix 通过。
- latest targeted/full counts 以当前复审时实跑为准，报告旧计数不作为结论依据。
- 当前设备 custom bridge 真实 HTTP 200；非本地设备仍仅声明 loopback fail-safe。

#### Reviewer Instructions
- Fresh session, fork_context=false, read-only。
- 直接读取最新工作树并尝试复现 Round 3 harness。
- 逐项 PASS/FAIL；任何 accepted blocking 未闭环则 FAIL-BLOCK。

### Reviewer Selection

| Reviewer | Reason Selected | Risk Area |
|---|---|---|
| architecture-adversary | Round 3 新 blocking 需要第四个 fresh session 独立验证 | Environment 数据流、默认安全语义、回归 |

### Reviewer Launch Records

| Reviewer | Internal Mechanism | Session / Job ID | Trace Source | Context Forked | Input Packet | Context Explicitly Excluded | Read-only |
|---|---|---|---|---|---|---|---|
| architecture-adversary | internal fresh subagent | `/root/proxy_allowlist_final_review` | `spawn_agent` | fork_context=false | Round 4 Review Input | main-agent history, reasoning, drafts, conclusions, full diff | yes |

### Reviewer Output

Verdict: **PASS WITH FOLLOW-UPS**。Blocking: none；Important: none。

五项 required checks 全部 PASS。Reviewer 独立重放：

- Agent allowlist → `automatic_proxy_allowed=False`、空 runtime policy。
- Environment Profile → Job persistence → Queue → Experiment 完整临时数据库链路，runtime
  收到 `False` 且 Job completed。
- 空 allowlist 保留 auto；Environment 显式 proxy 保留在 artifact，Agent 无 auto env。
- 通用 `network_allowlist` 日志和三份中文文档一致。
- daemon identity、cancel cleanup、direct path、active connection 与 concurrent policy
  回归未退化。

Minor follow-up：当前仓库集成测试从已编译 `_webui_run_config` 开始；本轮 reviewer 已用临时
数据库覆盖完整 Profile/persistence 链，后续可把该 harness 固化为长期自动化测试。

### Main Agent Closure Response

接受 minor follow-up，不阻断本次 closure。原因：生产执行路径已有持久化后配置集成测试，
Profile 编译本身已有独立测试，本轮 fresh reviewer 又完成完整临时数据库实测；将跨多个
service 的 harness 固化列为后续测试增强，不改变当前安全行为。

## Final Conclusion

**PASS WITH FOLLOW-UPS.** 四轮 fresh architecture-adversary 共发现并闭环 daemon 网络边界、
显式配置优先、relay 生命周期、取消泄漏、Context/Host 优先级、direct discovery 与
Agent/Environment allowlist 安全语义。当前实现不再把本机路径/IP/代理产品当作通用条件；
非本地 loopback target fail-safe，未声称未经设备实测的成功支持。
