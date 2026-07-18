# Problem P-001: 最新 Job 创建后立即 FileNotFoundError
- Status: diagnosed
- Created: 2026-07-19 06:34
- Updated: 2026-07-19 06:41
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
- Ruled out:
  - 将同一分钟更新的旧 resume run 误认为最新创建 Job。
- Fix criteria:
  - 从事件、配置、日志或代码路径确认具体缺失对象及触发机制，并排除主要替代原因。
- Current conclusion: 高置信确认 backend PATH 无法解析 runner 默认裸命令 `harbor`，subprocess 在创建阶段抛出 FileNotFoundError；生产事件缺少 traceback/filename 是剩余证据缺口。
- Related hypotheses:
  - H-001
  - H-002
- Resolution basis:
  - 诊断完成：H-002；E-002、E-003、E-004、E-005。未实施修复，不能标记 fixed。
- Close reason:
  - not closed

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
  - not closed

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
- Conclusion: `ManagedSubprocessHarborRunner` 默认执行 `harbor run`，但 daemon/backend PATH 中没有任何可执行的 `harbor`；真实 CLI 仅存在于仓库 `.venv/bin/harbor`。
- Repair design readiness: ready；应让 runner 使用 `harbor_cli_executable()` 的绝对路径，或在 daemon backend PATH 中显式加入当前源码 `.venv/bin`。
- Next step: 等待独立证据路径复核并向用户报告。
- Blocker:
  - none
- Close reason:
  - not closed

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
