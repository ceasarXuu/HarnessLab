# Harbor WebUI 功能覆盖清单

- Status: Tracking
- Created: 2026-06-28
- Updated: 2026-07-10
- Harbor baseline: 本机 `harbor` Python 包与 CLI help，`harbor_version` 由 OrnnLab runtime 读取为 `0.13.x`
- Goal: 让 demo 中可见的配置与操作逐步和 Harbor 支持能力 1:1 对等，避免做成只展示少量字段的假 WebUI。

> 文档定位：本文是 Harbor 能力覆盖跟踪清单和历史审查记录。产品范围以 [PRD](prd.md) 为准，技术边界以 [技术设计](technical-design.md) 为准，实施状态以 [工程计划与进度](engineering-plan.md) 为准。
>
> 2026-07-10 Stage 2 边界：所有当前可见操作均已通过 WebUI client 与 mock `Operation` 执行，不再由页面直接修改 demo seed state。表内的 `Partial` / `Backend only` / `Missing` 表示 Harbor/OrnnLab 后端能力或真实联调尚未完成，不表示前端仍在使用旧路由或 fake-only 成功状态。

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
   - `frontend/src/api/*`
   - `frontend/src/app/*`
   - `frontend/src/screens/*`
   - `frontend/src/ui/components/*`
   - `frontend/src/mocks/*`

## 2. 覆盖状态定义

| 状态 | 含义 |
|---|---|
| Covered | demo 中已有可见 UI，并能表达该 Harbor 概念或操作。 |
| Partial | demo 中有相近 UI，但字段、状态、操作或后端映射不完整。 |
| Backend only | OrnnLab 后端已有 API/服务，但 demo 未展示或不可操作。 |
| Missing | Harbor 支持，但 OrnnLab demo 与后端都未覆盖。 |
| Deferred | 低频或高风险操作，建议后续版本处理，但必须保留跟踪项。 |

## 2.1 2026-06-28 demo 可见性补齐记录

以下为历史审查记录，仅用于解释 v1.0.5 的收敛过程，不构成当前需求或 UI 设计依据。当时尝试露出的 Viewer、Upload、Hub 维护、额外排行榜筛选等入口，均已因缺少已确认的产品语义或 Harbor 对应能力而移除。当前可见能力、后端映射与完成状态一律以第 3-8 节的覆盖表为准。

该轮审查形成的有效结论已吸收到当前设计：New Job 只保留运行前需要决策的配置；Job 详情只保留明确的运行与排行榜操作；Dataset、Agent、Environment 和 System 只表达各自职责内的真实能力；所有写操作都经前端 `Operation` 契约，而不是由页面直接修改 demo 数据。

## 2.2 2026-06-28 对抗性审查补齐记录

