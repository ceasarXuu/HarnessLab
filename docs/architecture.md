# HarnessLab 技术架构设计

> 本文定义 HarnessLab 的核心架构。目标不是一次性写出所有实现细节，而是先确定稳定的系统边界、模块职责、扩展点和数据流，避免后续实现被某个 benchmark、某个 CLI agent 或某种 Docker 细节绑死。

## 1. 架构目标

HarnessLab 的架构必须优先满足四件事：

1. **Agent 接入轻量**：用户通过配置注册完整 CLI Agent profile，不为每个 agent 写业务代码。
2. **Benchmark 接入稳定**：Terminal-Bench 和 SWE-bench Pro 是 P0，但系统要能继续接入新的 terminal-style 或 patch-style benchmark。
3. **Run 可复现**：每次 run 都保存完整配置快照、任务快照、日志、结果、报告和 replay 所需元数据。
4. **故障可归因**：执行链路问题、benchmark 判断失败、部分得分和 usage parser warning 必须在数据模型上分开。

架构的中心不是 Web UI，也不是排行榜，而是 **Run Orchestrator + Adapter Contracts + Artifact Store**。

## 2. 关键外部事实

设计基于这些外部事实：

- Terminal-Bench 面向 terminal-native agents，任务包含 instruction、sandbox、verifier 和 oracle，官方 CLI 把 agent runtime 接到 sandboxed terminal 中执行并验证结果。参考：[Terminal-Bench](https://terminalbench.lol/)。
- Harbor 的任务结构包含 `instruction.md`、`task.toml`、`environment/`、`tests/` 和 verifier reward；它的 artifact collection 约定 `/logs/artifacts/`，并把 trial 输出组织成 agent/verifier/config/result/artifacts。参考：[Harbor task structure](https://www.harborframework.com/docs/tasks)、[Harbor artifact collection](https://www.harborframework.com/docs/run-jobs/results-and-artifacts)。
- SWE-bench 评测以 JSONL prediction 为输入，核心字段是 `instance_id`、`model_name_or_path`、`model_patch`，官方 evaluator 在 Docker 中应用 patch 并运行测试。参考：[SWE-bench evaluation guide](https://www.swebench.com/SWE-bench/guides/evaluation/)。
- SWE-bench Pro 是 Scale AI 发布的长周期软件工程 benchmark，包含公开、held-out 和商业集合，任务更长、更复杂，公开和授权数据获取能力必须被建模进 doctor/benchmark 状态。参考：[Scale Labs SWE-bench Pro](https://labs.scale.com/papers/swe_bench_pro)。

## 3. 架构原则

| 原则 | 约束 |
|---|---|
| Core owns lifecycle | 核心层拥有 run lifecycle、状态机、artifact layout 和失败分类。 |
| Adapters own translation | Benchmark adapter 只负责把外部 benchmark 翻译成 HarnessLab task/run contract。 |
| Agent is opaque | HarnessLab 不解析 agent 内部 harness，只执行完整 CLI profile 并收集可观测输出。 |
| Snapshot over reference | replay 读取 run 快照，不读取当前全局配置。 |
| Filesystem first | MVP 使用本地文件系统作为事实来源，不引入数据库。 |
| Docker behind boundary | Docker 是 sandbox provider 的实现，不泄漏到上层产品对象。 |
| Warnings are not failures | usage parser、artifact collection 这类附加能力失败不能污染 benchmark 主评分。 |

## 4. 顶层架构

```mermaid
flowchart TB
  CLI["CLI Commands"] --> App["Application Services"]

  App --> Registry["Agent Registry"]
  App --> Benchmarks["Benchmark Registry"]
  App --> Orchestrator["Run Orchestrator"]
  App --> Reports["Report Service"]
  App --> Doctor["Doctor Service"]

  Orchestrator --> Scheduler["Task Scheduler"]
  Orchestrator --> AgentRunner["Agent Runner"]
  Orchestrator --> Sandbox["Sandbox Provider"]
  Orchestrator --> Eval["Evaluation Coordinator"]
  Orchestrator --> Artifacts["Artifact Store"]
  Orchestrator --> Usage["Usage Collector"]
  Orchestrator --> Failure["Failure Classifier"]

  Benchmarks --> TerminalAdapter["Terminal Benchmark Adapter"]
  Benchmarks --> PatchAdapter["Patch Benchmark Adapter"]

  Sandbox --> Docker["Docker Provider"]
  Reports --> Artifacts
  Doctor --> Registry
  Doctor --> Benchmarks
  Doctor --> Sandbox
```

### 分层说明

| 层 | 职责 | 不做什么 |
|---|---|---|
| CLI | 参数解析、命令分发、用户输出。 | 不直接调 Docker，不直接读写 task 结果。 |
| Application Services | 编排 use case，例如 init、doctor、run、resume、replay、report open。 | 不实现 benchmark 细节。 |
| Core Domain | Run、TaskAttempt、AgentProfile、BenchmarkDescriptor、FailureClass 等核心模型。 | 不依赖 CLI、Docker、HTML。 |
| Adapter Layer | 接入 Terminal-Bench、SWE-bench Pro、未来 benchmark 和 agent profile templates。 | 不拥有全局 run 状态。 |
| Infrastructure | 文件系统、Docker、进程执行、PTY、浏览器打开报告。 | 不决定产品语义。 |

依赖方向必须单向：`CLI -> Application -> Core <- Adapters`，基础设施通过接口注入。Core 不能 import Docker SDK、HTML renderer 或具体 benchmark package。

## 5. 核心模块

### 5.1 CLI

命令：

```text
harnesslab init
harnesslab doctor
harnesslab agent list
harnesslab benchmark list
harnesslab benchmark info <benchmark>
harnesslab run --agent <profile> --benchmark <benchmark> --split <split>
harnesslab run resume <run-dir>
harnesslab run replay <run-dir>
harnesslab report open latest|<run-dir>
```

CLI 只负责：

- 解析参数。
- 加载全局配置路径。
- 调用 Application Service。
- 渲染简洁进度和摘要。
- 返回稳定 exit code。

CLI 不负责构建 Docker 命令，也不直接拼 report HTML。

稳定 exit code：

| Exit code | 语义 |
|---:|---|
| `0` | run 完成，所有 task 为 `success`；warning 不影响 exit code。 |
| `1` | run 完成，但存在 `execution_failure`。 |
| `2` | run 完成，无 execution failure，但存在 `benchmark_failure`。 |
| `3` | run 级失败，例如配置损坏、benchmark 数据不可用、Docker 不可达。 |
| `4` | run 完成，无 failure，但存在 `partial_success`。 |
| `130` | 用户中断，run 已进入 `paused`，可 resume。 |

如果同时存在多类问题，优先级为 `130 > 3 > 1 > 2 > 4 > 0`。`skipped` task 不单独决定 exit code；如果 skipped 是用户显式 limit/filter 的结果，按实际执行 task 结果返回。

### 5.2 Config Service

负责读写：

```text
~/.harnesslab/config.yaml
~/.harnesslab/agents/*.yaml
~/.harnesslab/benchmarks/
~/.harnesslab/runs/
```

设计要求：

- 所有配置解析都必须带 schema validation。
- `~`、env var、相对路径要在加载阶段规范化。
- 敏感值进入内存后可用于执行，但写入快照和报告前必须脱敏。
- 配置 schema 必须版本化：`schema_version`。

### 5.3 Agent Registry

职责：

- 检测本机 CLI Agent。
- 生成四类内置 profile 草稿。
- 加载用户手动编辑后的 profile。
- 校验 command、input_mode、auth、skills、usage parser。

Agent profile 是 opaque execution unit。核心字段：

```yaml
schema_version: 1
name: codex-default
kind: codex
command: "codex exec --full-auto {{instruction}}"
input_mode: argument
working_dir: workspace
timeout_sec: 3600
auth:
  inherit: true
  inherit_env:
    - OPENAI_API_KEY
  include_paths:
    - ~/.codex
  mount_ssh_socket: false
  mount_docker_socket: false
usage:
  parser: none
labels: {}
```

`auth.inherit` 是快捷开关，表示使用该 `kind` 的内置默认继承规则。规则展开后必须落到显式字段：

- `inherit_env`：允许传入 sandbox 的环境变量名称。
- `include_paths` / `exclude_paths`：允许挂载的本机配置路径。
- `mount_ssh_socket`：是否挂载 SSH agent socket。
- `mount_docker_socket`：是否挂载 Docker socket，默认禁止。

### 5.4 Benchmark Registry

职责：

- 注册可用 benchmark adapter。
- 暴露 benchmark metadata、split、数据准备状态。
- 为 run 创建 `BenchmarkPlan`。

核心接口：

```text
list_benchmarks() -> list[BenchmarkDescriptor]
get_info(name) -> BenchmarkInfo
inspect_data(name) -> BenchmarkDataState
prepare(name, split) -> BenchmarkPreparedState
plan_run(name, split, run_config) -> BenchmarkPlan
```

Benchmark Registry 不执行 agent，只生成可执行计划。

`BenchmarkDataState` 必须细化到 split：

```text
not_downloaded
downloading
partial
ready
corrupted
requires_auth
auth_failed
unsupported
```

### 5.5 Run Orchestrator

Run Orchestrator 是系统核心。职责：

1. 创建 run 目录。
2. 写入 run spec 和所有配置快照。
3. 通过 benchmark adapter 枚举 task。
4. 通过 scheduler 控制并发和 attempts。
5. 为每个 task 创建 sandbox。
6. 调用 Agent Runner。
7. 调用 Evaluation Coordinator。
8. 收集 artifacts、usage、logs、diff。
9. 分类 failure / warning。
10. 增量写入结果，支持 resume。
11. 完成后生成 report。

Orchestrator 必须是幂等和可恢复的：每个 task 状态落盘后，进程崩溃不应破坏已完成结果。

Task 完成时，Orchestrator 负责组装最终结果：

```text
AgentRunnerResult
  + EvaluationResult
  + ArtifactCollectionResult
  + UsageResult
  + FailureClassification
  -> TaskAttemptResult
```

这个组装逻辑可以实现为 `TaskAttemptAssembler`，但 ownership 属于 Orchestrator，不属于 adapter、agent runner 或 report service。

### 5.6 Task Scheduler

Scheduler 控制 run 内 task 并发和 attempts。MVP 不支持多个 run 同时执行的全局调度；如果用户并行启动多个 CLI 进程，互相之间不共享 scheduler。

核心输入：

```text
tasks
max_parallel
attempts
resource_pool
resume_policy
```

MVP 类型：

```text
ResourcePool
  max_parallel
  available_cpu_cores?
  available_memory_mb?
  available_disk_mb?

ResumePolicy
  skip_statuses: success | partial_success
  rerun_statuses: execution_failure | benchmark_failure | interrupted
```

核心输出：

```text
TaskAssignment(task_id, attempt, resource_reservation)
```

`resource_reservation` MVP 可只包含 slot id 和 task resource hint；不需要实现复杂资源调度，但接口必须保留 CPU/内存/磁盘字段。

资源策略：

- 默认 `max_parallel=4`，来自 PRD。
- 如果 Docker 资源不足，task 应排队而不是直接失败。
- 如果单个 task 的 resource hint 超过本机能力，preflight 阶段失败。
- `attempts > 1` 时，同一个 task 的多个 attempt 视作独立 assignment，但共享 task snapshot。

### 5.7 Evaluation Coordinator

Evaluation Coordinator 统一执行 benchmark verifier/evaluator。它不决定评分逻辑，只负责在正确环境中调用 adapter 的 evaluate contract。

```text
evaluate(task_plan, sandbox_handle, agent_result, artifacts) -> EvaluationResult
```

Evaluation Coordinator 持有两个 infrastructure port：

- `SandboxProvider`：用于 sandbox 内执行和 separate sandbox 生命周期。
- `HostProcessExecutor`：用于宿主机上的官方 evaluator 或 lightweight wrapper。

执行环境模式：

| 模式 | 用途 |
|---|---|
| `same_sandbox` | Terminal-style verifier 在 agent sandbox 内运行，使用 `SandboxProvider.exec(agent_sandbox, verifier_command)`。 |
| `separate_sandbox` | verifier 需要独立干净环境，由 Evaluation Coordinator 创建、执行、收集并销毁 verifier sandbox。 |
| `host_process` | patch-style benchmark 调用官方 evaluator 或本地 wrapper，使用 `HostProcessExecutor.exec()`。 |

Evaluation Coordinator 负责：

- 根据 `verifier_spec.environment_mode` 选择执行环境。
- 调用 verifier command 或 benchmark evaluator。
- 捕获 verifier stdout/stderr、退出码、duration。
- 把 adapter-specific evaluator 输出标准化为 `EvaluationResult`。
- 保证 separate verifier sandbox 的 cleanup，即使 verifier 失败也必须先收集 artifacts。

BenchmarkAdapter 的 `evaluate()` 负责解释 benchmark 原始输出；Evaluation Coordinator 负责执行和收集。

### 5.8 Usage Collector

Usage Collector 从 agent 原始日志或结构化日志中提取 token/cost。

```text
collect(stdout_path, stderr_path, parser_config, task_context) -> UsageResult
```

`parser_config` 来自 Agent Profile 的 `usage` 字段。MVP schema：

```text
UsageParserConfig
  parser: none | regex | json_path
  source: stdout | stderr | file
  path?
  patterns?
  json_paths?
```

`task_context` 只能包含 task id、agent profile labels、attempt dir 和 run dir，不能成为任意依赖注入容器。

`UsageResult`：

```text
status: ok | unknown | parse_error
tokens_in?
tokens_out?
total_tokens?
cost_usd?
model?
warnings[]
```

`unknown` 表示未配置 parser；`parse_error` 表示配置了 parser 但解析失败。两者都只生成 warning，不改变 benchmark 主评分。

### 5.9 Failure Classifier

Failure Classifier 的输入必须是结构化结果，不直接扫全文日志做主判断。

```text
classify(
  agent_result,
  evaluation_result,
  usage_result,
  artifact_result
) -> FailureClassification
```

日志内容可作为辅助信号，但优先级低于 stage、exit code、termination reason、evaluator code 和 explicit error code。

MVP failure code 枚举：

| Class | Codes |
|---|---|
| execution | `sandbox_create_failed`, `workspace_prep_failed`, `agent_spawn_error`, `agent_timeout`, `agent_signaled`, `agent_nonzero_exit`, `auth_failed`, `artifact_collection_failed`, `unknown_execution_error` |
| benchmark | `verifier_timeout`, `verifier_error`, `test_failed`, `no_valid_diff`, `patch_apply_failed`, `evaluator_error`, `wrong_answer`, `unknown_benchmark_failure` |
| warning | `usage_unknown`, `usage_parser_failed`, `artifact_optional_missing` |

Adapters 可以保留 benchmark 原始错误码，但必须映射到上述规范 code，报告才能跨 benchmark 比较。

## 6. Benchmark Adapter 架构

系统只定义两类一级 adapter contract：

```mermaid
flowchart LR
  BenchmarkAdapter --> TerminalStyle["TerminalStyleAdapter"]
  BenchmarkAdapter --> PatchStyle["PatchStyleAdapter"]
```

### 6.1 通用 BenchmarkAdapter

所有 adapter 必须提供：

```text
descriptor() -> BenchmarkDescriptor
inspect_data() -> BenchmarkDataState
prepare(split) -> PreparedBenchmark
list_tasks(split) -> list[TaskDescriptor]
create_task_plan(task, run_context) -> TaskPlan
evaluate(task, task_artifacts) -> EvaluationResult
snapshot(task) -> TaskSnapshot
```

核心对象：

```text
BenchmarkDescriptor
  name
  version
  homepage
  style: terminal | patch
  splits[]

TaskDescriptor
  task_id
  split
  estimated_timeout_sec
  resource_hint
  source_ref

TaskPlan
  instruction
  workspace_spec
  sandbox_spec
  verifier_spec
  artifact_spec
```

最小 spec 字段：

```text
WorkspaceSpec
  type: git_clone | local_copy | empty | benchmark_managed
  source_uri?
  target_path
  revision?
  clean: boolean

SandboxSpec
  image
  mounts[]
  env_vars[]
  network: none | restricted | full
  privileged: boolean
  resource_limits:
    cpu_cores?
    memory_mb?
    disk_mb?

VerifierSpec
  command?
  working_dir
  timeout_sec
  expected_exit_codes[]
  environment_mode: same_sandbox | separate_sandbox | host_process
  output_parser: reward_file | json | swe_evaluator | custom

ArtifactSpec
  base_dir
  globs[]
  required_paths[]
  max_size_bytes?
```

Workspace 生命周期：

```text
prepare workspace -> agent mutates workspace -> verifier/evaluator reads workspace -> artifact collector snapshots outputs
```

Artifacts 和 verifier 输出是两个 collection pass：verifier 输出用于评分，artifacts 用于复盘和报告。

`WorkspaceSpec.type` 语义：

| type | Orchestrator 行为 | SandboxProvider 行为 |
|---|---|---|
| `git_clone` | 根据 source_uri/revision 准备 repo workspace，并记录 base revision。 | 挂载或复制 workspace 到 sandbox。 |
| `local_copy` | 从本地路径复制干净 workspace。 | 挂载或复制 workspace 到 sandbox。 |
| `empty` | 创建空 workspace。 | 挂载空目录到 sandbox。 |
| `benchmark_managed` | 不自行准备 workspace，只调用 adapter 提供的 prepare hook 并接收 workspace handle/path。 | 根据 adapter 返回的 workspace handle/path 挂载或进入对应环境。 |

`benchmark_managed` 只用于外部 evaluator 强管理 workspace 的场景，例如某些 SWE-style evaluator。即使 workspace 由 benchmark 管理，Orchestrator 仍负责记录 workspace manifest、source ref 和 replay 所需 checksum。

### 6.2 TerminalStyleAdapter

用于 Terminal-Bench 和未来 Harbor-compatible terminal tasks。

运行形态：

```text
task metadata -> sandbox workspace -> agent executes instruction -> verifier script -> reward/result -> artifacts
```

设计要点：

- instruction 是 agent 输入。
- verifier 是 benchmark 原始评分来源。
- reward 文件、测试输出、agent logs 都进入 task artifact 目录。
- Terminal task 可以没有 git diff。
- verifier 失败是 Benchmark Failure；sandbox、agent、setup 失败是 Execution Failure。

### 6.3 PatchStyleAdapter

用于 SWE-bench Pro 和未来 SWE-style benchmark。

运行形态：

```text
instance metadata -> repo workspace -> agent edits repo -> collect git diff -> prediction JSONL -> official evaluator -> result
```

设计要点：

- Agent Runner 只负责让 CLI Agent 在 repo workspace 中完成任务。
- PatchStyleAdapter 负责把最终 diff 转成 benchmark prediction。
- 官方 evaluator 输出是 benchmark 主评分。
- `no_valid_diff`、`patch_apply_failed`、`test_failed` 要分开。
- replay 必须保留 instance metadata、base commit、diff、prediction JSONL 和 evaluator result。

## 7. Agent Runner 架构

Agent Runner 统一四种输入方式：

| input_mode | 语义 |
|---|---|
| `argument` | 将 `{{instruction}}` 渲染进命令。 |
| `file` | 写入 instruction 文件，通过 `{{instruction_file}}` 传入。 |
| `stdin` | 启动命令后向 stdin 写入 instruction。 |
| `tty` | 通过 PTY 注入 instruction，用于必须 TTY 的 CLI。 |

`tty` 时序：

1. execution target 先启动带 PTY 的 agent 命令；sandbox target 使用 `SandboxProvider.exec(pty=true)`，host target 使用 `HostProcessExecutor.exec(pty=true)`。
2. Agent Runner 等待可配置的 readiness signal，默认是固定短延迟加首屏输出；profile 可指定 prompt regex。
3. readiness 成功后写入 instruction，并发送 Enter。
4. 如果 readiness 超时，标记 `agent_spawn_error` 或 `agent_timeout`，由 Failure Classifier 归类。

Agent Runner 分为两层：

| 组件 | 层级 | 职责 |
|---|---|---|
| AgentCommandRenderer | Core | 渲染命令模板、准备 instruction 文件、决定 stdin/tty 输入内容。 |
| HostProcessExecutor | Infrastructure port | 管理宿主机进程/PTY 生命周期、日志流、timeout、exit status。 |
| SandboxProvider.exec | Infrastructure port | 管理 sandbox 内进程执行。 |

Runner pipeline：

```text
render command -> prepare input material -> execute process/pty -> capture logs/status -> write agent result
```

Agent Runner 不直接选择宿主机还是 sandbox 执行。Orchestrator 根据 TaskPlan/SandboxSpec 传入 execution target：

- target = `sandbox`：Agent Runner 调用 `SandboxProvider.exec()`。
- target = `host`：Agent Runner 调用 `HostProcessExecutor.exec()`，仅允许用于 doctor smoke 或 future non-sandbox mode。

MVP 正式 benchmark run 默认 target 必须是 `sandbox`。

Runner 需要输出：

```json
{
  "exit_code": 0,
  "started_at": "...",
  "finished_at": "...",
  "duration_ms": 1234,
  "stdout_path": "stdout.log",
  "stderr_path": "stderr.log",
  "termination_reason": "completed|timeout|signal|spawn_error"
}
```

Runner 不判断任务成功。任务成功只由 benchmark evaluator/verifier 决定。

## 8. Sandbox Provider 架构

MVP 只实现 Docker Provider，但 Core 只依赖 Sandbox Provider 接口。

```text
health_check()
create(task_plan, agent_profile, run_context) -> SandboxHandle
copy_in(files)
exec(command, user, env, cwd, timeout)
stream_logs()
copy_out(artifact_spec)
inspect_resources()
cleanup_orphans(run_id)
destroy()
```

Docker Provider 负责：

- 构建或拉取 benchmark/task 镜像。
- 挂载 workspace、logs、artifacts。
- 挂载授权路径和 env。
- 控制网络开关。
- 应用 CPU、内存、磁盘等资源限制。
- 注入 agent profile 所需资源。
- 提供 dry-run auth mount check 给 doctor。
- 按 run id 清理 orphaned sandbox。

Docker 细节不能泄漏到 BenchmarkAdapter 的返回值之外。Adapter 只声明 `SandboxSpec`，Provider 决定如何执行。

Sandbox 生命周期策略由 Orchestrator 决定：

- 正常 task 完成后默认销毁 sandbox。
- task 失败时仍默认销毁，但必须先完成 artifact collection。
- debug/post-mortem 保留 sandbox 是未来高级选项，不进入 MVP 默认路径。

## 9. Run 状态机

### 9.1 Run 状态

```text
created
preflight
running
paused
completed
failed
```

`paused` 表示用户中断或进程异常退出但 run 可 resume。`failed` 表示 run 级别不可继续，例如 run spec 损坏、benchmark 数据缺失且无法准备。

### 9.2 TaskAttempt 状态

```text
pending
preparing
agent_running
evaluating
collecting
success
partial_success
benchmark_failure
execution_failure
warning_only
skipped
```

状态必须增量写入 `<task-id>/result.json`。Resume 时：

- 跳过 `success` 和 `partial_success`。
- 默认重跑 `execution_failure` 和 `benchmark_failure`。
- 保留旧 result 为 attempt history。

## 10. Artifact Store

本地文件系统是 MVP 的唯一 artifact store。

```text
~/.harnesslab/runs/<agent>-<benchmark>-<split>-<timestamp>/
  run.yaml
  command.txt
  snapshots/
    config.snapshot.yaml
    agent-profile.snapshot.yaml
    benchmark.snapshot.yaml
    environment.snapshot.json
  results.json
  events.jsonl
  report.html
  tasks/
    <task-id>/
      task.snapshot.yaml
      attempts/
        1/
          instruction.md
          agent/
            stdout.log
            stderr.log
            result.json
          verifier/
            stdout.log
            stderr.log
            result.json
          artifacts/
            manifest.json
          diff.patch
          prediction.jsonl
          result.json
```

原则：

- 任何可复现输入都进入 `snapshots/` 或 `task.snapshot.yaml`。
- 任何执行输出都进入 task attempt。
- 顶层 `results.json` 是聚合索引，可以重建。
- `events.jsonl` 是 append-only 操作日志，服务排障和未来可视化。

## 11. 失败分类与结果模型

结果模型必须同时支持三层信息：

```text
Outcome: success | partial_success | failure
FailureClass: execution | benchmark | none
Warning[]: usage_unknown | usage_parser_failed | artifact_collection_failed
```

这样可以避免 usage parser 或 artifact collection 警告污染主评分。

Task result 最小结构：

```json
{
  "task_id": "example",
  "attempt": 1,
  "outcome": "failure",
  "failure_class": "benchmark",
  "failure_code": "test_failed",
  "benchmark_score": 0,
  "patch": {
    "path": "diff.patch",
    "format": "unified"
  },
  "duration_ms": 120000,
  "usage": {
    "status": "unknown"
  },
  "warnings": []
}
```

## 12. Resume 与 Replay

### Resume

Resume 读取原 run 目录：

1. 清理或标记旧 run 遗留的 orphaned sandbox。
2. 校验 run schema 和 benchmark snapshot。
3. 扫描 task result 和未完成 attempt。
4. 跳过 `success` 和 `partial_success`。
5. 将失败或中断 task 新建 attempt。
6. 增量更新 `results.json` 和 `report.html`。

Resume 修改原 run 目录，因为它表示继续同一个 run。

如果 task 停在 `preparing`、`agent_running`、`evaluating`、`collecting` 等中间状态，resume 不尝试续写原 attempt，而是把旧 attempt 标记为 `interrupted`，保留半写日志和 partial result，再创建新 attempt。

### Replay

Replay 创建新 run 目录：

1. 读取旧 run 快照。
2. 不读取当前全局 agent profile。
3. 校验 benchmark 数据和缓存是否满足旧快照。
4. 创建新的 run id。
5. 重新执行所有 task。

Replay 失败时必须说明缺失的是 agent 命令、auth、benchmark 数据、Docker 资源还是 snapshot 损坏。

ReplayValidator contract：

```text
validate(run_snapshot) -> ReplayReadiness
```

`ReplayReadiness`：

```text
ready: boolean
blockers:
  - stage
    code
    detail
```

必须校验：

- snapshot schema version 当前仍支持。
- benchmark name、version、split 与快照一致。
- benchmark/task 数据 checksum 一致；没有 checksum 时至少要求 source ref 和本地缓存 manifest 一致。
- agent command binary 仍存在；如果 profile 定义了 version command，则 version 输出要和快照一致，否则报告为 warning 或 blocker，由 profile 策略决定。
- auth include paths 和允许的 env 仍可用。
- sandbox image tag/digest 可用。

## 13. Report 架构

Report Service 输入只来自 artifact store，不直接调用 evaluator 或 Docker。

```text
results.json + snapshots + task attempts -> report model -> report artifact
```

Report model：

- run summary。
- agent profile summary。
- benchmark summary。
- failure summary。
- usage summary。
- task table。
- task detail links。
- replay command。
- original command。

MVP report artifact 必须可离线查看，不依赖运行中的 server。PRD 要求 HTML 为单文件；日志和大 artifacts 仍以相对链接指向 run 目录。生成时把 report 绝对路径写入报告内容，并在 CLI 中打印。

## 14. Doctor 架构

Doctor 是架构质量门禁，不只是用户提示。

检查项：

| 检查 | 失败级别 |
|---|---|
| config schema valid | error |
| agent command exists | error |
| Docker daemon reachable | error |
| auth include paths readable | error |
| auth paths mountable in Docker dry-run | error |
| benchmark adapter installed | error |
| split data state known | error |
| split data missing | warning or error by split |
| smoke task can be planned | error |
| usage parser valid | warning |

Doctor 输出结构化结果，CLI 渲染为人类可读表格。后续测试可以直接断言结构化 doctor result。

## 15. 日志与可观测性

每个 run 必须有三类日志：

1. `events.jsonl`：HarnessLab 自己的结构化事件。
2. `stdout.log` / `stderr.log`：agent 和 verifier 原始输出。
3. `result.json`：每个阶段的机器可读结果。

事件类型：

```text
run_created
preflight_started
task_started
sandbox_created
agent_started
agent_finished
evaluation_started
evaluation_finished
artifact_collection_finished
task_finished
run_finished
warning_emitted
error_emitted
```

日志原则：

- 原始日志不可丢。
- 结构化事件用于报告和 debug。
- 敏感 env 值进入日志前必须脱敏。
- 所有 error 必须带 stage、code、message、recoverable。

## 16. 测试架构

MVP 实现时测试按边界设计：

| 测试层 | 覆盖内容 |
|---|---|
| Unit | config validation、command rendering、state machine、failure classifier、usage parser。 |
| Contract | BenchmarkAdapter contract、SandboxProvider contract、AgentRunner contract。 |
| Integration | Docker smoke sandbox、fake terminal benchmark、fake patch benchmark。 |
| Golden | report model -> HTML snapshot。 |
| Resume/Replay | crash 后 resume、snapshot replay、缺失数据错误。 |

必须先内置 fake benchmark：

- `fake-terminal`：一个极小 terminal-style task，用于 doctor 和 CI smoke。
- `fake-patch`：一个极小 patch-style repo，用于验证 diff/prediction/evaluator flow。

Fake benchmark 不对用户开放为正式 benchmark，只作为测试工具，避免违反“不自创 benchmark”的产品定位。

## 17. 推荐模块边界

```text
HarnessLab
  CLI
  Application Services
  Core Domain
  Config
  Agent Registry
  Agent Command Renderer
  Host Process Executor
  Benchmark Registry
  Benchmark Adapters
  Task Scheduler
  Run Orchestrator
  Evaluation Coordinator
  Sandbox Providers
  Artifact Store
  Usage Collector
  Failure Classifier
  Report Service
  Test Fixtures
```

具体语言和文件树后置到实现方案。无论选择 Python、Rust 还是 Node，以上模块边界和 contract 不应改变。

## 18. 关键 ADR

### ADR-001：MVP 使用文件系统，不引入数据库

理由：

- run 是天然目录型 artifact。
- replay 需要完整文件快照。
- 本地 CLI 工具减少依赖更重要。

代价：多 run 查询和统计较弱。P1 可以增加索引文件，不必立刻上数据库。

### ADR-002：核心层不依赖 Docker

理由：Docker 是默认 sandbox 实现，不是产品模型。未来可能接远程 sandbox、CI 或 cloud worker。

### ADR-003：Benchmark 分为 terminal-style 和 patch-style

理由：Terminal-Bench 与 SWE-bench Pro 的执行模型根本不同；强行统一会导致抽象泄漏。统一点应放在 run lifecycle、artifact、result 和 failure taxonomy。

### ADR-004：Agent profile 是 opaque

理由：HarnessLab 评测的是完整 harness，不拆 model、prompt、skills。拆解差异由用户注册多个 profile 表达。

### ADR-005：Report 从 artifact store 生成

理由：报告必须可重建、可离线、可 replay。Report Service 不应该依赖活跃进程或外部 evaluator。

## 19. 架构风险

| 风险 | 应对 |
|---|---|
| Benchmark adapter 被具体实现污染 | 所有 adapter 必须通过 contract test。 |
| Docker 权限和认证继承复杂 | Doctor 做 dry-run mount check，并统一脱敏输出。 |
| CLI Agent 行为差异大 | Agent Runner 只承诺 process/pty/log contract，不承诺理解 agent 内部。 |
| SWE-bench Pro 数据准备重 | split 级数据状态建模，5 分钟承诺限定在已准备数据或 Terminal smoke。 |
| Resume 结果混乱 | 每次重跑创建新 attempt，旧 attempt 不覆盖。 |
| 报告变成技术债 | Report model 独立于 HTML renderer，HTML 可替换。 |

## 20. 开发顺序建议

1. Core models、artifact layout、state machine。
2. Config loader、agent registry、doctor 基础检查。
3. Fake terminal benchmark + Docker sandbox smoke。
4. Agent Runner 四种 input mode 中先实现 `argument/file/stdin`，`tty` 紧随其后。
5. Terminal-Bench adapter。
6. Report model + 单文件 HTML。
7. Resume。
8. Fake patch benchmark。
9. SWE-bench Pro adapter。
10. Replay。
11. Usage parser 和 warning 完整接入。

这一路径先证明架构闭环，再接入重 benchmark，避免一开始被 SWE-bench Pro 的数据和 evaluator 复杂度拖住核心抽象。
