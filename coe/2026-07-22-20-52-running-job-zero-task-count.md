# Problem P-001: 运行中 Job 的任务总数显示为 0
- Status: open
- Created: 2026-07-22 20:52
- Updated: 2026-07-22 20:59
- Objective: 以运行时、数据和代码证据确定当前 running Job 任务总数为 0 的根因。
- Symptoms:
  - 用户观察到当前正在运行的 Job 在 Jobs 列表中任务总数显示为 0。
- Expected behavior:
  - running Job 应展示其实际任务总数，并随每秒刷新保持正确。
- Actual behavior:
  - 当前 running Job 的任务总数显示为 0。
- Impact:
  - 本机 OrnnLab WebUI 的运行态 Job 进度可见性不正确。
- Reproduction:
  - 打开 Jobs 页面，观察当前 running Job 的“任务总数”列。
- Environment:
  - Ubuntu 本机；branch `main`；commit `007ff2b`；本机 dev service。
- Known facts:
  - 当前 Job `run-f230b2ad02e9` 的 API 返回 `trial.total=0`。
  - 同一 Job 的 Harbor 实时结果返回 `n_total_trials=10`、running 2、pending 8。
  - Run 的 `n_tasks` 与 `result_path` 均为 NULL，原生结果文件已存在于 `job_dir/harbor_job_name/result.json`。
- Ruled out:
  - 前端 DTO 转换或表格渲染覆盖非零任务数：后端直连 API 已返回 0。
- Fix criteria:
  - 根因必须由同一 Job 的 API DTO、数据库/Harbor 原始状态和代码路径证据共同确认；修复后原始复现中任务总数应为实际非零值。
- Current conclusion: 根因已确认：运行态 DTO 只读取尚未写入数据库的 `result_path`，且把代表全量 Dataset 的 `n_tasks=NULL` 回退成 0，因此忽略 Harbor 实时结果中的 `n_total_trials=10`。
- Related hypotheses:
  - H-001
  - H-002
- Resolution basis:
  - not satisfied
- Close reason:
  - not closed

## Hypothesis H-001: 后端未从运行态 Harbor 状态取得任务总数
- Status: confirmed
- Parent: P-001
- Claim: 当前 Job 的数据库或 Harbor 原生产物已有任务规模信息，但后端 Job DTO 的运行态映射没有读取该信息，因而返回 `total=0`。
- Layer: root-cause
- Factor relation: all_of
- Depends on:
  - none
- Rationale:
  - 任务计数由后端 DTO 提供；运行过程中 Harbor 的结果文件与终态结构可能不同。
- Falsifiable predictions:
  - If true: 同一 Job 的 API 返回 `total=0`，而数据库配置、Dataset 或 Harbor 运行目录存在可证明非零任务规模的字段或文件。
  - If false: API 已返回非零总数，或所有原始运行状态同样没有任务规模信息。
- Diagnostic evidence plan:
  - Prediction or clause under test: 对比同一 running Job 的 API DTO 与 SQLite、Job 配置、Harbor 原生目录。
  - Signal: Job ID、API progress、数据库行、配置 JSON、job_dir/结果文件中的任务规模。
  - Capture method: 只读调用 Jobs API、查询 SQLite schema/行、列出并读取相关非敏感产物结构。
  - Event name or marker:
    - running_job_task_count_probe
  - Correlation keys:
    - job_id
  - Differentiates from:
    - H-002 前端映射或渲染错误
  - Supports if:
    - API 为 0 且同一 Job 的原始状态可确定任务总数非零。
  - Refutes if:
    - API 返回非零值或原始执行层明确认为任务数为 0。
  - Instrumentation status: none
  - Instrumentation lifecycle:
    - none
- Evidence gate: satisfied
- Related evidence:
  - E-001
  - E-003
  - E-004
  - E-005
  - E-007
