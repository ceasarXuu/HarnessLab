# Subagent VS Review: Stage 2 Legacy API Residue

- Created: 2026-07-10T02:14:27+0800
- Updated: 2026-07-10T02:22:00+0800
- Report schema: adversarial-v1
- Task: Review Stage 2 contract-layer work, with emphasis on old API documentation and code residue.
- Report path: `vs_review/2026-07-10-stage-2-legacy-api-review.md`
- Review mode: fresh internal subagents
- Source session policy: no inherited main-agent context; approved CLI substitutes receive only the review packet
- Status: open

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

## Final Conclusion

Round 1 completed with a fresh Codex internal reviewer. The review requests changes. The work may proceed, but Stage 2 cannot be closed and `/api/webui/v1` cannot be presented as the active product path until F1, F2, and F3 are resolved and independently re-reviewed.
