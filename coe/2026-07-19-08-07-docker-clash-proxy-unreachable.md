# Problem P-001: Docker 任务容器无法自动使用宿主代理

- Status: resolved
- Created: 2026-07-19 08:07
- Updated: 2026-07-19 19:36
- Objective: OrnnLab 自动发现宿主标准代理配置，为 Docker trial 提供受限且可达的代理入口，并注入后续 Job。
- Symptoms:
  - 宿主机通过 Clash 可访问 Claude、npm，Harbor trial 下载 Claude Code 时连接超时。
- Expected behavior:
  - 用户无需维护 systemd relay 或逐个修改 Agent Profile；OrnnLab 自动完成代理发现、地址适配、注入和日志记录。
- Actual behavior:
  - Harbor 不继承宿主代理；回环代理地址在容器中指向容器自身。
- Impact:
  - 依赖宿主回环代理的所有 Docker Job 都可能在安装依赖或 Agent 运行阶段失败。
- Reproduction:
  - 宿主设置 `HTTPS_PROXY=http://127.0.0.1:7890`，运行需要访问 Claude 下载站的 Docker Job。
- Environment:
  - Ubuntu；Docker Engine；Clash Verge/Mihomo；Harbor 0.13.2；main。
- Known facts:
  - 宿主经 Clash 请求 Claude/npm 返回 200；容器无代理变量且直连超时。
  - Docker 专用 TCP relay 加代理注入可使原 Claude setup 成功。
  - 自动代理必须进入 Harbor Environment，才能覆盖 setup、Agent 和 Verifier。
  - Clash 对 Ubuntu archive/security 存在间歇性 502；同 URL 宿主直连成功。
- Ruled out:
  - Claude/npm 服务故障；原始 Claude 超时不是 Clash 目标规则故障。
- Fix criteria:
  - 自动发现标准代理变量；非回环代理直接继承；回环代理经 Docker 专用受限 relay 转换；setup、Agent、Verifier 全生命周期继承；可显式关闭；不得泄露代理凭据；有结构化日志、单元测试和真实 Docker 冒烟。
- Current conclusion: H-004/H-005 已确认并完成 capability-driven、Environment-lifecycle 修复；原失败任务等价复跑 10/10 终态，其中 5 个 Verifier 通过，剩余 5 个隔离为 Clash Ubuntu 上游 502/超时。
- Related hypotheses:
  - H-001
  - H-002
  - H-003
  - H-004
  - H-005
- Resolution basis:
  - H-001/H-002/H-003/H-004/H-005 evidence gates satisfied；自动代理真实容器 HTTP 200；四轮 fresh architecture review 最终 PASS WITH FOLLOW-UPS；最终等价 Job 的 setup、Agent、Verifier 均有真实成功证据。
- Close reason:
  - fixed

## Hypothesis H-001: Harbor trial 缺少可用代理是原始故障根因

- Status: confirmed
- Parent: P-001
- Claim: trial 未获得代理变量，并且宿主代理仅监听回环地址，两项共同导致容器外连超时。
- Layer: root-cause
- Factor relation: all_of
- Depends on:
  - none
- Falsifiable predictions:
  - If true: 宿主经代理成功；容器直连及访问宿主未转发端口失败；建立 Docker 专用 relay 并注入后成功。
  - If false: 容器在不注入代理时也能成功，或 relay 与注入后仍以相同方式失败。
- Diagnostic evidence plan:
  - Prediction or clause under test: 对比宿主、原容器、relay 容器三条路径。
  - Signal: HTTP 状态、连接错误、容器代理变量。
  - Capture method: curl、docker inspect、临时 socat、Claude setup 冒烟。
  - Event name or marker:
    - none
  - Correlation keys:
    - run-f41a9d84f101
  - Differentiates from:
    - 目标服务或 Clash 规则故障
  - Supports if:
    - 只有具备 Docker 可达 relay 和代理注入的容器成功。
  - Refutes if:
    - 原容器无需代理即可成功，或 relay 不能改变结果。
  - Instrumentation status: none
  - Instrumentation lifecycle:
    - none
- Evidence gate: satisfied
- Related evidence:
  - E-001
  - E-002
  - E-003
  - E-004
  - E-005
  - E-006
  - E-007
  - E-008
  - E-009
