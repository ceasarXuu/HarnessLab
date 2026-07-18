# Problem P-001: 最新 Job 创建后立即 FileNotFoundError
- Status: fixed
- Created: 2026-07-19 06:34
- Updated: 2026-07-19 07:14
- Objective: 确认最新创建 Job `run-549da9f6acf8` 的失败根因并给出证据充分的处理方向。
- Symptoms:
  - Job `claude-code-ds-test-copy-copy` 创建约 1 秒后失败。
- Expected behavior:
  - Job 进入 Harbor 执行阶段并产出 job 目录、结果或明确的任务级失败。
- Actual behavior:
  - run 状态为 failed，failure_class 为 harbor_internal_error，failure_code 为 FileNotFoundError。
- Impact:
  - 最新 Job 未开始有效 benchmark 执行，score 与 result_path 均为空。
- Reproduction:
  - 读取 `ornnlab.sqlite` 中按 created_at 最新的 run：`run-549da9f6acf8`。
- Environment:
  - Ubuntu 24.04 x86_64；OrnnLab 0.2.0；Harbor 0.13.2；Docker 29.6.1；main@8448c66。
- Known facts:
  - created_at=2026-07-18T22:32:51Z，finished_at=2026-07-18T22:32:52Z。
  - job_dir=`/home/zhangxu/ornnlab-data/jobs/test`，result_path 为空。
  - failure_summary=`[Errno 2] No such file or directory`。
  - 新 Harbor 配置与能力文件已写入，但 job.log 仍是 2026-07-17 的旧文件。
  - 新建与 resume 现已共用 `harbor_cli_executable()` 解析 Harbor CLI。
- Ruled out:
  - 将同一分钟更新的旧 resume run 误认为最新创建 Job。
- Fix criteria:
  - 从事件、配置、日志或代码路径确认具体缺失对象及触发机制，并排除主要替代原因。
- Current conclusion: 根因已修复；默认 runner 在 backend PATH 不含 `.venv/bin` 时仍解析到当前 Python 同目录的绝对 Harbor CLI，并通过真实 Harbor/Docker 生命周期验证。
- Related hypotheses:
  - H-001
  - H-002
- Resolution basis:
  - H-002；E-002、E-003、E-004、E-005、E-006、E-007、E-008、E-009。
- Close reason:
  - 修复与回归证据满足 P-001 fix criteria；未自动重跑包含外部模型成本的原 Job。

## Hypothesis H-001: Job 配置引用的迁移前输入路径不存在
- Status: refuted
- Parent: P-001
- Claim: 最新 Job 从复制配置继承了 macOS 路径或不存在的 Dataset/task 路径，Harbor 在执行前读取该路径时抛出 FileNotFoundError。
- Layer: root-cause
- Factor relation: single
- Depends on:
  - none
- Rationale:
  - 数据刚从 macOS 迁移到 Ubuntu，且 Job 名称显示为复制配置；失败发生在 1 秒内。
- Falsifiable predictions:
  - If true: webui_job_configs、事件 payload 或 Harbor 配置中存在一个当前主机不存在的输入路径，并与异常调用链一致。
  - If false: 所有配置输入路径均存在，异常明确指向可执行文件、工作目录或其他对象。
- Diagnostic evidence plan:
  - Prediction or clause under test: 配置中的路径必须逐项解析并检查存在性，异常事件应指向其中之一。
  - Signal: run 配置、experiment_events payload、harbor.config.json 和路径存在性探针。
  - Capture method: 只读查询 SQLite，读取 JSON 配置，并对明确路径执行 lstat。
  - Event name or marker:
    - run.failed
  - Correlation keys:
    - run-549da9f6acf8
    - exp-37e96ac38982
  - Differentiates from:
    - H-002 Harbor 启动命令或 cwd 缺失
  - Supports if:
    - 事件/配置中的缺失输入路径与 FileNotFoundError 调用链相符。
  - Refutes if:
    - 输入路径全部存在且错误发生在 subprocess 启动边界。
  - Instrumentation status: none
  - Instrumentation lifecycle:
    - none
