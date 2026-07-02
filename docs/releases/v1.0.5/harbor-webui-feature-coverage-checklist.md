# Harbor WebUI 功能覆盖清单

- Status: Tracking
- Created: 2026-06-28
- Updated: 2026-06-28
- Harbor baseline: 本机 `harbor` Python 包与 CLI help，`harbor_version` 由 OrnnLab runtime 读取为 `0.13.x`
- Goal: 让 demo 中可见的配置与操作逐步和 Harbor 支持能力 1:1 对等，避免做成只展示少量字段的假 WebUI。

## 1. 证据来源

本清单以三类证据为准：

1. 本机 Harbor 模型 introspection：
   - `harbor.models.job.config.JobConfig`
   - `DatasetConfig`
   - `AgentConfig`
   - `EnvironmentConfig`
   - `VerifierConfig`
   - `RetryConfig`
   - `MetricConfig`
   - `PluginConfig`
   - `TrialConfig`
   - `harbor.models.task.config.TaskConfig`
2. 本机 Harbor CLI help：
   - `harbor run --help`
   - `harbor dataset --help`
   - `harbor task --help`
   - `harbor job --help`
   - `harbor leaderboard --help`
   - `harbor auth --help`
   - `harbor publish --help`
3. OrnnLab 当前实现：
   - `ornnlab/api/*.py`
   - `ornnlab/models/*.py`
   - `ornnlab/services/harbor_engine.py`
   - `frontend/src/data/demo.ts`
   - `frontend/src/pages/*`
   - `frontend/src/components/*`

## 2. 覆盖状态定义

| 状态 | 含义 |
|---|---|
| Covered | demo 中已有可见 UI，并能表达该 Harbor 概念或操作。 |
| Partial | demo 中有相近 UI，但字段、状态、操作或后端映射不完整。 |
| Backend only | OrnnLab 后端已有 API/服务，但 demo 未展示或不可操作。 |
| Missing | Harbor 支持，但 OrnnLab demo 与后端都未覆盖。 |
| Deferred | 低频或高风险操作，建议后续版本处理，但必须保留跟踪项。 |

## 2.1 2026-06-28 demo 可见性补齐记录

本次根据清单先补齐 Harbor WebUI demo 中原本不可见的配置项和操作入口，目标是让用户在页面上能看到 Harbor 支持能力的完整信息架构。注意：这些新增项仍是 demo seed data 与前端状态，不能等同于真实 Harbor API 已接管。

已补齐的可见面包括：

1. New Job：展开 JobConfig 字段，不再只保留 source/agent/environment/concurrency/attempts；新增 job_name、jobs_dir、task 白名单、extra instructions、agent import/env/kwargs/skills/MCP、环境 backend、force_build/delete、资源限制、mounts、docker compose、verifier、timeout、retry、artifacts、metric、plugins、upload、visibility、share targets。
2. Jobs：Job drawer 新增 job_dir、split、暂停/恢复、Open viewer、Upload、harbor.capability.json、failure code、计入排行榜开关等入口；不展示 summarize/analyze/share/download 等未明确收敛的入口。
3. Datasets / Tasks：Dataset drawer 新增 registry_url/path、download_dir、manifest、拉取更新、发布；task 行级只保留 run single task 操作。
4. Agents：Agent drawer 新增 env readiness、kwargs、runtime、skills、MCP，以及 validate、compile、edit、delete 操作。
5. Leaderboard：表格新增 metric、split、submission/report path 与 submit/open viewer/share 行级操作。
6. System：补 cache 与 Harbor doctor/maintenance 状态入口；auth 只保留在 Header，plugins 归入 New Job，sync 归入 Dataset manifest editor。

后续仍需把这些可见入口逐项接入真实 API，并在对应表格中从 `Partial` / `Backend only` / `Missing` 更新为真实覆盖状态。

## 2.2 2026-06-28 对抗性审查补齐记录

根据对抗性审查，本次继续补齐上一轮仍不可见的 Harbor 能力面：

