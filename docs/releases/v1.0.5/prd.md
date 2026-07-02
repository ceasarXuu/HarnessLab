# PRD: OrnnLab Harbor WebUI v1.0.5

- Status: Draft
- Created: 2026-06-29
- Updated: 2026-06-29
- Owner / requester: OrnnLab
- Source request: 将前面讨论过的 Harbor WebUI 功能逻辑整理成早期 PRD 草稿，后续随着前端完善持续补充。

## Requester Review Summary

- Key decisions: v1.0.5 先做 Harbor WebUI 产品化；OrnnLab 仍是基于 Harbor 的实验控制台，不改写为脱离 Harbor 的独立产品。
- Important exceptions: 当前仍保持 mock 数据，不接入后端；但 UI 上不能出现 Harbor 没有真实能力支撑的 fake 功能。
- Must-confirm before implementation: Harbor CLI/Hub 对 dataset、plugin、cache、leaderboard 提交等能力的精确命令边界仍需继续核验。
- Status reason: 已有一轮较完整的前端功能讨论，但后端接口、真实执行链路、权限和异常状态还未完全定稿。

## 1. Background And Product Intent

OrnnLab v1.0.5 的核心目标是把 Harbor 的常用 CLI 操作转成可交互的本地 WebUI。用户不应再依赖记忆命令、手写参数或手动翻文件来完成日常实验管理。

产品定位是“基于 Harbor 的本地实验控制台”。当前版本先完成 Harbor WebUI 产品形态：围绕 Job、Dataset、Agent、Environment、Leaderboard、System 六个一级页面，提供创建、运行、查看、管理、诊断等核心流程。

## 2. Goals And Success Criteria

- 用 WebUI 交互替代 Harbor 的主要 CLI 操作。
- 页面可见功能与 Harbor/OrnnLab 真实能力双向一致：Harbor 有的常用能力 WebUI 应能找到；WebUI 有的按钮后端必须能落到真实能力。
- Jobs 作为默认第一个 tab，承载所有 run/trial 相关流程。
- Dataset 承载 task 管理；Task 不作为一级页面。
- Agent 承载 built-in harness 与用户 custom agent 管理。
- Environment 承载 OrnnLab-local 的可复用环境模板管理；模板底层映射 Harbor `EnvironmentConfig` 和 task `[environment]` 字段。New Job 只从这些模板中选择，不在新建 Job 流程里展开底层环境细节。
- Leaderboard 按 dataset 展示排名，一次只展示一个 dataset。
- System 只展示 OrnnLab 服务、Harbor CLI、Docker、Storage、CPU/GPU/可用存储等系统健康与真实系统操作。

## 3. Users And Usage Context

目标用户是本机运行 Harbor benchmark 的开发者、研究者和 OrnnLab 使用者。

典型使用场景：

- 查看已有 Harbor Job 的状态、分数、成本、tokens、运行时长和产物。
- 新建 JobConfig 并运行 Harbor run。
- 浏览 Harbor 支持的 dataset 和 task，下载或删除本地 dataset。
- 配置 built-in 或 custom agent/harness。
- 查看某个 dataset 下的 leaderboard 排名，并从排名跳转到对应 Job。
- 检查本机 OrnnLab、Harbor、Docker、Storage 与资源状态。

## 4. Scope

### In Scope

- 顶部导航：Jobs、Datasets、Agents、Environment、Leaderboard、System。
- Header：品牌、Harbor 标识、Hub 连接状态、多语言切换、light/dark 切换。
- Jobs：Job 列表、新建 Job 二级页面、Job 详情抽屉、Trial 列表、事件日志、产物路径、取消/重试等运行操作。
- New Job：按功能领域和使用频率分子 tab 配置 Job，不把所有参数平铺；不承载环境配置细节。
- Datasets：展示 Harbor dataset 全集，支持下载、取消下载、删除本地数据、查看 task。
- Agents：展示 Agent Name、Harness、模型、状态；支持新建 custom agent、删除 custom agent。
- Environment：管理 OrnnLab-local Environment 模板，支持搜索、新建、复制、删除 custom 模板、查看并编辑配置详情；新建和复制使用二级页面，详情使用右侧抽屉，抽屉打开后直接展示可编辑表单。
- Leaderboard：按 dataset 搜索/切换，展示排名、Agent Name、Harness、模型、得分、成本、tokens、耗时和 Job。
- System：展示服务健康、检查更新、重启服务、清理 Docker 缓存、清理 `~/.cache/harbor`、CPU/GPU/可用存储。
- Storybook：所有正式组件和页面状态需要持续纳入 Storybook。
- API 契约：前端功能与后端接口以 `docs/architecture/frontend-api-contract.md` 为准。

