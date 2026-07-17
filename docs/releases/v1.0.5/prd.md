# PRD: OrnnLab Harbor WebUI v1.0.5

- 状态：Active
- 更新：2026-07-18
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
| Agents | 基于 Harbor Harness 的可复用 Agent 配置 | 新建 Agent 页面；详情按 Harness 能力开放可编辑配置 |
| Environment | OrnnLab-local 可复用环境模板 | 新建/复制二级页面；custom 详情抽屉直接编辑；built-in 只读 |
| Leaderboard | 单 Dataset 的可比较 Job 排名 | Dataset 搜索和切换；Job 详情抽屉；移除收录 |
| System | OrnnLab dev service、Harbor、Docker、存储和资源健康 | 检查/安装更新、重启/停止应用级服务、清理 Docker 或 Harbor 本地缓存 |

Header 只保留品牌、Hub 连接状态、语言和主题切换。运行 Job、Docker 状态与通知中心不在 Header 出现。

## 4. 核心流程

### 4.1 Jobs 与 New Job

Jobs 是默认页面。列表展示名称与 ID、状态、Dataset、Agent Name、Harness、模型、Trial 进度、得分、成本、Token 消耗、运行时长和创建时间。点击行打开可拖拽宽度的详情抽屉；抽屉展示事件、绝对日志路径、绝对产物路径和可展开 Trial 列表。Trial 只展示 Harbor 结果中可确认的状态、得分、成本、Token、时长、重试次数和日志路径；不存在的运行中百分比、分析路径或验证器内部字段不得伪造。

“恢复”只对服务端确认具备 Harbor 原生恢复产物的失败或中断 Job 展示，不能仅凭状态推断。恢复 Operation 成功、失败或取消后都要刷新 Job 列表和详情；失败时在对应 Job 抽屉内展示产品化错误，并保留事件日志用于诊断。

Job 详情提供“复制”入口。复制是只读的草稿准备动作：点击后进入 New Job 页面，不创建、不排队也不执行 Job。新草稿回填原 Job 的完整配置，名称追加 `-copy`，工作目录保持不变；已删除的 Agent、模型、Dataset 或 Environment 使用当前资源目录的默认值。原 Job 没有可读取的 OrnnLab 配置时留在详情页并提示无法复制。

“新建 Job”是 Jobs 下的二级页面，包含：

- 基础：Job 名称、工作目录、Dataset、已配置 Agent、本次运行模型、Environment、并发、每个 Task 重复次数、debug 模式、是否计入排行榜、备注。
- Tasks：Dataset 的 Task 白名单。默认全部选择；支持搜索、单项开关、对当前搜索结果的批量开启/关闭，以及附加指令文件路径。
- 验证器：仅支持“使用 Dataset 默认验证器”与“跳过验证”。跳过验证时强制不计入排行榜。
- 运行策略：总/Agent/Verifier/Agent 初始化/环境构建超时倍率、失败重试次数与异常命中规则、重试间隔和高级参数。失败重试默认关闭。

Agent 配置可以引用 Harbor 内置 Harness，也可以引用自定义 Harness，并保存该模板允许使用的模型集合。New Job 选择 Agent 后，必须从该集合中选择一个本次运行使用的具体模型；后端校验归属后将其写入 Harbor `AgentConfig.model_name`。凭证、Skills、MCP、专属参数和超时仍由 Agent 模板展开。New Job 不包含输出、Hub 上传、插件、环境细节或 CLI `--yes` 等非交互参数。

### 4.2 Datasets 与 Tasks

Dataset 列表展示来源、路径、大小、Task 数、下载状态和操作。未下载 Dataset 可下载或取消下载；已下载 registry Dataset 可拉取更新、迁移位置或删除本地数据；本地导入 Dataset 仅可重新定位或移除登记，不能由 OrnnLab 删除原目录。删除与清理都必须二次确认。

#### Dataset 存储位置管理