1. New Job：补 debug、quiet、agent/environment allow host、environment import/env/kwargs、全量 environment backend、suppress override warnings、override_cpus、TPU、verifier max timeout、agent setup timeout、environment build timeout、retry wait/min/max 等字段，并进入配置预览。CLI `--yes` 不进入 WebUI，由执行层处理非交互运行。`env_file` 作为 CLI 文件输入形式不进入 WebUI，环境变量统一收敛到 Environment 模板。
2. Jobs：取消独立 Trial diagnostics 模块，改为 Job Trials 表格行展开，仅展示 retries 与 log path；Job 操作只保留与当前 Harbor 产品语义明确对应的暂停/恢复、Open viewer、Upload 与排行榜开关。
3. Datasets / Tasks：保留 registry 拉取更新与发布入口；Task config explorer、manifest add/init/remove 等用户价值不足或已被收敛的操作在 v1.0.5 不展示。
4. Agents：补 adapter init/review、setup/max timeout、extra_allowed_hosts、compatible models 和 adapter review 状态。
5. Leaderboard：补 agent/status/date/comparability filters、uploaded URL、submission id、config hash、agent snapshot hash、open job/open report/download 操作。
6. System / Header：补 Harbor auth 全局状态，以及 login/logout/status、upload、leaderboard submit、job share 的维护入口。

本次仍只承诺 demo 可见性，不改变“真实 API 接管仍需后续逐项实现”的边界。

## 2.3 2026-06-30 Environment 双向一致性修正

本机 Harbor CLI 顶层命令没有 `environment` / `environments` 一等资源管理命令。Environment 真实能力来自两个层面：

1. task manifest 的 `[environment]` 字段：`docker_image`、`os`、`cpus`、`memory_mb`、`storage_mb`、`gpus`、`gpu_types`、`tpu`、`env`、`skills_dir`、`healthcheck`、`workdir`、`network_mode`、`allowed_hosts`。
2. Job / Trial 运行时 EnvironmentConfig：`type`、`import_path`、`force_build`、`delete`、`cpu_enforcement_policy`、`memory_enforcement_policy`、`override_cpus`、`override_memory_mb`、`override_storage_mb`、`override_gpus`、`override_tpu`、`mounts`、`extra_docker_compose`、`env`、`kwargs`、`extra_allowed_hosts`。

2026-07-01 用户确认：OrnnLab 需要在 Harbor 上构建一层 Environment 管理。v1.0.5 当前 demo 将 Environment 页定义为 OrnnLab-local 环境模板管理层：可搜索、可新建 custom 模板、可复制 built-in/custom 模板、可删除 custom 模板、可打开详情抽屉、可被 New Job 下拉引用。新建和复制使用二级页面，详情使用右侧抽屉；抽屉不再区分只读态和编辑态，打开后直接展示可编辑表单。该 CRUD 只管理 OrnnLab 本地模板，不是 Harbor 原生 Environment 资源命令；每个模板必须展开为 Harbor 真实支持的 EnvironmentConfig / task `[environment]` 字段。

## 3. Jobs / JobConfig 覆盖清单

### 3.1 Job 列表与生命周期操作