### Out Of Scope

- 当前阶段不接入真实后端。
- 不做独立的 Tasks 一级页面。
- 不在 header 放运行 Job 入口，运行入口收敛到 Jobs。
- 不在 header 放 Docker 状态，Docker 状态收敛到 System。
- 不做通知中心等 Harbor 无直接支撑的 demo-only 功能。
- 不把 Environment 模板 CRUD 表述为 Harbor 原生命令；它属于 OrnnLab-local 管理层。
- 不在页面中展示 CLI 命令作为主要交互说明。
- 不展示无实际产品意义的面包屑、副标题或冗余系统检查块。

## 5. Core User Journey

### Jobs

用户进入默认 Jobs 页面后，可以搜索已有 Job，点击任意行打开右侧详情抽屉。列表需展示 Job Name、Job ID、状态、dataset、agent/harness、模型、trial 进度、得分、成本、tokens、运行时长、创建时间。

用户点击“新建 Job”进入二级页面 `/jobs/new`。页面按配置领域分组：基础、Tasks、验证器、运行策略、输出。基础区保留高频 Agent/Harness、Environment 选择和“计入排行榜”开关；`jobs_dir` 支持手动输入，也支持通过系统文件夹选择器选择。Environment 是 OrnnLab-local 环境模板，不在 New Job 中展开资源、镜像、mount、环境变量等细节。Tasks 展示 split、额外说明文件与 Task 白名单列表，默认全选；用户可搜索过滤、单项开关、全部开启或全部关闭。存在搜索过滤时，全部开启/全部关闭只作用于当前搜索结果；无搜索过滤时作用于完整 task 列表。验证器 tab 默认使用 Dataset 内置验证器；只有选择“自定义验证器”时才展开验证器入口、验证器环境变量、参数和最大验证时长；选择“跳过验证”时不展示底层 disable 开关，并强制本次 Job 不计入排行榜。运行策略 tab 使用“超时策略、失败重试、高级参数”组织配置，不直接平铺 Harbor CLI 参数；失败重试默认关闭，用户启用后才展开失败重试次数、重试场景和重试间隔；高级参数默认收起，展开后展示 Agent 初始化/环境构建超时倍率和“不重试的原始错误（命中规则）”列表。用户确认配置后运行 Job，回到 Jobs 列表并打开新 Job 详情。

### Datasets

用户进入 Datasets 页面后可以搜索 dataset，查看来源、路径、大小、task 数、状态和操作。用户可以导入本地 Dataset 路径，作为本地可运行 dataset 加入列表；该动作不上传、不复制文件。未下载 dataset 可下载并显示进度，可取消下载；已下载 dataset 可删除本地数据，删除前二次确认。

点击 dataset 行打开右侧详情抽屉，查看 dataset manifest、下载路径、registry 信息、splits 和 task 列表。Task 作为 dataset 子资源展示，可从 task 创建单 task Job。

### Agents

用户进入 Agents 页面后可以搜索 Agent，也可以新建 custom Agent。列表展示 Agent Name、Harness、类型、模型、状态和操作。built-in Agent 不可删除；custom Agent 可删除，删除前二次确认。

点击 agent 行打开右侧详情抽屉，查看 harness、运行参数、env、kwargs、skills、MCP、超时和允许 host 等配置。

### Environment

用户进入 Environment 页面后可以搜索 OrnnLab-local 环境模板。列表展示 Environment Name、profile 类型、`type`、`docker_image`、`network_mode`、CPU/Memory 策略、运行时覆盖项和操作。

