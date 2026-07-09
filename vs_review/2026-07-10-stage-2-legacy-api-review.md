# Subagent VS Review: Stage 2 Legacy API Residue

- Created: 2026-07-10T02:14:27+0800
- Updated: 2026-07-10T07:05:00+0800
- Report schema: adversarial-v1
- Task: Review Stage 2 contract-layer work, with emphasis on old API documentation and code residue.
- Report path: `vs_review/2026-07-10-stage-2-legacy-api-review.md`
- Review mode: fresh internal subagents
- Source session policy: no inherited main-agent context; approved CLI substitutes receive only the review packet
- Status: closed — Stage 2 frontend contract closure approved; Stage 3 backend implementation remains open.

## Round 1: Legacy API boundary review

### Review Input

#### Objective
Falsify the claim that v1.0.5 Stage 2 has established a clean migration path from old backend API semantics to the `/api/webui/v1` product contract without retaining a legacy API compatibility layer.

#### Review Target
Code implementation, contract documentation, and planning documentation.

#### Target Locations

- `frontend/src/api/`
- `frontend/src/app/App.tsx`
- `frontend/src/mocks/`
- `frontend/src/mocks/mswHandlers.ts`
- `docs/architecture/frontend-api-contract.md`
- `docs/releases/v1.0.5/`
- `ornnlab/api/`

#### Change Introduction
Stage 2 added structured Jobs/Datasets DTOs, a `/api/webui/v1` HTTP client, and an offline mock client. Pages have not yet been migrated from direct mock seed imports.

#### Risk Focus

- Old route names or old response semantics still presented as official architecture.
- Legacy adapters or fallback paths accidentally introduced in the frontend.
- Pages, stories, or MSW still reading old routes or bypassing the contract client.
- Documents contradicting the decision to directly break old APIs rather than maintaining two API sets.

#### User-Perspective Review Focus

- A future developer can identify one official API entry point without mistaking historical routes for supported interfaces.

#### Implementation Completeness Focus

- Verify Stage 2 is described as partial while production pages still use mock seed data.
- Verify the HTTP client and mock client are not misrepresented as finished backend integration.

#### Target Benefit Focus

- Claimed benefit: reducing future integration rework by establishing one API contract.
- Baseline, target, and runtime measurement are not yet available; challenge any claim that this benefit is already proven.

#### Assumptions To Attack

- Old route references exist only as migration evidence.
- `/api/webui/v1` is the sole forward-looking frontend route prefix.
- No legacy adapter exists or is planned.
- Mock client data remains isolated from production screen entry points after Stage 2 completion claims.

#### Adversarial Lenses

- implementation-completeness
- architecture
- documentation
- testing
- maintenance

#### Verification Status

- Commit under review: `1738e45`.
- Validated before review: unit tests, typecheck, lint, production build, Storybook smoke, and e2e passed.
- No real backend contract smoke has run.

#### Reviewer Instructions

- Fresh session with no inherited main-agent context.
- Read target files directly.
- Do not modify files.
- Cite evidence paths and line numbers when possible.
- Challenge incomplete wiring and historical-route ambiguity.

### Internal Subagent Unavailable Fallback

Not applicable. Codex internal subagent tooling is available in this session.

### Reviewer Timeout Policy

| Complexity | Initial Wait | Extension | Max Attempts Per Role | Blocking Closure Behavior |
|---|---:|---:|---:|---|
| complex | 15 minutes | 10 minutes | 2 | cannot pass if review is unavailable |

### Reviewer Selection

| Reviewer | Reason Selected | Risk Area |
|---|---|---|
| implementation-completeness-adversary | The primary risk is that Stage 2 exists only as protocol/mock scaffolding or leaves old API paths as de facto production integrations. | Legacy residue, production-path wiring, and documentation truthfulness |

### Reviewer Launch Records

| Reviewer | Internal Mechanism | Session / Job ID | Trace Source | Context Forked | Input Packet | Context Explicitly Excluded | Read-only |
|---|---|---|---|---|---|---|---|
| implementation-completeness-adversary | `multi_agent_v1__spawn_agent` | `019f4834-fce7-7050-bffe-c487b8a1ea1b` | current task tool trace | false | Round 1 Review Input | main-agent history, reasoning, conclusions, full diff | yes |

