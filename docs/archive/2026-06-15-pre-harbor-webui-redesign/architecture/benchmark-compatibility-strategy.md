# HarnessLab Benchmark 兼容策略

## 核心原则

> **HarnessLab 维护唯一 Runtime。Benchmark 只提供数据（任务集 + 验证逻辑）。**

- HarnessLab 自己管理容器生命周期
- HarnessLab 自己安装和驱动 Agent
- HarnessLab 自己执行 Verifier 并收集结果
- Adapter 只做**数据转换**，不做运行时调用

---

## 统一任务目录结构

每个 benchmark 任务转换后都变成同一套目录结构：

```
.harnesslab/tasks/<task-id>/
├── task.toml              # 超时、资源、网络策略
├── instruction.md         # 任务描述
├── environment/
│   ├── Dockerfile         # 复用官方镜像或自研
│   └── docker-compose.yaml
├── tests/
│   ├── test.sh            # 入口脚本（必须写入结果到约定路径）
│   └── ...                # 辅助脚本/数据
└── solution/
    └── solve.sh           # 参考答案（可选）
```

**Verifier 契约**：`tests/test.sh` 执行完毕后必须向约定路径写入结果。

---

## 三类 Benchmark 兼容策略

### 类型 A：有官方 Docker 镜像 + 有官方验证库

**代表**：SWE-Bench、Multi-SWE-Bench

| 维度 | 策略 |
|------|------|
| **环境** | 直接复用官方 Docker 镜像，`FROM swebench/xxx`，只叠加最小依赖（uv、git、logs 目录） |
| **验证** | 不调用官方 CLI。把官方库作为 Python 依赖，在 `tests/test.sh` 中 `import swebench.harness.grading`，调用 grading 函数，最后写入结果文件 |
| **数据** | Adapter 从 HuggingFace dataset 读取 instance，生成任务目录 |
| **示例** | `tests/test.sh` 内嵌 Python 脚本：`from swebench.harness.grading import get_logs_eval; ...; echo 1 > /harnesslab/verifier/result.json` |

### 类型 B：有官方 Docker 环境 + 有测试脚本

**代表**：Terminal-Bench

| 维度 | 策略 |
|------|------|
| **环境** | 复用官方 `docker-compose.yaml` 和 `Dockerfile`，Harbor 方式：将原 `client` service 重命名为 `main`，保留环境定义 |
| **验证** | 改造官方 `run-tests.sh` 为 `tests/test.sh`，**在末尾注入结果写入逻辑**。原测试逻辑完全保留 |
| **数据** | Adapter 遍历 `terminal-bench-core-0.1.1/<task-id>/`，复制 `task.yaml` 的 instruction、tests/、solution.sh |
| **示例** | 在 `run-tests.sh` 尾部追加：`EXIT_CODE=$?; echo $EXIT_CODE > /harnesslab/verifier/result.json; exit $EXIT_CODE` |

### 类型 C：无官方镜像 + 无官方验证库

**代表**：Aider Polyglot、LiveCodeBench、自定义 Benchmark

| 维度 | 策略 |
|------|------|
| **环境** | 从通用基础镜像构建（`buildpack-deps:jammy`、`python:3.11-slim`），自行安装语言工具链 |
| **验证** | 完全自研 verifier。Aider 类调用原生测试工具（pytest、gradle、cargo）；LiveCodeBench 类用 Python 脚本 import solution 并比对输出 |
| **数据** | Adapter 从 GitHub / HuggingFace 拉取原始数据，生成任务目录 |
| **示例** | `tests/test.sh`：`pytest /workspace/test_*.py; echo $? > /harnesslab/verifier/result.json` |

---

## Adapter 职责边界

**Adapter 只做这些**：
1. 从数据源读取 benchmark 原始任务
2. 渲染模板，生成统一任务目录（task.toml + instruction.md + environment/ + tests/ + solution/）
3. 转换 docker-compose 格式（如将 terminal-bench 的 `client` service 重命名为 `main`）

**Adapter 不做这些**：
- ❌ 调用外部 runner CLI（如 `tb run`、`run_evaluation.py`）
- ❌ 管理容器生命周期
- ❌ 安装或执行 agent
- ❌ 收集结果（由框架核心统一做）

---

## HarnessLab Runtime 核心职责

```
CLI: harnesslab run --agent claude-ds --benchmark terminal-bench
  |
  v
Benchmark Adapter (数据转换)
  |-- 读取 .benchmarks/terminal-bench/...
  |-- 生成 .harnesslab/tasks/<task-id>/
  |
  v
HarnessLab Core Runtime
  |-- BaseEnvironment: docker compose up (使用任务目录的 environment/)
  |-- AgentDriver: 在容器内安装并执行 agent (使用 AgentProfile)
  |-- Verifier: 在容器内执行 tests/test.sh
  |-- 读取 /harnesslab/verifier/result.json
  |-- BaseEnvironment: docker compose down
  |
  v
Result
```

| 组件 | 职责 |
|------|------|
| **BaseEnvironment** | `start()` / `stop()` / `exec()` / `upload_file()` / `download_file()`。基于 docker compose，支持 override 层叠 |
| **AgentDriver** | `setup(env)` 在容器内安装 agent；`run(instruction, env)` 驱动 agent 执行。内置 ClaudeCodeDriver、CodexDriver、CustomDriver |
| **Verifier** | 执行 `tests/test.sh`，从约定路径读取结果。框架级超时控制 |
| **Trial** | 编排：环境启动 → agent setup → agent run → verifier → 环境销毁 |

---

## 当前 HarnessLab 需要改什么

| 当前状态 | 目标状态 | 改造方式 |
|---------|---------|---------|
| TerminalBenchAdapter 调用 `tb run` | TerminalBenchAdapter 生成任务目录 | 重写 adapter：遍历 task 目录，渲染模板 |
| SWEBenchProAdapter 调用外部 runner | SWEBenchProAdapter 生成任务目录 + 内嵌 grading 库 | 重写 adapter：HF dataset → 任务目录；verifier 内嵌 `swebench` 库 |
| AgentProfile 配置被外部 runner 忽略 | AgentProfile 驱动 AgentDriver 在容器内生效 | 新增 AgentDriver trait，实现 ClaudeCodeDriver / CustomDriver |
| 无统一容器抽象 | BaseEnvironment 管理所有容器 | 新增 BaseEnvironment trait，实现 DockerEnvironment |
| Verifier 由外部 runner 执行 | Verifier 由框架核心执行 | 统一 verifier 契约：执行 test.sh，读取结果文件 |