点击 Environment 行打开右侧详情抽屉，直接编辑 Harbor 真实支持的环境字段：`type` / `import_path`、`network_mode` / `allowed_hosts`、`docker_image`、`os`、CPU/Memory/Storage/GPU/TPU、`healthcheck`、`workdir`、`mounts`、`env`、`kwargs`、`force_build`、`delete`、资源覆盖项和 `extra_docker_compose`。`skills_dir` 后续收敛到 Agents 中管理，不在 Environment UI 暴露。custom 模板允许删除，删除前二次确认。新建 Environment 和复制模板进入二级页面，保存后回到列表并打开对应详情抽屉。New Job 仅通过下拉选择这些模板。

Environment 字段交互按 Harbor 字段类型分配控件：`type`、`os`、`network_mode`、CPU/Memory enforcement policy 使用下拉列表；`force_build`、`delete` 使用 switch；CPU/Memory/Storage/GPU 数量使用数字输入；`tpu` 和 `override_tpu` 使用 TPU type 下拉 + Topology X/Y/Z 数字输入，并由 UI 组合成 Harbor 的 `TYPE=TOPOLOGY` 值；`allowed_hosts`、`extra_allowed_hosts`、`gpu_types` 使用逗号分隔的 tag 输入；`env`、`kwargs` 使用 Key-Value 多行输入；`mounts`、`healthcheck` 使用 JSON 多行输入；`extra_docker_compose` 和目录类字段使用路径输入。`allowed_hosts` 只表示 task `[environment]` 基线，运行时增量 host 使用 `extra_allowed_hosts`。

### Leaderboard

用户进入 Leaderboard 后先选择 dataset。页面一次只展示一个 dataset 的排名，dataset 下拉支持搜索。排名列表展示 rank、Agent Name、Harness、模型、得分、trials、成本、tokens、耗时和 Job ID。

点击 Job ID 打开与 Jobs 页面一致的 Job 详情抽屉。用户可以将某个 Job 从 leaderboard 移除，本质是把该 Job 设置为不进入 leaderboard，并更新排名。

### System

用户进入 System 后查看 OrnnLab Service、Harbor CLI、Docker、Storage、CPU 占用、GPU 占用、可用存储空间。

OrnnLab Service 支持检查更新和重启服务。Docker 行支持清理 Harbor 匹配规则下的 Docker 缓存。Storage 行支持清理 `~/.cache/harbor`。所有破坏性或可能影响体验的操作必须二次确认，弹窗不加副标题，标题和内容左对齐。

## 6. Interaction And Information Design

- 采用 Harbor 官方风格作为视觉基准：工具型、密集、低装饰、高信息利用率。
- 左右边距只保留安全边距，避免工具类页面浪费宽度。
- 详情页统一使用右侧抽屉，适用于 Job、Dataset、Agent，以及 Leaderboard 中点击 Job。
- 列表是一级页面主体；创建、详情、确认、下载进度等作为二级页面或局部状态出现。
- 搜索框、下拉列表、按钮高度需要统一；dark/light 下组件边框、下拉弹层和安全边距必须一致。
- 弹窗默认不添加副标题；除非明确要求，所有弹窗标题和内容左对齐。
- Toast 自动 3 秒消失，显示 `3s/2s/1s` 倒计时；关闭按钮不显示描边。
- 多语言和 light/dark 切换放在 header。

## 7. Product Rules And State Logic

- Job 是 run 的父概念；Trial 是 Job 下的单次 task 执行记录。
- Dataset 是 Task 的父概念；Task 不独立进入一级导航。
- Agent Name 是用户自定义名称；Harness 是执行适配层，例如 `claude-code`、`codex-cli`、`custom-harness`。
- Leaderboard 的 `metric` 和 `split` 必须来自 JobConfig 与 Dataset manifest，不允许前端随意编造。
- New Job 中“是否进入 leaderboard”默认打开，可随时修改。
- Dataset 未下载时路径和大小展示“未下载”；如果 Harbor 官方不能获取远端体积，不伪造大小。
- 清理 Docker 缓存只清理 Harbor 匹配规则内资源，不清理用户其他 Docker 资源。
- 清理本地缓存对应 `~/.cache/harbor`。
- 登录/认证相关功能先收敛为 header 中的 Hub 状态，不在 System 或其他页面重复展开。

## 8. Edge Cases, Errors, And Recovery