- 每次下载 Harbor registry Dataset 时，用户通过本机系统目录选择器选择该 Dataset 的父目录；路径框只展示所选路径，不要求手工输入。OrnnLab 默认填充上一次选择的目录。
- OrnnLab 在所选父目录中创建并管理一个基于 Dataset 名称和版本的唯一子目录，例如 `terminal-bench@2.0`。用户不编辑该子目录名。
- 同一个 Dataset 只维护一个本地副本和一个当前位置。用户变更位置时，OrnnLab 将已下载目录移动到新选择的父目录，移动成功后更新记录；移动失败则保留原位置并明确报错。
- 如果目标父目录下已存在同名 Dataset 子目录，下载或迁移被阻止；不自动合并、续传、覆盖或清空既有内容。
- OrnnLab 只删除它创建并登记的 Dataset 子目录，绝不删除用户选择的父目录。
- 用户在 OrnnLab 外移动或删除已下载目录后，路径状态显示为“不可用”；用户可选择该 Dataset 当前所在目录重新定位，或移除该条目录记录。
- 导入本地 Dataset 不复制也不移动；OrnnLab 只登记其原始目录、加载其中的 Task，并在路径不可用时允许重新定位或移除登记。

Dataset 详情抽屉展示来源、本地路径、大小、Registry 与 Task 列表。Task 列表支持搜索；不提供 Dataset 级 `split` 筛选，因为当前 Harbor Dataset contract 没有该通用配置。点击“运行单 Task”进入 New Job 并预选择该 Task，而不是伪造一个独立的单 Task 执行接口。

已下载或本地导入的 Dataset 应从每个 Task 的 `task.toml` 和 `environment/` 目录解析运行环境摘要，在 Task 展开区展示操作系统、预构建镜像、镜像 Registry 声明的平台、Dockerfile/Docker Compose 定义、构建超时、网络模式、资源要求和工作目录。远程但未下载的 Registry Dataset 只展示 Registry 已提供的 Task 名称，并提示下载后可查看运行环境。v1.0.5 不判断本机兼容性、不构建替代镜像，也不因解析结果阻止 Job；镜像平台查询失败时只显示“未知”。

### 4.3 Agents

Agent = Harness x 用户配置。Agent Name 是用户命名，Harness 是 Harbor AgentName 或 custom import path。

Harbor 内置的 30 种 `AgentName` 在 OrnnLab 中只作为 Harness 模板目录，不作为已配置 Agent 展示，也不自动写入 Agent 存储。首次初始化时 Agent 目录为空；用户进入“新建 Agent”二级页面时，Agent Name 与 Harness 均为空，不做默认选择或自动命名。用户主动选择 Harness、填写名称及该 Harness 支持的配置并保存成功后，才产生可供 Job 复用的 Agent。

Agent 详情抽屉不设置独立保存按钮。用户修改配置后由前端短暂防抖并自动保存；连续修改必须串行提交最新值，旧请求不得覆盖较新的本地编辑。保存失败继续通过统一 Operation 错误反馈展示，失败值不得被标记为已保存。

所有新建/保存类表单使用统一合法性反馈：操作按钮仅在无写权限或请求执行中禁用，不因字段缺失而静默禁用；用户提交不合法表单后，只在具体字段下方显示红色错误文字，不显示顶部错误摘要，也不为输入组件增加红色描边；提交时自动聚焦第一个错误字段。首次提交后错误随编辑实时更新。API 或异步 Operation 拒绝请求时，使用独立的红色错误文字反馈，不在页面底部重复展示。该规范由共享组件和 Storybook 状态约束管理。

- Harness 模板：由当前 Harbor 安装动态提供，只读、不可直接运行，不出现在 Agent 列表中。
- Agent profile：用户创建后可编辑、删除；创建后 Harness 不可更换。可选模型集合、环境变量、Skills 来源、MCP server、kwargs、安装与运行超时都保存在 profile。
- MCP server 只支持 Harbor `stdio`（command + args）、`sse` 或 `streamable-http`（URL）配置。WebUI 不承诺安装、部署或 compose sidecar。
- Skills 来源可填写单个 `SKILL.md` 目录或包含多个 Skill 目录的集合目录，支持系统目录选择。
- Agent 的“环境变量”不是凭证库。新增变量时优先从当前 Harness 的已验证变量目录中选择，也允许切换为自定义变量名；该目录由 Harbor descriptor 与适配器实际读取变量共同组成，不能把不完整的 `ENV_VARS` 当作全集。已选择变量不得重复。Claude Code 提供 Anthropic API、Claude OAuth、Amazon Bedrock 三种认证方式，切换时只保留当前方式相关的认证变量，公共运行变量和用户自定义变量不受影响，内部模式开关不直接展示。`MAX_THINKING_TOKENS` 等同时具有 CLI 参数描述符的设置归入高级参数，不在环境变量中重复展示。取值方式默认“自定义”，展示值输入框并保存用户填写的字符串；切换为“继承系统变量”时不展示值输入框，只在运行时读取 OrnnLab 进程中的同名变量，API 用 `null` 表示继承。自定义值会保存在本地 Agent profile 中，因此敏感凭证应优先使用继承方式，直至独立 Secret 管理能力落地。