### Reviewer Timeout Records

| Reviewer Output Key | Reviewer Role | Attempt | Session / Job ID | Waited | Status | Reason | Action |
|---|---|---:|---|---:|---|---|---|
| round-1-reviewer | implementation-completeness-adversary | 1 | `019f4834-fce7-7050-bffe-c487b8a1ea1b` | 6 minutes | completed | reviewer completed within the initial 15-minute window | completed |

### Reviewer Outputs

#### round-1-reviewer

##### Summary

The reviewer did not find a frontend legacy adapter, but found that the backend still exposes only legacy routes and the production React app still boots from local mock seeds. The reviewer classified Stage 2 as contract scaffolding rather than a completed forward migration.

##### Blocking Findings

- `F1` Backend `/api/webui/v1` does not exist; legacy routes remain the live server contract.
  - Broken assumption: the typed frontend contract can already be treated as an active backend migration.
  - Failure scenario: a client requests `/api/webui/v1/jobs` or `/api/webui/v1/datasets` before Stage 3 routes exist.
  - Trigger condition: any actual API-mode request.
  - Impact: 404 responses and old response semantics remain externally active.
  - Proof needed: FastAPI routers for `/api/webui/v1` with the new response envelope and operation model; old frontend-facing route names retired as the product contract.
  - Evidence: `ornnlab/app.py:31`, `ornnlab/api/experiments.py:12`, `ornnlab/api/runs.py:8`, `ornnlab/api/benchmarks.py:5`, `ornnlab/api/agents.py:8`, `ornnlab/api/leaderboard.py:7`, `ornnlab/api/system.py:7`.

- `F2` The production frontend bypasses the contract client and still runs directly on seed state.
  - Broken assumption: the production path has already moved to the WebUI contract.
  - Failure scenario: the backend is unavailable or upgraded while the UI continues to render populated local state.
  - Trigger condition: app startup, dataset filtering, or New Job launch.
  - Impact: migration incompleteness is hidden and removal of old routes is not exercised.
  - Proof needed: App and screen entry points consume data hooks backed by `WebUiClient`, with no direct production imports from `frontend/src/mocks/`.
  - Evidence: `frontend/src/app/App.tsx:4`, `frontend/src/app/App.tsx:70`, `frontend/src/app/App.tsx:95`, `frontend/src/app/App.tsx:162`, `frontend/src/api/webUiClient.ts:11`.

- `F3` Storybook, MSW, and tests do not yet validate the WebUI contract end to end.
  - Broken assumption: the offline/mock path protects route and envelope accuracy.
  - Failure scenario: a contract route or response shape drifts while stories inject props directly and handlers expose incomplete or mismatched resources.
  - Trigger condition: future client hookup or contract evolution.
  - Impact: incorrect route residue can survive green visual and e2e checks.
  - Proof needed: MSW handlers mirror contract routes and `ApiResponse` envelopes exactly; screen/App stories exercise client or hooks rather than raw seed props.
  - Evidence: `frontend/src/app/App.stories.tsx:16`, `frontend/src/screens/Screens.stories.tsx:5`, `frontend/src/mocks/mswHandlers.ts:8`, `frontend/src/mocks/mswHandlers.ts:15`, `docs/architecture/frontend-api-contract.md:37`.

##### Non-blocking Risks

- `R1` Route naming drifts between `GET /leaderboards` in `docs/releases/v1.0.5/engineering-plan.md:129` and singular `GET /leaderboard` in `docs/architecture/frontend-api-contract.md:517`.
- `R2` `requestJson()` casts arbitrary JSON to `ApiResponse<T>` without runtime shape validation: `frontend/src/api/webUiClient.ts:22`.
- `R3` The feature checklist retains historical old endpoint names in “current backend” cells, which can be misread as implementation guidance: `docs/releases/v1.0.5/harbor-webui-feature-coverage-checklist.md:192`, `:207`.

##### User-Perspective Checks

- Usability risk: API mode cannot yet reveal a backend failure because the app is locally seeded (`frontend/src/app/App.tsx:70`).
- Ease-of-use risk: backend migration currently does not change UI behavior because the HTTP client is unused (`frontend/src/api/webUiClient.ts:11`).
- Ease-of-understanding risk: Storybook may appear healthy while the product contract is wrong because stories inject props directly (`frontend/src/screens/Screens.stories.tsx:52`).