- Conclusion: confirmed；API/Harbor 同 Job 对照与结果读取代码共同证明运行态真实结果被忽略，并被 NULL 回退为 0。
- Repair design readiness: ready; user confirmation required before implementation
- Next step: 请求用户确认后设计并实施从安全归属的 Harbor 原生结果路径读取运行态进度，并补充日志与回归测试。
- Blocker:
  - none
- Close reason:
  - not closed

## Hypothesis H-002: 前端丢失后端已返回的非零任务总数
- Status: refuted
- Parent: P-001
- Claim: 后端 API 已为当前 Job 返回非零任务总数，但前端 DTO 转换、轮询合并或表格渲染将其覆盖为 0。
- Layer: root-cause
- Factor relation: any_of
- Depends on:
  - none
- Rationale:
  - 页面每秒刷新并通过前端领域模型展示计数，转换或合并是独立故障层。
- Falsifiable predictions:
  - If true: 当前 Job 的 `/jobs` 或 `/jobs/{id}` 响应包含非零总数，而页面或前端模型显示 0。
  - If false: 后端 API 自身已经返回 0。
- Diagnostic evidence plan:
  - Prediction or clause under test: 直接读取后端和 Vite proxy 的当前 Job JSON，并追踪前端映射字段。
  - Signal: 两个 API 响应的 task 计数字段和前端映射/渲染代码。
  - Capture method: 只读 curl 响应并检查 `webUiClient`、ViewModel 与 Jobs 表格代码。
  - Event name or marker:
    - running_job_frontend_mapping_probe
  - Correlation keys:
    - job_id
  - Differentiates from:
    - H-001 后端运行态映射缺失
  - Supports if:
    - API 返回非零，但前端转换或渲染结果为 0。
  - Refutes if:
    - 后端和代理 API 均返回 0。
  - Instrumentation status: none
  - Instrumentation lifecycle:
    - none
- Evidence gate: satisfied
- Related evidence:
  - E-002
  - E-003
- Conclusion: refuted；后端直连 API 已经返回 `trial.total=0`，前端没有机会取得非零值。
- Repair design readiness: not applicable
- Next step: 关闭该分支。
- Blocker:
  - none
- Close reason:
  - Refuted by backend API response E-003.

## Evidence E-001: 待采集后端运行态快照
- Related hypotheses:
  - H-001
- Direction: neutral
- Type: probe
- Source: pending
- Prediction or plan link:
  - H-001 API、SQLite 与 Harbor 状态对比计划
- Matched signal:
  - none
- Correlation keys:
  - job_id pending
- Raw content:
  ```text
  pending
  ```
- Interpretation: 尚未采集。
- Time: 2026-07-22 20:52

## Evidence E-002: 待采集前端映射快照
- Related hypotheses:
  - H-002
- Direction: neutral
- Type: probe
- Source: pending
- Prediction or plan link:
  - H-002 API 与前端字段对比计划
- Matched signal:
  - none
- Correlation keys:
  - job_id pending
- Raw content:
  ```text
  pending
  ```
- Interpretation: 尚未采集。
- Time: 2026-07-22 20:52

## Evidence E-003: 后端 API 已返回零任务总数
- Related hypotheses:
  - H-001
  - H-002
- Direction: supports H-001; refutes H-002
- Type: probe
- Source: `GET /api/webui/v1/jobs/run-f230b2ad02e9`
- Prediction or plan link:
  - H-001/H-002 对比 API 输出的计划
- Matched signal:
  - API `trial.total=0`、全部终态计数为 0
- Correlation keys:
  - job_id `run-f230b2ad02e9`
- Raw content:
  ```text
  status=running
  trial={total:0,completed:0,passed:0,notPassed:0,errored:0}
  runtimeSeconds=216
  ```
- Interpretation: 0 已由后端 DTO 产生，不是前端转换或渲染将非零值覆盖为 0。
- Time: 2026-07-22 20:56

## Evidence E-004: Harbor 实时结果明确包含十个任务
- Related hypotheses:
  - H-001
