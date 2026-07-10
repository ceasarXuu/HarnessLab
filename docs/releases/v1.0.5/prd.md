# PRD: OrnnLab Harbor WebUI v1.0.5

- 状态：Active
- 更新：2026-07-10
- 定位：基于 Harbor 的本地实验控制台

## 1. 产品意图

v1.0.5 的目标是用本地 WebUI 承接 Harbor 的日常实验工作流，减少用户记忆 CLI 参数、手工查看本地文件和在多个命令间切换的成本。OrnnLab 不替代 Harbor，不把 Harbor 不支持的能力伪装为产品功能；它将 Harbor、OrnnLab 服务和本机系统能力组织成可操作的界面。

当前前端默认运行 mock 数据模式。真实 API 已按同一契约提供，后续联调按资源逐步切换到 API 模式，页面不直接访问旧接口，也不回退到 mock 成功结果。

## 2. 产品原则

- 双向一致：可见按钮和配置必须落到 Harbor、OrnnLab 服务或本机系统的真实能力；已确认的 Harbor 能力应有合理入口。
- 无代码交互：页面不展示 CLI 命令作为主要操作方式；确认、错误和进度通过 WebUI 状态表达。
- 资源分层：Dataset 包含 Task，Job 包含 Trial；Task 与 Trial 都不是一级页面。
- 低噪声工具界面：Jobs、Datasets、Agents、Environment、Leaderboard、System 为唯一一级导航；详情用右侧抽屉，创建用二级页面。
- 配置责任明确：Agent profile 管理 Harness 运行配置；Environment 模板管理 Harbor `EnvironmentConfig`；New Job 只引用二者并配置运行级参数。

## 3. 信息架构

| 一级页面 | 责任 | 关键二级交互 |
|---|---|---|
| Jobs | Job 列表、运行状态、结果与恢复 | 新建 Job；Job 详情抽屉；取消、重试、恢复；排行榜收录 |
| Datasets | Harbor registry 与本地导入 Dataset | 详情抽屉；下载、取消下载、拉取更新、删除本地数据、导入、Task 单项运行入口 |
| Agents | Harbor 内置 Harness 目录与 custom Agent profile | 新建 Agent 页面；custom 详情可编辑；built-in 详情只读 |
| Environment | OrnnLab-local 可复用环境模板 | 新建/复制二级页面；custom 详情抽屉直接编辑；built-in 只读 |
| Leaderboard | 单 Dataset 的可比较 Job 排名 | Dataset 搜索和切换；Job 详情抽屉；移除收录 |
| System | OrnnLab、Harbor、Docker、存储和资源健康 | 检查/安装更新、重启服务、清理 Docker 或 Harbor 本地缓存 |

Header 只保留品牌、Hub 连接状态、语言和主题切换。运行 Job、Docker 状态与通知中心不在 Header 出现。

## 4. 核心流程

### 4.1 Jobs 与 New Job

Jobs 是默认页面。列表展示名称与 ID、状态、Dataset、Agent Name、Harness、模型、Trial 进度、得分、成本、Token 消耗、运行时长和创建时间。点击行打开可拖拽宽度的详情抽屉；抽屉展示事件、绝对日志路径、绝对产物路径和可展开 Trial 列表。Trial 只展示 Harbor 结果中可确认的状态、得分、成本、Token、时长、重试次数和日志路径；不存在的运行中百分比、分析路径或验证器内部字段不得伪造。

“新建 Job”是 Jobs 下的二级页面，包含：

- 基础：Job 名称、工作目录、Dataset、已配置 custom Agent、Environment、并发、每个 Task 重复次数、debug 模式、是否计入排行榜、备注。
- Tasks：Dataset 的 Task 白名单。默认全部选择；支持搜索、单项开关、对当前搜索结果的批量开启/关闭，以及附加指令文件路径。
- 验证器：仅支持“使用 Dataset 默认验证器”与“跳过验证”。跳过验证时强制不计入排行榜。
- 运行策略：总/Agent/Verifier/Agent 初始化/环境构建超时倍率、失败重试次数与异常命中规则、重试间隔和高级参数。失败重试默认关闭。

New Job 只可选择 custom Agent profile。built-in Agent 是 Harbor Harness 能力目录，没有可持久化的模型、凭证或运行配置；用户需要先基于 Harness 创建 custom Agent profile。New Job 不包含输出、Hub 上传、插件、环境细节或 CLI `--yes` 等非交互参数。

### 4.2 Datasets 与 Tasks

Dataset 列表展示来源、路径、大小、Task 数、下载状态和操作。未下载 Dataset 可下载或取消下载；已下载 registry Dataset 可拉取更新或删除本地数据；本地导入 Dataset 只能删除本地数据。删除与清理都必须二次确认。