- Evidence gate: satisfied
- Related evidence:
  - E-001
  - E-002
- Conclusion: 新 Job 配置中的 jobs_dir 已是 Ubuntu 路径，配置和能力产物均成功写入；异常发生在后续进程启动边界。
- Repair design readiness: blocked until Status is confirmed and Evidence gate is satisfied
- Next step: closed
- Blocker:
  - none
- Close reason:
  - H-001 已被运行时路径、产物与独立审计证据否定。

## Hypothesis H-002: Harbor 启动链引用不存在的可执行文件或工作目录
- Status: confirmed
- Parent: P-001
- Claim: OrnnLab 调用 Harbor 时构造的 executable 或 cwd 在 Ubuntu 上不存在，导致 subprocess 创建阶段抛出 FileNotFoundError。
- Layer: interaction
- Factor relation: single
- Depends on:
  - none
- Rationale:
  - 失败分类为 Harbor internal error，且没有 result_path，符合执行前 subprocess 边界失败。
- Falsifiable predictions:
  - If true: 事件 traceback 或调用代码显示 FileNotFoundError 来自 subprocess，且目标 executable/cwd 不存在。
  - If false: Harbor 已成功启动，异常来自其内部读取某个配置输入文件。
- Diagnostic evidence plan:
  - Prediction or clause under test: traceback 应定位到 subprocess spawn，或 Harbor 日志应证明进程已经启动。
  - Signal: 事件 traceback、OrnnLab service log、Harbor invocation code 和命令可用性。
  - Capture method: 读取结构化事件与日志，追踪代码调用边界，执行 command -v/lstat 探针。
  - Event name or marker:
    - harbor.subprocess.failed
  - Correlation keys:
    - run-549da9f6acf8
  - Differentiates from:
    - H-001 迁移后输入路径缺失
  - Supports if:
    - traceback 明确止于 spawn 且 executable/cwd 缺失。
  - Refutes if:
    - Harbor 进程已有输出并在读取配置路径时失败。
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
- Conclusion: `ManagedSubprocessHarborRunner` 默认执行 `harbor run`，但 daemon/backend PATH 中没有任何可执行的 `harbor`；真实 CLI 仅存在于仓库 `.venv/bin/harbor`。
- Repair design readiness: implemented；runner 已统一使用 `harbor_cli_executable()` 绝对路径并保留显式命令覆盖。
- Next step: closed
- Blocker:
  - none
- Close reason:
  - E-006 至 E-009 已完成失败复现、实现回归、真实 Harbor/Docker 和部署重载验证。

## Evidence E-001: 最新创建 run 的结构化失败记录
- Related hypotheses:
  - H-001
  - H-002
- Direction: neutral
- Type: observation
- Source: `/home/zhangxu/.ornnlab/data/ornnlab.sqlite` runs 表
- Prediction or plan link:
  - P-001 症状与目标 run 身份确认
- Matched signal:
  - failed / harbor_internal_error / FileNotFoundError
- Correlation keys:
  - run-549da9f6acf8
  - exp-37e96ac38982
- Raw content:
  ```text
  harbor_job_name=claude-code-ds-test-copy-copy
  created_at=2026-07-18T22:32:51Z
  finished_at=2026-07-18T22:32:52Z
  job_dir=/home/zhangxu/ornnlab-data/jobs/test
  result_path=null
  failure_class=harbor_internal_error
  failure_code=FileNotFoundError
  failure_summary=[Errno 2] No such file or directory
  ```
- Interpretation: 确认最新 Job 在有效结果产生前发生文件不存在错误，但单凭摘要无法识别缺失对象。
- Time: 2026-07-19 06:34

## Evidence E-002: 事件顺序和产物时间证明失败发生在 Harbor spawn 前
- Related hypotheses:
  - H-001
  - H-002