- Conclusion: confirmed
- Repair design readiness: ready
- Next step: 以自动化产品能力替代手工恢复配置。
- Blocker:
  - none
- Close reason:
  - not closed

## Hypothesis H-002: 产品缺少宿主代理到 Docker 环境的适配边界

- Status: confirmed
- Parent: P-001
- Claim: OrnnLab 的服务生命周期没有托管 Docker 可达 relay，Job 配置编译也没有把标准宿主代理归一化后合并到 EnvironmentConfig.env。
- Layer: interaction
- Factor relation: all_of
- Depends on:
  - H-001
- Falsifiable predictions:
  - If true: 应找不到代理发现/relay 生命周期服务，Job 编译只使用用户 profile，Harbor 只消费显式 Agent/Environment env。
  - If false: 现有自动代理适配器已经存在，只是配置或调用链未启用。
- Diagnostic evidence plan:
  - Prediction or clause under test: 追踪应用启动、Job 编译、Harbor env 注入三条代码路径。
  - Signal: 生命周期注册点、配置合并点和现有测试覆盖。
  - Capture method: 静态代码检索、调用链阅读、针对性失败测试。
  - Event name or marker:
    - docker_proxy_detection
    - docker_proxy_bridge_started
    - docker_proxy_injected
  - Correlation keys:
    - application lifecycle
    - run id
  - Differentiates from:
    - 已有功能仅未启用
  - Supports if:
    - 三条路径均不存在自动适配逻辑。
  - Refutes if:
    - 已有适配逻辑能覆盖回环代理且只是运行配置错误。
  - Instrumentation status: permanent-observability-candidate
  - Instrumentation lifecycle:
    - promote after repair
- Evidence gate: satisfied
- Related evidence:
  - E-010
- Conclusion: confirmed
- Repair design readiness: ready
- Next step: 在异步执行期注入应用级 ContainerProxyRuntime policy，并由 lifespan 管理 relay 生命周期。
- Blocker:
  - none
- Close reason:
  - not closed

## Hypothesis H-003: 执行期 runtime 能自动替代手工恢复方案

- Status: confirmed
- Parent: P-001
- Claim: 应用级 runtime 在 Job 执行期发现代理、为回环代理建立受限 relay、把模板写入 Agent config 并只通过 Harbor 子进程传递实际地址，可在不修改 Profile 和系统服务的情况下消除原症状。
- Layer: fix-validation
- Factor relation: all_of
- Depends on:
  - H-001
  - H-002
- Falsifiable predictions:
  - If true: artifact 只含模板；Harbor child 获得实际派生值；新 Docker 容器经自动 relay 请求 Claude 成功；关闭手工配置后服务仍能发现代理。
  - If false: 必须保留 systemd/Profile 才能成功，或实际代理 URL 泄露到 artifact。
- Diagnostic evidence plan:
  - Prediction or clause under test: 自动 runtime 完整替代手工 relay/Profile。
  - Signal: 单元/集成测试、artifact 内容、真实 Docker HTTP 状态、生产 backend 日志和 Profile 状态。
  - Capture method: pytest、真实容器 curl、服务重启、API 脱敏检查。
  - Event name or marker:
    - docker_proxy_detection
    - docker_proxy_bridge_started
    - docker.proxy.injected
  - Correlation keys:
    - application lifecycle
    - disposable Docker smoke
  - Differentiates from:
    - 旧 systemd relay 或 Agent Profile 仍在生效
  - Supports if:
    - 手工方案移除后自动 runtime 的真实请求与全量回归通过。
  - Refutes if:
    - 移除手工配置后原症状复现。
  - Instrumentation status: permanent-observability-candidate
  - Instrumentation lifecycle:
    - retain as permanent observability
- Evidence gate: satisfied
- Related evidence:
  - E-011
  - E-012
  - E-013
  - E-014
- Conclusion: confirmed
- Repair design readiness: implemented and validated
- Next step: none
- Blocker:
  - none
- Close reason:
  - fixed

## Hypothesis H-004: 当前实现错误地把本地 rootful Linux 当作通用 Docker 部署