该轮对抗性审查把 CLI 参数与产品操作分离：CLI 的非交互确认、输出降噪和文件输入不进入 WebUI；用户价值不足的 manifest 作者工具、诊断面板和无确认语义的 Hub 入口不进入 v1.0.5。后续实现以当前覆盖表为准，不把当时的 demo 可见性当作真实 API 接管的证据。

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
| 启动 job | `harbor run` / `harbor job start` | 旧 `/api/experiments/{id}/run` 待破坏性升级为 `/api/webui/v1/jobs` Operation | New Job 通过 `WebUiClient.createJob` 提交并轮询 mock Operation | Partial | Stage 3 直接升级后端 Job API，不维护旧 experiments run 契约。 |
| Job list | job artifact / OrnnLab runs | 旧 `/api/experiments`、`/api/runs/{id}` 待破坏性升级为 `/api/webui/v1/jobs` | Jobs 表格：name/id/status/dataset/agent/model/trials/score/cost/tokens/duration/created time | Partial | Stage 3 接真实 Harbor job id、failure class/code。 |
| Job detail | Harbor viewer/job result | 旧 run events/report 待破坏性升级为 `/api/webui/v1/jobs/{jobId}` 子资源 | 右侧 drawer：overview、events、可展开 trials、绝对 artifact paths、状态操作 | Partial | Stage 3 接真实 config/result/job.log 与状态 evidence。 |
| 取消 run | OrnnLab subprocess cancel + Harbor artifacts | 旧 run/experiment cancel 待破坏性升级为 `/api/webui/v1/jobs/{jobId}/cancel` Operation | Job drawer 通过 `WebUiClient.cancelJob` 提交并轮询 mock Operation | Partial | Stage 3 接真实 cancel API，并展示 cancel evidence。 |
| Retry / rerun | Harbor retry config；OrnnLab clone/template | `clone`、`save-template`、run retry config | Job drawer 失败态通过 `WebUiClient.retryJob` 提交 mock Operation | Partial | Stage 3 区分 retry failed trial、clone config、rerun whole job。 |
| Resume job | `harbor job resume` | 无专门 API | Job drawer 暂停态通过 `WebUiClient.resumeJob` 提交 mock Operation | Partial | Stage 3 确认真实 resume 语义并展示失败状态。 |
| Summarize job | `harbor job summarize` | report summary 读取，未接 Harbor summarize | 不展示 | Deferred | 后续确认是否需要可视化摘要生成。 |
| Download job | `harbor job download` | 无 API | 未展示 | Deferred | 后续如需时定义 Hub job 导入语义。 |
| Upload job | `harbor upload` / `harbor run --upload` | 无 API | 未展示 | Deferred | 后续定义 Upload dialog：visibility、share targets、uploaded URL。 |
| Share job | `harbor job share` | 无 API | 不展示 | Deferred | 与 Upload 能力边界确认后再进入 UI。 |
| Leaderboard inclusion | JobConfig leaderboard / submit policy | 无 API | New Job 基础 tab 与 Job detail drawer 展示“计入排行榜”开关 | Partial | 接真实 JobConfig 与 leaderboard submission 状态。 |

### 3.2 JobConfig 字段覆盖

| JobConfig 字段域 | Harbor 支持项 | 当前 demo 可见项 | 状态 | 下一步 |
|---|---|---|---|---|
| 基础 | `job_name`、`jobs_dir`、dataset、agent、environment profile、`debug`、leaderboard inclusion、notes | New Job 基础 tab 已展示；`jobs_dir` 支持手动输入和文件夹选择；model 内包在 Agent profile，environment 只选择已配置 profile，不作为 Job 级细节配置暴露；`env_file` 不进入 WebUI | Covered | 后端接入时校验字段名与 JobConfig schema 对齐；桌面壳/后端接入原生文件夹选择后回填绝对路径；CLI `--yes` 不作为用户配置项；环境变量由 Environment 模板统一承载。 |
| Tasks | `split`、`task_names`、`extra_instruction_paths` | New Job Tasks tab 以 Task 白名单列表和额外说明文件承载；默认全选，支持搜索过滤、单项开关、全部开启/全部关闭；搜索后批量开关只作用于当前过滤结果 | Covered | 后端接入时用 dataset manifest 驱动 task 列表，并将用户选择映射为 Harbor `task_names`；校验 extra instruction 路径。 |
| 尝试与并发 | `n_attempts`、`n_concurrent_trials` | attempts、concurrency | Covered | 字段名和生成配置需对齐 Harbor。 |
| Timeout | `timeout_multiplier`、`agent_timeout_multiplier`、`verifier_timeout_multiplier`、`agent_setup_timeout_multiplier`、`environment_build_timeout_multiplier` | 运行策略 tab 展示“超时策略”，标准/严格/放宽/自定义；高级区展示 setup/build timeout | Partial | 后端接入时将策略映射到 Harbor multiplier，并补充边界校验。 |
| Retry | `RetryConfig.max_retries`、include/exclude exceptions、wait multiplier/min/max wait | 运行策略 tab 默认不启用失败重试；启用后展开失败重试次数、重试场景、重试间隔；原始 exclude 放在默认收起的高级区，以“不重试的原始错误（命中规则）”列表维护 | Partial | 后端接入时将产品化场景映射到 Harbor exception names，并区分 job retry 与 UI Retry 按钮。 |
| Artifacts | `--artifact`，JobConfig artifacts | New Job 不展示；Job detail 展示运行后 artifact paths | Deferred | 如需配置 artifact collection，后续先定义产品语义再进入 UI。 |
| Extra instructions | `extra_instruction_paths` | Tasks tab 已展示 | Covered | 后端接入时校验路径存在性。 |
| Metrics | `MetricConfig.type` | New Job 不展示；Leaderboard 展示排名 metric | Deferred | 后续如果用户需要选择评分口径，再按 dataset/leaderboard 语义设计。 |
| Plugins | `plugins`，`PluginConfig.import_path/kwargs`；CLI `harbor plugins` | New Job 不展示 | Deferred | 接真实 `harbor plugins list` 后进入插件管理或高级扩展入口。 |
| Hub upload/share | `--upload`、`--public/--private`、`--share-org` | New Job 与 Job detail 均不展示 | Deferred | 上传属于运行后分发动作，接真实 Hub 认证、上传和组织权限后再设计。 |

