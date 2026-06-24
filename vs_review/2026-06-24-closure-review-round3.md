# Closure Review Report (Round 3)

## Summary

**Verdict: pass-with-fixes** — All 4 blocking (F1–F4) and 10 non-blocking (R1–R10) findings from Rounds 1 & 2 are closed by the v1.1 revisions. Five new minor-to-moderate defects were discovered (N1–N5), none blocking at plan stage. N3 (type-non-modification vs "delete" decisions) needs clarification before entering Phase 2 implementation so the developer knows whether to emit empty strings, keep fields with dummy values, or relax types.

---

## Per-finding closure verification

| ID | Original severity | Closed? | Evidence (file:line) | Residual risk |
|---|---|---|---|---|
| **F1** | blocking | **yes** | `01-vite-dev-proxy-missing.md:60` — `'http://127.0.0.1:8765'`; `01:30` — port table with `cli.py:24` and `settings.py` citations | None. Default target now matches `ornnlab/cli.py:24` (`default=8765`) and `ornnlab/settings.py:18` (`port: int = 8765`) |
| **F2** | blocking | **yes** | `03-contract-gap-vs-backend.md:43-75` — 31-row endpoint table; rows map 1:1 to `ornnlab/api/*.py` decorators | Very low. Row 22 (SSE stream) omits `after: int = 0` query param but is Deferred — see N1 |
| **F3** | blocking | **yes** | `03-contract-gap-vs-backend.md:83-140` — per-field tables for `ExperimentRecord` (7 fields), `AgentRecord` (7), `KpiMetric` (5), `AlertItem` (3), `LeaderboardSeed` (4). Each has `决策` column: 保留/派生/删除 | Moderate. "删除" decisions conflict with "不在本 PR 修改 types/console.ts" — see N3 |
| **F4** | blocking | **yes** | `03-contract-gap-vs-backend.md:146-158` — extended `ApiClient` interface with `query?: Record<string, string\|number\|boolean>` on both `get` and `post`; `URLSearchParams` implementation note | None. Existing call sites (no query) remain compatible |

| ID | Original severity | Closed? | Evidence (file:line) | Residual risk |
|---|---|---|---|---|
| **R1** | major | **yes** | `README.md:65-66` — "BUG-WEB-04 PR-A … 独立 PR，不依赖 02/03"; `02:48` — "04 基础设施独立 PR 先合"; `04:69-74` — PR-A (仅基础设施) / PR-B (View 接入) split | None. PR slicing is clear across all 3 docs |
| **R2** | major | **yes** | `04-loading-error-empty-states.md:51-65` — error abstraction boundary table + normalization rules; `ApiError` → network/HTTP layer, native `Error` → mapper, `AsyncState.error` → UI container; "不新建 MapperError 子类" | None. Clear 3-layer boundary; `instanceof ApiError` for status-aware rendering |
| **R3** | major | **yes** | `03-contract-gap-vs-backend.md:176-181` — revised criteria: 1:1 copy → no mapper; enum/rename/aggregate → mapper; mapper list narrowed to 4 functions | None. Criteria match external reviewer's recommendation |
| **R4** | minor | **yes** | `02-views-not-consuming-api.md:49` — "SSE … 不在本立项范围（R4）"; `04:49` — "SSE … 不在本立项范围（R4）"; `03:194` — SSE stream row marked Deferred; `README.md:72` — Deferred section | None. SSE boundary explicitly stated in 4 of 6 docs |
| **R5** | target-benefit | **yes** | `README.md:104-111` — 4 quantitative checkboxes: script exit 0, ≥1 real API 2xx, ≥1 View renders backend text, ≥1 specific-input→specific-DOM-text assertion | Low. e2e "real API 2xx" requires backend running; script doesn't start it — see N4 |
| **R6** | minor | **yes** | `01-vite-dev-proxy-missing.md:24-31` — port layout table with dev 4173, preview 4173, e2e 4174, backend 8765; all with source citations | None. Table is clear and accurate vs `vite.config.ts:14-20`, `package.json:16` |
| **R7** | minor | **yes** | `03-contract-gap-vs-backend.md:79-81` — "ExperimentRun 缺 job_dir 字段" section; `03:200` AC — "ExperimentRun 接口包含 job_dir: string \| null" | None. Gap documented; implementation fix specified |
| **R8** | minor | **yes** | `01-vite-dev-proxy-missing.md:89` — AC item: "后端启动命令可一行执行：python -m ornnlab web" with `cli.py#L22-L24` citation | None. Command matches `ornnlab/cli.py:22-24` exactly |
| **R9** | maintenance | **yes** | `03-contract-gap-vs-backend.md:210-213` — "Maintenance Follow-up（R9 defer 到 v0.1.5）" section; references openapi-typescript tool | None. Deferred to correct target release |
| **R10** | major | **yes** | `05-integration-test-gap.md:41` — "≥1 个 View 的 happy path 测试必须做特定输入 → 特定 DOM 文本断言"; concrete example with `{ name: "exp-001", status: "completed" }` → assert `exp-001` text + `complete` label | None. Concrete example removes ambiguity about assertion depth |