- Direction: supports
- Type: probe
- Source: experiment_events、webui_job_configs 与 `/home/zhangxu/ornnlab-data/jobs/test`
- Prediction or plan link:
  - H-001/H-002 对 Harbor 是否已经启动的区分信号
- Matched signal:
  - config/capability 于 06:32 写入；running 后立即 failed；job.log 仍停留在 2026-07-17
- Correlation keys:
  - run-549da9f6acf8
  - events 63/64
- Raw content:
  ```text
  2026-07-18T22:32:52Z harbor.job.running
  artifacts.config_path=/home/zhangxu/ornnlab-data/jobs/test/harbor.config.json
  2026-07-18T22:32:52Z harbor.job.failed FileNotFoundError
  harbor.config.json mtime=2026-07-19 06:32
  harbor.capability.json mtime=2026-07-19 06:32
  job.log mtime=2026-07-17 15:15
  ```
- Interpretation: OrnnLab 已完成配置构建和文件写入，但新的 Harbor 子进程没有产生任何日志，失败点位于 create_subprocess_exec。
- Time: 2026-07-19 06:37

## Evidence E-004: 使用 backend 实际 PATH 精确复现 spawn 失败
- Related hypotheses:
  - H-002
- Direction: supports
- Type: reproduction
- Source: 独立日志审计路径；backend 实际 PATH + `.venv/bin/python` + `asyncio.create_subprocess_exec`
- Prediction or plan link:
  - H-002 subprocess spawn 应在同一运行环境产生同型 FileNotFoundError
- Matched signal:
  - `create_subprocess_exec('harbor', '--version')` 抛出 errno 2，filename 为 harbor
- Correlation keys:
  - backend runtime environment
  - run-549da9f6acf8
- Raw content:
  ```text
  FileNotFoundError: [Errno 2] No such file or directory: 'harbor'
  ```
- Interpretation: 在生产 backend 的命令解析环境中可确定性复现 runner 下一步的失败机制。
- Time: 2026-07-19 06:39

## Evidence E-005: 独立数据库和文件审计排除输入路径与 Docker
- Related hypotheses:
  - H-001
  - H-002
- Direction: supports
- Type: external-review
- Source: Subagent 对 SQLite、JSONL、Dataset、Job 产物、Docker 与 backend PATH 的只读审计
- Prediction or plan link:
  - H-001/H-002 的主要替代原因区分
- Matched signal:
  - jobs_dir、Dataset 和 Docker 均可用；新 native job 目录/result/job.log 不存在；backend 不能解析 harbor
- Correlation keys:
  - run-549da9f6acf8
- Raw content:
  ```text
  jobs_dir exists and is writable
  terminal-bench@2.0 exists
  Docker server 29.6.1 is available
  native job directory absent
  result_path=null
  backend command -v harbor => no result
  ```
- Interpretation: 两条独立证据路径一致支持 spawn executable 缺失，并排除 Dataset、Job 输出根目录和 Docker 作为本次直接原因。
- Time: 2026-07-19 06:40

## Evidence E-006: 修复前失败测试固定三个契约缺口
- Related hypotheses:
  - H-002
- Direction: supports
- Type: test
- Source: `tests/python/test_harbor_subprocess.py`
- Prediction or plan link:
  - H-002 修复验证必须覆盖绝对 resolver、spawn 诊断和启动日志
- Matched signal:
  - 修复前 3 failed：默认裸 harbor、缺少可操作异常、缺少结构化日志
- Correlation keys:
  - pre-fix targeted test run
- Raw content:
  ```text
  3 failed, 4 passed
  expected .venv/bin/harbor, actual harbor
  expected Harbor CLI executable not found, actual No such file or directory
  expected harbor_subprocess.start, caplog empty
  ```
- Interpretation: 测试在实现前确定性捕获生产根因和可观测性缺口，避免先改后解释。
- Time: 2026-07-19 07:03

## Evidence E-007: resolver 与日志实现通过针对性回归
- Related hypotheses:
  - H-002