- Status: confirmed
- Parent: P-001
- Claim: 默认 `docker network inspect bridge`、绑定该 gateway、只支持 Harbor subprocess 等实现假设不能覆盖 Docker Desktop、rootless、远程 context 或非默认 daemon，因而当前“自动继承”可能是本机特有 feature。
- Layer: environment
- Factor relation: any_of
- Depends on:
  - H-003
- Falsifiable predictions:
  - If true: 代码没有 daemon locality/capability 分类；非本地 rootful 场景仍进入同一 gateway relay 路径；测试仅用 127.0.0.1 fake gateway 和本机 Docker。
  - If false: 生产代码已按 daemon 类型选择 portable 策略，并对不支持场景清晰降级，测试覆盖各能力分支。
- Diagnostic evidence plan:
  - Prediction or clause under test: 审计代理发现、Docker target 检测、relay bind、Harbor engine 与测试矩阵。
  - Signal: 代码分支、Docker context/daemon 检测、官方能力约束、独立 reviewer 反例。
  - Capture method: 静态审计、官方文档核对、纯单元能力矩阵和 fresh adversarial review。
  - Event name or marker:
    - docker_proxy_target_detected
  - Correlation keys:
    - Docker context
    - daemon endpoint
  - Differentiates from:
    - 仅本机配置错误
  - Supports if:
    - 非本地场景被错误映射到默认 bridge relay，或缺少明确支持/拒绝路径。
  - Refutes if:
    - capability-driven strategy 对所有声明场景都有正确行为和测试。
  - Instrumentation status: permanent-observability-candidate
  - Instrumentation lifecycle:
    - promote after repair if confirmed
- Evidence gate: satisfied
- Related evidence:
  - E-015
  - E-016
  - E-017
  - E-018
  - E-019
  - E-020
  - E-021
  - E-022
  - E-023
  - E-024
- Conclusion: confirmed
- Repair design readiness: implemented and validated
- Next step: 将完整 Profile→Job→Queue→Experiment reviewer harness 固化为后续自动化增强。
- Blocker:
  - none
- Close reason:
  - fixed

## Hypothesis H-005: 只向 Agent 注入代理会遗漏 Verifier 网络

- Status: confirmed
- Parent: P-001
- Claim: `AgentConfig.env` 只能覆盖 Agent setup/运行；Verifier 通过 Environment 执行，因而仍会在下载验证依赖时直连失败。
- Layer: integration
- Factor relation: all_of
- Depends on:
  - H-003
- Falsifiable predictions:
  - If true: Claude 安装和模型调用成功，但 Verifier 下载 `uv` 失败；将模板移到 Environment 后 setup、Agent、Verifier 均获得同一代理。
  - If false: 只配置 Agent env 时 Verifier 也能稳定下载依赖，或 Environment 注入不改变结果。
- Diagnostic evidence plan:
  - Prediction or clause under test: 等价复跑并分别观察 Agent setup、模型调用和 Verifier 下载。
  - Signal: Claude bootstrap、trajectory、Verifier `uv`/CPython/PyPI 下载和 reward。
  - Capture method: WebUI Job copy、Harbor trial log、Verifier stdout、原生 result.json。
  - Event name or marker:
    - harbor_subprocess.runtime_config_prepared
  - Correlation keys:
    - run-d20feca1d748
    - run-7d456ad95206
  - Differentiates from:
    - Clash 对单个 Ubuntu mirror 的选择性 502
  - Supports if:
    - Agent 成功而 Verifier 直连失败；Environment 注入后相同 Verifier 下载成功。
  - Refutes if:
    - Verifier 在 Agent-only 配置下已继承代理，或 Environment 修复后仍以相同方式失败。
  - Instrumentation status: permanent
  - Instrumentation lifecycle:
    - retained
- Evidence gate: satisfied
- Related evidence:
  - E-025
  - E-026
  - E-028
  - E-029
  - E-030
- Conclusion: confirmed
- Repair design readiness: implemented and validated
- Next step: none
- Blocker:
  - none
- Close reason:
  - fixed

## Evidence E-001: 原 Job 的 Claude bootstrap 连接超时

