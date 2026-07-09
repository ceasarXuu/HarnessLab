# Subagent VS Review: Stage 2 Legacy API Residue

- Created: 2026-07-10T02:14:27+0800
- Updated: 2026-07-10T02:14:27+0800
- Report schema: adversarial-v1
- Task: Review Stage 2 contract-layer work, with emphasis on old API documentation and code residue.
- Report path: `vs_review/2026-07-10-stage-2-legacy-api-review.md`
- Review mode: blocked_due_to_review_unavailable
- Source session policy: no inherited main-agent context; approved CLI substitutes receive only the review packet
- Status: blocked

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

- Internal subagent unavailable reason: the current runtime exposes no fresh internal subagent mechanism.
- Local CLI discovery commands:
  - `command -v claude`
  - `command -v claude-code`
  - `command -v codex`
  - `command -v codex-cli`
  - `command -v opencode`
  - `command -v pi`
- Discovered CLI candidates:
  - `/Users/xuzhang/.local/bin/claude`
  - `/Volumes/XU-1TB-NPM/global/bin/codex`
  - `/opt/homebrew/bin/opencode`
  - `/Volumes/XU-1TB-NPM/global/bin/pi`
- User-recommended alternative agent requested: no
- User-recommended agent command: n/a
- User-recommended agent verification: n/a
- User approval requested: pending
- User-approved CLI command: n/a
- User decision: pending
- Fallback outcome: blocked_due_to_review_unavailable

### Reviewer Timeout Policy

| Complexity | Initial Wait | Extension | Max Attempts Per Role | Blocking Closure Behavior |
|---|---:|---:|---:|---|
| complex | 15 minutes | 10 minutes | 2 | cannot pass if review is unavailable |

### Reviewer Selection

| Reviewer | Reason Selected | Risk Area |
|---|---|---|
| implementation-completeness-adversary | The primary risk is that Stage 2 exists only as protocol/mock scaffolding or leaves old API paths as de facto production integrations. | Legacy residue, production-path wiring, and documentation truthfulness |

### Reviewer Launch Records

No reviewer launched. Explicit user approval is required before invoking a discovered local CLI substitute.

### Reviewer Timeout Records

No reviewer attempt started.

### User Decision After Failed Review

- Decision: pending
- User-visible reason: a local CLI substitute requires explicit approval before invocation.

## Final Conclusion

The independent adversarial review has not started. Awaiting explicit approval for one discovered local CLI reviewer command.