Dataset 详情抽屉展示来源、本地路径、大小、Registry 与 Task 列表。Task 列表支持搜索；不提供 Dataset 级 `split` 筛选，因为当前 Harbor Dataset contract 没有该通用配置。点击“运行单 Task”进入 New Job 并预选择该 Task，而不是伪造一个独立的单 Task 执行接口。

### 4.3 Agents

Agent = Harness x 用户配置。Agent Name 是用户命名，Harness 是 Harbor AgentName 或 custom import path。

- built-in：来自 Harbor 的 Harness 目录，只读、不可删除、不可在 WebUI 中直接编辑。
- custom：可新建、编辑、删除；模型列表、环境变量、Skills 来源、MCP server、kwargs、安装与运行超时都保存在 Agent profile。
- MCP server 只支持 Harbor `stdio`（command + args）、`sse` 或 `streamable-http`（URL）配置。WebUI 不承诺安装、部署或 compose sidecar。
- Skills 来源可填写单个 `SKILL.md` 目录或包含多个 Skill 目录的集合目录，支持系统目录选择。

### 4.4 Environment

Environment 是 OrnnLab-local 模板层，映射 Harbor `EnvironmentConfig`。它不是 Harbor 顶层资源，也不引入 Harbor 不支持的 Docker 镜像、网络模式、健康检查、工作目录或 GPU 型号枚举。

- 基础：名称、`type` 或 `import_path`、环境变量。
- 网络：`extra_allowed_hosts` 白名单。
- 高级：`force_build`、`delete`、CPU/Memory policy、`override_cpus`、`override_memory_mb`、`override_storage_mb`、`override_gpus`、`override_tpu`、mounts、`extra_docker_compose`、kwargs。
- `override_tpu` 是 Harbor 未枚举的 `type=topology` 值；界面用类型文本和 Topology X/Y/Z 数字输入组合。

built-in 模板只读且可复制为 custom；custom 模板可编辑与删除。New Job 只选择模板，不暴露模板内部字段。

### 4.5 Leaderboard 与 System

Leaderboard 一次只展示一个 Dataset，支持搜索并切换 Dataset。排名展示 Agent Name、Harness、模型、得分、Trial、成本、Token、时长和 Job ID。移除操作将 Job 的 `includeInLeaderboard` 设为 false；跳过验证的 Job 不可重新加入。

System 展示 OrnnLab Service、Harbor CLI、Docker、Storage、CPU、GPU 与可用存储。系统操作有明确后果与二次确认：Docker 缓存清理只作用于 Harbor 规则匹配的资源；Storage 清理作用于 `~/.cache/harbor`；重启依赖受支持的服务管理器；检查更新与安装更新对应 OrnnLab npm 发行包。

## 5. 交互与视觉规则

- 采用 Harbor 工具界面风格：紧凑、高信息利用率、少装饰、仅保留安全边距。
- 搜索框、下拉、按钮和菜单必须使用共享组件，dark/light 与中英文下尺寸、边框和弹层宽度一致。
- Job、Dataset、Agent、Environment 详情使用可调整宽度的右侧抽屉；抽屉内容在窄宽度下不横向溢出。
- 弹窗默认没有副标题，标题和内容左对齐。Toast 3 秒自动消失并显示倒计时。
- 所有用户可见文案必须进入 i18n；新增组件必须有 Storybook story 与必要状态。

## 6. 非目标

- 不提供 Tasks 一级页面。
- 不维护 `/api/experiments`、`/api/runs`、`/api/benchmarks` 等旧产品 API，也不创建 legacy adapter。
- 不在 New Job 中支持 custom verifier、`split`、输出管理、Hub 上传、插件管理、`env_file` 或 CLI 确认开关。
- 不把 Hub 认证、Docker 状态、通知、插件或命令罗列重复放到无关页面。
- 不在 v1.0.5 承诺 MCP server 的安装、容器部署或运行编排。

## 7. 验收

- 六个一级页面与上述流程可通过 WebUI 访问；没有额外一级 Tasks 或 New Job tab。
- API 模式只访问 `/api/webui/v1`，mock 模式与 API 模式遵循相同的写操作约束和 Operation 生命周期。
- built-in 资源不可产生可编辑但无法保存的伪交互；custom 资源的写操作均返回 Operation。
- 所有可见字段均在 Harbor/OrnnLab/本机能力边界内；未知或不支持字段被 API 拒绝。
- Storybook、前端单元测试、前端生产构建与后端 API 集成测试通过；完整命令与当前进度见工程计划。
