# Subagent VS Review: v0.1.4 web-connectivity 接入方案

- Created: 2026-06-23T00:00:00+08:00
- Updated: 2026-06-23T00:00:00+08:00
- Report schema: adversarial-v1
- Task: 评审 `docs/releases/v0.1.4/web-connectivity/` 中的前后端调通方案（README + 01..05 五份子文档）
- Report path: `vs_review/2026-06-23-web-connectivity-plan-review.md`
- Review mode: Round 1 = main-agent self-adversarial pass (degraded);
  Round 2 = external CLI independent review via `claude-ds-pro` (per
  agent-vs-censorship skill, user-approved replacement path)
- Source session policy: Round 1 不构成独立评审；Round 2 由用户明确选择"调用外部 CLI 独立评审"后启用
- Status: blocked — 4 blocking findings accepted, awaiting documentation fixes + closure review

## Round 1: 接入方案的对抗性评审（降级）

### Review Input

#### Objective

把 v0.1.4 的前端（Vue 3 控制台）与已实现的 FastAPI 后端真正调通，使界面显示真实数据，且具备最小的 loading/error/empty 体验与回归测试。

#### Review Target

工程修复立项计划（设计 + 计划 + 验收），共 6 份文档：

- [docs/releases/v0.1.4/web-connectivity/README.md](file:///Volumes/XU-1TB-NPM/projects/HarnessLab/docs/releases/v0.1.4/web-connectivity/README.md)
- [01-vite-dev-proxy-missing.md](file:///Volumes/XU-1TB-NPM/projects/HarnessLab/docs/releases/v0.1.4/web-connectivity/01-vite-dev-proxy-missing.md)
- [02-views-not-consuming-api.md](file:///Volumes/XU-1TB-NPM/projects/HarnessLab/docs/releases/v0.1.4/web-connectivity/02-views-not-consuming-api.md)
- [03-contract-gap-vs-backend.md](file:///Volumes/XU-1TB-NPM/projects/HarnessLab/docs/releases/v0.1.4/web-connectivity/03-contract-gap-vs-backend.md)
- [04-loading-error-empty-states.md](file:///Volumes/XU-1TB-NPM/projects/HarnessLab/docs/releases/v0.1.4/web-connectivity/04-loading-error-empty-states.md)
- [05-integration-test-gap.md](file:///Volumes/XU-1TB-NPM/projects/HarnessLab/docs/releases/v0.1.4/web-connectivity/05-integration-test-gap.md)

#### Target Locations

- 前端：[frontend/vite.config.ts](file:///Volumes/XU-1TB-NPM/projects/HarnessLab/frontend/vite.config.ts)、[frontend/src/api/client.ts](file:///Volumes/XU-1TB-NPM/projects/HarnessLab/frontend/src/api/client.ts)、[frontend/src/types/console.ts](file:///Volumes/XU-1TB-NPM/projects/HarnessLab/frontend/src/types/console.ts)、`frontend/src/views/*.vue`、[frontend/src/App.vue](file:///Volumes/XU-1TB-NPM/projects/HarnessLab/frontend/src/App.vue)、[frontend/src/data/consoleSnapshot.ts](file:///Volumes/XU-1TB-NPM/projects/HarnessLab/frontend/src/data/consoleSnapshot.ts)
- 后端：[ornnlab/app.py](file:///Volumes/XU-1TB-NPM/projects/HarnessLab/ornnlab/app.py)、`ornnlab/api/*.py`、[ornnlab/settings.py](file:///Volumes/XU-1TB-NPM/projects/HarnessLab/ornnlab/settings.py)、[ornnlab/cli.py](file:///Volumes/XU-1TB-NPM/projects/HarnessLab/ornnlab/cli.py)
- 脚手架：[scripts/test-after-change-web.sh](file:///Volumes/XU-1TB-NPM/projects/HarnessLab/scripts/test-after-change-web.sh)、[frontend/playwright.config.ts](file:///Volumes/XU-1TB-NPM/projects/HarnessLab/frontend/playwright.config.ts)

#### Change Introduction

立项 5 个修复子项：dev/preview proxy、View 切换到 `ornnLabApi`、契约对齐 + mapper 层、统一 async-state、API/View 集成测试与 CI 编排。Phase 顺序：0 契约梳理 → 1 proxy → 2（03→02+04）→ 3 测试基建。SSE 实时接入与生产部署形态显式延后。

#### Risk Focus

- 默认端口与启动命令是否与后端真实配置一致
- mapper / async-state / `StatePanel` 是否过度工程（针对 4 个静态页面）
- "Views 切到真实数据" 与 "loading/error/empty" 同 PR 的耦合是否会让 PR 过大不可 review
- 契约对齐策略：UI 模型 vs 后端 schema 演化时由谁负责更新
- 测试基建是否会受 `consoleSnapshot.ts` 迁移路径影响

#### User-Perspective Review Focus

- 开发者本地启动顺序文档是否充分（FastAPI 必须先起？端口冲突？）
- 错误态文案是否会泄露技术栈/SQL/堆栈
- 空态文案能否引导用户到正确动作（"去创建 experiment"）

#### Implementation Completeness Focus

- BUG-WEB-01 的 proxy `target` 默认值是否与代码中真实端口一致
- BUG-WEB-03 中"补齐 agents/system/benchmarks 客户端"是否枚举了**所有**已有 router（`runs / templates / agents/validate / agents/{id}/compile`...）
- BUG-WEB-04 中 `ApiError`（已存在于 client.ts）与新引入的 `AsyncState.error` 是否会出现职责重叠
- BUG-WEB-05 中是否充分覆盖 SSE 的"显式不在范围"边界

#### Target Benefit Focus

文档宣称"调通"，但没有定义"调通"的可观测目标（请求成功率？首屏耗时？错误覆盖率？）。这是 benefit warning。

#### Assumptions To Attack

- 假设后端默认监听 `http://127.0.0.1:8000` —— **可疑**：[ornnlab/cli.py](file:///Volumes/XU-1TB-NPM/projects/HarnessLab/ornnlab/cli.py) 默认 `--port 8765`、[ornnlab/settings.py](file:///Volumes/XU-1TB-NPM/projects/HarnessLab/ornnlab/settings.py) 默认 `port: int = 8765`
- 假设 Vite dev server 端口 4173 与 Playwright preview 4174 互不干扰 —— [frontend/package.json](file:///Volumes/XU-1TB-NPM/projects/HarnessLab/frontend/package.json) 的 e2e 脚本用 4174，但 dev/preview 均为 4173，需澄清
- 假设 mapper 层一定值得引入（4 个页面是否构成"分层成本必要"）
- 假设 `consoleSnapshot.ts` 可以"作为 fixture 保留"——但当前类型是 UI 模型而非后端 schema，作 fixture 价值有限

#### Adversarial Lenses

architecture, implementation-completeness, maintenance, testing, user-perspective (developer onboarding)

#### Verification Status

- 文档检查：本人直接读了所有 6 份 markdown、`vite.config.ts`、`api/client.ts`、`settings.py`、`cli.py`、所有 router 文件。
- 静态分析：grep 确认 `@/api` 零引用。
- 未执行：未实际跑 `npm run dev` 或 `npm run e2e`；未启动 FastAPI 验证 proxy 行为。

#### Reviewer Instructions

不适用（见下方 launch record，无 fresh subagent）。

### Reviewer Timeout Policy

| Complexity | Initial Wait | Extension | Max Attempts Per Role | Blocking Closure Behavior |
|---|---:|---:|---:|---|
| normal | n/a (degraded) | n/a | n/a | cannot pass if review is unavailable; user decision required |

### Reviewer Selection

| Reviewer | Reason Selected | Risk Area |
|---|---|---|
| architecture-adversary（**未实际启动**） | 计划文档同时涉及分层（mapper / AsyncState）、契约演化与可执行性，架构视角最能挑战是否过度工程或漏项 | 架构边界 + 实现完备性 + 可执行性 |

### Reviewer Launch Records

| Reviewer | Internal Mechanism | Session / Job ID | Trace Source | Context Forked | Input Packet | Context Explicitly Excluded | Read-only |
|---|---|---|---|---|---|---|---|
| architecture-adversary | **unavailable** — current Trae runtime exposes no fresh internal subagent / Task tool; only `Skill` invocation is available, which does not isolate context | n/a | n/a | n/a | Round 1 Review Input (intended) | n/a | n/a |

按 skill 协议 #11：runtime 不支持 fresh internal subagent 时不得伪造独立评审。本轮以 **main-agent self-adversarial pass** 给出诚实的降级评审视图，结论仅供参考，不能等同于独立评审。

### Reviewer Timeout Records

| Reviewer Output Key | Reviewer Role | Attempt | Session / Job ID | Waited | Status | Reason | Action |
|---|---|---:|---|---:|---|---|---|
| self-pass-1 | architecture-adversary（degraded） | 1 | n/a | 0 | blocked_due_to_review_unavailable | runtime 无 fresh subagent 机制 | user decision required |

### User Decision After Failed Review

- 必须由用户决定：retry（等待 runtime 支持 / 让我用其他可用 reviewer 工具） / narrow scope / change reviewer type（如调用外部 CLI 评审） / accept risk（接受仅 self-pass 结论） / blocked
- User-visible reason: Trae 内置 agent 没有 fresh internal subagent；可用的 `Skill` 只能加载 prompt，不隔离 main-agent 上下文，按协议不算独立评审。

### Reviewer Outputs

#### self-pass-1 （main-agent self-adversarial pass，**不构成独立评审**）

##### Summary

按 architecture-adversary 视角对方案做自检，发现 **2 个 blocking 事实性错误** 与 **5 个 non-blocking 风险**。最重的问题是 BUG-WEB-01 默认 API target 与实际后端端口不一致，会让"按文档照做"的 dev 联调直接失败；其次是 BUG-WEB-03 没有枚举 runs / templates / agents 子端点，可能导致"客户端补齐"工作再次留半。

##### Blocking Findings

- F1: BUG-WEB-01 把 proxy target 默认写成 `http://127.0.0.1:8000`，但后端代码默认端口是 `8765`
  - Broken assumption: 后端默认端口 = 8000
  - Failure scenario: 工程师按文档配置后，`/api` proxy 指向 8000，FastAPI 实际监听 8765；所有请求返回 ECONNREFUSED，BUG-WEB-02/04/05 的所有验收都无法通过
  - Trigger condition: 任何按文档默认值进行的首次联调
  - Impact: Phase 1 立即失败，连带 Phase 2/3 阻塞；让此修复计划本身丧失"调通"目标
  - Proof needed: 修改文档默认值为 `http://127.0.0.1:8765`，并显式引用 [ornnlab/cli.py#L24](file:///Volumes/XU-1TB-NPM/projects/HarnessLab/ornnlab/cli.py#L24) 与 [ornnlab/settings.py#L18-L19](file:///Volumes/XU-1TB-NPM/projects/HarnessLab/ornnlab/settings.py#L18-L19) 作为来源；或把端口变量统一从环境变量 `ORNNLAB_PORT` 读取

- F2: BUG-WEB-03 "补齐缺失访问器" 不完备，遗漏了 runs / templates / agents 的细分操作
  - Broken assumption: 现有 `ornnLabApi` 只缺 agents/system/benchmarks
  - Failure scenario: 文档列出的需要补齐项有 agents/system/benchmarks，但 [ornnlab/api/agents.py](file:///Volumes/XU-1TB-NPM/projects/HarnessLab/ornnlab/api/agents.py) 还有 `POST /api/agents`（create）、`PUT /{id}`（update）、`DELETE /{id}`（soft delete）、`POST /validate`、`POST /{id}/compile`；[ornnlab/api/runs.py](file:///Volumes/XU-1TB-NPM/projects/HarnessLab/ornnlab/api/runs.py) 还有 `GET /{id}/events`；当前 `ornnLabApi` 的 `templates()` 也只覆盖 list/create，但 [ornnlab/api/templates.py] 可能还有 update/delete（未一一核对）。BUG-WEB-02 切 View 时若 Agents View 需要展示 + 操作 agents，会再次发现客户端缺方法
  - Trigger condition: View 进入到 Agents 增删改流程
  - Impact: 计划再度出现"补齐又遗漏"的循环，违背立项目的
  - Proof needed: 在 BUG-WEB-03 增加一张完整端点清单表，逐一列出 method + path + 客户端方法名 + 已覆盖/待补齐状态；以 router 文件为契约源

##### Non-blocking Risks

- R1: BUG-WEB-02 与 BUG-WEB-04 强绑同 PR，可能让 PR 过大、不可 review
  - Broken assumption: "同 PR 提交" 一定优于分两步落地
  - Failure scenario: 一个 PR 改 4 个 view + AsyncState 原语 + StatePanel + fixture 迁移，diff 行数大，回归面广
  - Trigger condition: 实际执行时
  - Impact: review 时间长、回滚成本高
  - Proof needed: 改为先合并 04（仅引入 AsyncState + StatePanel，不动 view），再用一个 view 单独切换作为示范 PR，再批量推进余下 view

- R2: AsyncState 与现有 `ApiError` 职责重叠
  - Broken assumption: 需要新建一个 `AsyncState` discriminated union
  - Failure scenario: `client.ts` 已经定义了 `ApiError`，加上 `AsyncState.error` 后会有两层错误抽象；mapper 层抛错 vs fetch 抛错的归一化路径不清晰
  - Trigger condition: mapper 抛业务错误时
  - Impact: 维护成本上升、错误处理路径分叉
  - Proof needed: 明确"`ApiError` 只表网络/HTTP 层；mapper 抛 `MapperError` 或直接复用 `ApiError`；UI 层只看 `AsyncState`" 的边界

- R3: mapper 层对 4 个静态页面是否过度工程
  - Broken assumption: 必须引入 mapper 才能解耦
  - Failure scenario: 当前 view 实际只渲染少量字段，mapper 层多写一份 viewmodel 类型，长期可能再被 inline
  - Trigger condition: view 字段需求与后端 schema 接近时
  - Impact: 增加冗余文件，违背"最小化"
  - Proof needed: 给出每个 view 真实使用的字段清单；如果 ≤ 5 个字段且语义对齐，允许直接消费 `ornnLabApi` 类型而不引入 mapper；mapper 仅对 KPI 派生这种"多对一聚合"使用

- R4: BUG-WEB-05 把 SSE 测试推迟，但 BUG-WEB-04 / BUG-WEB-02 没有显式声明"本轮不测 SSE"
  - Broken assumption: SSE 不在本立项范围已经写在 README，子文档读者会自然继承
  - Failure scenario: 实施 02/04 时被诱导临时加 EventSource 接入
  - Trigger condition: 在 02/04 PR 中触手可及看到 SSE 端点
  - Impact: 范围蔓延、与 bugfix/04 重复劳动
  - Proof needed: 在 02 和 04 的"不在范围"小节里显式提一句"SSE 实时事件等待 bugfix/04"

- R5: "调通"缺乏可观测指标（benefit warning）
  - Broken assumption: 验收里有"显示真实数据"已经足够
  - Failure scenario: 实施后没有量化目标，难以判断"再调一调还是收尾"
  - Trigger condition: 评审 PR 时
  - Impact: 推进节奏不清晰
  - Proof needed: 在 README "验收" 补一条"`scripts/test-after-change-web.sh` 退出码 0；e2e smoke 中至少 1 个真实 API 请求返回 2xx"

##### User-Perspective Checks

- Usability：风险 → 见 R1（PR 过大不可 review）
- Ease of use：风险 → 见 F1（默认端口错，按文档照做无法成功）
- Ease of understanding：通过（README 结构清晰、phase 图直观）

##### Implementation Completeness Checks

| Plan Item | Expected Behavior | Production Code Path | Integration Entry | Test Evidence | Runtime / Log Evidence | Mock / Stub Exposure | Status | Finding Link |
|---|---|---|---|---|---|---|---|---|
| Vite proxy 默认 target | dev/preview `/api` 命中后端 | `frontend/vite.config.ts` (missing) | n/a | missing | missing | none | not-started | F1 |
| 客户端补齐 agents/system/benchmarks | `ornnLabApi.*` 暴露完整端点 | `frontend/src/api/client.ts`（待改） | n/a | missing | missing | none | partial (枚举不全) | F2 |
| View 切到真实 API | 4 个 view 用 `ornnLabApi` | `frontend/src/views/*.vue`（待改） | n/a | missing | missing | snapshot blocks completion | not-started | n/a (计划文档完整) |
| mapper 层 | viewmodel 派生 | `frontend/src/api/mappers/`（待建） | n/a | missing | missing | none | not-started | R3 (是否需要) |
| AsyncState + StatePanel | 三态 UI | `frontend/src/utils/asyncState.ts` + `frontend/src/components/StatePanel.vue`（待建） | n/a | missing | missing | none | not-started | R2 |
| 集成测试 + 脚本编排 | typecheck/lint/vitest/e2e 串起 | `scripts/test-after-change-web.sh`（待改） | n/a | missing | missing | none | not-started | n/a |

##### Target Benefit Checks

| Claimed Benefit | Baseline | Target | Measurement Method | Comparison Evidence | Result | Regression / Side Effect | Status | Finding Link |
|---|---|---|---|---|---|---|---|---|
| 前端可见真实后端数据 | 当前 0%（全 snapshot） | 4/4 view 渲染真实数据 | 浏览器 Network + e2e 断言 | missing | unmeasured | UI 在后端不可用时可能空白（被 BUG-WEB-04 兜底） | unmeasured | R5 |
| 错误态不再裸露异常 | 当前无错误处理 | 任一接口 5xx → UI 显示用户级文案 | 手动 smoke + 单测 | missing | unmeasured | 错误文案泄露 SQL/栈的反向风险 | unmeasured | R5 |
| dev 联调一行命令可达 | 当前缺 proxy | `npm run dev` 即可联调 | 手动 | missing | unmeasured | 端口冲突 | unmeasured | F1 |

##### Required Fixes

- 修正 BUG-WEB-01 默认 target 端口为 8765 并引用 settings/cli 源码（对应 F1）
- 在 BUG-WEB-03 增加端点清单表，覆盖 runs/events、templates 全集、agents 增删改与 compile/validate（对应 F2）
- 在 BUG-WEB-04 显式定义 `ApiError` 与 `AsyncState.error` 的归一化边界（对应 R2）

##### Missing Tests

- 没有为 proxy 配置准备 smoke：建议在 e2e 中加一条"GET `/api/system/status` 返回 200"断言（对应 R5）

##### Missing Logs / Observability

- 验收里没有要求前端发出统一的 `console.error` 或 Sentry-like 钩子；本立项可不引入第三方，但应记录"未配置前端 telemetry 是已知缺口"

##### Evidence

- [ornnlab/cli.py#L24](file:///Volumes/XU-1TB-NPM/projects/HarnessLab/ornnlab/cli.py#L24) — `web.add_argument("--port", type=int, default=8765)`
- [ornnlab/settings.py#L18-L19](file:///Volumes/XU-1TB-NPM/projects/HarnessLab/ornnlab/settings.py#L18-L19) — `port: int = 8765`
- [ornnlab/api/agents.py#L11-L58](file:///Volumes/XU-1TB-NPM/projects/HarnessLab/ornnlab/api/agents.py#L11-L58) — 列出 list/create/get/compile/validate/update/delete 全集
- [ornnlab/api/runs.py#L29-L31](file:///Volumes/XU-1TB-NPM/projects/HarnessLab/ornnlab/api/runs.py#L29-L31) — `GET /runs/{id}/events` 未在 `ornnLabApi` 中
- [frontend/src/api/client.ts#L134-L172](file:///Volumes/XU-1TB-NPM/projects/HarnessLab/frontend/src/api/client.ts#L134-L172) — 当前 `ornnLabApi` 仅覆盖部分端点
- [frontend/vite.config.ts#L1-L22](file:///Volumes/XU-1TB-NPM/projects/HarnessLab/frontend/vite.config.ts#L1-L22) — 无 proxy
- [docs/releases/v0.1.4/web-connectivity/01-vite-dev-proxy-missing.md](file:///Volumes/XU-1TB-NPM/projects/HarnessLab/docs/releases/v0.1.4/web-connectivity/01-vite-dev-proxy-missing.md) — 文档示例 `'http://127.0.0.1:8000'`

### Main Agent Response

| Reviewer | Finding | Broken Assumption / Failure Scenario | Severity | Decision | Evidence / Reason | Action Taken | Follow-up |
|---|---|---|---|---|---|---|---|
| self-pass-1 | F1: proxy 默认端口错 | 后端默认 8765，文档写 8000，导致首次联调失败 | blocking | accept | [cli.py#L24](file:///Volumes/XU-1TB-NPM/projects/HarnessLab/ornnlab/cli.py#L24) | 待用户决策评审路径后修订 BUG-WEB-01 默认值 + 引用源码 | 触发对修订后文档的 closure review |
| self-pass-1 | F2: 客户端补齐枚举不全 | 漏 runs/events、agents 增删改/compile/validate、templates update/delete | blocking | accept | [agents.py](file:///Volumes/XU-1TB-NPM/projects/HarnessLab/ornnlab/api/agents.py)、[runs.py](file:///Volumes/XU-1TB-NPM/projects/HarnessLab/ornnlab/api/runs.py) | 修订 BUG-WEB-03 增加完整端点清单表 | closure review |
| self-pass-1 | R1: 02+04 同 PR 过大 | 一次提交改动面太广 | major | accept | 经验判断 | 修订 02/04，允许"先 04 原语 + 1 个 view 示范"分两步 | 在 README/02/04 标注 |
| self-pass-1 | R2: ApiError 与 AsyncState.error 边界 | 抽象重叠 | major | accept | [client.ts#L1-L10](file:///Volumes/XU-1TB-NPM/projects/HarnessLab/frontend/src/api/client.ts#L1-L10) | 在 BUG-WEB-04 增"错误抽象边界"小节 | n/a |
| self-pass-1 | R3: mapper 层过度 | 4 个 view 不一定需要 mapper | major | accept (limited) | 当前 view 字段较少 | 在 BUG-WEB-03 改为"原则上不引入 mapper；仅在多对一聚合（KPI）时新增" | n/a |
| self-pass-1 | R4: SSE 边界 | 范围蔓延风险 | minor | accept | README 已说，但 02/04 没复述 | 在 02/04 各加一句"SSE 见 bugfix/04" | n/a |
| self-pass-1 | R5: benefit 量化缺失 | 调通缺可观测目标 | target-benefit warning | accept | README "验收" 缺指标 | 在 README "验收" 增加"e2e smoke 含 ≥1 个真实 API 2xx 断言；脚本退出码 0" | n/a |

### Closure Status

- Blocking findings found: yes（F1、F2）
- Accepted blocking findings fixed: no（等待用户对评审降级的决策后再统一改文档）
- Blocking re-review completed: no
- Blocking re-review passed: n/a
- Blocking re-review round links: n/a
- Blocking re-review launch records: n/a
- Rejected findings backed by evidence: n/a
- Deferred findings documented: n/a
- Implementation completeness gaps resolved or accepted by user: no
- Target benefit warnings recorded: yes (R5)
- Blocked reason: 1) fresh internal subagent 不可用；2) F1/F2 是 self-pass 发现的事实性缺陷，仍需用户确认是否信任降级评审结论
- Allowed to proceed: no（按 skill 协议）

## Final Conclusion

Round 1 = degraded self-pass；Round 2 = `claude-ds-pro` 独立评审已完成，见下方 Round 2 详细记录。两轮合并共 **4 个 blocking** + **10 个 non-blocking**，全部由 main agent 给出 `accept` 决定，**未实施前评审尚未 close**。需要在更新 6 份文档后做一次 closure review。

---

## Round 2: External CLI independent review (claude-ds-pro)

### Review Input

与 Round 1 完全相同的 review packet，但通过 stdin 传给独立的 `claude-ds-pro` CLI 进程，不共享本会话上下文。

完整 prompt 保存在 [.codex_tmp/claude-reviews/prompt.txt](file:///Volumes/XU-1TB-NPM/projects/HarnessLab/.codex_tmp/claude-reviews/prompt.txt)，输出保存在 [.codex_tmp/claude-reviews/2026-06-23-web-connectivity-external-review.md](file:///Volumes/XU-1TB-NPM/projects/HarnessLab/.codex_tmp/claude-reviews/2026-06-23-web-connectivity-external-review.md)。

### Reviewer Timeout Policy

| Complexity | Initial Wait | Extension | Max Attempts Per Role | Blocking Closure Behavior |
|---|---:|---:|---:|---|
| normal | 8 min | none used | 1 | cannot pass while blocking findings unresolved |

### Reviewer Selection

| Reviewer | Reason Selected | Risk Area |
|---|---|---|
| external-architecture-adversary (claude-ds-pro) | 用户明确选择「调用外部 CLI 独立评审」；独立进程、独立 LLM、独立上下文，能挑战 self-pass 可能继承的盲点 | 架构边界 + 实现完备性 + 契约对齐 |

### Reviewer Launch Records

| Reviewer | Internal Mechanism | Session / Job ID | Trace Source | Context Forked | Input Packet | Context Explicitly Excluded | Read-only |
|---|---|---|---|---|---|---|---|
| external-architecture-adversary | `claude-ds-pro -p --permission-mode dontAsk --max-budget-usd 5.00`，通过 stdin 传 prompt | command_id `8512ac84-c09b-4e8b-8f3a-a93a9f537b02`；claude session id `0f542f0a-b91b-47e3-8b34-3496fac0ef61` | `.codex_tmp/claude-reviews/run.log`（stdout 末尾 `REVIEW_WRITTEN:` 行） | fork_context=false（独立 CLI 进程） | prompt.txt 完整内容 | main-agent 会话历史、推理、self-pass 结论（在 prompt 中明确写明"不要被它绑架，鼓励独立得出可能更严厉的结论"） | yes（指令"只读，禁止修改任何源码或文档"） |

### Reviewer Timeout Records

| Reviewer Output Key | Reviewer Role | Attempt | Session / Job ID | Waited | Status | Reason | Action |
|---|---|---:|---|---:|---|---|---|
| external-1 | external-architecture-adversary | 1 | 0f542f0a-... | ~3 min | completed | review written to disk | completed |

### Reviewer Outputs

#### external-1

完整输出见 [.codex_tmp/claude-reviews/2026-06-23-web-connectivity-external-review.md](file:///Volumes/XU-1TB-NPM/projects/HarnessLab/.codex_tmp/claude-reviews/2026-06-23-web-connectivity-external-review.md)。以下是按 skill 模板的关键提取。

##### Summary

发现 **4 个 blocking** + **10 个 non-blocking**。Verdict: `blocked`。同意 self-pass 的 F1/F2/R1/R2/R4/R5，新增 F3、F4 两个 blocking 与 R6-R10 五个 non-blocking。对 self-pass 的 R3（mapper 过度）持部分不同意意见。

##### Blocking Findings

- **F1** (与 self-pass F1 一致)：BUG-WEB-01 proxy default target `8000` vs 后端真实 `8765`
  - Evidence: `ornnlab/cli.py:24`、`ornnlab/settings.py:18`、`01-vite-dev-proxy-missing.md:54`
- **F2** (扩展 self-pass F2)：BUG-WEB-03 端点枚举遗漏 **11 个**：`system.dockerOrphans`、`agents.create/validate/update/delete`、`experiments.create/delete/listEvents/listRuns`、`runs.listEvents`、`templates.delete`；后端 31 端点对照 ornnLabApi 13 已覆盖 + 计划补 6 个，仍漏 11
  - Evidence: 7 个 router 文件逐行列出（见外部报告内表格）
- **F3 (NEW)**：UI viewmodel 字段在后端**无数据源**
  - `ExperimentRecord.owner` / `target` 后端无；`AgentRecord.health` 三态枚举、`queue`、`lastHeartbeat` 后端无；`LeaderboardSeed.successRate / experiments` 后端无；`ExperimentState` 仅 3 态但后端 7+ 态
  - Trigger: mapper 实现时逐字段对齐 → 字段无输入 → UI 空白或 undefined
  - Evidence: `frontend/src/types/console.ts:3,20-45`、`ornnlab/models/experiment.py:25-33`、`frontend/src/api/client.ts:51-61`
- **F4 (NEW)**：`apiClient.post` 无法传递 query param
  - `system.doctor(logs=true)` 实际是 `POST /api/system/doctor?logs=true`（FastAPI 把原始类型默认参数解析为 query）；当前 `post()` 把参数当 body
  - Evidence: `frontend/src/api/client.ts:122-129`、`ornnlab/api/system.py:15-17`、`ornnlab/api/experiments.py:100-101`

##### Non-blocking Risks

- R1 (与 self-pass 一致)：02+04 同 PR 过大；明确建议 04 基础设施独立 PR 先合
- R2 (扩展 self-pass)：`ApiError` vs `AsyncState.error` 边界；建议统一为 `ApiError`，移除 `| Error` 回退
- R3 (与 self-pass **不同意**)：mapper 层判据更细——枚举映射/字段重命名仍应保留 mapper；仅 1:1 复制可消除
- R4 (与 self-pass 一致)：SSE 边界需在 BUG-WEB-03 显式声明 deferred
- R5 (与 self-pass 一致)：benefit 量化；补充覆盖率/退出码/typecheck/真实 API 2xx 断言
- **R6 (NEW)**：`vite.config.ts` preview port 4173 与 e2e CLI 覆写 4174 的关系未在文档说明
- **R7 (NEW)**：`ExperimentRun` 缺 `job_dir` 字段（后端 `RunView` 有）
- **R8 (NEW)**：BUG-WEB-01 把"后端启动文档"defer 出范围，但 AC 又依赖后端已启动，造成首次开发体验断裂
- **R9 (NEW maintenance warning)**：无 OpenAPI 自动类型生成方案，长期 schema 演化漂移风险
- **R10 (NEW)**：BUG-WEB-05 测试断言深度可能退化为占位测试；缺"特定输入→特定 DOM 内容"的深度断言要求

##### Disagreements With Self-Pass

外部 reviewer 对 self-pass 在 R3（mapper 过度工程）的结论持**部分不同意**：完全去除 mapper 会让 View 中散落 inline 转换，建议保留枚举映射 / 字段重命名场景下的 mapper，仅消除 1:1 复制。

外部 reviewer 还指出 self-pass 把 degraded 评审视为"不可据此通过"的论断**说服力不足**：F1–F4 都有独立于评审机制的代码证据，修正后可直接 `pass-with-fixes`。

##### Evidence

完整证据见外部报告。所有关键 finding 已交叉验证 `ornnlab/cli.py`、`ornnlab/settings.py`、`ornnlab/api/*.py`、`frontend/src/api/client.ts`、`frontend/src/types/console.ts`。

### Main Agent Response (Round 2)

| Reviewer | Finding | Broken Assumption / Failure Scenario | Severity | Decision | Evidence / Reason | Action Taken | Follow-up |
|---|---|---|---|---|---|---|---|
| external-1 | F1 | 后端默认 8765 而文档写 8000 | blocking | accept (与 R1 self-pass F1 合并) | `cli.py:24` | 待修订 BUG-WEB-01 | closure review |
| external-1 | F2 | 11 个端点遗漏 | blocking | accept (扩展 R1 self-pass F2) | 31 端点对照表 | 修订 BUG-WEB-03 增加完整端点清单 | closure review |
| external-1 | F3 | UI viewmodel 字段无后端源 | blocking | accept | `types/console.ts:20-45` 等 | 在 BUG-WEB-03 增"viewmodel 字段数据源决策"节，逐字段给出 删除/派生/后端补 决定 | closure review |
| external-1 | F4 | apiClient.post 无 query param | blocking | accept | `client.ts:122-129` | 在 BUG-WEB-03 增 apiClient 能力扩展条款 | closure review |
| external-1 | R1 | 02+04 同 PR 过大 | major | accept | self-pass 同意 | 在 README/02/04 改"04 基础设施独立 PR 先合" | n/a |
| external-1 | R2 | ApiError 边界 | major | accept (升级 self-pass R2) | self-pass 同意 | 在 BUG-WEB-04 增"错误抽象边界"节，统一为 ApiError | n/a |
| external-1 | R3 | mapper 判据更细 | major | accept (修正 self-pass R3) | external 论据合理 | BUG-WEB-03 mapper 判据改为"仅 1:1 复制可消除；枚举/聚合/重命名保留" | n/a |
| external-1 | R4 | SSE 子文档需复述 | minor | accept | self-pass 同意 | BUG-WEB-03 端点表标注 Deferred | n/a |
| external-1 | R5 | benefit 量化 | target-benefit warning | accept | self-pass 同意 | README 验收加可观测项 | n/a |
| external-1 | R6 | preview port 误导 | minor | accept | `vite.config.ts:18-20` | BUG-WEB-01 加端口布局说明 | n/a |
| external-1 | R7 | ExperimentRun 缺 job_dir | minor | accept | `models/experiment.py:48` | BUG-WEB-03 在端点表内标注；本次不直接改 client.ts（plan-only） | n/a |
| external-1 | R8 | 后端启动文档 defer | minor | accept | 01 AC 与依赖错配 | BUG-WEB-01 增"启动后端一行命令"AC 项 | n/a |
| external-1 | R9 | 无 OpenAPI 自动类型 | maintenance warning | defer | v0.1.5 PRD 范畴 | 在 BUG-WEB-03 增"maintenance follow-up"节，记录 openapi-typescript 评估 | tracked to v0.1.5 |
| external-1 | R10 | 测试断言深度 | major | accept | `05:37-38` 只规定数量 | BUG-WEB-05 增"≥1 个 view 测试需做特定输入→特定 DOM 文本"要求 | n/a |

### Closure Status (Round 2)

- Blocking findings found: yes（F1、F2、F3、F4）
- Accepted blocking findings fixed: no（需修订 6 份计划文档）
- Blocking re-review completed: no
- Blocking re-review passed: n/a
- Implementation completeness gaps resolved or accepted by user: no
- Target benefit warnings recorded: yes (R5)
- Rejected findings backed by evidence: n/a (no reject)
- Deferred findings documented: yes (R9 → v0.1.5)
- Allowed to proceed: no（按协议必须先修订文档并跑一次 closure review）

## Final Conclusion (overall)

外部独立评审在 self-pass 基础上**新增 2 个 blocking + 5 个 non-blocking**，并对 self-pass 的 R3 mapper 判据做了重要修正。共需对 6 份计划文档做以下修订才能进入实施：

1. **[P0]** 修正 `01` 的 default port → 8765，加端口布局与启动命令说明
2. **[P0]** 在 `03` 增加 31 端点完整清单表 + viewmodel 字段数据源决策 + apiClient query-param 能力扩展条款
3. **[P0]** 修正 `03` mapper 判据
4. **[P1]** 在 `02/04/README` 拆 PR 切片；`04` 明确 `ApiError` 边界
5. **[P2]** `05` 增加测试断言深度要求；`README` 增加量化验收
6. **[P2]** R9 (OpenAPI auto types) defer 到 v0.1.5 并在 `03` 记录

修订完成后跑一次 closure review（再调一次 `claude-ds-pro`），如均 pass 则 verdict 升为 `pass-with-fixes`，进入 Phase 1 实施。

**当前状态：blocked。需要你确认是否进行上述文档修订。**

## Round 3: Closure Review（针对修订后文档）

### Review Input (Round 3)

#### Objective

验证 v0.1.4 web-connectivity 计划的 6 份修订后文档（commit `dfd2567`）是否真正闭合 Round 1 self-pass 与 Round 2 external-1 提出的全部 blocking + non-blocking findings；找出修订过程中可能引入的新缺陷、新一致性问题、或未覆盖的边界。

#### Review Target

修订后的 6 份计划文档（Version v1.1，Updated 2026-06-24）：

- [docs/releases/v0.1.4/web-connectivity/README.md](file:///Volumes/XU-1TB-NPM/projects/HarnessLab/docs/releases/v0.1.4/web-connectivity/README.md)
- [01-vite-dev-proxy-missing.md](file:///Volumes/XU-1TB-NPM/projects/HarnessLab/docs/releases/v0.1.4/web-connectivity/01-vite-dev-proxy-missing.md)
- [02-views-not-consuming-api.md](file:///Volumes/XU-1TB-NPM/projects/HarnessLab/docs/releases/v0.1.4/web-connectivity/02-views-not-consuming-api.md)
- [03-contract-gap-vs-backend.md](file:///Volumes/XU-1TB-NPM/projects/HarnessLab/docs/releases/v0.1.4/web-connectivity/03-contract-gap-vs-backend.md)
- [04-loading-error-empty-states.md](file:///Volumes/XU-1TB-NPM/projects/HarnessLab/docs/releases/v0.1.4/web-connectivity/04-loading-error-empty-states.md)
- [05-integration-test-gap.md](file:///Volumes/XU-1TB-NPM/projects/HarnessLab/docs/releases/v0.1.4/web-connectivity/05-integration-test-gap.md)

#### Change Introduction

针对 Round 1/2 共 4 个 blocking + 7 个 non-blocking findings 做了 v1.0 → v1.1 修订（commit `dfd2567`，6 files changed, 259 insertions, 48 deletions）。每份文档头部新增 `Revision Notes` 字段记录本轮修改对应的 finding ID。

#### Risk Focus

- 修订内容是否真正解决了原 findings，而非"打补丁"
- 修订之间是否引入新冲突（如 README PR 切片描述 vs 02/04 子文档表述不一致）
- 31 端点清单表是否真的覆盖了后端所有 router 端点（核对 `ornnlab/api/*.py`）
- viewmodel 字段决策的"派生 / 删除"是否会与现有 UI 实现冲突（View 组件需要这些字段）
- 量化验收指标是否真的可观测（避免再次出现"unmeasured"）

#### Reviewer Instructions

不要修改任何文件。直接读取目标文档与相关源码（`ornnlab/api/*.py`、`frontend/src/api/client.ts`、`frontend/src/types/console.ts`、`ornnlab/cli.py`、`ornnlab/settings.py`），逐 finding 验证闭合状态。

### Reviewer Selection (Round 3)

| Reviewer | Reason Selected | Risk Area |
|---|---|---|
| closure-validator (via claude-ds-pro) | Round 2 已使用同 reviewer 发现新 finding，保持基线一致便于对比；用户明确批准 | finding 闭合验证 + 修订引入的新缺陷 |

### Reviewer Launch Records (Round 3)

| Reviewer | Internal Mechanism | Session / Job ID | Trace Source | Context Forked | Input Packet | Context Explicitly Excluded | Read-only |
|---|---|---|---|---|---|---|---|
| closure-validator | approved_external_cli_substitute | claude-ds-pro pid 27593 stream (one-shot `-p` mode) | claude-ds-pro stdout → `vs_review/.round3-output.md`；reviewer 自行写出 `vs_review/2026-06-24-closure-review-round3.md` | no | `vs_review/.round3-prompt.md`（neutral packet） | main-agent 完整对话历史、Round 1/2 内部推理、修订过程 todo 列表、claude-ds-pro 用户级 `~/.claude/projects/*` 仅作 sandbox 配置告警来源、未注入 | yes（prompt 中明确 read-only；reviewer 仅生成报告未触碰目标文档） |

### Reviewer Timeout Policy (Round 3)

| Complexity | Initial Wait | Extension | Max Attempts Per Role | Blocking Closure Behavior |
|---|---:|---:|---:|---|
| normal | 12 min | 6 min (×1) | 2 | cannot pass if review is unavailable |

### Reviewer Timeout Records (Round 3)

| Reviewer Output Key | Reviewer Role | Attempt | Session / Job ID | Waited | Status | Reason | Action |
|---|---|---:|---|---:|---|---|---|
| closure-validator-1 | closure-validator | 1 | claude-ds-pro one-shot | ≈6 min | completed | reviewer 输出完整 closure report，verdict = pass-with-fixes | none |

### Reviewer Outputs (Round 3)

#### closure-validator-1（external CLI 独立 reviewer，approved replacement）

完整报告：[`vs_review/2026-06-24-closure-review-round3.md`](./2026-06-24-closure-review-round3.md)

##### Summary

**Verdict: pass-with-fixes**。Rounds 1 & 2 的 4 个 blocking（F1–F4）+ 10 个 non-blocking（R1–R10）findings **全部闭合**。修订引入 5 个新缺陷（N1–N5），plan 阶段均非 blocking，但 N3 需在 Phase 2 实施前澄清。

##### All Closed Findings (14/14)

| ID | Original severity | Closure Evidence |
|---|---|---|
| F1 | blocking | `01:60` proxy default = 8765；引用 `cli.py:24` / `settings.py:18` |
| F2 | blocking | `03:43-75` 31 行端点表逐行匹配 `ornnlab/api/*.py`，已交叉验证 |
| F3 | blocking | `03:83-140` 5 个 UI 类型逐字段 保留/派生/删除 决策 |
| F4 | blocking | `03:146-158` ApiClient.get/post 扩展 query 参数 |
| R1 | major | README `:65-66` + 02 `:48` + 04 `:69-74` PR-A / PR-B 切片三处一致 |
| R2 | major | 04 `:51-65` ApiError vs AsyncState.error 边界表 + 归一化规则 |
| R3 | major | 03 `:176-181` mapper 判据修正（1:1 直传 / 枚举聚合分流） |
| R4 | minor | README/02/03/04/05 五处显式声明 SSE 不在范围 |
| R5 | benefit warning | README `:104-111` 4 项量化指标 |
| R6 | minor | 01 `:24-31` 端口布局表与源文件交叉引用 |
| R7 | minor | 03 `:79-81` job_dir gap 节 + AC `:200` |
| R8 | minor | 01 `:89` 一行后端启动 AC + `cli.py#L22-L24` 引用 |
| R9 | maintenance | 03 `:210-213` openapi-typescript defer 到 v0.1.5 |
| R10 | major | 05 `:41` 具体输入→具体 DOM 文本断言 + 具体例子 |

##### New Defects (N1–N5)

| ID | Severity | Summary | Trigger |
|---|---|---|---|
| N1 | minor | `03:66` SSE stream 行 Query Params 标为 `—`，但 `experiments.py:128` 有 `after: int = 0` | Deferred 端点；v0.1.5 实施时需补 |
| N2 | minor | `02:48` "含 04 基础设施" 措辞歧义，可能被读成"和 04 PR-A 同 PR"（与 PR 切片冲突） | 读者解读，README/04 已有正确表述托底 |
| **N3** | **moderate** | `03:136`（6 字段删除）vs `03:193`（不修改 types/console.ts）冲突。mapper 输出现有类型但被删字段无数据源 | Phase 2 实施时开发者不确定填 `""` / 删字段 / View 模板删 cell |
| N4 | minor | `test-after-change-web.sh` 不启动 backend，R5 #2 "real API 2xx" 在 bare CI 会 skip | CI 编排；05 已声明"后端不可用时跳过或 xfail" |
| N5 | minor | "删除"决策对 Vue 模板影响（如 Dashboard alerts 整段消失）未在 02 显式列出 | TS 会捕获不匹配；实施可发现，但 02 缺 UX 影响说明 |

##### Required Fixes Before Phase 2

1. **[P1]** 解决 N3：在 03 §决策汇总 加一句明确"删除 = mapper 输出 `""` / `"—"`（保持类型不变）" 或 "02 中将从 types/console.ts 移除已删除字段"
2. **[P2]** 解决 N2：02 `:48` 措辞改为"View 切换 PR 必须以 04 PR-A 已先合并为前置；可与 04 PR-B 同 PR，或独立后续 PR"
3. **[P2]** 解决 N5：在 02 修复方案补一条"明确列出删除字段对应的模板段落（Dashboard alerts、ExperimentsView Owner 等）的处理方式"
4. **[P3]** 解决 N1：03 行 22 Query Params 补 `after: int = 0`（plan-only 修正）
5. **[P3]** 解决 N4：评估是否扩展 `test-after-change-web.sh` 启动 backend，或显式将 R5 #2/#3 标为 conditional

### Main Agent Response (Round 3)

| Reviewer | Finding | Severity | Decision | Action Taken | Follow-up |
|---|---|---|---|---|---|
| closure-validator-1 | F1–F4, R1–R10 闭合 | — | accept (已闭合) | 无需再修改 | 进入 Phase 1 实施 |
| closure-validator-1 | N1 SSE row query param | minor | accept | 在 03 行 22 Query Params 列加 `after: int = 0` 并标 Deferred | 与 SSE 实施同步 |
| closure-validator-1 | N2 02 措辞歧义 | minor | accept | 修订 02 `:48` 措辞，去除歧义 | n/a |
| closure-validator-1 | N3 删除决策 vs 类型不动 | moderate | accept | 在 03 §决策汇总 加 N3 澄清条款：删除字段 = mapper 输出 `""`，保持 types/console.ts 不变；同时在 02 加一条 AC 标注 | n/a |
| closure-validator-1 | N4 CI 不启 backend | minor | accept (defer) | 在 README R5 验收节加 "metric #2/#3 为 conditional：CI 需先 `python -m ornnlab web` 才能验证；否则按 xfail 处理"；脚本扩展放 v0.1.5 | tracked |
| closure-validator-1 | N5 模板影响未列 | minor | accept | 在 02 修复方案补一条"删除字段对应模板段落处置清单"，引用 03 字段决策 | n/a |

### Closure Status (Round 3)

- Blocking findings (R1/R2): 0 new blocking
- All Rounds 1 & 2 accepted findings re-reviewed: yes，verdict pass-with-fixes
- Implementation completeness gaps resolved or accepted by user: pending（N3/N5 待 main agent 落地后再确认）
- Target benefit warnings recorded: yes（N4 已记录为 conditional 验收）
- Rejected findings backed by evidence: n/a
- Deferred findings documented: yes（N1 与 SSE 同步，N4 部分 defer）
- Allowed to proceed: **yes，待 N1–N5 修订完成后即可进入 Phase 1 (BUG-WEB-01)**




