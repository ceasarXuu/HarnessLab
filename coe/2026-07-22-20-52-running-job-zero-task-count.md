# Problem P-001: 运行中 Job 的任务总数显示为 0
- Status: fixed
- Created: 2026-07-22 20:52
- Updated: 2026-07-22 21:31
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
- Current conclusion: 已修复并部署：running DTO 安全读取当前 Harbor Job 原生结果；事件写入和历史镜像完成脱敏。
- Related hypotheses:
  - H-001
  - H-002
  - H-003
- Resolution basis:
  - H-001、H-003；E-003、E-004、E-005、E-008、E-009
- Close reason:
  - 复现等价 API 回归、全量门禁、部署健康、schema 迁移与历史脱敏均通过。

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

## Hypothesis H-003: 安全读取原生运行结果可恢复实时计数
- Status: confirmed
- Parent: P-001
- Claim: 当 running Run 的数据库 `result_path` 为空时，仅从安全单层 `job_dir/harbor_job_name/result.json` 读取进度，可返回 Harbor 的真实任务计数且不会误读共享父目录旧结果。
- Layer: fix-validation
- Factor relation: all_of
- Depends on:
  - H-001
- Rationale:
  - H-001 已证明权威结果存在于当前 Harbor Job 原生子目录，问题仅在 DTO 未读取。
- Falsifiable predictions:
  - If true: 原生结果 total 10、父目录旧结果 total 99 时，API 返回 10；不安全 job name 不会越界读取。
  - If false: API 仍返回 0/99，或路径穿越能够影响计数。
- Diagnostic evidence plan:
  - Prediction or clause under test: 定向 API 回归与本机当前 Job 重启后对照。
  - Signal: API trial total/completed/errored 与同一原生 result.json 一致。
  - Capture method: 隔离 TestClient 回归；部署后只读 curl 与 jq 对照。
  - Event name or marker:
    - job_progress.live_result_loaded
  - Correlation keys:
    - job_id
  - Differentiates from:
    - 共享父目录 fallback 或前端伪造计数
  - Supports if:
    - 定向回归和部署后当前 Job 均返回原生实时值。
  - Refutes if:
    - 任一验证返回 0、父目录旧值或越界值。
  - Instrumentation status: permanent-observability-candidate
  - Instrumentation lifecycle:
    - retain as permanent observability
- Evidence gate: satisfied
- Related evidence:
  - E-008
  - E-009
- Conclusion: confirmed；等价运行态 API 回归证明 native total 10 优先于共享父目录 stale total 99，部署后 API、迁移和脱敏状态均正确。
- Repair design readiness: implemented and validated
- Next step: 关闭案件；后续 running Job 由 `job_progress.live_result_loaded` debug 日志提供诊断证据。
- Blocker:
  - none
- Close reason:
  - Fix validation E-008 and deployment validation E-009 passed.

## Evidence E-008: 修复回归与全量门禁通过
- Related hypotheses:
  - H-003
- Direction: supports
- Type: fix-validation
- Source: `tests/python/test_webui_running_progress.py`、`tests/python/test_event_payload_security.py`、`scripts/test-after-change-web.sh`
- Prediction or plan link:
  - H-003 隔离 API 回归
- Matched signal:
  - native total 10 优先于共享父目录 stale total 99；不安全 job name 返回 0；事件数据库和镜像均无测试 secret
- Correlation keys:
  - isolated test jobs
- Raw content:
  ```text
  Python: 176 passed, 3 skipped
  Pyright: 0 errors, 0 warnings
  Frontend: 32 files, 117 tests
  Launcher: 27/27
  ```
- Interpretation: 代码级修复满足计数、路径隔离、事件脱敏和历史清理回归；仍需部署后验证用户当前 Job。
- Time: 2026-07-22 21:18

## Evidence E-009: 部署迁移和当前 Job 最终计数验证通过
- Related hypotheses:
  - H-003
- Direction: supports
- Type: fix-validation
- Source: 本机重启后的后端 API、5173 proxy、SQLite schema 与 backend log
- Prediction or plan link:
  - H-003 部署后 API、路径和脱敏验证
- Matched signal:
  - 当前 Job total 10；后端与 proxy 一致；schema 9；数据库完整 running config 计数 0；历史 JSONL 清理 10 条
- Correlation keys:
  - job_id `run-f230b2ad02e9`
  - commit `69ca3b3`
- Raw content:
  ```text
  API: status=failed total=10 completed=9 passed=5 notPassed=4 errored=1
  Proxy: total=10 completed=9 passed=5 notPassed=4 errored=1
  schema_version=9
  running_events_with_full_config=0
  event_history.redacted databaseEvents=0 mirrorFiles=9 mirrorEvents=10
  ```
- Interpretation: 修复已部署；当前 Job 的最终关系满足 10=9+1，数据库 migration 和历史镜像清理均完成。运行态原始复现由 E-008 的等价 API 用例验证。
- Time: 2026-07-22 21:31
