# Problem P-001: Harbor 容器缺少 OrnnLab 所有权与闭环回收
- Status: fixed
- Created: 2026-07-23 01:01
- Updated: 2026-07-23 01:18
- Objective: 让 OrnnLab 创建的全部 Harbor Docker 容器带有可迁移的运行归属标签，并在任务终止与服务启动恢复时安全回收残留资源。
- Symptoms:
  - 历史 Harbor Compose 容器在 Job 结束后仍以 Exited 状态残留。
  - System 孤儿扫描无法发现这些残留容器。
- Expected behavior:
  - 每个由 OrnnLab 发起的 Harbor Docker 容器都可由稳定标签关联到 OrnnLab run。
  - 正常结束、异常结束和 OrnnLab 重启后均执行幂等回收并留下结构化证据。
- Actual behavior:
  - Harbor 原生 Compose 容器只有 Compose 标签，没有 `ornnlab.run_id`。
  - OrnnLab 只扫描 `ornnlab.run_id`，且只生成手工审核计划，不执行自动回收。
- Impact:
  - Docker 容器与 Compose 网络可持续占用本机资源，System 健康检查仍可能显示正常。
- Reproduction:
  - 强制终止运行中的 Harbor/OrnnLab 进程，使 Harbor 的 `docker compose down` 无法完成，再执行 OrnnLab Docker 孤儿扫描。
- Environment:
  - Ubuntu；OrnnLab main；Python 3.12；Harbor 0.13.x；Docker Compose v5.3.1。
- Known facts:
  - E-001：生产者没有写入扫描器要求的标签。
  - E-002：现有回收器只输出 `docker rm -f` dry-run 计划。
  - E-003：Harbor 提供自定义 Environment `import_path` 与 `kwargs` 扩展边界。
  - E-004：本机历史残留容器均为 Harbor Compose 容器且退出码为 137。
  - E-005：配置、sidecar 覆盖、实例隔离、保留策略和回收单测通过。
  - E-006：随机真实 Compose 项目的双服务/网络标签与回收冒烟通过。
  - E-007：真实 Harbor `hello-world@1.0` 容器与网络运行时标签完整且结束后残留为 0。
  - E-008：项目全量质量门通过。
- Ruled out:
  - 普通 Job 失败必然泄漏：Harbor 正常异常路径仍在 finally 中调用 stop；只有清理未运行或执行失败才会残留。
- Fix criteria:
  - Harbor 配置测试证明 Docker Environment 被替换为 OrnnLab 所有权环境且保留用户配置。
  - Compose 覆盖测试证明 main 与 sidecar 均带 `ornnlab.run_id`。
  - 回收测试证明只删除目标 run 标签资源、失败可观察且重复调用幂等。
  - Job 终止和启动恢复测试证明自动回收已接入。
  - 真实 Docker 冒烟证明容器标签可见且清理后无目标 run 容器。
- Current conclusion: 所有权协议已贯通 Harbor 创建、Job 终态回收、启动恢复和 System 扫描，并由真实 Harbor 运行与全量回归验证。
- Related hypotheses:
  - H-001
- Resolution basis:
  - H-001；E-005、E-006、E-007、E-008
- Close reason:
  - not closed

## Hypothesis H-001: 容器生产与孤儿扫描的所有权协议断链
- Status: confirmed
- Parent: P-001
- Claim: OrnnLab 未在 Harbor Docker Environment 创建容器前注入 `ornnlab.run_id`，而孤儿扫描只接受该标签，因此 Harbor 残留既不能被识别也不能进入自动回收。
- Layer: root-cause
- Factor relation: all_of
- Depends on:
  - none
- Rationale:
  - 容器创建由 Harbor 私有 Compose 生命周期完成，OrnnLab 仅传递标准 `EnvironmentConfig`；现有扫描器却假定标签已经存在。
- Falsifiable predictions:
  - If true: Harbor Job 配置中没有 OrnnLab Docker 扩展与 run ID，扫描器只过滤 `label=ornnlab.run_id`，历史 Harbor 容器不含该标签。
  - If false: 当前创建路径已经向全部 Harbor Compose 服务写入该标签，或扫描器无需该标签即可证明资源归属。