### 4.4 Environment

Environment 是 OrnnLab-local 模板层，映射 Harbor `EnvironmentConfig`。它不是 Harbor 顶层资源，也不引入 Harbor 不支持的 Docker 镜像、网络模式、健康检查、工作目录或 GPU 型号枚举。

- 基础：名称、`type` 或 `import_path`、环境变量。
- 网络：`extra_allowed_hosts` 白名单。
- 高级：`force_build`、`delete`、CPU/Memory policy、`override_cpus`、`override_memory_mb`、`override_storage_mb`、`override_gpus`、`override_tpu`、mounts、`extra_docker_compose`、kwargs。
- `override_tpu` 是 Harbor 未枚举的 `type=topology` 值；界面用类型文本和 Topology X/Y/Z 数字输入组合。

Agent 是 OrnnLab 保存的可复用运行配置，组合 Harness、可选模型集合、环境变量、Skills、MCP、专属参数和超时。Harbor 内置项只是 Harness 模板，不能直接用于 Job；用户保存 Agent profile 后才能在 New Job 中选择。New Job 不暴露 Agent 的其他内部字段，但必须从所选 Agent 的可选模型集合中单选本次运行模型。

### 4.5 Leaderboard 与 System

Leaderboard 一次只展示一个 Dataset，支持搜索并切换 Dataset。排名展示 Agent Name、Harness、模型、得分、Trial、成本、Token、时长和 Job ID。移除操作将 Job 的 `includeInLeaderboard` 设为 false；跳过验证的 Job 不可重新加入。

System 使用分组健康看板展示 OrnnLab Service、Harbor CLI、Docker、Harbor Cache、CPU、GPU 与可用存储，不再把差异较大的系统组件塞入通用表格。看板分为“服务与依赖”“存储”“主机资源”；卡片直接展示核心状态、关键值、路径、错误原因和操作，不增加详情抽屉。

每类组件使用专属状态语义：OrnnLab Service 区分运行、启动、重启、降级、停止和错误；Harbor CLI 区分已安装与未安装；Docker 区分运行中、未启动、未安装和连接异常；CPU/GPU 使用负载等级，未检测到 GPU 不得显示为笼统的 unavailable；可用存储区分充足、偏低和严重不足。Docker CLI 已安装不代表 daemon 正常，只有实际连接成功才能显示“运行中”。

OrnnLab 不安装、不内置也不识别具体 Docker Runtime。Docker 卡片通过当前 Docker Context 和标准 Docker CLI/API 展示 CLI 路径、Context、Client/Server 版本及 daemon 连接状态。页面只展示本地化的错误摘要，原始错误保留在 API 和服务日志中供诊断。

Docker 卡片始终提供一组“启动命令 + 运行”快捷控件。命令由当前用户自行填写并持久化，OrnnLab 不推断 Docker Desktop、Colima、OrbStack 等产品的启动方式。命令只允许单个可执行文件及参数，不支持管道、重定向、组合命令或其他 Shell 操作符，并以 `shell=false` 运行。输入框失焦时自动保存；运行后通过异步 Operation 轮询当前 Docker Context，daemon 可连接后刷新卡片。失败时页面只显示摘要，完整命令输出写入服务日志。Docker Runtime 的安装、关闭和故障处理仍由用户负责。

OrnnLab Service 指应用级 dev service：用户可主动启动、关闭、重启并查看状态；服务异常退出后可由应用级守护进程按退避策略重启。该能力只管理当前用户会话中的 OrnnLab 前后端进程，不安装系统服务，也不做开机或登录自启动。

系统操作有明确后果与二次确认：Docker 缓存清理只作用于 Harbor 规则匹配的资源；Storage 清理作用于 `~/.cache/harbor`；检查更新与安装更新对应 OrnnLab npm 发行包。

## 5. 交互与视觉规则