### 3.3 Agent 配置字段覆盖

| Harbor AgentConfig 字段 | 当前 OrnnLab 支持 | 当前 demo | 状态 | 下一步 |
|---|---|---|---|---|
| `name` | `AgentProfile.harbor.agent` | New Job 选择 agent；Agents 表格展示名称 | Covered | 需从真实 agents API 拉取。 |
| `import_path` | Agent compile 后写入 `harbor_import_path` | New Agent 与 Agent detail 支持配置 custom import path | Partial | Stage 3 校验/持久化 import path。 |
| `model_name` | `AgentProfile.harbor.model` | Agent detail 以可增删 model-name 列表配置；New Job 选择 Agent 后带出默认 model | Partial | Stage 3 校验 Agent-compatible model。 |
| `kwargs` | `AgentProfile.harbor.kwargs` | Agent Advanced tab 提供 key/value 编辑 | Partial | Stage 3 持久化与 schema 校验。 |
| `env` | `AuthProfile.inherit_env`、编译时 resolve env | Agent Basic tab 提供 API key / Base URL env 与 agent env 编辑，不展示 secret 值 | Partial | Stage 3 实现 secret readiness。 |
| `skills` | `SkillsProfile.paths` | Agent Skills tab 支持技能源路径与文件夹选择 | Partial | Stage 3 校验路径、挂载与执行时解析。 |
| `mcp_servers` | `McpProfile.config_paths`，Harbor `MCPServerConfig` | Agent MCPs tab 支持 MCP server 列表与部署/传输配置 | Partial | Stage 3 生成真实 task 环境与 MCP connection。 |
| `override_timeout_sec` | runtime agent timeout | Agent detail 展示 runtime default，未作为编辑项 | Deferred | 后续按实际 Harness 支持范围开放。 |
| `override_setup_timeout_sec` | runtime setup timeout | Agent detail 展示 setup timeout，未作为编辑项 | Deferred | 后续按实际 Harness 支持范围开放。 |
| `max_timeout_sec` | Harbor 支持 | Agent detail 展示 max timeout，未作为编辑项 | Deferred | 后续按实际 Harness 支持范围开放。 |
| `extra_allowed_hosts` | Harbor 支持 | 网络访问配置收敛在 Environment 模板，不在 Agent 层重复展示 | Out of scope | 由 Environment 的 allowlist / runtime override 统一承载。 |

### 3.4 Environment / Verifier 覆盖

| 字段域 | Harbor 支持项 | 当前 demo | 状态 | 下一步 |
|---|---|---|---|---|
| Environment 模板 | OrnnLab-local profile，底层映射 task `[environment]` 字段和 Job/Trial `EnvironmentConfig`：`type` / `import_path`、`force_build`、`delete`、resources、mounts、env/kwargs、extra_allowed_hosts、runtime overrides | Environment 一级页展示模板；new/copy 使用二级页面；detail 用抽屉；抽屉打开即编辑；custom 可 delete；New Job 只下拉选择模板 | Partial | 后端实现本地模板 API，并在创建 JobConfig 时展开为 Harbor 真实字段。 |