---

## New defects introduced by the revisions

### N1 — SSE stream query param omission (minor)

- **Location**: `03-contract-gap-vs-backend.md:66` (row 22)
- **Finding**: The `GET /api/experiments/{id}/events/stream` row shows `—` for Query Params, but the actual code at `ornnlab/api/experiments.py:128` accepts `after: int = 0`. Since this endpoint is Deferred, the omission is non-blocking. However, when the endpoint is later implemented, omitting `after` from the plan means it could be missed again.
- **Severity**: minor (Deferred endpoint; v0.1.5 or later scope)

### N2 — 02 PR-slicing wording ambiguity (minor)

- **Location**: `02-views-not-consuming-api.md:48`
- **Finding**: "View 切换可在同一 PR（含 04 基础设施）或后续 PR 中进行" — the parenthetical "含 04 基础设施" is ambiguous. It could be read as "in the same PR that also includes 04 PR-A infrastructure," which contradicts the PR separation strategy that 04 PR-A is independent. The intended meaning is likely "with 04 PR-A already merged," but a reader could misinterpret it.
- **Severity**: minor (README.md:65-66 and 04:69-74 resolve the ambiguity with explicit PR-A/PR-B split)

### N3 — "Delete" decisions vs type-non-modification constraint (moderate)

- **Location**: `03-contract-gap-vs-backend.md:136` (decision summary) vs `03:193` (scope boundary)
- **Finding**: Six viewmodel fields are marked **删除**: `ExperimentRecord.owner`, `AgentRecord.owner`, `AgentRecord.queue`, `AgentRecord.lastHeartbeat`, `AlertItem.*`. But `03:193` says "不在本 PR 修改 types/console.ts 的语义边界." If the types aren't modified, the mapper functions (which claim to return the existing `ExperimentRecord` / `AgentRecord` types) must still produce values for `owner: string`, `queue: string`, `lastHeartbeat: string`, and `AlertItem[]`. The plan doesn't specify whether:
  - (a) Mappers emit `""` / `"—"` for deleted fields (keeping the type unchanged)
  - (b) The types are relaxed to make deleted fields optional (contradicting the type-non-modification rule)
  - (c) View templates are changed to not render these fields, but the fields remain in the type with dummy values

  **Impact if unresolved during implementation**: Developer uncertainty about whether to delete template cells entirely or just fill with `"—"`. The DashboardView's entire "Priority alerts" section (`DashboardView.vue:30-41`) renders `AlertItem[]` — if this is truly deleted, that section disappears; the 02 doc should acknowledge this UX regression.
- **Severity**: moderate (plan-stage ambiguity; implementation would surface the concrete choices but could cause rework)

