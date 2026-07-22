# Problem P-001: Harness 与 LiteLLM 价格看起来相同
- Status: diagnosed
- Created: 2026-07-22 15:36
- Updated: 2026-07-22 15:40
- Objective: 解释手动 Agent 中 Harness 上报与 LiteLLM 价格为何显示相同，并确认展示价格没有混淆实际计费来源。
- Symptoms:
  - 用户观察到同一手动 Agent 中 Harness 上报与 LiteLLM 都显示了 LiteLLM 价格。
- Expected behavior:
  - 两种来源的展示含义与实际成本计算路径应清晰可辨。
- Actual behavior:
  - 两种来源都展示输入缓存未命中、输入缓存命中和输出三项价格。
- Impact:
  - 可能使用户误以为 Harness 上报和 LiteLLM 按同一单价计算，影响成本数据可信度判断。
- Reproduction:
  - 打开 `claude-code-deepseek-v4-pro` Agent，对比 `deepseek-v4-pro` 的 LiteLLM 来源与 `deepseek-v4-flash` 的 Harness 上报来源。
- Environment:
  - Ubuntu 本机，OrnnLab main `66e1919`，2026-07-22。
- Known facts:
  - Agent 的 `deepseek-v4-pro` 配置为 `litellm`，`deepseek-v4-flash` 配置为 `reported`。
  - 价格预览 API 对两个模型均返回 `source=litellm` 的目录条目。
  - 前端对两种非自定义来源复用同一个预览对象，只切换说明文案。
  - 后端 `reported` 返回 Harbor `cost_usd`，`litellm` 使用 Token 与快照费率重算。
- Ruled out:
  - none
- Fix criteria:
  - 本案例只做诊断；如确认存在误导性设计，修复需用户另行授权并以展示和实际计费路径验证为准。
- Current conclusion: 页面显示相同是当前实现有意复用 LiteLLM 目录参考价造成的；两种来源的实际成本计算并不相同。Harness 没有可供配置页预览的固定单价，这种展示容易让用户误解数据来源。
- Related hypotheses:
  - H-001
  - H-002
- Resolution basis:
  - H-001、H-002；E-001 至 E-004
- Close reason:
  - not closed

## Hypothesis H-001: 两种模式故意复用同一目录价预览
- Status: confirmed
- Parent: P-001
- Claim: Agent 编辑器对 `reported` 与 `litellm` 调用同一个价格预览接口，因此价格数据来源相同，只有计费说明不同。
- Layer: interaction
- Factor relation: part_of
- Depends on:
  - none
- Rationale:
  - 上一需求要求 Harness 与 LiteLLM 模式都显示价格，而 Harness 只上报运行后总成本，没有统一的预配置单价。
- Falsifiable predictions:
  - If true: 两种来源渲染同一个 `ModelPricingPreviewDto`，Harness 文案将其标记为参考价。
  - If false: Harness 模式存在独立价格查询或从 Harness 读取固定单价。
- Diagnostic evidence plan:
  - Prediction or clause under test: 两种来源渲染同一个目录预览，仅说明文案不同。
  - Signal: 前端组件的数据加载与分支渲染代码。
  - Capture method: 读取 `AgentModelSettings.tsx` 和真实 API 响应。
  - Event name or marker:
    - none
  - Correlation keys:
    - Agent `claude-code-deepseek-v4-pro`
  - Differentiates from:
    - H-002
  - Supports if:
    - 两种来源共享 `getModelPricing` 返回值，`reported` 仅切换说明文案。
  - Refutes if:
    - Harness 有独立固定单价数据源。
  - Instrumentation status: none
  - Instrumentation lifecycle:
    - none
- Evidence gate: satisfied
- Related evidence:
  - E-001
  - E-002
  - E-003
- Conclusion: `reported` 与 `litellm` 共享同一个 LiteLLM 目录预览；前者仅通过说明文案声明这是参考价。
- Repair design readiness: ready; no repair requested
- Next step: 向用户解释并等待是否授权调整展示语义。
- Blocker:
  - none
- Close reason:
  - not closed

## Hypothesis H-002: 展示复用但实际计费来源仍然分离
- Status: confirmed
- Parent: P-001
- Claim: `reported` 实际读取 Harbor 的 `cost_usd`，`litellm` 则在 Job 创建时固化目录单价并按 Token 重算；相同展示不意味着相同计费。
- Layer: root-cause
- Factor relation: part_of
- Depends on:
  - H-001
- Rationale:
  - 当前产品契约定义了两条不同的成本路径。
- Falsifiable predictions:
  - If true: Job 快照和 `calculate_cost` 会按 `source` 分支，`reported` 快照不包含三项费率。
  - If false: 两种来源最终都通过 LiteLLM 三项费率计算成本。