##### Implementation Completeness Checks

| Plan Item | Expected Behavior | Production Code Path | Integration Entry | Test Evidence | Runtime / Log Evidence | Mock / Stub Exposure | Status | Finding Link |
|---|---|---|---|---|---|---|---|---|
| Typed `/api/webui/v1` client | Client exists and is consumed by the app | Defined but unused | `frontend/src/api/webUiClient.ts:11` | mock client test only | none | mock client maps seeds | partial | F2 |
| Page migration to hooks/client | App/screens read through client | Direct seed state remains | `frontend/src/app/App.tsx:70` | app/e2e assert seed values | none | high | missing | F2 |
| Backend breaking upgrade | New routes replace old product contract | Old routes only | `ornnlab/app.py:31` | none | none | n/a | missing | F1 |
| Offline/mock contract path | MSW and stories mirror contract | Stories bypass client; handler set incomplete | `frontend/src/mocks/mswHandlers.ts:8` | no route/envelope assertion | none | high | incomplete | F3 |
| Partial Stage 2 documentation | All authority docs state not integrated | Mixed wording | v1.0.5 documents | conflicting evidence | none | n/a | inconsistent | F3 |

##### Target Benefit Checks

| Claimed Benefit | Baseline | Target | Measurement Method | Comparison Evidence | Result | Regression / Side Effect | Status | Finding Link |
|---|---|---|---|---|---|---|---|---|
| Future integration rework reduction | no baseline | one consumable product contract | API-mode smoke and route-removal test | `typecheck`, 14 unit tests, 10 e2e tests only exercise seeded UI | unmeasured | green gates can be misread as integration proof | unmeasured | F1, F2, F3 |

##### Required Fixes

- Migrate App and screen entries to data hooks backed by `WebUiClient`; remove direct mock seed imports from production paths.
- Implement `/api/webui/v1` routers with `ApiResponse<T>` and `Operation`, then retire old frontend-facing route names.
- Rewrite MSW handlers to the contract routes and envelopes, or remove them until client-backed stories exist.
- Align authority documents so they consistently state: Stage 2 is partial, no legacy adapter, backend migration has not happened, and demo-only items remain `Partial`.

##### Missing Tests

- Live frontend-to-FastAPI smoke for `/api/webui/v1`.
- A test that fails if App silently falls back to local seeds in API mode.
- MSW contract tests for route names and `ApiResponse` shape.
- Backend tests proving old frontend-facing routes are retired or explicitly non-canonical.

##### Missing Logs / Observability

- No visible mock/API mode indicator.
- No request/error instrumentation around `webUiClient`.
- No automated residue check for legacy route names across frontend-facing docs, mocks, and handlers.

### Main Agent Response

| Reviewer | Finding | Broken Assumption / Failure Scenario | Severity | Decision | Evidence / Reason | Action Taken | Follow-up |
|---|---|---|---|---|---|---|---|
| implementation-completeness-adversary | F1 | Treating Stage 2 scaffolding as an active backend migration causes `/api/webui/v1` requests to fail. | blocking for Stage 3+ | accept | The engineering plan explicitly lists Stage 3 as not started; the finding correctly prevents premature completion claims. | No code change in this review-only round. | Implement Stage 3, then run a fresh closure review. |
| implementation-completeness-adversary | F2 | The app appears connected while directly reading seed data. | blocking for Stage 2 completion | accept | `App.tsx` imports mock seeds directly. The plan already lists hooks/page migration as unfinished; this is a real completion blocker. | No code change in this review-only round. | Implement data hooks and Jobs/Datasets migration, then run a fresh closure review. |
| implementation-completeness-adversary | F3 | Green Storybook/e2e can conceal route or envelope drift. | blocking for Stage 2 completion | accept | Existing stories mostly inject props and MSW resources do not yet mirror the typed client surface. | No code change in this review-only round. | Add contract-accurate handlers, client-backed stories, and route/envelope tests; then run a fresh closure review. |
| implementation-completeness-adversary | R1 | Singular/plural leaderboard route ambiguity may produce mismatched client/backend work. | medium | accept | The two authority paths use different route spellings. | No code change in this review-only round. | Resolve in the API contract and plan before Stage 3 implementation. |
| implementation-completeness-adversary | R2 | Malformed payloads pass an unchecked cast. | medium | accept | `requestJson` has no envelope validation. | No code change in this review-only round. | Add response validation and API error tests while implementing hooks. |
| implementation-completeness-adversary | R3 | Historical route references can be mistaken for target routes. | medium | accept | The checklist presents old routes as current-backend evidence without a uniform legacy marker. | No code change in this review-only round. | Clarify historical-route labels in the documentation cleanup batch. |