- 采用 Harbor 工具界面风格：紧凑、高信息利用率、少装饰、仅保留安全边距。
- 搜索框、下拉、按钮和菜单必须使用共享组件，dark/light 与中英文下尺寸、边框和弹层宽度一致。
- Job、Dataset、Agent、Environment 详情使用可调整宽度的右侧抽屉；抽屉内容在窄宽度下不横向溢出。
- 弹窗默认没有副标题，标题和内容左对齐。Toast 3 秒自动消失并显示倒计时。
- 异步操作进度必须依附于具体对象或控件展示，例如 Dataset 下载行；一级页面和列表底部不得追加全局“正在加载/操作进行中/已完成 100%”状态。搜索加载通过控件 busy 状态表达，操作失败仍使用统一错误反馈。
- 可添加列表在空状态下只展示“添加”操作；新增空行属于临时编辑态，用户未输入内容并将焦点移出整行时自动销毁。Key/Value 等多字段行在行内切换焦点时不得误删。
- 所有用户可见文案必须进入 i18n；新增组件必须有 Storybook story 与必要状态。
- 下拉列表在完整选项集超过 10 项时自动展示内部搜索；10 项及以下保持紧凑列表。服务端搜索下拉由调用方显式保持搜索态，不能因当前结果数减少而隐藏搜索框。
- 所有 Dataset 选择器在选中值和菜单选项中展示本地状态标签：已下载、未下载或路径不可用；搜索仍只按 Dataset 名称和版本匹配，不把状态标签当作搜索关键字。

## 6. 非目标

- 不提供 Tasks 一级页面。
- 不维护 `/api/experiments`、`/api/runs`、`/api/benchmarks` 等旧产品 API，也不创建 legacy adapter。
- 不在 New Job 中支持 custom verifier、`split`、输出管理、Hub 上传、插件管理、`env_file` 或 CLI 确认开关。
- 不把 Hub 认证、Docker 状态、通知、插件或命令罗列重复放到无关页面。
- 不在 v1.0.5 承诺 MCP server 的安装、容器部署或运行编排。
- 不根据 Dataset Task 的运行环境摘要和镜像平台推断本机兼容性，也不管理重建镜像。
- 不做系统级开机自启动、登录自启动、launchd、systemd 或 Windows Service 安装。

## 7. 验收

- 六个一级页面与上述流程可通过 WebUI 访问；没有额外一级 Tasks 或 New Job tab。
- API 模式只访问 `/api/webui/v1`，mock 模式与 API 模式遵循相同的写操作约束和 Operation 生命周期。
- built-in 资源不可产生可编辑但无法保存的伪交互；custom 资源的写操作均返回 Operation。
- Agent 详情修改无需点击保存；停止输入后自动提交，连续编辑只保留最新配置，失败时展示真实错误。
- 所有可见字段均在 Harbor/OrnnLab/本机能力边界内；未知或不支持字段被 API 拒绝。
- Given 用户选择任意可写父目录, when 下载 registry Dataset, then OrnnLab 在该目录创建唯一 Dataset 子目录、持久化路径并在后续列表和 New Job 中无感加载。
- Given 已登记路径在外部丢失, when 用户查看 Dataset, then UI 显示路径不可用并提供重新定位或移除登记；不会伪装为已下载。
- Given 已下载 Dataset 迁移位置, when 新目标中不存在同名子目录且移动成功, then 旧路径消失、新路径成为唯一受管理副本；移动失败时旧路径和登记保持不变。
- Given 用户导入已有本地 Dataset, when 移除登记, then OrnnLab 仅删除自身记录，不删除用户原始目录。
- Given Dataset 已下载或已导入, when 用户展开一个 Task, then UI 展示 Harbor Task 配置中实际存在的运行环境摘要且不输出兼容性结论；未下载 Dataset 明确提示下载后可查看。
- Given OrnnLab dev service 已启动, when 后端或前端子进程异常退出, then 应用级守护进程按退避策略重启对应服务，并在 System 页和日志中展示真实状态。
- Given Docker CLI 已安装但 daemon 未启动, when 用户查看 System, then Docker 卡片显示“未启动”和连接原因，且清理 Docker 缓存不可用；不得显示 Available 或 Healthy。
- Given 用户主动停止 OrnnLab dev service, when 停止完成, then 前后端端口释放且服务不会自动复活。
- Storybook、前端单元测试、前端生产构建与后端 API 集成测试通过；完整命令与当前进度见工程计划。