- Direction: supports
- Type: fix-validation
- Source: `ornnlab/services/harbor_subprocess.py` 与 Harbor 相关测试
- Prediction or plan link:
  - H-002 修复后默认命令应为绝对 CLI，显式 override 保持兼容，空命令拒绝
- Matched signal:
  - 16 个 Harbor/Experiment 针对性测试通过，ruff/format 通过
- Correlation keys:
  - post-fix targeted test run
- Raw content:
  ```text
  16 passed
  All checks passed
  default resolver: ORNNLAB_HARBOR_CLI -> PATH -> sys.executable sibling
  ```
- Interpretation: 修复统一了新建与 resume 的 executable 解析，并增加 start/spawn_failed 稳定日志且不记录 env/PATH。
- Time: 2026-07-19 07:06

## Evidence E-008: 全量质量门和真实 Harbor/Docker 测试通过
- Related hypotheses:
  - H-002
- Direction: supports
- Type: fix-validation
- Source: `scripts/test-after-change-web.sh` 主体门禁、全量 Python、真实 Harbor cancel-recovery
- Prediction or plan link:
  - H-002 修复不得破坏 Python、前端、launcher 或真实 Harbor 生命周期
- Matched signal:
  - Python 124 passed；前端 108 passed；launcher 27 passed；真实 Harbor 2 passed
- Correlation keys:
  - 2026-07-19 post-fix quality gate
- Raw content:
  ```text
  Python: 124 passed, 3 skipped
  Frontend: 108 passed
  Launcher: 27 passed
  Real Harbor/Docker: 2 passed in 196.27s
  ```
- Interpretation: 真实 Harbor 已越过原 spawn 边界并完成执行与取消恢复，原 FileNotFoundError 机制不再出现。全栈 shell 脚本的随机端口释放超时为已知独立环境问题。
- Time: 2026-07-19 07:13

## Evidence E-009: 本机 daemon 重载后解析绝对 Harbor CLI
- Related hypotheses:
  - H-002
- Direction: supports
- Type: fix-validation
- Source: `ornnlab dev restart`、backend 实际 PATH 探针与前端代理 live
- Prediction or plan link:
  - P-001 修复必须进入实际本机开发服务
- Matched signal:
  - 服务全健康；backend PATH 环境下默认 command 为 `.venv/bin/harbor run`；live=ok
- Correlation keys:
  - 2026-07-19 07:14 daemon reload
- Raw content:
  ```text
  dev service reload: ok
  default command: ['/home/zhangxu/HarnessLab/.venv/bin/harbor', 'run']
  frontend proxy live: ok
  ```
- Interpretation: 实际部署的 backend 已加载修复，不依赖 daemon PATH 中存在裸 harbor。
- Time: 2026-07-19 07:14

## Evidence E-003: backend PATH 无法解析默认 Harbor 命令
- Related hypotheses:
  - H-002
- Direction: supports
- Type: environment
- Source: `/proc/<daemonPid>/environ`、`/proc/<backendPid>/environ`、`ornnlab/services/harbor_subprocess.py`
- Prediction or plan link:
  - H-002 若为真，runner 的目标 executable 在 backend PATH 中不存在
- Matched signal:
  - command=`harbor run`；daemon/backend harbor_matches=[]；`.venv/bin/harbor --version` 返回 0.13.2
- Correlation keys:
  - run-549da9f6acf8
  - daemonPid 3644144
  - backendPid 3644152
- Raw content:
  ```text
  ORNNLAB_HARBOR_SUBPROCESS_COMMAND=None
  default command: harbor run
  daemon harbor_matches=[]
  backend harbor_matches=[]
  /home/zhangxu/HarnessLab/.venv/bin/harbor --version => 0.13.2
  ```
- Interpretation: runner 请求的裸 executable 不可解析，直接解释无文件名的 FileNotFoundError、1 秒失败和无新 job.log。
- Time: 2026-07-19 06:37