### N4 — e2e "real API 2xx" not self-contained in CI script (minor)

- **Location**: `README.md:109` vs `scripts/test-after-change-web.sh:23-30`
- **Finding**: R5 metric #2 requires "e2e smoke 中至少 1 个真实 API 请求返回 2xx." The `test-after-change-web.sh` script runs `npm --prefix frontend run e2e` but does **not** start the FastAPI backend. The e2e script (`package.json:16`) builds the frontend and starts Playwright against a preview server. Without a backend, `/api/system/status` returns 404 (from Vite preview, not from FastAPI). The 05 doc says "后端不可用时跳过或 xfail" — but that means the metric is conditionally observable, not guaranteed. In CI, unless CI also starts `python -m ornnlab web`, R5 metric #2 will always skip.
- **Severity**: minor (the skipped/xfail behavior is documented in 05; but R5's presence in README as a quantified gate implies it passes in CI, which it won't without backend orchestration)

### N5 — Cross-document template-change traceability gap (minor)

- **Location**: 03 viewmodel decisions ↔ 02 View changes ↔ actual Vue templates
- **Finding**: The "delete" decisions in 03 require specific template changes that are not explicitly traced to 02. Concrete impacts verified against source:
  - `DashboardView.vue:56` — `<span>Owner</span>` header + `:68` — `{{ experiment.owner }}` cell → field deleted
  - `DashboardView.vue:30-41` — entire alerts section rendering `v-for="alert in snapshot.alerts"` → `AlertItem.*` deleted; section vanishes
  - `ExperimentsView.vue:38-39` — `<dt>Owner</dt><dd>{{ experiment.owner }}</dd>` → field deleted
  - `AgentsView.vue:51` — `<span>Queue</span>` header + `:65` — `{{ agent.queue }}` cell → field deleted
  - `AgentsView.vue:53` — `<span>Heartbeat</span>` header + `:68` — `{{ agent.lastHeartbeat }}` cell → field deleted
  - `AgentsView.vue:63` — `<small>{{ agent.owner }}</small>` → field deleted
  - `LeaderboardView.vue:26-27` — `<span>Success</span>` / `<span>Experiments</span>` headers → fields marked "派生或删除"

  The 02 doc says "改造每个 View" but doesn't enumerate which template cells must change. A developer implementing 02 would need to cross-reference 03's tables with the source templates themselves. While TypeScript would catch type mismatches, the plan could reduce implementation risk by listing the specific template sections that require removal vs data-binding-only changes.
- **Severity**: minor (implementation discoverable; not a logical error in the plan)

---

## Cross-document consistency check

| Check | Result | Detail |
|---|---|---|
| README PR slicing vs 02/04 | **Consistent** | README `:65-66`/`:68` matches 04 `:69-74` (PR-A/PR-B split) and 02 `:48` (04 infra merged first). See N2 for minor wording ambiguity in 02. |
| SSE scope consistency | **Consistent** | README `:72`, 02 `:49`, 03 `:66`/`:194`, 04 `:49`, 05 `:46` all explicitly exclude SSE. No divergence. |
| Mapper criteria across docs | **Consistent** | 03 `:176-181` defines criteria; 02 `:41` references "BUG-WEB-03 R3 判据." Single source of truth. |
| Phase dependency linearity | **Consistent** | README `:77-93` diagram matches 01 Phase 1, 03→02+04 Phase 2, 05 Phase 3. No circular dependencies. |
| Viewmodel decisions vs type file | **Tension** | 03 `:193` says "不在本 PR 修改 types/console.ts" but `:136` marks 6 fields as **删除**. See N3. |
| Quantified metrics vs CI script | **Partial** | README R5 metrics reference `test-after-change-web.sh`; the script runs vitest+e2e but doesn't start the backend. See N4. |
| AC cross-references | **Consistent** | 02 AC `:57` references "依赖 BUG-WEB-04 基础设施已先行合并"; 05 AC `:54` references `test-after-change-web.sh`. All cross-doc AC dependencies traceable. |