- Related hypotheses: H-001
- Direction: supports
- Type: reproduction
- Source: run-f41a9d84f101 trial logs
- Prediction or plan link: H-001 容器无可用代理时外连失败。
- Matched signal: curl 连接 downloads.claude.ai:443 超时。
- Correlation keys: run-f41a9d84f101
- Raw content: `curl: (28) Failed to connect to downloads.claude.ai port 443`
- Interpretation: Agent setup 失败发生在容器外连边界。
- Time: 2026-07-19 08:00

## Evidence E-002: 宿主代理是仅回环监听的 Clash

- Related hypotheses: H-001
- Direction: supports
- Type: environment
- Source: 宿主 env、ss、Clash 安全配置
- Prediction or plan link: H-001 的回环不可达条件。
- Matched signal: 代理变量为 127.0.0.1:7890，allow-lan=false。
- Correlation keys: host
- Raw content: `LISTEN 127.0.0.1:7890; allow-lan: false`
- Interpretation: 原地址不能从 Docker namespace 访问。
- Time: 2026-07-19 08:06

## Evidence E-003: 宿主经 Clash 可访问目标服务

- Related hypotheses: H-001
- Direction: supports
- Type: probe
- Source: 宿主 curl
- Prediction or plan link: 排除目标服务与 Clash 规则故障。
- Matched signal: Claude bootstrap 与 npm registry 均 HTTP 200。
- Correlation keys: host
- Raw content: `HTTP/2 200`
- Interpretation: 故障只存在于容器代理路径。
- Time: 2026-07-19 08:08

## Evidence E-004: trial 没有代理变量

- Related hypotheses: H-001
- Direction: supports
- Type: config
- Source: docker inspect（仅筛选 proxy key）
- Prediction or plan link: H-001 的未注入条件。
- Matched signal: HTTP_PROXY/HTTPS_PROXY/ALL_PROXY 大小写变量均不存在。
- Correlation keys: active trial containers
- Raw content: `proxy env keys: []`
- Interpretation: Harbor 没有自动继承宿主代理。
- Time: 2026-07-19 08:08

## Evidence E-005: 容器三条未适配路径均失败

- Related hypotheses: H-001
- Direction: supports
- Type: experiment
- Source: trial 容器 curl
- Prediction or plan link: H-001 回环地址和未监听网关均不可达。
- Matched signal: 直连超时；127.0.0.1:7890 与 bridge gateway:7890 拒绝连接。
- Correlation keys: active trial containers
- Raw content: `exit 28; exit 7; exit 7`
- Interpretation: 单纯复制宿主代理地址不能解决问题。
- Time: 2026-07-19 08:09

## Evidence E-006: Docker 专用 relay 改变结果

- Related hypotheses: H-001
- Direction: supports
- Type: experiment
- Source: 临时 socat 与两个 Compose bridge 容器
- Prediction or plan link: H-001 若成立，relay 加注入应成功。
- Matched signal: 两个容器均约 0.5 秒获得 HTTP 200。
- Correlation keys: 172.18.0.0/16; 172.19.0.0/16
- Raw content: `http=200`
- Interpretation: Docker 可达 relay 是充分修复条件之一。
- Time: 2026-07-19 08:10

## Evidence E-007: 手工恢复服务与 Profile 生效

- Related hypotheses: H-001
- Direction: supports
- Type: fix-validation
- Source: systemd service、Agent Profile API
- Prediction or plan link: 持久 relay 与代理注入应可重复工作。
- Matched signal: service enabled/active；六个代理变量已保存。
- Correlation keys: claude-code-deepseek-v4-pro
- Raw content: `enabled; active`
- Interpretation: 手工方案恢复后续 Job 的必要网络条件。
- Time: 2026-07-19 08:13

## Evidence E-008: 全新容器经 relay 访问 Claude 成功

- Related hypotheses: H-001
- Direction: supports
- Type: fix-validation
- Source: curlimages/curl Docker smoke
- Prediction or plan link: 新容器使用注入地址应成功。
- Matched signal: HTTP 200。
- Correlation keys: disposable container
- Raw content: `http=200`
- Interpretation: 修复不依赖旧 trial 容器状态。
- Time: 2026-07-19 08:15

## Evidence E-009: 原 task 类型的完整 Agent setup 成功