- Diagnostic evidence plan:
  - Prediction or clause under test: `reported` 与 `litellm` 的 Job 快照和计算函数存在可观察分支。
  - Signal: Agent 配置、Job 私有快照、成本计算代码与已有 Job 原始结果。
  - Capture method: 查询 SQLite 配置并读取 `model_pricing.py`、`webui_job_service.py`。
  - Event name or marker:
    - `webui.job.configured`
  - Correlation keys:
    - Agent `claude-code-deepseek-v4-pro`
  - Differentiates from:
    - 两种来源实际都按 LiteLLM 价格重算
  - Supports if:
    - `reported` 返回原始 `cost_usd`，`litellm` 使用三项快照费率。
  - Refutes if:
    - 两种来源进入相同计算分支。
  - Instrumentation status: none
  - Instrumentation lifecycle:
    - none
- Evidence gate: satisfied
- Related evidence:
  - E-001
  - E-003
  - E-004
- Conclusion: 两种来源的实际计算分支不同；历史 Job 的真实金额对照也证明结果并不相等。
- Repair design readiness: ready; no repair requested
- Next step: 向用户解释并等待是否授权调整展示语义。
- Blocker:
  - none
- Close reason:
  - not closed

## Evidence E-001: 手动 Agent 的两模型来源配置
- Related hypotheses:
  - H-001
  - H-002
- Direction: supports
- Type: config
- Source: `GET /api/webui/v1/agents?limit=100`
- Prediction or plan link:
  - H-002 需要确认实际比较对象使用不同来源。
- Matched signal:
  - `deepseek-v4-pro=litellm`，`deepseek-v4-flash=reported`
- Correlation keys:
  - Agent `claude-code-deepseek-v4-pro`
- Raw content:
  ```text
  deepseek-v4-pro: source=litellm
  deepseek-v4-flash: source=reported
  ```
- Interpretation: 用户的 Agent 确实同时配置了两种不同来源，不是界面标签读取错误。
- Time: 2026-07-22 15:36

## Evidence E-002: 价格预览统一来自 LiteLLM 目录
- Related hypotheses:
  - H-001
- Direction: supports
- Type: probe
- Source: `GET /api/webui/v1/model-pricing/preview?modelName=...`
- Prediction or plan link:
  - H-001 的统一目录预览预测。
- Matched signal:
  - 两个响应的 `source` 都是 `litellm`。
- Correlation keys:
  - `deepseek-v4-pro`
  - `deepseek-v4-flash`
- Raw content:
  ```text
  deepseek-v4-pro: miss=0.435 hit=0.003625 output=0.87 source=litellm
  deepseek-v4-flash: miss=0.14 hit=0.0028 output=0.28 source=litellm
  ```
- Interpretation: 页面可用的预览价格不是 Harness 独立费率，而是当前安装的 LiteLLM 目录价。
- Time: 2026-07-22 15:36

## Evidence E-003: 展示与成本计算使用不同分支
- Related hypotheses:
  - H-001
  - H-002
- Direction: supports
- Type: code-location
- Source: `frontend/src/ui/components/AgentModelSettings.tsx:24-34,135-158`；`ornnlab/services/model_pricing.py:15-69`
- Prediction or plan link:
  - H-001 的共享预览预测；H-002 的成本分流预测。
- Matched signal:
  - 所有模型只调用 `loadPricing(model)`；非自定义来源共用 `ResolvedPricing`；`reported` 在 `calculate_cost` 中直接返回 `usage.cost_usd`。
- Correlation keys:
  - Agent `claude-code-deepseek-v4-pro`
- Raw content:
  ```text
  frontend: response = await loadPricing(model)
  frontend: pricing.source === 'reported' ? pricingHarnessReferenceNote : pricingLiteLlmNote
  backend: if source == "reported": return {modelName, source}
  backend: if snapshot.source == "reported": return usage.cost_usd
  ```
- Interpretation: 相同的是配置页参考单价的数据源，不是实际成本算法。
- Time: 2026-07-22 15:39

## Evidence E-004: 历史 Job 的 Harness 上报额与 LiteLLM 重算额不同
- Related hypotheses:
  - H-002
- Direction: supports
- Type: reproduction
- Source: `run-371699db5dee` 的 Harbor `result.json` 与当前 `deepseek-v4-pro` LiteLLM 目录价
- Prediction or plan link:
  - H-002 预测两种实际成本不会因为页面预览相同而相同。
- Matched signal:
  - Harness 上报 `$8.324371`；按 LiteLLM 目录与同一批 Token 重算为 `$0.255605246`。
- Correlation keys:
  - Job `run-371699db5dee`
- Raw content:
  ```text
  harness_reported_cost_usd=8.324371000000001
  uncached_input_tokens=298892
  cache_tokens=8137472
  output_tokens=110447
  litellm_recomputed_cost_usd=0.255605246
  ```
- Interpretation: 实际成本路径有显著差异；对于 Claude Code 接 DeepSeek 代理，Harness 上报总额还可能沿用 Claude Code 自身的成本口径，因此不能把目录参考价理解为 Harness 单价。
- Time: 2026-07-22 15:40