### Closure Status

- Blocking findings found: yes
- Accepted blocking findings fixed: no
- Blocking re-review completed: no
- Blocking re-review passed: no
- Blocking re-review round links:
  - n/a; implementation has not started
- Blocking re-review launch records:
  - n/a
- Rejected findings backed by evidence: n/a
- Deferred findings documented: no
- Implementation completeness gaps resolved or accepted by user: no
- Target benefit warnings recorded: yes
- Blocked reason: accepted blocking findings require Stage 2 page/hook migration and Stage 3 backend work before closure.
- Allowed to proceed: yes, only as an explicitly partial Stage 2 implementation; no readiness or integration-complete claim is permitted.

## Implementation Update: 2026-07-10

The following remediation has landed after Round 1. This is not a closure review.

| Finding | Remediation | Current state |
|---|---|---|
| F2 | Added runtime `api`/`mock` modes, resource hooks, DTO/ViewModel mappers, and migrated App-level Jobs/Datasets reads through `WebUiClient`. API mode does not fall back to seed data. | Partially resolved. Agents, Environments, Leaderboard, System and detail-side resources still use fixture state. |
| F3 | Replaced MSW's unrelated legacy-shaped mocks with `/api/webui/v1` Jobs/Datasets routes backed by the mock client; added route/envelope tests and an API-unavailable App story. | Partially resolved. The remaining resources need contract clients, handlers and client-backed state stories. |
| R1 | Standardized the planned resource path as `GET /leaderboard`. | Resolved in authority documentation. |
| R2 | Added runtime `ApiResponse` envelope validation plus malformed-response and transport-failure tests. | Resolved for the current HTTP client surface. |
| R3 | Marked historical `/api/*` entries as current legacy-backend evidence and named their Stage 3 replacement. | Resolved in the v1.0.5 coverage checklist. |
| F1 | No frontend-only fix is valid: the backend still does not serve `/api/webui/v1`. | Open; Stage 3 scope. |

The report remains open. A fresh independent closure review is required only after the remaining Stage 2 read-resource migration and the Stage 3 backend route upgrade are complete.

## Round 2: Residue and Migration-Completeness Re-review

### Review Input

#### Objective

Falsify the current claim that the Stage 2 remediation removed old API bypasses for the migrated resources and accurately records the remaining migration work.

#### Review Target

Stage 2 frontend contract implementation and the v1.0.5 planning/contract documentation.

#### Target Locations

- `frontend/src/api/`
- `frontend/src/app/App.tsx`
- `frontend/src/mocks/`
- `frontend/src/mocks/mswHandlers.ts`
- `frontend/src/app/App.stories.tsx`
- `frontend/src/screens/`
- `docs/releases/v1.0.5/`
- `docs/architecture/frontend-api-contract.md`
- `docs/architecture/frontend-webui-governance.md`
- `ornnlab/api/`

#### Change Introduction

Round 1 remediation added runtime API/mock modes, envelope validation, generic resource hooks, Jobs/Datasets client migration, MSW handler tests and an API-unavailable story. The implementation intentionally remains mock-first and documents un-migrated resources as pending.

#### Risk Focus

- Old `/api/*` route names or response shapes still used as an active frontend path.
- Direct mock imports or mock fallback paths that bypass the WebUI client in production runtime.
- Contract routes, WebUiClient methods, MSW handlers and stories drifting from one another.
- Documentation claiming completed migration where code is fixture-backed, or wrongly presenting legacy backend routes as the target contract.
- Unverified API-mode failure/loading behavior and any test evidence that could run against a stale server.

#### Reviewer Instructions

Use a fresh context, read targets directly, do not modify files, and return only high-signal findings with severity, concrete evidence paths/lines, and the condition required to close each finding. Explicitly distinguish Stage 2 work from the Stage 3 backend route upgrade.