- Related hypotheses: H-001
- Direction: supports
- Type: fix-validation
- Source: terminal-bench task image
- Prediction or plan link: 原始 Claude 安装症状应消失。
- Matched signal: apt、bootstrap 与 claude --version 全部成功。
- Correlation keys: fix-code-vulnerability:2.0
- Raw content: `2.1.214 (Claude Code)`
- Interpretation: 原始失败链已由 relay 与注入修复。
- Time: 2026-07-19 08:17

## Evidence E-010: 三条代码路径均不存在自动代理适配

- Related hypotheses: H-002
- Direction: supports
- Type: code-location
- Source: `ornnlab/app.py`、`webui_job_service.py`、`experiment_service.py`、Harbor 0.13.2 env 链路
- Prediction or plan link: H-002 三条代码路径。
- Matched signal: app lifespan 未管理代理 runtime；Job 创建只持久化 profile；真正 Harbor config 在 `_run_one()` 执行期生成；Harbor 只消费显式 env。
- Correlation keys: application lifecycle; job compile
- Raw content: `create_app -> QueueWorkerService; create_job -> harbor_overrides; _run_one -> builder.build; EnvironmentConfig.env -> docker compose exec -e`
- Interpretation: 缺口位于应用级 runtime 与执行期 config policy 合并边界；在 Job 创建期修改 Profile 或快照会引入错误生命周期。
- Time: 2026-07-19 08:22

## Evidence E-011: 自动代理契约与子进程安全边界通过测试

- Related hypotheses: H-003
- Direction: supports
- Type: fix-validation
- Source: `test_container_proxy_runtime.py`、`test_harbor_subprocess.py`
- Prediction or plan link: H-003 的发现、relay、显式配置优先、模板与 child env 分离。
- Matched signal: 代理针对性测试、Harbor config 和 worker 回归全部通过。
- Correlation keys: targeted regression
- Raw content: `31 passed; pyright 0 errors; ruff all checks passed`
- Interpretation: 派生地址不进入 artifact，大小写冲突与不安全代理会在 Harbor 前失败。
- Time: 2026-07-19 08:47

## Evidence E-012: 无手工端口的真实 Docker 请求成功

- Related hypotheses: H-003
- Direction: supports
- Type: fix-validation
- Source: ContainerProxyRuntime + disposable curl container
- Prediction or plan link: H-003 自动 relay 应替代 systemd socat。
- Matched signal: runtime 动态 policy 注入后 Claude bootstrap HTTP 200。
- Correlation keys: disposable Docker smoke
- Raw content: `http=200`
- Interpretation: 原症状在自动 runtime 路径上消失。
- Time: 2026-07-19 08:45

## Evidence E-013: 本机已移除手工代理配置并加载自动模式

- Related hypotheses: H-003
- Direction: supports
- Type: fix-validation
- Source: systemd、Agent API、dev service backend.log
- Prediction or plan link: 排除旧手工方案仍在生效。
- Matched signal: 手工 unit not-found；Agent proxy keys 为空；backend 日志发现标准代理；前后端健康。
- Correlation keys: local dev service restart
- Raw content: `docker_proxy_detection mode=auto; profile proxy keys=[]; status=running`
- Interpretation: 当前部署不再依赖 systemd relay 或逐 Profile 配置。
- Time: 2026-07-19 08:52

## Evidence E-014: 全栈质量门与启停回归通过

- Related hypotheses: H-003
- Direction: supports
- Type: fix-validation
- Source: `test-after-change-web.sh` 各门禁与修正后的 `test-run-dev-api.sh`
- Prediction or plan link: 自动 runtime 生命周期不得破坏服务、Harbor、前端或 launcher。
- Matched signal: Python 133 passed/3 skipped；前端 108 passed；launcher 27 passed；构建与 Storybook 成功；API 启停通过。
- Correlation keys: post-fix full gate
- Raw content: `133 passed; 108 passed; 27 passed; test-run-dev-api exit 0`
- Interpretation: 代理生命周期与 worker 关闭没有引入全栈回归；端口门禁的 TIME_WAIT 假失败也已改为真实 connect 探针。
- Time: 2026-07-19 09:01

