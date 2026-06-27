# PRD: v1.0.5 Harbor WebUI 产品化

- Status: Draft
- Created: 2026-06-28
- Updated: 2026-06-28
- Owner / requester: project maintainer
- Source request: v1.0.5 版本先做 Harbor 的 WebUI 产品，可完全接管 Harbor 服务。

## Requester Review Summary

- Key decisions:
  - OrnnLab 仍然是基于 Harbor 的实验控制台；v1.0.5 版本优先补齐 Harbor WebUI 能力。
  - Harbor 继续负责 benchmark 执行、agent 执行、环境生命周期、验证和原始 artifacts。
  - OrnnLab Web 负责把 Harbor 的核心 CLI/API 能力产品化为本地单用户 WebUI。
  - v1.0.5 UI 以官方 Harbor Hub (`https://hub.harborframework.com/`) 为视觉和信息结构基准。
  - 产品核心不是展示 Harbor 信息，而是用 UI 交互替代 Harbor 日常 CLI 操作。
- Important exceptions:
  - 不重写 Harbor core，不 fork Harbor，不把 v1.0.5 做成多租户云平台。
  - “完全接管”指用户日常使用 Harbor 的主要操作不再必须回到 CLI；不是覆盖 Harbor 内部所有维护命令。
- Must-confirm before implementation:
  - v1.0.5 首发是否必须覆盖 Harbor Hub upload/share/leaderboard submit。
  - 是否接受先覆盖 Docker backend，其他 Harbor environment backend 作为后续版本。
- Status reason:
  - 方向明确，但完整 Harbor WebUI 的 launch slice、Hub 范围和首发 backend 范围还需确认，因此当前为 Draft。

## 1. Background And Product Intent

当前 Web 已经接入 OrnnLab 的 agents、experiments、runs、reports、leaderboard
和 system doctor，并且后端默认通过 `harbor run --config` 走真实 Harbor
subprocess 执行。

但当前 Web 仍不是 Harbor 的完整 UI。Harbor 的 task/dataset 管理、JobConfig
完整配置、trial/trajectory 浏览、upload/share、leaderboard submit、plugins、
environment resource 参数等核心能力仍主要由 CLI 暴露。

v1.0.5 的产品意图是：在 OrnnLab 仍作为“基于 Harbor 的实验控制台”的前提下，
先把 Harbor 的日常工作流补成可用的 WebUI。用户应能通过 Web 完成 Harbor
主要操作，CLI 退回为高级用户、调试和自动化入口；后续版本继续围绕实验控制台
做更完整的产品化改造。

## 2. Goals And Success Criteria

1. 用户可在 Web 中完成一个 Harbor job 的配置、启动、观察、取消、复用和诊断。
2. 用户可在 Web 中管理 Harbor-facing agent 配置，并理解配置如何映射到 Harbor。
3. 用户可在 Web 中选择或导入 dataset/task，并控制 task filter 与任务数量。
4. 用户可在 Web 中查看 Harbor 原始 artifacts、result、job log、trial trajectory。
5. 用户遇到 Docker、Harbor、dataset、agent 或 verifier 错误时，Web 给出可执行的恢复动作。
6. 普通评测用户不需要记忆 `harbor run` 的大段参数即可完成日常评测。
7. 每个核心页面都必须对应一个被 UI 替代的 Harbor CLI 工作流，且能展示等价命令或 JobConfig 作为可审计证据。

## 3. Users And Usage Context

- 本地开发者：想快速评估一个 coding agent 在 benchmark 上的表现。
- Agent/模型调优者：需要比较不同 agent、模型、skills、MCP 配置和环境参数。
- 维护者：需要排查 Harbor job、Docker、dataset、artifact 和 leaderboard 结果。

使用环境默认是本地单用户机器，通过 npm `ornnlab` 启动 WebUI；Docker 是真实
benchmark 执行的默认环境前提。

## 4. Scope

### In Scope

- Harbor JobConfig 可视化配置：
  - agent、model、agent env、agent kwargs、skills、MCP config；
  - dataset/task、version、include/exclude、n_tasks；
  - n_attempts、n_concurrent、timeout、retry；
  - environment type、Docker 资源、mounts、delete/force-build；
  - verifier env、custom verifier、verification 开关；
  - artifacts 与 job output directory。
- Harbor job 生命周期：
  - create、run、cancel、retry/clone、resume/reuse 配置；
  - running 状态、events、job log、result、report；
  - failed/interrupted/cancelled 的诊断与恢复建议。