Environment 字段控件约束：枚举字段使用下拉，布尔字段使用 switch，资源数量使用数字输入，TPU 使用 type 下拉 + topology 维度数字输入，host / GPU type 使用 tag 输入，`env` / `kwargs` 使用 Key-Value 多行输入，`mounts` / `healthcheck` 使用 JSON 多行输入，路径字段使用路径输入。`allowed_hosts` 与 `extra_allowed_hosts` 必须分开：前者属于 task `[environment]` network baseline，后者属于 Job/Trial `EnvironmentConfig` runtime override。
| Verifier | override/max timeout、env、import_path、kwargs、disable | New Job 验证器 tab 以“验证方式”下拉作为主入口：默认使用 Dataset 验证器；自定义模式展开 import_path、env、kwargs、max timeout；跳过验证模式映射 disable，并强制本次 Job 不计入排行榜 | Partial | 后端接入时将 UI mode 映射为 Harbor `VerifierConfig` / disable 语义，并在 API 层拒绝跳过验证的 leaderboard submission。 |

## 4. Datasets / Tasks 覆盖清单

### 4.1 Dataset catalog 与操作

| Harbor 能力 | Harbor 证据 | 当前后端 | 当前 demo | 状态 | 下一步 |
|---|---|---|---|---|---|
| Dataset list | `harbor dataset list` | 旧 `/api/benchmarks` 静态返回待破坏性升级为 `/api/webui/v1/datasets` | Datasets 表格通过 WebUI client 读取契约 fixture；本地导入通过 Operation | Partial | Stage 3 接 Harbor registry list，支持分页、registry source。 |
| Dataset detail | `DatasetConfig`，registry/local fields | 无专门 dataset API | drawer 默认展示 task 数、source、本地 path/size、registry 与 task 列表，task 列表支持 split 筛选与搜索；底层 digest/ref/manifest 命令不默认展示 | Partial | 接真实 dataset detail API，必要时增加高级 metadata 折叠区。 |
| Dataset download | `harbor dataset download` / `harbor download` | 无 | 列表和 drawer 按下载状态展示下载、取消、拉取更新、删除本地数据 | Partial | 接真实 download/cancel/delete/pull API。 |
| Dataset local import/init | `harbor dataset init`、`harbor add`、`harbor run --path` | 无 | Datasets 页“导入本地 Dataset”mock 表单，登记本地路径 | Partial | 接真实本地路径选择、manifest 探测与 JobConfig source。 |
| Dataset visibility | `harbor dataset visibility` | 无 | Dataset drawer 不展示 leaderboard inclusion | Deferred | 若 Harbor dataset visibility 进入 v1.0.5，再定义独立 dataset 可见性 UI。 |
| Publish dataset | `harbor publish` | 无 | 未展示 | Deferred | Publish wizard。 |
| Manifest add/remove/sync | `harbor add/remove/sync` | 无 | 不在详情抽屉展示 CLI 命令 | Deferred | 后续作为 Dataset Editor + manifest diff 处理。 |

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
| Task list | Dataset 下属 task | Dataset drawer 展示 task name，支持搜索和展开描述 | Partial | 接真实 dataset manifest 后展示完整 task 集合；高级 manifest 字段不默认展示。 |
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
| List agents | 当前旧后端 `GET /api/agents`（Stage 3 将破坏性升级为 `/api/webui/v1/agents`） | Agents 表格与详情通过 WebUI client 读取契约 fixture | Partial | Stage 3 接真实 API。 |
| Built-in agents | Harbor `--agent` 支持 agent name/ACP registry shorthand | 表格展示 built-in rows | Partial | 拉取 Harbor built-ins/registry shorthand 列表。 |
| Custom agent create | 当前旧后端 `POST /api/agents`，AgentProfile v2（Stage 3 待升级） | Add custom agent 按钮未接 | Backend only | 创建/编辑抽屉。 |
| Agent validate | 当前旧后端 `POST /api/agents/validate`（Stage 3 待升级） | 未展示 | Backend only | 表单实时校验。 |
| Agent compile | 当前旧后端 `POST /api/agents/{id}/compile`（Stage 3 待升级） | 未展示 | Backend only | Compile action + 编译结果。 |
| Agent update/delete | 当前旧后端 `PUT/DELETE /api/agents/{id}`（Stage 3 待升级） | 未展示 | Backend only | Settings/edit/delete 操作。 |
| Agent auth/env readiness | `AuthProfile.inherit_env/include_paths` | 未展示 | Backend only | 显示缺失 secret，不泄露值。 |
| Skills/MCP | `SkillsProfile` / `McpProfile` | 未展示 | Backend only | Agent detail 增加配置区。 |
| Runtime backend/timeouts | `RuntimeProfile.backend/agent_timeout/setup_timeout` | 未展示 | Backend only | Agent detail 增加 runtime 区域。 |
| Adapter review/dev | `harbor adapter init/review` | 未展示 | Deferred | Developer tools。 |