- Direction: supports
- Type: observation
- Source: `/home/zhangxu/ornnlab-data/jobs/test/claude-code-ds-proxy-rerun-20260719-220957-copy/result.json`
- Prediction or plan link:
  - H-001 原始 Harbor 状态存在非零任务规模
- Matched signal:
  - `n_total_trials=10`、`n_running_trials=2`、`n_pending_trials=8`
- Correlation keys:
  - job_id `run-f230b2ad02e9`
  - harbor_job_name `claude-code-ds-proxy-rerun-20260719-220957-copy`
- Raw content:
  ```text
  n_total_trials=10
  n_completed_trials=0
  n_running_trials=2
  n_pending_trials=8
  finished_at=null
  ```
- Interpretation: Harbor 已在运行期给出准确总数，API 的 0 与执行层事实矛盾。
- Time: 2026-07-22 20:57

## Evidence E-005: Run 在运行期不保存 result_path
- Related hypotheses:
  - H-001
- Direction: supports
- Type: code-location
- Source: `ornnlab/services/experiment_service.py:384`、`ornnlab/services/webui_job_service.py:349,472`
- Prediction or plan link:
  - H-001 运行态结果文件没有进入 DTO 读取路径
- Matched signal:
  - `_mark_run_running` 只写 `job_dir` 和 `harbor_job_name`；`result_path` 仅在 engine 返回后写入；`_job_dto` 只按数据库 `result_path` 读取结果，空值返回 `{}`
- Correlation keys:
  - job_id `run-f230b2ad02e9`
- Raw content:
  ```text
  database: n_tasks=NULL, n_attempts=1, result_path=NULL,
            job_dir=/home/zhangxu/ornnlab-data/jobs/test,
            harbor_job_name=claude-code-ds-proxy-rerun-20260719-220957-copy
  expected_total = (n_tasks if not NULL else 0) * attempts
  _result_payload(NULL) -> {}
  result_path is persisted only after await self.engine.run(...) returns
  ```
- Interpretation: 当用户选择全部任务使 `n_tasks=NULL` 时，运行态 DTO 既不使用 Harbor 已存在的原生 `result.json`，又把未知总数回退为 0；该机制完整解释症状，并会持续到 Job 终态写回 `result_path`。
- Time: 2026-07-22 20:58

## Evidence E-006: 运行事件包含未脱敏的 Agent 环境变量
- Related hypotheses:
  - H-001
- Direction: neutral
- Type: diagnostic-log
- Source: SQLite `experiment_events` 中 `harbor.job.running` 事件
- Prediction or plan link:
  - H-001 数据库/事件运行态快照检查
- Matched signal:
  - `payload_json.config.agent.env` 包含认证类环境变量及其原值
- Correlation keys:
  - job_id `run-f230b2ad02e9`
- Raw content:
  ```text
  Sensitive values intentionally omitted. The event payload persists agent.env without redaction.
  ```
- Interpretation: 这不导致任务数为 0，但属于调查中发现的独立安全风险；相关凭据应轮换，事件写入需要单独修复与历史数据处置方案。
- Time: 2026-07-22 20:54

## Evidence E-007: 五分钟运行后错误计数仍持续
- Related hypotheses:
  - H-001
- Direction: supports
- Type: reproduction
- Source: 后端 API 与 Harbor `result.json` 二次同步快照
- Prediction or plan link:
  - H-001 该机制会持续到终态写回 `result_path`，不是启动瞬间竞态
- Matched signal:
  - API runtime 299 秒时仍为 total 0；Harbor 同时仍为 total 10、running 2、pending 8
- Correlation keys:
  - job_id `run-f230b2ad02e9`
- Raw content:
  ```text
  API: status=running runtimeSeconds=299 trial.total=0
  Harbor: n_total_trials=10 n_running_trials=2 n_pending_trials=8
  ```
- Interpretation: 症状在运行五分钟后稳定存在，排除仅发生于 Harbor 初始化前的短暂竞态。
- Time: 2026-07-22 20:59