| Harbor 能力 | Harbor 证据 | 当前后端 | 当前 demo | 状态 | 下一步 |
|---|---|---|---|---|---|
| 创建 JobConfig | `JobConfig`，`harbor run --config` | `HarborConfigBuilder.to_job_config_payload` | New Job 表单 + 右上角 JobConfig 入口 | Partial | 表单字段需要扩展到完整 JobConfig。 |
| 启动 job | `harbor run` / `harbor job start` | `POST /api/experiments/{id}/run`，worker 执行 Harbor | New Job 的 `运行 Job` 只更新 demo seed state | Partial | demo 要展示真实 API 对接状态，区分 mock/demo 与 real backend。 |
| Job list | job artifact / OrnnLab runs | `GET /api/experiments`，`GET /api/runs/{id}` | Jobs 表格：name/status/dataset/agent/model/trials/score/cost/updated | Partial | 增加 Harbor job id、job dir、started/finished time、failure class/code。 |
| Job detail | Harbor viewer/job result | `GET /api/runs/{id}`，events/report | 右侧 drawer：overview、events、trials、artifact paths | Partial | 增加 config/result/job.log/summary/upload/share/resume 入口。 |
| 取消 run | OrnnLab subprocess cancel + Harbor artifacts | `POST /api/runs/{id}/cancel`，`POST /api/experiments/{id}/cancel` | Job drawer 有 Cancel 按钮但 demo 未接 API | Partial | 按钮接真实 cancel API，并展示 cancel evidence。 |
| Retry / rerun | Harbor retry config；OrnnLab clone/template | `clone`、`save-template`、run retry config | Job drawer 有 Retry 按钮但未定义行为 | Partial | 区分 retry failed trial、clone config、rerun whole job。 |
| Resume job | `harbor job resume` | 无专门 API | Job detail 按状态展示暂停/恢复入口 | Partial | 接真实 resume/cancel API，并展示失败状态。 |
| Summarize job | `harbor job summarize` | report summary 读取，未接 Harbor summarize | 不展示 | Deferred | 后续确认是否需要可视化摘要生成。 |
| Download job | `harbor job download` | 无 API | Jobs 有 Import 按钮但未定义来源 | Missing | Import from Hub 表单：job id/url、download target、conflict policy。 |
| Upload job | `harbor upload` / `harbor run --upload` | 无 API | Job detail 展示 Upload 入口 | Partial | Upload dialog：visibility、share targets、uploaded url。 |
| Share job | `harbor job share` | 无 API | 不展示 | Deferred | 与 Upload 能力边界确认后再进入 UI。 |
| Leaderboard inclusion | JobConfig leaderboard / submit policy | 无 API | New Job 基础 tab 与 Job detail drawer 展示“计入排行榜”开关 | Partial | 接真实 JobConfig 与 leaderboard submission 状态。 |

### 3.2 JobConfig 字段覆盖

| JobConfig 字段域 | Harbor 支持项 | 当前 demo 可见项 | 状态 | 下一步 |
|---|---|---|---|---|
| 基础 | `job_name`、`jobs_dir`、dataset、agent、environment profile、`debug`、leaderboard inclusion、notes | New Job 基础 tab 已展示；model 内包在 Agent profile，environment 只选择已配置 profile，不作为 Job 级细节配置暴露；`env_file` 不进入 WebUI | Covered | 后端接入时校验字段名与 JobConfig schema 对齐；CLI `--yes` 不作为用户配置项；环境变量由 Environment 模板统一承载。 |
| Tasks | `split`、`task_names` | New Job Tasks tab 以 Task 白名单列表承载；默认全选，支持搜索过滤、单项开关、全部开启/全部关闭；搜索后批量开关只作用于当前过滤结果 | Covered | 后端接入时用 dataset manifest 驱动 task 列表，并将用户选择映射为 Harbor `task_names`。 |
| 尝试与并发 | `n_attempts`、`n_concurrent_trials` | attempts、concurrency | Covered | 字段名和生成配置需对齐 Harbor。 |
| Timeout | `timeout_multiplier`、`agent_timeout_multiplier`、`verifier_timeout_multiplier`、`agent_setup_timeout_multiplier`、`environment_build_timeout_multiplier` | 无 | Missing | Runtime/Advanced 增加 timeout controls。 |
| Retry | `RetryConfig.max_retries`、include/exclude exceptions、wait multiplier/min/max wait | 无 | Missing | 增加 Retry 区域，区分 job retry 与 UI Retry 按钮。 |
| Artifacts | `artifacts`，`ArtifactConfig.source/destination/exclude` | Job detail 展示 artifact paths，New Job 不可配置 | Partial | New Job 增加 artifact path 列表。 |
| Extra instructions | `extra_instruction_paths` | Runtime tab 已展示 | Covered | 后端接入时校验路径存在性。 |
| Metrics | `metrics`，`MetricConfig.type/kwargs` | Leaderboard 展示 score，不能配置 metric | Missing | New Job 增加 metric selector；Leaderboard 展示 metric breakdown。 |
| Plugins | `plugins`，`PluginConfig.import_path/kwargs`；CLI `harbor plugins` | 输出 tab 已展示 plugin import_path 与空列表状态 | Partial | 接真实 `harbor plugins list`。 |
| Hub upload/share | `--upload`、`--public/--private`、`--share-org`、`--share-user` | 输出 tab 已展示 upload/share targets | Partial | 接真实 Hub 认证、上传和权限状态。 |