---

## 31-endpoint inventory audit

Verified by direct comparison of `03-contract-gap-vs-backend.md:43-75` table against all 7 `ornnlab/api/*.py` files.

| Router file | Endpoints in code | Endpoints in 03 table | Match? |
|---|---|---|---|
| `system.py` | 3 (`status`, `doctor`, `docker-orphans`) | 3 (rows 1–3) | ✅ |
| `agents.py` | 7 (`list`, `create`, `get`, `compile`, `validate`, `update`, `delete`) | 7 (rows 4–10) | ✅ |
| `benchmarks.py` | 1 (`list`) | 1 (row 11) | ✅ |
| `experiments.py` | 12 (`list`, `create`, `get`, `run`, `cancel`, `delete`, `clone`, `save-template`, `report`, `events`, `events/stream`, `runs`) | 12 (rows 12–23) | ✅ |
| `runs.py` | 4 (`get`, `cancel`, `events`, `report`) | 4 (rows 24–27) | ✅ |
| `templates.py` | 3 (`list`, `create`, `delete`) | 3 (rows 28–30) | ✅ |
| `leaderboard.py` | 1 (`list`) | 1 (row 31) | ✅ |
| **Total** | **31** | **31** | ✅ |

### Query param verification

| Row | Endpoint | 03 table says | Actual code | Match? |
|---|---|---|---|---|
| 2 | `POST /api/system/doctor` | `logs: bool = False` | `logs: bool = False` (`system.py:16`) | ✅ |
| 15 | `POST /api/experiments/{id}/run` | `wait: bool = False` | `wait: bool = False` (`experiments.py:37`) | ✅ |
| 21 | `GET /api/experiments/{id}/events` | `after: int = 0` | `after: int = 0` (`experiments.py:101`) | ✅ |
| 22 | `GET /api/experiments/{id}/events/stream` | `—` | `after: int = 0` (`experiments.py:128`) | ❌ See N1 |
| 26 | `GET /api/runs/{run_id}/events` | `after: int = 0` | `after: int = 0` (`runs.py:30`) | ✅ |
| 31 | `GET /api/leaderboard` | `benchmark: str \| None = None` | `benchmark: str \| None = None` (`leaderboard.py:11`) | ✅ |

### ornnLabApi coverage verification

Current `frontend/src/api/client.ts:134-173` provides exactly 13 accessors. All 13 are correctly marked ✅ in the 03 table. The `leaderboard()` method manually constructs query string (`client.ts:170`) — the F4 apiClient extension would formalize this.

Status distribution: 13 covered ✅ + 17 待补 + 1 Deferred = 31 ✅

---

## Viewmodel decision feasibility

### Fields marked "删除" — template impact

Verified against actual Vue templates (`frontend/src/views/*.vue`):

| Deleted field | Template locations using it | Impact |
|---|---|---|
| `ExperimentRecord.owner` | `DashboardView.vue:55,68` (Owner column header + cell); `ExperimentsView.vue:38-39` (Owner in detail grid) | Both Views render an "Owner" column/slot. Deleting this field means templates must remove these cells. |
| `AgentRecord.owner` | `AgentsView.vue:63` (`<small>{{ agent.owner }}</small>`) | Subtitle under agent name. Can be removed without structural change. |
| `AgentRecord.queue` | `AgentsView.vue:50,65` (Queue column header + cell) | Table column must be removed. |
| `AgentRecord.lastHeartbeat` | `AgentsView.vue:53,68` (Heartbeat column header + cell) | Table column must be removed. |
| `AlertItem.*` (title, detail, severity) | `DashboardView.vue:30-41` (entire alerts section) | **Entire "Priority alerts" section** (lines 23-42) disappears. This is a visible UX regression that the 02 doc doesn't acknowledge — what replaces it? |
| `LeaderboardSeed.successRate` | `LeaderboardView.vue:26,37` (Success column) | Marked "派生或删除"; if deleted, column removed |
| `LeaderboardSeed.experiments` | `LeaderboardView.vue:27,38` (Experiments column) | Marked "派生或删除"; if deleted, column removed |