## 6. Leaderboard 覆盖清单

| 项目 | Harbor / OrnnLab 证据 | 当前 demo | 状态 | 下一步 |
|---|---|---|---|---|
| Dataset-scoped ranking | 当前旧后端 `GET /api/leaderboard?benchmark=`，`harbor leaderboard submit`；Stage 3 目标为 `GET /api/webui/v1/leaderboard` | 通过 WebUI client 支持 dataset 搜索 + 下拉切换；一次展示一个 dataset | Partial | Stage 3 接真实 leaderboard API。 |
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
| Cache | `harbor cache clean` | System 展示 Docker / Storage 缓存清理确认与 Operation 状态 | Partial | Stage 3 接真实 cleanup plan，不直接破坏性删除。 |
| Plugins | `harbor plugins list`，JobConfig plugins | 未展示 | Missing | Integrations 页面或 New Job plugin picker。 |
| View artifacts | `harbor view` | Job drawer 展示绝对 artifact paths；不展示未接管的 Viewer 按钮 | Deferred | 后续定义受管启动 viewer 或内嵌 Artifact viewer。 |
| Analyze trajectories | `harbor analyze` | 不展示 | Deferred | 后续确认具体用户场景后再进入 Job/Trial detail。 |
| Publish | `harbor publish` | 未展示 | Deferred | Dataset/Task publish wizard。 |
| Upload/share | `harbor upload` / `harbor job share` | 未展示 | Deferred | 后续先定义真实认证、上传与分享语义。 |
| Real-vs-demo boundary | Harbor real subprocess exists in backend | 前端通过 contract fixture 与 mock Operation 演示；API 模式不回退成功 | Partial | Stage 3 接真实 API，不能伪装成 real state。 |

## 8. 当前 demo 已覆盖的可见操作

| 页面 | 已有可见操作 | 真实程度 |
|---|---|---|
| Jobs | 搜索、新建 Job、点击行打开 Job drawer、取消/重试/恢复、查看 events/trials/artifacts、计入排行榜开关 | 全部通过 `WebUiClient` 与 mock Operation；真实 Harbor 执行待 Stage 3。 |
| New Job | 选择 Dataset/Agent/Environment，填写并发/重复/debug/备注，通过 Tasks 白名单选择任务，通过右上角 JobConfig 复制配置，运行 Job | 表单写操作经 `createJob` Operation；`env_file` 不展示，环境变量进入 Environment 模板。 |
| Datasets | 搜索、本地导入、下载/取消/删除、点击行打开 Dataset drawer、查看 task、运行单 Task、拉取更新 | 全部通过 WebUI client 与 mock Operation；发布未展示。 |
| Agents | 搜索、新建、点击行打开 Agent drawer、配置、删除 custom Agent | 全部通过 WebUI client 与 mock Operation；真实 Agent profile 持久化待 Stage 3。 |
| Environment | 搜索、新建 custom 模板、复制模板、删除 custom 模板、点击行打开可编辑 Environment drawer | 全部通过 WebUI client 与 mock Operation；CRUD 语义是 OrnnLab-local 模板管理，不是 Harbor 原生命令。 |
| Leaderboard | dataset 搜索、dataset 下拉切换、排名表、打开 Job 抽屉、移除 Job | 全部通过 WebUI client 与 mock Operation；真实排名/提交待 Stage 3。 |
| System | 查看 OrnnLab/Harbor/Docker/Storage/资源监控状态，检查更新、重启、缓存清理 | 全部通过 WebUI client 与 mock Operation；真实系统探针与执行待 Stage 3。 |

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