## Evidence E-015: 用户拒绝本机特有实现作为完成标准

- Related hypotheses: H-004
- Direction: supports
- Type: user-feedback
- Source: 2026-07-19 用户反馈
- Prediction or plan link: H-004 的跨设备部署目标。
- Matched signal: 用户明确要求适应 OrnnLab 在其他位置和设备上的部署。
- Correlation keys: portability review
- Raw content: `不要做成本机特有feature,要适应ornnlab在其他位置设备上的部署`
- Interpretation: 本机 rootful Linux 的 HTTP 200 只能证明一个平台分支，不能关闭产品问题。
- Time: 2026-07-19 09:10

## Evidence E-016: fresh 架构审查复现跨 daemon 网络边界错误

- Related hypotheses: H-004
- Direction: supports
- Type: independent-review
- Source: `/root/proxy_portability_arch_review`
- Prediction or plan link: H-004 daemon locality 与 gateway bind 假设。
- Matched signal: remote、Desktop、rootless 的 gateway 位于 daemon/VM/namespace，而 listener 在 OrnnLab OS bind；显式 Profile 也在自动探测之后才生效。
- Correlation keys: architecture-adversary round 1
- Raw content: `FAIL / BLOCK`
- Interpretation: 旧实现是本地 rootful Linux 默认 bridge 专用策略，H-004 确认。
- Time: 2026-07-19 17:10

## Evidence E-017: Docker 官方边界支持 capability 分类

- Related hypotheses: H-004
- Direction: supports
- Type: external-evidence
- Source: Docker Context、Rootless、Desktop host networking 与 dockerd host-gateway 官方文档
- Prediction or plan link: target discovery repair design。
- Matched signal: Context endpoint 可以指向远程 daemon；rootless daemon 在 user namespace；Desktop 使用 VM/专用 host DNS；host-gateway 默认来自 daemon default bridge。
- Correlation keys: Docker target capability
- Raw content: `context endpoint; SecurityOptions=rootless; host.docker.internal; host-gateway`
- Interpretation: 必须先识别 daemon locality/runtime，不能把 daemon gateway 直接等同 OrnnLab 本机接口。
- Time: 2026-07-19 17:14

## Evidence E-018: capability 与生命周期矩阵测试通过

- Related hypotheses: H-004
- Direction: refutes
- Type: fix-validation
- Source: `test_docker_proxy_target.py`、`test_container_proxy_runtime.py`、`test_experiment_service.py`
- Prediction or plan link: H-004 修复后的反证条件。
- Matched signal: DOCKER_HOST/context、local rootful、Desktop、rootless、remote、显式配置优先、不可 bind、policy 回收均有稳定分支。
- Correlation keys: portability unit matrix
- Raw content: `144 passed, 3 skipped; pyright 0; scoped ruff pass`
- Interpretation: 生产策略按能力选择；不支持的 loopback target 在 Harbor 前给可恢复错误。
- Time: 2026-07-19 17:31

## Evidence E-019: 当前设备仍通过新 target 策略完成真实容器请求

- Related hypotheses: H-003; H-004
- Direction: refutes
- Type: fix-validation
- Source: ContainerProxyRuntime + disposable curl container
- Prediction or plan link: 重构不能破坏已确认的本地分支。
- Matched signal: target 分类为 local-rootful-linux，单 Job relay 请求 Claude bootstrap HTTP 200，退出后 policy 回收。
- Correlation keys: real Docker portability baseline
- Raw content: `strategy=host-relay target=local-rootful-linux relays=1; docker_http_status=200`
- Interpretation: capability 重构保持当前环境可用，不依赖固定 IP 或 Clash 产品识别。
- Time: 2026-07-19 17:33

## Evidence E-020: 全栈回归与 watcher 环境故障隔离

- Related hypotheses: H-004
- Direction: refutes
- Type: regression
- Source: `test-after-change-web.sh` 与 polling Storybook smoke
- Prediction or plan link: 跨设备改造不得破坏其他系统边界。
- Matched signal: Python 144/3、前端 108、类型、lint、build 通过；Storybook 初次被设备 inotify 配额阻断，polling 模式通过。
- Correlation keys: full quality gate
- Raw content: `144 passed; 108 passed; build pass; Smoke tests passed`
- Interpretation: 代码门禁通过；ENOSPC 属于宿主 watcher 配额且已形成可移植排障记录。
- Time: 2026-07-19 17:38