- Dataset 下载失败或取消后，应回到未下载状态，并清理已下载部分。
- 删除 dataset、删除 custom agent、重启服务、清理缓存必须二次确认。
- built-in Agent 删除操作不可用。
- Job 失败时详情中展示失败状态、failure code、事件日志和相关产物路径。
- 后端或 Harbor 不支持的能力不得展示为可点击按钮；可作为待确认需求记录在文档中。
- System 检查更新：如果已有新版本，弹窗确认是否升级；如果已是最新版本，toast 提示已是最新版本。

## 9. Content And Terminology

- 使用 `Job`、`Dataset`、`Task`、`Trial`、`Agent Name`、`Harness`、`Leaderboard`、`System` 作为核心术语。
- `Agent` 不再指 `claude-code`、`codex-cli` 这类执行器；这些称为 `Harness`。
- 得分支持两种格式：百分比，例如 `72.5%`；分数，例如 `87/100`。
- Tokens 单位展示为 `M`，不使用 `/M`。
- 操作文案优先使用无代码表达，不在 UI 上展示 CLI 命令作为主要说明。

## 10. Acceptance Criteria

- 一级导航只有 Jobs、Datasets、Agents、Environment、Leaderboard、System。
- Jobs 是默认第一个 tab。
- New Job 是 Jobs 的二级页面，不是一级 tab。
- New Job 不展示 Environment 配置子 tab，只从 OrnnLab-local Environment 模板下拉选择。
- Job、Dataset、Agent 详情都通过右侧抽屉展示。
- Leaderboard 一次只展示一个 dataset，支持 dataset 搜索和切换。
- System 不包含 Jobs 内部运行组件，例如 verifier retry。
- Demo 可见功能必须能在接口规范中找到对应后端接口或待确认项。
- 前端仍保持 mock 数据，但 mock 字段不得扩散出 demo-only 产品能力。
- 所有新组件和关键页面状态需纳入 Storybook。

## 11. Review Checklist And Sign-off Questions

- Harbor dataset list 是否能返回未下载 dataset 的远端大小？
- Harbor plugin 的真实机制、发现方式、启用方式和配置边界是什么？
- Harbor Hub 登录态是否只由 header 管理，还是需要单独账号页面？
- Docker 缓存清理的安全匹配规则是什么？
- Leaderboard 提交、上传、分享是否属于 v1.0.5 首期范围？
- System 中 CPU/GPU/Storage 的采样频率、单位和跨平台差异如何定义？

## 12. Clarification Decision Log

| Topic | Decision | Rationale | Source |
|---|---|---|---|
| 产品定位 | v1.0.5 先做 Harbor WebUI，OrnnLab 仍是基于 Harbor 的实验控制台 | 避免把产品定位写成脱离 Harbor 的新产品 | 前序讨论 |
| 一级导航 | Jobs、Datasets、Agents、Environment、Leaderboard、System | Environment 作为 OrnnLab-local 模板管理层，避免 New Job 暴露环境细节；模板 CRUD 映射 Harbor 配置字段但不是 Harbor 原生命令 | 2026-07-01 用户确认 |
| Tasks | Task 是 Dataset 子概念 | Dataset 页面展示 task 全集与详情 | 前序讨论 |
| Trials | Trial 是 Job 子概念 | Run/Trial 都收纳在 Job 下 | 前序讨论 |
| New Job | Jobs 下二级页面 | 新建流程不是独立一级产品域 | 前序讨论 |
| 详情页 | Job、Dataset、Agent 使用右侧抽屉 | 列表保持主上下文，详情不打断浏览 | 前序讨论 |
| Agent 命名 | Agent Name 是自定义名，Harness 是执行适配层 | 区分用户命名与 Harbor 执行器 | 前序讨论 |
| Header | 保留 Hub、语言、主题；移除运行 Job、Docker、通知 | 避免 header 堆叠非全局能力 | 前序讨论 |
| System | 只保留真实系统健康和系统操作 | 移除认证重复、命令列表、verifier 等不属于系统页的内容 | 前序讨论 |
| 弹窗 | 默认无副标题，标题和内容左对齐 | 减少说明噪音，保持工具页面清晰 | 前序讨论 |