### 3.3 Agent 配置字段覆盖

| Harbor AgentConfig 字段 | 当前 OrnnLab 支持 | 当前 demo | 状态 | 下一步 |
|---|---|---|---|---|
| `name` | `AgentProfile.harbor.agent` | New Job 选择 agent；Agents 表格展示名称 | Covered | 需从真实 agents API 拉取。 |
| `import_path` | Agent compile 后写入 `harbor_import_path` | Agents detail 展示 adapter/import path | Partial | 增加创建/编辑 import path UI。 |
| `model_name` | `AgentProfile.harbor.model` | Agents 表格与 Agent detail 展示 models；New Job 通过 Agent 选择隐式带出默认 model | Partial | 支持多 model 和 agent-compatible model list，必要时进入 Agent 配置流修改。 |
| `kwargs` | `AgentProfile.harbor.kwargs` | 未展示 | Backend only | Agent detail 增加 kwargs editor。 |
| `env` | `AuthProfile.inherit_env`、编译时 resolve env | 未展示 | Backend only | Agent detail 显示 env readiness，不泄露 secret。 |
| `skills` | `SkillsProfile.paths` | 未展示 | Backend only | Agent detail / New Job 增加 skills picker。 |
| `mcp_servers` | `McpProfile.config_paths`，Harbor `MCPServerConfig` | 未展示 | Backend only | Agent detail 增加 MCP config list。 |
| `override_timeout_sec` | runtime agent timeout | 未展示 | Backend only | Agent runtime 区域展示。 |
| `override_setup_timeout_sec` | runtime setup timeout | 未展示 | Backend only | Agent runtime 区域展示。 |
| `max_timeout_sec` | Harbor 支持 | 未支持 | Missing | 加入 advanced runtime。 |
| `extra_allowed_hosts` | Harbor 支持 | 未展示 | Missing | Agent network policy editor。 |

### 3.4 Environment / Verifier 覆盖

| 字段域 | Harbor 支持项 | 当前 demo | 状态 | 下一步 |
|---|---|---|---|---|
| Environment 模板 | OrnnLab-local profile，底层映射 task `[environment]` 字段和 Job/Trial `EnvironmentConfig`：`type` / `import_path`、`force_build`、`delete`、resources、mounts、env/kwargs、extra_allowed_hosts、runtime overrides | Environment 一级页展示模板；new/copy 使用二级页面；detail 用抽屉；抽屉打开即编辑；custom 可 delete；New Job 只下拉选择模板 | Partial | 后端实现本地模板 API，并在创建 JobConfig 时展开为 Harbor 真实字段。 |

Environment 字段控件约束：枚举字段使用下拉，布尔字段使用 switch，资源数量使用数字输入，TPU 使用 type 下拉 + topology 维度数字输入，host / GPU type 使用 tag 输入，`env` / `kwargs` 使用 Key-Value 多行输入，`mounts` / `healthcheck` 使用 JSON 多行输入，路径字段使用路径输入。`allowed_hosts` 与 `extra_allowed_hosts` 必须分开：前者属于 task `[environment]` network baseline，后者属于 Job/Trial `EnvironmentConfig` runtime override。
| Verifier | override/max timeout、env、import_path、kwargs、disable | New Job 验证 tab 以“验证方式”下拉作为主入口：默认使用 Dataset 验证器；自定义模式展开 import_path、env、kwargs、max timeout；跳过验证模式映射 disable | Partial | 后端接入时将 UI mode 映射为 Harbor `VerifierConfig` / disable 语义，并对跳过验证的 leaderboard 影响做限制。 |

## 4. Datasets / Tasks 覆盖清单

### 4.1 Dataset catalog 与操作