## Evidence E-021: 第二轮 closure 发现 daemon 混用与取消泄漏

- Related hypotheses: H-004
- Direction: supports
- Type: independent-review
- Source: `/root/proxy_portability_closure_review`
- Prediction or plan link: capability 与生命周期 closure。
- Matched signal: `DOCKER_CONTEXT`/`DOCKER_HOST` 优先级混用；prepare 第二个 relay 时取消可遗留第一个 listener；默认 auto 与 `extra_allowed_hosts` 白名单冲突。
- Correlation keys: architecture-adversary round 2
- Raw content: `FAIL-BLOCK; active_server_count_after_cancel=1`
- Interpretation: 第一轮修复仍有可复现缺口，必须继续修而不能按本机 HTTP 200 关闭。
- Time: 2026-07-19 17:45

## Evidence E-022: 第二轮反例修复与全量门禁通过

- Related hypotheses: H-004
- Direction: refutes
- Type: fix-validation
- Source: target/runtime/Experiment 矩阵、`test-after-change-web.sh`、真实 Docker custom network
- Prediction or plan link: Round 3 required closure checks。
- Matched signal: Context 覆盖 Host 且所有 daemon 查询显式锁定同一 target；取消事务回收；allow-host 跳过 auto；direct 不依赖 discovery；并发 policy 互不关闭。
- Correlation keys: portability closure regression
- Raw content: `Python 153 passed/3 skipped; frontend 108; launcher 27; pyright 0; Docker HTTP 200`
- Interpretation: 第二轮所有 accepted blocking 均有生产代码、回归和真实当前平台证据；等待第三轮 fresh reviewer。
- Time: 2026-07-19 17:56

## Evidence E-023: 第三轮发现 Environment 白名单漏接

- Related hypotheses: H-004
- Direction: supports
- Type: independent-review
- Source: `/root/proxy_portability_final_closure`
- Prediction or plan link: allowlist 默认安全语义。
- Matched signal: Environment Profile 把白名单编译到 override，但运行路径只检查 Agent，生产 harness 观察到 auto 仍注入。
- Correlation keys: architecture-adversary round 3
- Raw content: `FAIL-BLOCK; automatic_proxy_allowed=True`
- Interpretation: 网络策略有两个模型来源，必须统一判定，不能只修一个表面字段。
- Time: 2026-07-19 17:59

## Evidence E-024: 第四轮完整数据流闭环通过

- Related hypotheses: H-004
- Direction: refutes
- Type: independent-fix-validation
- Source: `/root/proxy_allowlist_final_review`、full quality gate
- Prediction or plan link: Round 4 five closure checks。
- Matched signal: Agent/Environment allowlist 均跳过 auto；空白名单保留 auto；显式 Environment proxy 保留；Profile→Job→Queue→Experiment 临时数据库链路完成。
- Correlation keys: architecture-adversary round 4
- Raw content: `PASS WITH FOLLOW-UPS; Python 156 passed/3 skipped; no blocking/important`
- Interpretation: 所有 accepted blocking 已闭环；剩余项仅为把 reviewer 的完整跨 service harness 固化为长期测试。
- Time: 2026-07-19 18:05

## Evidence E-025: 原 Job resume 被 Harbor lock 一致性保护拒绝

- Related hypotheses: H-005
- Direction: neutral
- Type: reproduction
- Source: `run-f41a9d84f101` resume operation
- Prediction or plan link: 不修改旧 lock/artifact，改用等价 copy 复跑。
- Matched signal: Harbor 报现有 `lock.json` 与 resolved job lock 不一致。
- Correlation keys: `op-6711b9870952`
- Raw content: `FileExistsError: Job directory ... already has a lock.json that does not match`
- Interpretation: 原目录不可安全原地 resume；唯一名称的等价 Job 是非破坏性验证路径。
- Time: 2026-07-19 18:49

## Evidence E-026: Agent-only 复跑越过原故障但 Verifier 仍直连失败