### Reviewer Selection

- Role: `code-reviewer`
- Rationale: cross-cutting migration completeness, contract consistency, tests and documentation need one adversarial implementation review.
- Freshness policy: `fork_context=false`; reviewer receives only this navigation packet.

### Reviewer Launch Record

- Reviewer role: `code-reviewer`
- Mechanism: `multi_agent_v1__spawn_agent`
- Agent id: `019f4864-2e1b-7a63-ab57-18a6a5de28a3`
- Nickname: Herschel
- Context policy: fresh session, `fork_context=false`
- Review result: completed

### Reviewer Output

Verdict: **request changes**. The reviewer verified that active HTTP traffic is confined to `/api/webui/v1`; no React path calls the old `/api/experiments`, `/api/runs`, `/api/benchmarks`, `/api/agents`, `/api/leaderboard` or `/api/system` routes. It also confirmed that the backend replacement remains Stage 3 work.

Findings:

1. **Blocking**: Job/Dataset details still receive `events`、`trialRows`、`taskRows` from direct `mocks/` imports in `App.tsx`, despite those subresources being part of the Stage 2 contract surface. The reviewer cited `frontend/src/app/App.tsx:8,269,325`, `frontend/src/screens/DatasetsPage.tsx:63`, `frontend/src/screens/JobsPage.tsx:53`, and `docs/architecture/frontend-api-contract.md:442,471`.
2. **Medium**: MSW exposes unmigrated Agents、Environments、Leaderboard、System paths using raw UI fixture shapes, without matching `WebUiClient` methods or contract DTOs. The reviewer cited `frontend/src/mocks/mswHandlers.ts:39`, `frontend/.storybook/preview.ts:55`, `frontend/src/domain/harbor.ts:166,240`, and `docs/architecture/frontend-api-contract.md:104`.
3. **Low**: Summary-level documentation still has stale wording that says pages have not migrated to resource hooks, even though Jobs/Datasets lists have. The reviewer cited `docs/architecture/frontend-api-contract.md:6`, `docs/releases/v1.0.5/engineering-plan.md:19`, and `frontend/src/app/App.tsx:79-80`.

### Main Agent Response

| Finding | Decision | Rationale | Required follow-up |
|---|---|---|---|
| Blocking detail-flow fixture bypass | accept | The list-level migration does not satisfy Stage 2 while Job events/trials and Dataset tasks bypass the same client boundary. | Add contract DTO/client methods/hooks for these subresources; wire live App detail paths through them; remove direct fixture imports; then run a fresh re-review. |
| Medium unmigrated MSW contract drift | accept | A `/api/webui/v1` handler that returns a raw UI fixture misrepresents contract coverage. | Either remove those handlers until their resources migrate, or add matching contract DTO/client/hook ownership. The next Stage 2 implementation batch should choose one path explicitly. |
| Low stale summary wording | accept | The current wording understates completed Jobs/Datasets work and obscures the actual remaining scope. | Update summary docs as part of the remediation batch. |

Because a blocking finding was accepted, Round 3 is required as a fresh independent scope validation before implementation begins.

## Round 3: Closure-Scope Validation

### Reviewer Launch Record

- Reviewer role: `code-reviewer`
- Mechanism: `multi_agent_v1__spawn_agent`
- Agent id: `019f4869-87b4-72d0-b46c-55a2be3a0429`
- Nickname: Fermat
- Context policy: fresh session, `fork_context=false`
- Review result: completed

### Reviewer Output

Verdict: **request changes**. The reviewer confirmed the old backend route replacement remains Stage 3 and found no active React consumers of `/api/experiments`、`/api/runs` or `/api/benchmarks`.

It independently confirmed the previously accepted Job event/trial and Dataset task fixture bypass. It also found that `mswHandlers.ts` advertises details plus Agents、Environments、Leaderboard and System HTTP routes without a corresponding contract DTO/client/hook boundary. Finally, it found mock write flows in the live App: New Job creation, leaderboard mutations, Dataset local actions, Agent/Environment mutations and System toast-only actions.

### Main Agent Response