| Harbor 能力 | Harbor 证据 | 当前后端 | 当前 demo | 状态 | 下一步 |
|---|---|---|---|---|---|
| Dataset list | `harbor dataset list` | `GET /api/benchmarks` 静态返回两项 | Datasets 表格 seed 数据 + 本地导入 mock row | Partial | 接 Harbor registry list，支持分页、registry source。 |
| Dataset detail | `DatasetConfig`，registry/local fields | 无专门 dataset API | drawer 展示 version/tasks/source/digest/updated 和 task 列表 | Partial | 增加 registry_url/path、download_dir、task filters、manifest path。 |
| Dataset download | `harbor dataset download` / `harbor download` | 无 | Download 按钮未接行为 | Missing | Dataset detail 加 download action。 |
| Dataset local import/init | `harbor dataset init`、`harbor add`、`harbor run --path` | 无 | Datasets 页“导入本地 Dataset”mock 表单，登记本地路径 | Partial | 接真实本地路径选择、manifest 探测与 JobConfig source。 |
| Dataset visibility | `harbor dataset visibility` | 无 | Dataset drawer 不展示 leaderboard inclusion | Deferred | 若 Harbor dataset visibility 进入 v1.0.5，再定义独立 dataset 可见性 UI。 |
| Publish dataset | `harbor publish` | 无 | 未展示 | Deferred | Publish wizard。 |
| Manifest add/remove/sync | `harbor add/remove/sync` | 无 | Manifest 工具区跟踪，不放入顶部快捷操作 | Partial | Dataset editor + manifest diff。 |

### 4.2 DatasetConfig 字段覆盖

| DatasetConfig 字段 | 当前 demo | 状态 | 下一步 |
|---|---|---|---|
| `path` | 本地导入 mock 表单和详情 path 展示 | Partial | 接真实本地 dataset/task 路径选择与校验。 |
| `name` / `version` / `ref` | name/version 展示；ref 无 | Partial | 增加 ref 与完整 dataset id。 |
| `registry_url` / `registry_path` | source 文本，非真实字段 | Partial | 明确 registry selector。 |
| `overwrite` | 无 | Missing | 下载/同步时加 conflict policy。 |
| `download_dir` | 无 | Missing | Download target picker。 |
| `task_names` | New Job Tasks tab 通过白名单列表选择；Dataset detail 展示 task 列表 | Covered | 接真实 dataset manifest 后展示完整 task 集合。 |
| `exclude_task_names` | WebUI 不暴露排除规则，避免与白名单模型重复 | Out of scope | 如后续需要高级模式，再单独设计。 |
| `n_tasks` | WebUI 不暴露数量截断，用户通过白名单明确选择任务 | Out of scope | 如后续需要抽样/随机抽样，再单独设计。 |

### 4.3 Task 支持项覆盖

| Harbor 能力 | Harbor 证据 | 当前 demo | 状态 | 下一步 |
|---|---|---|---|---|
| Task list | Dataset 下属 task | Dataset drawer 展示部分 task name/description/os/state | Partial | 增加 task id/source/ref/path/verifier/environment/steps。 |
| Run single task | `harbor trial start` / `harbor run --task` | Dataset drawer 有 Run single task 按钮 | Partial | 接真实 API，生成单 task Job。 |
| Task download | `harbor task download` | 未展示 | Missing | 不放在 task 行级快捷操作，后续如需要进入 Task detail。 |
| Start environment | `harbor task start-env` | 未展示 | Missing | 不放在 task 行级快捷操作，后续如需要进入 Task detail。 |
| Debug/check | `harbor task debug` / `harbor task check` / `harbor check` | 未展示 | Missing | 不放在 task 行级快捷操作，后续如需要进入 Task detail / diagnostics panel。 |
| Task init/update/annotate/migrate | `harbor task init/update/annotate/migrate` | 未展示 | Deferred | Authoring tools，后续版本。 |
| Task visibility | `harbor task visibility` | 未展示 | Deferred | Task settings。 |
| Task config fields | schema/package metadata/verifier/agent/environment/solution/source/steps/artifacts | 不展示，仅保留任务列表与可执行任务操作 | Out of scope | v1.0.5 不提供 Task config explorer，避免暴露用户用不到的 manifest 细节。 |

## 5. Agents 覆盖清单