- Related hypotheses: H-005
- Direction: supports
- Type: reproduction
- Source: `run-d20feca1d748`
- Prediction or plan link: H-005 的分层差异。
- Matched signal: Claude 安装、DeepSeek 模型调用和 trajectory 成功；QEMU Verifier 请求 `astral.sh` 时 `curl: (7)`，进程环境无代理。
- Correlation keys: `qemu-startup__8j6ze95`
- Raw content: `Claude trajectory written; verifier: curl: (7) Couldn't connect to server`
- Interpretation: 原始网络问题已在 Agent 层修复，但容器全生命周期语义尚未闭环。
- Time: 2026-07-19 19:06

## Evidence E-027: Clash 对 Ubuntu mirror 返回选择性 502

- Related hypotheses: H-001; H-005
- Direction: neutral
- Type: external-evidence
- Source: 宿主 curl direct/proxy 对照与 trial apt 输出
- Prediction or plan link: 区分 OrnnLab relay 故障和代理上游规则/节点故障。
- Matched signal: 同一宿主上 `archive.ubuntu.com` 经 Clash 502、直连 200；Claude/GitHub 经 Clash 200。
- Correlation keys: `127.0.0.1:7890`; Ubuntu archive
- Raw content: `proxy archive=502; direct archive=200; proxy downloads.claude.ai=200`
- Interpretation: relay 正确转发了 Clash 响应；不得在 OrnnLab 中加入 Ubuntu 域名或本机直连特判。
- Time: 2026-07-19 18:56

## Evidence E-028: Environment 模板需要运行期临时解析

- Related hypotheses: H-005
- Direction: supports
- Type: implementation-diagnostic
- Source: `run-cec9f06c9c70` 与 ManagedSubprocessHarborRunner
- Prediction or plan link: Environment 负责全生命周期，但 Harbor 不会替其展开 OrnnLab 模板。
- Matched signal: 未解析时 curl 报 `${ORNNLAB_CONTAINER_HTTPS_PROXY}` 为坏主机；临时解析后 setup 使用实际 relay。
- Correlation keys: `harbor_subprocess.runtime_config_prepared`
- Raw content: `curl: (5) Unsupported proxy syntax; fixed setup relay=172.17.0.1:<ephemeral>`
- Interpretation: OrnnLab artifact 保留模板，runner 只为本次 Harbor 子进程生成并清理已解析临时配置。
- Time: 2026-07-19 19:13

## Evidence E-029: Environment 生命周期实现通过全量门禁

- Related hypotheses: H-005
- Direction: refutes
- Type: regression
- Source: `test-after-change-web.sh` 与 targeted proxy tests
- Prediction or plan link: 显式配置优先、临时配置清理、缺失 runtime 值失败、全栈不回归。
- Matched signal: Python、前端、launcher、类型、lint、构建和 Storybook 均通过。
- Correlation keys: commit `a150e6a`
- Raw content: `157 passed/3 skipped; frontend 108; launcher 27; pyright 0; targeted subprocess 12 passed`
- Interpretation: 修复使用 Harbor 标准 EnvironmentConfig，不依赖 Clash、固定地址、路径或设备。
- Time: 2026-07-19 19:21

## Evidence E-030: 最终等价 Job 证明 setup、Agent、Verifier 全生命周期可用

- Related hypotheses: H-001; H-003; H-004; H-005
- Direction: refutes
- Type: fix-validation
- Source: `run-7d456ad95206` 原生 Harbor result 与 trial logs
- Prediction or plan link: 最终真实任务验收。
- Matched signal: 10/10 终态；5 个 reward=1；Claude/DeepSeek 成功；多个 Verifier 成功下载 uv、33.8 MiB CPython、PyPI 包并通过测试。
- Correlation keys: Harbor job `b6bf26c8-8b6f-4b74-98cc-8a1eb53dada0`
- Raw content: `n_completed_trials=10; n_errored_trials=5; mean=0.5; runtime=1366s`
- Interpretation: 原始 Claude 网络故障及 Verifier 漏注入均已修复；剩余 5 个失败隔离为 4 个 Ubuntu 502 和 1 个 apt setup 300 秒超时。
- Time: 2026-07-19 19:36