| Finding | Decision | Rationale | Required follow-up |
|---|---|---|---|
| Detail subresource fixture bypass | accept | This independently confirms the Round 2 blocking finding. Jobs/Datasets cannot be called migrated while their detail data bypasses the contract. | Stage 2: add DTOs, client methods and hooks for events, trials and Dataset tasks; remove their direct fixture imports from the live App. |
| Unmigrated MSW HTTP surface | accept | HTTP mocks must prove a contract boundary, not expose raw UI fixture types under the future product URL. | Stage 2: remove unowned resource routes from MSW, or implement matching DTO/client/hook boundaries before retaining them. |
| Mock write flows active in the runtime App | defer to Stage 2 write-boundary design and Stage 5 implementation | The user has explicitly kept the frontend in mock mode, so local interaction simulation remains permitted for design work. However, it must not silently remain available as a fake success path in `api` mode. | Stage 2: introduce an explicit write-action boundary that distinguishes mock simulation from API mode. Stage 5: implement operation-backed writes. |
| Top-level progress wording drift | accept | The summary is internally inconsistent with the completed Jobs/Datasets list migration. | Stage 2 documentation remediation. |

### Closure Status

- Blocking findings found: yes.
- Blocking findings accepted: yes.
- Fresh scope-validation re-review completed: yes, Round 3.
- Blocking findings fixed: no.
- Stage 2 closure: **not eligible**. The report remains open until the accepted read-resource and MSW-boundary remediation lands and passes another fresh implementation re-review.
- Stage 3 dependency: backend `/api/webui/v1` implementation remains open and is not a reason to represent Stage 2 as completed.

## Round 4: Detail Resource, MSW and API-Mode Write Remediation

### Implementation Scope

- Added typed `JobEventDto` and `TrialDto`, corresponding `WebUiClient` methods and resource hooks.
- Migrated Job/Dataset detail resources, Job events, Job trials, Dataset tasks and New Job task selection to client/hook reads.
- Removed unowned Agents、Environments、Leaderboard and System routes from MSW; only Jobs/Datasets contract routes remain.
- Made `api` mode a strict read boundary: it does not seed un-migrated resources and it disables mock-only writes.
- Updated authority documentation to record the exact partial migration state.

### Reviewer Launch Record

- Reviewer role: `code-reviewer`
- Mechanism: `multi_agent_v1__spawn_agent`
- Agent id: `019f4887-bfd5-7962-a8f2-7eeea3844315`
- Nickname: Gibbs
- Context policy: fresh session, `fork_context=false`
- Review result: request changes

### Reviewer Output and Main Agent Response

| Finding | Decision | Remediation |
|---|---|---|
| `api` mode still let a user open the Dataset delete confirmation dialog; the handler returned only after the dialog appeared. | accept | Disabled Dataset table `Download`、`Cancel download` and `Delete` actions when `allowMockWrites=false`; retained handler guards; added API-mode assertions that every rendered Dataset delete action is disabled. |
| `listJobEvents(id)` ignored `id`, so every Job received the same fixture event stream. | accept | Added `jobId` ownership to mock event fixtures; `listJobEvents(id)` now filters by it; extended client and MSW tests to compare two Job IDs. |

### Verification

- Targeted regression: `npm run test -- src/app/App.api.test.tsx src/api/detailResources.test.ts src/mocks/mswHandlers.test.ts` — 8 passed.
- Full unit suite: `npm run test -- --run` — 11 files, 30 passed.
- Static gates: `npm run typecheck`, `npm run lint` — passed.
- Production build: `npm run build` — passed.
- Storybook smoke: `npm run storybook:test` — passed.
- End-to-end regression: `npm run e2e` — 10 passed. The port `4174` preflight found no stale preview listener.

## Round 5: Fresh Remediation Re-review

### Reviewer Launch Record

- Reviewer role: `code-reviewer`
- Mechanism: `multi_agent_v1__spawn_agent`
- Agent id: `019f4890-0912-78f0-90b9-bef0508351d9`
- Nickname: Heisenberg
- Context policy: fresh session, `fork_context=false`
- Review result: approve

### Reviewer Output

The reviewer found **no blocking issue** in the repaired Dataset write path or Job event path. It confirmed that Dataset table and drawer write controls receive `allowMockWrites`/`writeDisabled`, the callbacks retain defensive guards, and Job event fixtures are filtered by `jobId` through both mock client and MSW. The reviewer also confirmed the two regression suites cover the corrected paths.