| Harbor / OrnnLab 能力 | 当前后端 | 当前 demo | 状态 | 下一步 |
|---|---|---|---|---|
| List agents | `GET /api/agents` | Agents seed 表格 | Partial | demo 接真实 API。 |
| Built-in agents | Harbor `--agent` 支持 agent name/ACP registry shorthand | 表格展示 built-in rows | Partial | 拉取 Harbor built-ins/registry shorthand 列表。 |
| Custom agent create | `POST /api/agents`，AgentProfile v2 | Add custom agent 按钮未接 | Backend only | 创建/编辑抽屉。 |
| Agent validate | `POST /api/agents/validate` | 未展示 | Backend only | 表单实时校验。 |
| Agent compile | `POST /api/agents/{id}/compile` | 未展示 | Backend only | Compile action + 编译结果。 |
| Agent update/delete | `PUT/DELETE /api/agents/{id}` | 未展示 | Backend only | Settings/edit/delete 操作。 |
| Agent auth/env readiness | `AuthProfile.inherit_env/include_paths` | 未展示 | Backend only | 显示缺失 secret，不泄露值。 |
| Skills/MCP | `SkillsProfile` / `McpProfile` | 未展示 | Backend only | Agent detail 增加配置区。 |
| Runtime backend/timeouts | `RuntimeProfile.backend/agent_timeout/setup_timeout` | 未展示 | Backend only | Agent detail 增加 runtime 区域。 |
| Adapter review/dev | `harbor adapter init/review` | 未展示 | Deferred | Developer tools。 |

## 6. Leaderboard 覆盖清单

| 项目 | Harbor / OrnnLab 证据 | 当前 demo | 状态 | 下一步 |
|---|---|---|---|---|
| Dataset-scoped ranking | `GET /api/leaderboard?benchmark=`，`harbor leaderboard submit` | 支持 dataset 搜索 + 下拉切换；一次展示一个 dataset | Covered | 接真实 leaderboard API。 |
| Rank | score desc, finished desc | 展示 `#1/#2` | Covered | 后端返回 rank 或前端计算。 |
| Agent / model | runs.agent_id；Harbor agent model | 展示 agent/model | Covered | 接真实 row。 |
| Score | runs.score | 展示 score | Covered | 增加 metric name、pass@k、confidence。 |
| Trials | Harbor result stats | 展示 trials | Partial | 增加 total/completed/failed/cancelled/errored。 |
| Cost | Harbor result / token usage | demo 展示 cost，后端无字段 | Partial | 后端 schema 增加 token/cost。 |
| Duration | started/finished | demo 展示 duration，后端当前只返回 finished_at | Partial | 后端增加 started_at/finished_at/duration。 |
| Job id / link | run id / Harbor job id | 展示 job id | Partial | 行点击跳转/打开 Job drawer。 |
| Dataset version/split | benchmark_version、split、comparability_key | 未展示 | Missing | Leaderboard header 增加 version/split/comparability key。 |
| Submitted/uploaded state | `harbor leaderboard submit` 依赖 uploaded job | 未展示 | Missing | 增加 submission status、uploaded job URL、submission id。 |
| Reproducibility metadata | report_path、config hash、agent snapshot hash | 未展示 | Missing | 展开行或 drawer 展示 evidence。 |
| Filters | dataset filter only | 只有 dataset 搜索 | Partial | 增加 agent/model/status/date filters。 |
| Actions | submit/download/open job/open report/share | 无行级操作 | Missing | Action menu。 |

## 7. System / Hub / Cross-cutting 缺口

| 能力域 | Harbor 支持 | 当前 demo | 状态 | 下一步 |
|---|---|---|---|---|
| Auth | `harbor auth login/status/logout` | Header 无 auth panel | Missing | Header 增加 Auth 状态和登录/登出。 |
| Cache | `harbor cache clean` | System 未展示 cache | Missing | Cleanup plan，不直接破坏性删除。 |
| Plugins | `harbor plugins list`，JobConfig plugins | 未展示 | Missing | Integrations 页面或 New Job plugin picker。 |
| View artifacts | `harbor view` | Job drawer 展示 Open viewer 与绝对 artifact paths | Partial | 受管启动 viewer 或内嵌 Artifact viewer。 |
| Analyze trajectories | `harbor analyze` | 不展示 | Deferred | 后续确认具体用户场景后再进入 Job/Trial detail。 |
| Publish | `harbor publish` | 未展示 | Deferred | Dataset/Task publish wizard。 |
| Upload/share | `harbor upload` / `harbor job share` | Job detail 只展示 Upload；Share 暂不展示 | Partial | 先接 Upload，Share 后续确认语义。 |
| Real-vs-demo boundary | Harbor real subprocess exists in backend | 前端仍为 seed data | Partial | Demo 内标注/切换真实 API，不能伪装成 real state。 |