**Assessment**: The mapper decisions are technically correct (fields genuinely have no backend source), but the 02 doc should explicitly list the template sections that require removal vs data-binding-only changes. The DashboardView alerts section deletion is the most impactful — removing it without a replacement leaves a layout gap. This is implementation-level concern that doesn't block the plan, but the 02 AC should include a note about the alerts section replacement strategy.

---

## Quantitative acceptance observability

### R5 metrics vs test-after-change-web.sh

| R5 metric | Measurable by script? | Mechanism | Caveat |
|---|---|---|---|
| 1. `test-after-change-web.sh` exit 0 | **Yes** | Script already runs typecheck + lint + vitest + e2e + storybook (`script:23-30`). Exit code 0 = all pass. | Requires 05 to add real tests; without them, tests pass vacuously. |
| 2. ≥1 real API request returns 2xx | **Conditional** | e2e smoke test (`script:29`) runs Playwright against preview server. Backend must be running separately. | Script does NOT start `python -m ornnlab web`. In bare CI, this test would skip (per 05 "后端不可用时跳过或 xfail"). Not self-contained. |
| 3. ≥1 View renders backend data text | **Conditional** | Same e2e dependency as #2. Requires real backend for non-mocked assertion. | Vitest View integration tests (with mock fetch) could partially cover this; e2e verifies the real path. |
| 4. ≥1 View test has specific-input→specific-DOM-text | **Yes** | Vitest View integration test with `@vue/test-utils` + mock fetch. Does NOT require backend. | Fully self-contained; test runs in JSDOM with mocked network. |

**Assessment**: Metrics #1 and #4 are fully self-contained and will pass/fail deterministically in CI. Metrics #2 and #3 depend on backend availability — the script will skip them when the backend isn't running. For CI to validate #2/#3, either (a) CI must start `python -m ornnlab web` before the script, or (b) the "至少 1 个" threshold is understood as "passes when backend is available, skipped otherwise." The 05 doc acknowledges this ("后端不可用时跳过或 xfail"), but the README's presentation of these as unconditional gates is slightly misleading.

---

## Final verdict

**pass-with-fixes**

All 4 blocking findings (F1–F4) and all 10 non-blocking findings (R1–R10) are substantively closed. The v1.1 revisions address every broken assumption from Rounds 1 & 2 with concrete, verifiable changes.

The 5 new defects (N1–N5) are non-blocking at plan stage:

- **N1** (SSE query param): Fix trivially when the endpoint is implemented.
- **N2** (wording ambiguity): Clarify in 02 before Phase 2 starts.
- **N3** (type-non-modification vs delete): **Highest priority** — resolve before Phase 2 by specifying whether deleted fields get empty/dummy values or whether types/console.ts will be relaxed. Also, add a note in 02 about the Dashboard alerts section removal.
- **N4** (CI script backend dependency): Accept as-is (documented skip behavior) or extend `test-after-change-web.sh` to optionally start the backend.
- **N5** (template-change traceability): Low priority; TypeScript will catch mismatches during implementation.

**Recommended pre-implementation actions:**
1. Clarify N3: add a sentence in 03 §"决策汇总" stating "删除 = mapper 输出空字符串 `""` / `"—"`（保持类型不变）" or "02 中将从 types/console.ts 移除已删除字段."
2. Add a bullet in 02's修复方案 acknowledging that the Dashboard alerts section will be removed and what (if anything) replaces it.
3. Optionally fix N1 by adding `after: int = 0` to row 22's Query Params column.