### Closure Status

- Round 2/3 accepted detail-resource fixture bypass: **resolved for Jobs/Datasets**.
- Round 2/3 accepted unowned MSW surface: **resolved**; un-migrated resources no longer masquerade as WebUI HTTP endpoints.
- Round 4 API-mode Dataset write escape: **resolved**.
- Round 4 Job-event cross-Job fixture leakage: **resolved**.
- Fresh implementation re-review: **passed**.
- Stage 2 overall: **still in progress**, not closed. Agents、Environments、Leaderboard、System read resources remain fixture-backed and lack typed client/hook ownership. Operation-backed writes, permission states and their Storybook status matrices are still pending.
- Stage 3 remains required to implement the actual backend `/api/webui/v1` routes. No document may describe the product as real-API integrated before that work and a backend contract review complete.

## Round 5 Conclusion (Historical, Superseded)

在 Round 5 结束时，已接受的 Job/Dataset detail、MSW 边界和 API-mode mock-write 发现项均已关闭；当时 Agents、Environments、Leaderboard 和 System 尚未迁移，因此 Stage 2 仍为 partial。该历史结论已由 Round 6 的完整闭环审查取代。

## Round 6: Stage 2 Closure Review

### Closure Scope

- Jobs、Datasets、Agents、Environments、Leaderboard、System 与 Header Hub 状态均已迁移到 DTO、client、hook、ViewModel 边界。
- 生产 `app`、`screens`、`ui/components` 不直接导入 fixture 或调用旧路由；mock 只经 `WebUiClient`、MSW、Storybook 和测试使用。
- 所有可见写操作返回并轮询 `Operation`；HTTP、mock 与 MSW 同时支持 `GET /operations/{id}` 和 `POST /operations/{id}/cancel`。
- API 模式请求失败只呈现错误状态，不回退 mock 成功。

### Independent Reviewers

| Reviewer | Agent ID | Scope | Result |
|---|---|---|---|
| `code-reviewer`（Pascal） | `019f4904-9cdf-7c13-b0d9-e71726279424` | Operation 轮询、Dataset 详情刷新、排行榜专用 Dataset、Operation 取消、写后资源刷新与 fixture ID 一致性 | Approve |
| `verifier`（Boyle） | `019f4904-9c53-7a60-b297-c5ea7a979cfc` | Hub 状态语义、Storybook loading/error 覆盖、权威契约与当前 UI 路由一致性 | Pass |

### Findings And Resolution

| Finding | Resolution | Regression Evidence |
|---|---|---|
| 瞬时轮询失败会清空进行中的 Operation | 保留最后有效 Operation，按失败次数指数退避并继续轮询 | `hooks.test.tsx` |
| Dataset 抽屉使用过期下载状态 | 抽屉动作从刷新后的详情 DTO 推导 | `App.test.tsx` |
| 排行榜选择器使用通用 Dataset 列表 | 只读取 `/leaderboard/datasets`，写后刷新该资源 | `App.api.test.tsx` |
| 通用 Operation 取消接口缺失 | 补齐 HTTP、mock、MSW 和测试 | `operationWrites.test.ts`、`mswHandlers.test.ts` |
| 排行榜 mock 选择项与写后状态不一致 | 每次读取都从当前排名数据派生；所有排行榜 Job ID 均有对应 Job 详情 | `operationWrites.test.ts` |
| Header Hub 连接状态硬编码 | 读取 `/system/hub-connection`；读取失败显示 UI 状态 `unavailable` | `App.api.test.tsx`、`App.stories.tsx` |

### Final Gates

- `npm run typecheck` — passed.
- `npm test` — 14 files, 45 tests passed.
- `npm run lint` — passed.
- `npm run build` — passed.
- `npm run storybook:test` — passed.
- `lsof -nP -iTCP:4174 -sTCP:LISTEN` — no stale listener.
- `npm run e2e` — 10 desktop/mobile tests passed.

### Final Status

Stage 2：**Done**。本结论只表示前端契约层和离线 mock 治理完成，不表示真实 Harbor/OrnnLab 后端已经接入。Stage 3 必须直接升级旧后端路由到 `/api/webui/v1`，不得恢复旧产品接口、legacy adapter 或 API→mock 成功 fallback。