## 8. 当前 demo 已覆盖的可见操作

| 页面 | 已有可见操作 | 真实程度 |
|---|---|---|
| Jobs | 搜索、Import 按钮、新建 Job、点击行打开 Job drawer、暂停/恢复、Open viewer、Upload、查看 events/trials/artifacts、计入排行榜开关 | 多数为 demo state；暂停/恢复、Upload、Viewer 未接 API。 |
| New Job | 选择 Dataset/agent/environment，填写 concurrency/attempts/debug/notes，通过 Tasks 白名单选择要运行的 task，通过右上角 JobConfig 入口查看配置，Run Job | 表单字段少于 Harbor JobConfig；`env_file` 不展示，环境变量进入 Environment 模板；Run 只更新前端 demo state。 |
| Datasets | 搜索、Import/Download 按钮、点击行打开 Dataset drawer、查看 task、Run single task、拉取更新/发布 | 主要为 seed 数据；按钮未接 API。 |
| Agents | 查看 agent 列表、点击行打开 Agent drawer、Agent settings/Add custom agent 按钮 | 主要为 seed 数据；后端有 agents API 但 demo 未接。 |
| Environment | 搜索、新建 custom 模板、复制模板、删除 custom 模板、点击行打开可编辑 Environment drawer | 主要为 seed 数据；新建/复制为二级页面，详情抽屉打开即编辑；CRUD 语义是 OrnnLab-local 模板管理，不是 Harbor 原生命令。 |
| Leaderboard | dataset 搜索、dataset 下拉切换、排名表 | 主要为 seed 数据；后端有 `/api/leaderboard` 但 demo 未接。 |
| System | 查看 Harbor/Docker/Storage/Local cache 状态、系统级清理动作 | 主要为 seed 数据；后端有 system API 但 demo 未接。 |

## 9. 建议的实现优先级

### P0：让 Web 真实接管 Harbor run 主链路

1. New Job 表单扩展到 Harbor JobConfig P0 字段：
   - job name、jobs dir、dataset name/version/ref、task whitelist、
   - agent/import path/model/env/kwargs、
   - environment profile selection、
   - attempts、concurrency、timeouts、retry、artifacts。
2. 前端从 seed data 切到真实 API：
   - agents、benchmarks/datasets、experiments/runs、leaderboard、system。
3. Jobs：
   - Run、Cancel、Retry/Clone、Save template、events stream、report、artifact paths。
4. Datasets：
   - registry list、dataset detail、task list、task filter preview、download action。
5. Agents：
   - create/edit/validate/compile/delete，展示 env readiness、skills、MCP、runtime。
6. Leaderboard：
   - 接真实 `/api/leaderboard`，补 version/split/duration/cost/report/job link。

### P1：结果解释和复用

1. Artifact viewer / `harbor view` 入口。
2. `harbor analyze` / `harbor job summarize`。
3. Dataset manifest editor：add/remove/sync + diff。
4. Plugins picker。
5. Auth status。

### P2：Hub 闭环

1. Upload/share。
2. Leaderboard submit。
3. Publish dataset/task。
4. Dataset/task visibility。

### P3：作者与开发者工具

1. Task init/update/annotate/migrate。
2. Adapter init/review。
3. Package authoring workflow。

## 10. 长期检查规则

每次新增 Harbor WebUI 能力时，必须同步更新本清单：

1. 新增 UI 操作必须标注替代的 Harbor CLI/API/模型字段。
2. 如果 UI 只是 demo seed data，状态必须保持 `Partial`，不得标成 `Covered`。
3. 如果后端已有 API 但前端未接，状态使用 `Backend only`。
4. 所有写入、上传、分享、清理、删除类操作必须有确认或预览。
5. 任何无法首版覆盖的 Harbor 字段必须保留在表格中，不能从清单消失。
6. 每次 Harbor 版本升级后，必须重新运行模型字段 introspection 和 CLI help 对比。
