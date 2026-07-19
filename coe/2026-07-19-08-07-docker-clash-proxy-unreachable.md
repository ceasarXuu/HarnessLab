# Problem P-001: Docker 任务容器无法自动使用宿主代理

- Status: fixed
- Created: 2026-07-19 08:07
- Updated: 2026-07-19 09:01
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
- Ruled out:
  - Claude/npm 服务故障；Clash 规则故障。
- Fix criteria:
  - 自动发现标准代理变量；非回环代理直接继承；回环代理经 Docker 专用受限 relay 转换；可显式关闭；不得泄露代理凭据；有结构化日志、单元测试和真实 Docker 冒烟。
- Current conclusion: OrnnLab 已自动发现宿主代理，在执行期托管 Docker bridge relay，并通过受管 Harbor 子进程安全注入 Agent。
- Related hypotheses:
  - H-001
  - H-002
  - H-003
- Resolution basis:
  - H-001、H-002、H-003；E-006、E-009、E-011、E-012、E-013、E-014。
- Close reason:
  - not closed

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