- Harbor artifacts 和 trial 体验：
  - result.json、job.log、harbor.config.json、harbor.capability.json；
  - trial 列表、trajectory 链接、任务级状态；
  - 对接 `harbor view` 或实现等价浏览入口。
- Dataset / task 工作流：
  - 展示可用 benchmark/dataset；
  - 支持本地 path 与 registry dataset；
  - 支持 task filter 与任务预览。
- Web 诊断：
  - Harbor 版本、Docker daemon、context、orphan containers、stale runs；
  - agent config 编译错误、dataset resolution 错误、verifier 错误。
- CLI-to-UI replacement:
  - `harbor run`：由 JobConfig 向导、Review 确认、Run 按钮、实时状态和 cancel/retry 交互替代；
  - `harbor dataset` / `harbor task`：由 Datasets/Tasks 列表、搜索、详情、task filter 和任务预览替代；
  - `harbor view`：由 Artifacts/Trials 浏览、trajectory 链接和原始路径入口替代；
  - `harbor upload` / share：若进入 v1.0.5 范围，由 Upload/Share 表单替代，并保留 public/private/share target 选择；
  - `harbor leaderboard submit`：若进入 v1.0.5 范围，由 Leaderboard 提交交互替代；
  - `harbor check` / doctor 类命令：由 System 诊断、阻断原因和恢复动作替代。

### Out Of Scope

- 重写 Harbor 执行引擎。
- 替代 Harbor Hub 云端服务。
- 多租户、团队权限、账号体系。
- 修改 Harbor 上游协议或私有 fork 作为默认路径。
- 覆盖每一个低频 Harbor CLI 管理命令，例如内部 cache 维护命令。

## 5. Core User Journey

1. 用户打开 OrnnLab WebUI，看到 Harbor/Docker/数据目录健康状态。
2. 用户进入 Agents，创建或选择一个 Harbor agent profile。
3. 用户进入 Runs/Experiments，选择 dataset/task、attempt、并发和环境参数。
4. Web 展示即将生成的 Harbor JobConfig 摘要，用户确认后启动。
5. Web 实时展示 run 状态、job log、events 和 trial 进度。
6. 完成后，Web 展示 summary、score、artifacts、trajectory 和 leaderboard 入口。
7. 如果失败，Web 展示失败分类、原始证据路径和可执行恢复动作。
8. 用户可 clone/retry/save template，复用同一组 Harbor 配置。

## 6. Interaction And Information Design

- Navigation:
  - Datasets
  - Tasks
  - Organizations
  - Jobs
  - System
  - Agents
  - Artifacts
  - Leaderboard
  - Templates
- Visual baseline:
  - 复刻官方 Harbor Hub 的顶部导航、白底/深色模式、monospace 字体、细边框表格、黑色主按钮、胶囊式模式切换和代码块样式。
  - 首页优先呈现 Harbor-style `Dataset / Tasks` 表格和 `Publish your first dataset` 指引，不再使用重卡片式运营仪表盘作为第一视觉。
  - 列表页沿用官方 `/datasets` 的标题、搜索框、表格、分页和空/加载骨架布局语言。
  - 任何暂未接入真实能力的官方 UI 元素不得做成误导性假入口；若保留入口，必须明确指向真实本地路径、外部官方路径或禁用状态。
- JobConfig UI 应分步组织，不把 Harbor 参数一次性平铺：
  - Agent
  - Dataset
  - Environment
  - Verification
  - Runtime
  - Review
- 每个高级参数默认折叠，但必须可见、可解释、可复制为原始 JobConfig。
- Web 页面必须始终保留原始 Harbor artifact 链接，避免产品层摘要掩盖事实。
- UI 中的每个主要操作都应回答“这替代了哪条 CLI 命令、会生成什么配置、失败后怎么恢复”。

## 7. Product Rules And State Logic

- Harbor subprocess 是默认执行边界。
- Web 生成的每个 job 必须持久化 `harbor.config.json`。
- Web 不允许静默 fallback 到模拟执行。
- Web 不允许只包装一段自由文本命令让用户手动复制执行；日常路径必须是表单、选择器、按钮、状态流和恢复动作。
- Web 可以展示等价 CLI 命令，但展示目的仅限审计、复制到自动化或高级调试。
- 如果 Docker daemon 不可用，Web 允许配置 job，但运行前必须阻断并给出启动/修复动作。
- 如果 run 已被用户取消，后续 Harbor 结果不得覆盖用户可见终态；但执行事实应作为诊断事件保留。
- 所有删除类操作默认软删除、归档或可恢复，不做不可恢复删除。