- Diagnostic evidence plan:
  - Prediction or clause under test: 创建配置缺少标签注入，而扫描和回收完全依赖该标签。
  - Signal: 配置构建代码、Harbor EnvironmentFactory 扩展契约、扫描器命令和历史容器 inspect 状态。
  - Capture method: 只读检查相关源码、CLI 与 Docker runtime 标签。
  - Event name or marker:
    - `ornnlab.run_id`
  - Correlation keys:
    - run_id
  - Differentiates from:
    - Docker daemon 单次故障导致扫描失败
  - Supports if:
    - 构建配置未注入标签，扫描只按标签过滤，实际残留不含标签。
  - Refutes if:
    - 创建路径和实际容器均已有标签但扫描仍遗漏。
  - Instrumentation status: permanent-observability-candidate
  - Instrumentation lifecycle:
    - 将回收结果事件与结构化日志作为永久可观察性保留
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
- Conclusion: 生产者、发现器和回收器没有共享同一所有权协议，机制与症状一致。
- Repair design readiness: ready
- Next step: 进入常规发布与用户确认的对抗性审查门。
- Blocker:
  - none
- Close reason:
  - not closed

## Evidence E-001: 配置生产路径未注入所有权
- Related hypotheses:
  - H-001
- Direction: supports
- Type: code-location
- Source: `ornnlab/services/harbor_engine.py`；Harbor `environments/factory.py`
- Prediction or plan link:
  - H-001 创建配置缺少标签注入预测
- Matched signal:
  - 默认环境仅为 `{"type":"docker","delete":true}`；Harbor 支持 `import_path` 和 `kwargs`，但 OrnnLab 未使用。
- Correlation keys:
  - run_id
- Raw content:
  ```text
  environment = overrides.get("environment", {"type": "docker", "delete": True})
  if config.import_path is not None: create_environment_from_import_path(..., **config.kwargs)
  ```
- Interpretation: 当前没有任何创建前标签注入边界，但存在无需修改 Harbor 安装的官方扩展点。
- Time: 2026-07-23 01:01

## Evidence E-002: 扫描器仅按不存在的标签发现且不执行回收
- Related hypotheses:
  - H-001
- Direction: supports
- Type: code-location
- Source: `ornnlab/services/docker_orphan_service.py`
- Prediction or plan link:
  - H-001 扫描和回收完全依赖标签预测
- Matched signal:
  - `docker ps -a --filter label=ornnlab.run_id`；cleanup plan 固定为 dry-run/manual review。
- Correlation keys:
  - run_id
- Raw content:
  ```text
  ORNNLAB_RUN_LABEL = "ornnlab.run_id"
  "dry_run": True,
  "manual_review_required": True,
  ```
- Interpretation: 未打标的 Harbor 容器永远不能进入现有发现结果，发现后也没有自动清理闭环。
- Time: 2026-07-23 01:01

## Evidence E-003: Harbor 支持自定义 Environment 构造参数
- Related hypotheses:
  - H-001
- Direction: supports
- Type: code-location
- Source: Harbor 0.13.x `environments/factory.py` 与 `models/trial/config.py`
- Prediction or plan link:
  - H-001 的最小可验证修复边界
- Matched signal:
  - `EnvironmentConfig.import_path` 选择自定义类，`EnvironmentConfig.kwargs` 原样传入构造器。
- Correlation keys:
  - run_id
- Raw content:
  ```text
  if config.import_path is not None:
      return cls.create_environment_from_import_path(..., **env_constructor_kwargs)
  ```
- Interpretation: OrnnLab 可通过 Harbor 公布的扩展边界管理标签，无需本机补丁或安装位置特判。
- Time: 2026-07-23 01:01

## Evidence E-004: 历史残留缺少 OrnnLab 标签
- Related hypotheses:
  - H-001
- Direction: supports
- Type: observation
- Source: 2026-07-22 本机 `docker ps -a` 与 `docker inspect`
- Prediction or plan link:
  - H-001 实际残留不含标签预测
- Matched signal:
  - 4 个 Harbor Compose 容器均为 `Exited (137)`，具有 `com.docker.compose.*` 标签但没有 `ornnlab.run_id`。
- Correlation keys:
  - historical compose project