## 8. Edge Cases, Errors, And Recovery

- Docker CLI 已安装但 daemon 未启动：提示当前 context、socket、`colima start` 或 Docker Desktop 启动动作。
- Harbor CLI/API 不可用：提示依赖同步或 Harbor version mismatch。
- Dataset 不存在或 registry 下载失败：展示 dataset 名称、版本和 registry 源。
- Agent env 缺失：展示缺失变量名，不泄露 secret 值。
- Job cancel 超时：展示 SIGTERM/SIGKILL 清理证据和 orphan container 扫描入口。
- result.json 缺失：标记为 protocol/interrupted，不合成成功。
- Web 断连后恢复：根据 SQLite 状态和 Harbor artifacts reconcile。

## 9. Content And Terminology

- 用户可见主术语使用 Harbor 原生概念：
  - Job
  - Trial
  - Dataset
  - Task
  - Agent
  - Environment
  - Artifact
  - Verifier
- OrnnLab 术语作为产品组织层：
  - Experiment 表示用户意图容器；
  - Template 表示可复用配置；
  - Report 表示面向用户的摘要外壳。

## 10. Acceptance Criteria

- Given Docker daemon 可用，when 用户在 Web 创建并运行 Harbor job，then Web 生成的 `harbor.config.json` 可被 `harbor run --config` 直接执行。
- Given 用户不懂 Harbor CLI 参数，when 用户完成 JobConfig 向导，then Web 展示可理解摘要和原始 JobConfig。
- Given 用户需要完成 Harbor 日常 run workflow，when 用户只使用 Web 表单、按钮和状态视图，then 不需要手动输入 `harbor run`、dataset/task filter 或 cancel 命令。
- Given run 正在执行，when 用户点击 cancel，then Harbor subprocess 被终止并写入清理证据。
- Given run 完成，when 用户打开 report，then 能看到 score、result、job log、trial artifacts 和原始路径。
- Given dataset/version 不存在，when 用户启动 run，then Web 阻断执行并展示 dataset resolution 错误。
- Given Docker daemon 未启动，when 用户启动 run，then Web 不执行 Harbor job，并展示启动 Docker/Colima 的恢复动作。
- Given Harbor Hub upload 属于 v1.0.5 范围，when 用户上传 job，then 可选择 public/private 与 share target。

## 11. Review Checklist And Sign-off Questions

- “完全接管 Harbor”是否要求首版覆盖 Harbor Hub upload/share？
- v1.0.5 是否只承诺 Docker backend，其他 environment backend 后续补？
- 是否允许 Web 先复用 `harbor view` 做 artifact/trajectory 浏览，而不是首版自研完整 viewer？
- 是否把 `Experiment` 继续作为 OrnnLab 包装层，还是 UI 主导航直接改为 `Jobs`？

## 12. Clarification Decision Log

| Topic | Decision | Rationale | Source Round |
|---|---|---|---|
| Version direction | v1.0.5 先做 Harbor WebUI 能力 | 用户明确要求该版本先做 Harbor WebUI，同时 OrnnLab 仍是基于 Harbor 的实验控制台 | Initial request + correction |
| Product boundary | OrnnLab Web 接管 Harbor 日常工作流，不重写 Harbor core | 保留 Harbor 执行权威，Web 做产品层 | Initial inference |
| Official UI baseline | 复刻官方 Harbor Hub 的导航、表格、代码块和整体视觉密度 | 用户要求直接参考官方界面并尽量一致 | UI follow-up |
| Interaction priority | 用 UI 交互替代 Harbor 日常 CLI 操作 | 用户明确指出关键在于替代 CLI 操作 | CLI replacement correction |
| Status | Draft | Hub 范围、首发 backend 范围和 launch slice 仍需确认 | Initial inference |

## 13. Open Questions And Risks

- Harbor CLI/API 可能变更，v1.0.5 需要明确支持的 Harbor 版本范围。
- 如果首版覆盖所有 Harbor environment backend，交付面会显著扩大。
- Harbor Hub auth/share 涉及外部账号和权限，可能需要单独 PRD。
- Trial/trajectory viewer 如果自研，前端复杂度会明显高于复用 `harbor view`。

## 14. Implementation Notes

- 所有 v1.0.5 文档默认中文为主。
- 日志和诊断是验收要求，不是实现细节：每个 Harbor job 必须保留 config、capability、result、job log 和 cleanup evidence。
- 本地单用户是默认产品假设；账号、租户和团队协作不进入 v1.0.5 首版。