- Raw content:
  ```text
  regex-log__rdral9y-main-1
  qemu-startup__cp52uhk-main-1
  build-cython-ext__s4oagdu-main-1
  fix-code-vulnerability__2ychepj-main-1
  ExitCode=137; ornnlab.run_id absent
  ```
- Interpretation: 运行时证据与生产者/扫描器协议断链完全一致；137 解释清理未完成，但不单独区分 OOM 与强制停止。
- Time: 2026-07-23 01:01

## Evidence E-005: 定向所有权与生命周期测试通过
- Related hypotheses:
  - H-001
- Direction: supports
- Type: fix-validation
- Source: pytest、Ruff、Pyright
- Prediction or plan link:
  - P-001 配置、sidecar、实例隔离、保留策略和自动回收 fix criteria
- Matched signal:
  - 定向 44 passed；新增生命周期 5 passed / 1 skipped；Ruff 与 Pyright 均为 0。
- Correlation keys:
  - run_id；instance_id
- Raw content:
  ```text
  44 passed
  5 passed, 1 skipped
  All checks passed!
  0 errors, 0 warnings, 0 informations
  ```
- Interpretation: 配置与回收协议的主要分支均有自动化回归，静态边界成立。
- Time: 2026-07-23 01:12

## Evidence E-006: 真实双服务 Compose 标签与回收通过
- Related hypotheses:
  - H-001
- Direction: supports
- Type: fix-validation
- Source: `ORNNLAB_REAL_DOCKER_OWNERSHIP=1` pytest
- Prediction or plan link:
  - P-001 main 与 sidecar 标签及回收 fix criteria
- Matched signal:
  - 随机项目创建 main/sidecar；容器和默认网络标签一致；扫描命中 2；删除 2 个容器和 1 个网络；复扫为 0。
- Correlation keys:
  - random project；run_id；instance_id
- Raw content:
  ```text
  tests/python/test_real_docker_ownership_cleanup.py 1 passed
  removed_containers=2; removed_networks=1; remaining=0
  ```
- Interpretation: 所有权覆盖和显式 Docker ID 回收在真实 daemon 上有效且限定于随机实例/run。
- Time: 2026-07-23 01:13

## Evidence E-007: 真实 Harbor 端到端标签与自然清理通过
- Related hypotheses:
  - H-001
- Direction: supports
- Type: fix-validation
- Source: Harbor `hello-world@1.0` + oracle 真实运行；Docker runtime inspect
- Prediction or plan link:
  - P-001 Harbor 扩展点与正常结束清理 fix criteria
- Matched signal:
  - 运行容器与网络均含四个 OrnnLab 标签和 ownership overlay；任务结束后容器、网络与卷扫描均为空。
- Correlation keys:
  - run_id=real-harbor-smoke；session_id=hello-world__oDMVL9W
- Raw content:
  ```text
  docker.ownership.compose_prepared run_id=real-harbor-smoke service_count=1 network_count=1 volume_count=0
  ornnlab.managed=true
  ornnlab.instance_id=000e755204d443d0b9fde984e6ec3ac0
  ornnlab.run_id=real-harbor-smoke
  ornnlab.cleanup=auto
  1 passed in 143.45s
  post-run container count=0; network count=0
  ```
- Interpretation: 修复真实进入 Harbor Environment 创建链，且没有破坏 Harbor 正常清理。
- Time: 2026-07-23 01:18

## Evidence E-008: 全量质量门通过
- Related hypotheses:
  - H-001
- Direction: supports
- Type: fix-validation
- Source: `bash scripts/test-after-change-web.sh`
- Prediction or plan link:
  - P-001 回归测试 fix criteria
- Matched signal:
  - Python、前端、构建、Storybook 与 launcher 全绿；自定义 Environment 兼容回归后 Python 全量仍通过。
- Correlation keys:
  - repository main worktree
- Raw content:
  ```text
  Python 187 passed, 4 skipped
  Ruff passed
  Pyright 0 errors, 0 warnings
  Frontend 32 files, 117 tests passed
  lint/typecheck/build passed
  Storybook smoke/static build passed
  launcher 27 passed
  ```
- Interpretation: 所有权改动未引入已知后端、前端或启动生命周期回归。
- Time: 2026-07-23 01:16
