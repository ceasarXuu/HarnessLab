# Subagent VS Review: npm package reservation

- Created: 2026-06-03T03:40:10+0800
- Updated: 2026-06-03T03:48:00+0800
- Report schema: adversarial-v1
- Task: Reserve the public `harnesslab` npm package name and `harnesslab` CLI command.
- Report path: `vs_review/2026-06-03-npm-package-reservation-review.md`
- Review mode: fresh internal subagents
- Source session policy: no inherited main-agent context
- Status: open

## Round 1: reservation implementation and release readiness

### Review Input

#### Objective
Reserve both the npm package name `harnesslab` and CLI command name `harnesslab`
with a minimal public package that avoids publishing the full Rust workspace.

#### Review Target
Implementation, release process, documentation, security, and validation for
the npm reservation package.

#### Target Locations
- `package.json`
- `bin/harnesslab.js`
- `README.md`
- `docs/development-operations.md`
- `docs/playbooks/npm-package-reservation.md`
- `.gitignore`
- command: `npm run smoke:npm-bin`
- command: `npm pack --dry-run`
- command: `npm publish --access public`

#### Change Introduction
Added a root npm package named `harnesslab` at version `0.1.0`, exposed a
`harnesslab` bin shim, limited published files to README, license, package
metadata, and the shim, and documented npm reservation validation and 2FA
handling.

#### Risk Focus
- Accidental publication of repository internals, local artifacts, or secrets.
- CLI shim making false claims about production distribution status.
- npm package metadata or bin mapping failing after install.
- 2FA/token handling leaking credentials or leaving an unreproducible release path.
- Validation being too narrow to prove the package and command are really reserved.

#### Assumptions To Attack
- `files` limits the npm tarball to the intended four files.
- `bin.harnesslab` works when installed from the packed tarball.
- `.env.local` and npm token material are ignored and never committed.
- The reservation package is acceptable even though it does not ship the Rust CLI.
- Publishing can proceed only after npm 2FA is satisfied.

#### Adversarial Lenses
- release
- security
- implementation
- testing
- observability
- documentation

#### Verification Status
- `npm run smoke:npm-bin` passed.
- `npm pack --dry-run` showed four files in the tarball.
- Temporary-prefix tarball install passed using
  `node_modules/.bin/harnesslab --version` and `--help`.
- Registry check returned `404` before attempted publish.
- `npm publish --access public` failed twice with npm 2FA `E403`; publication is
  blocked pending OTP or a granular token with bypass 2FA.

#### Reviewer Instructions
- Fresh internal subagent session.
- No inherited main-agent context.
- Read target files directly.
- Do not modify files.
- Cite evidence paths and line numbers when possible.

### Reviewer Timeout Policy

| Complexity | Initial Wait | Extension | Max Attempts Per Role | Blocking Closure Behavior |
|---|---:|---:|---:|---|
| normal | 10 minutes | none | 2 | cannot pass if review is unavailable |

### Reviewer Selection

| Reviewer | Reason Selected | Risk Area |
|---|---|---|
| release-ops-adversary | npm package publication is a release/packaging operation with partial-success and 2FA states. | package contents, publish sequence, verification |
| security-adversary | Token and OTP handling are in scope, and the package must not leak secrets or repo internals. | secret handling, supply chain, install behavior |

### Reviewer Launch Records

| Reviewer | Internal Mechanism | Session / Job ID | Trace Source | Context Forked | Input Packet | Context Explicitly Excluded | Read-only |
|---|---|---|---|---|---|---|---|
| release-ops-adversary | multi_agent_v1.spawn_agent code-reviewer | 019e89da-68f1-7980-8696-b3ddd33a7e73 | spawn_agent tool call | fork_context=false | Round 1 Review Input | main-agent history, reasoning, drafts, conclusions, full diff unless read from repo | yes |
| security-adversary | multi_agent_v1.spawn_agent security-reviewer | 019e89da-a202-7d53-8b12-dda43f535db5 | spawn_agent tool call | fork_context=false | Round 1 Review Input | main-agent history, reasoning, drafts, conclusions, full diff unless read from repo | yes |

### Reviewer Timeout Records

| Reviewer Output Key | Reviewer Role | Attempt | Session / Job ID | Waited | Status | Reason | Action |
|---|---|---:|---|---:|---|---|---|
| release-ops-adversary-r1 | release-ops-adversary | 1 | 019e89da-68f1-7980-8696-b3ddd33a7e73 | <10 minutes | completed | reviewer returned output | completed |
| security-adversary-r1 | security-adversary | 1 | 019e89da-a202-7d53-8b12-dda43f535db5 | <10 minutes | completed | reviewer returned output | completed |

### Reviewer Outputs

#### release-ops-adversary-r1

##### Summary
`REQUEST CHANGES`. The package is minimized correctly and local pack/install
checks pass, but the objective is not complete until npm publish succeeds and
registry-backed verification passes.

##### Blocking Findings
- Objective not achieved: the public package name is still unreserved, so the
  registry-backed `harnesslab` command is not actually claimed yet.
  - Broken assumption: local package preparation is enough to reserve the npm name.
  - Failure scenario: another publisher claims `harnesslab` before the real publish.
  - Trigger condition: npm registry still returns `404` for `harnesslab`.
  - Impact: package and CLI name are not reserved.
  - Proof needed: successful `npm publish`, `npm view harnesslab name version bin --json`,
    and registry-backed command smoke.
- Release path is not reproducible under auth failure because the playbook does
  not define a deterministic credential preflight.
  - Broken assumption: `npm whoami` success is enough to predict publish success.
  - Failure scenario: logged-in account or token still hits 2FA `E403`.
  - Trigger condition: publish with a session or token that cannot satisfy npm 2FA.
  - Impact: future releasers repeat trial-and-error around auth.
  - Proof needed: documented auth-mode preflight and blessed publish paths.

##### Non-blocking Risks
- `--version` can drift from the package version because it was hardcoded in the
  shim.
  - Broken assumption: maintainers will always update duplicated versions.
  - Failure scenario: `package.json` is bumped but CLI prints the old version.
  - Trigger condition: future version bump.
  - Impact: published command misreports version.
  - Proof needed: shim reads version from `package.json` or a test catches drift.

##### Required Fixes
- Publish the package for real and prove registry reservation.
- Strengthen auth-mode preflight and release record guidance.
- Remove duplicated version source in the shim.

##### Missing Tests
- Post-publish registry smoke via `npx harnesslab --version` and `--help`.
- Clean-prefix install smoke from the registry package.
- Negative CLI smoke for unsupported args.

##### Missing Logs / Observability
- Durable release record capturing successful publish command, auth mode, and
  registry outputs.

##### Evidence
- `package.json:14` defines `bin.harnesslab` and `files`.
- `bin/harnesslab.js:11` discloses that this is a reservation shim.
- `docs/playbooks/npm-package-reservation.md:47` documents publish.
- Reviewer commands: `npm run smoke:npm-bin`, `node --check`, `npm pack --dry-run`,
  local tarball command smoke passed; registry checks returned `404` / `E404`.

#### security-adversary-r1

##### Summary
Risk level: `MEDIUM`. The package itself is low-complexity and mostly safe, but
the publish documentation taught unsafe token handling and `.npmrc` was not
ignored.

##### Blocking Findings
- Publish playbook teaches command-line token exposure.
  - Broken assumption: inline `NODE_AUTH_TOKEN=<token> npm publish` is safe enough
    for a playbook.
  - Failure scenario: token leaks through shell history, process inspection, or
    copied logs.
  - Trigger condition: future maintainer follows the inline token example.
  - Impact: npm token theft and package takeover.
  - Proof needed: remove inline token example and document non-echo token handling.

##### Non-blocking Risks
- `.env.local` is ignored, but `.npmrc` is not.
  - Broken assumption: all common local npm credential files are ignored.
  - Failure scenario: project-scoped `.npmrc` with `_authToken` is committed.
  - Trigger condition: maintainer switches from `.env.local` to `.npmrc`.
  - Impact: credential leakage.
  - Proof needed: `.npmrc` ignore rule and tracked-file guard.
- Broad `*.local` ignore can hide review-relevant files.
  - Broken assumption: every `*.local` file is secret-only.
  - Failure scenario: local fixture or note is hidden from review.
  - Trigger condition: future tracked-worthy file uses `.local` suffix.
  - Impact: missed review artifact.
  - Proof needed: narrower ignore patterns.
- Dependency audit is not automated for this package path.
  - Broken assumption: dependency-free state will remain obvious.
  - Failure scenario: future dependency is added without audit coverage.
  - Trigger condition: package gets dependencies.
  - Impact: supply-chain risk.
  - Proof needed: dependency/script/tarball guard.

##### Required Fixes
- Remove inline token publish example and replace it with non-history,
  non-command-line token-entry flow.
- Add explicit warning against committing or logging tokens.
- Ignore `.npmrc`, and preferably local env secret files.

##### Missing Tests
- Guard that fails if `.env.local` or `.npmrc` becomes tracked.
- Docs sanity check for unsafe token examples.
- Package surface guard for dependencies/lifecycle scripts/tarball contents.

##### Missing Logs / Observability
- Document publish evidence: exact tarball list, npm pack metadata, auth mode,
  and post-publish registry outputs.

##### Evidence
- `package.json:17` limits published files.
- `package.json:25` has no lifecycle scripts.
- `bin/harnesslab.js:19` only parses args and does not execute input.
- `.gitignore:8` ignored `*.local` before the fix.
- `docs/playbooks/npm-package-reservation.md:66` contained inline token guidance
  before the fix.

### Main Agent Response

| Reviewer | Finding | Broken Assumption / Failure Scenario | Severity | Decision | Evidence / Reason | Action Taken | Follow-up |
|---|---|---|---|---|---|---|---|
| release-ops-adversary | Package not actually published yet | Local preparation does not reserve npm name | blocking | accept | npm publish failed with 2FA `E403`; registry remained `404` | Waiting for OTP or bypass-2FA token | Final publish and registry smoke required |
| release-ops-adversary | Auth path not deterministic | `npm whoami` is not enough to predict publish success | blocking | accept | 2FA `E403` occurred after `npm whoami` succeeded | Added `npm profile get --json`, ignored secret checks, OTP-first publish path, and publish evidence requirements to playbook | Round 2 re-review |
| release-ops-adversary | Version drift | Shim duplicated `0.1.0` separately from package metadata | non-blocking | accept | `package.json` and shim both carried version data | Shim now reads `require("../package.json").version`; smoke script asserts version match | Round 2 re-review |
| security-adversary | Unsafe inline token guidance | Token can leak through shell history or process inspection | blocking | accept | Playbook contained inline `NODE_AUTH_TOKEN=<token>` publish command | Removed inline token value example; documented OTP-first path, interactive read, ignored `.env.local` loading, and explicit secret warnings | Round 2 re-review |
| security-adversary | `.npmrc` not ignored | Common npm token file could be committed | non-blocking | accept | `.gitignore` did not include `.npmrc` | Replaced broad `*.local` with `.env`, `.env.local`, `.npmrc`; smoke script checks ignore and tracked status | Round 2 re-review |
| security-adversary | Broad `*.local` ignore | Non-secret `.local` files could be hidden | non-blocking | accept | `.gitignore` used `*.local` | Narrowed ignore entries to `.env`, `.env.local`, `.npmrc` | Round 2 re-review |
| security-adversary | Missing package surface guard | Future deps or tarball files could drift | non-blocking | accept | No prior npm package guard script | Added `scripts/verify-npm-reservation-package.sh`; `npm run smoke:npm-bin` checks version, unsupported args, secret files, and dry-run tarball file list | Round 2 re-review |

### Closure Status

- Blocking findings found: yes
- Accepted blocking findings fixed: partially; all local code/docs findings fixed, real publish pending OTP
- Blocking re-review completed: pending
- Blocking re-review passed: pending
- Blocking re-review round links:
  - Round 2 pending
- Blocking re-review launch records:
  - Round 2 pending
- Rejected findings backed by evidence: n/a
- Deferred findings documented: n/a
- Blocked reason: pending OTP or bypass-2FA token for npm publish
- Allowed to proceed: pending

## Round 2: accepted local fixes closure

### Review Input

#### Objective
Verify accepted local fixes before another npm publish attempt.

#### Review Target
Security, release documentation, CLI shim, and npm package validation script.

#### Target Locations
- `.gitignore`
- `package.json`
- `bin/harnesslab.js`
- `scripts/verify-npm-reservation-package.sh`
- `docs/playbooks/npm-package-reservation.md`
- `README.md`

#### Change Introduction
Removed unsafe inline token guidance, added explicit secret ignore rules,
changed the shim to read version from package metadata, and added a smoke script
that checks CLI behavior, secret-file tracking, and npm dry-run tarball contents.

#### Risk Focus
- Secret leakage through docs, scripts, `.gitignore`, and npm auth flow.
- Package contents accidentally expanding.
- CLI shim or tests becoming misleading or unsafe.
- Fixed docs still teaching unsafe token handling.

#### Assumptions To Attack
- `.env.local` and `.npmrc` are ignored and not tracked.
- The playbook no longer teaches command-line token exposure.
- The shim cannot drift from `package.json` version.
- The smoke script catches package surface drift.

#### Adversarial Lenses
- security
- release
- testing
- documentation

#### Verification Status
- `npm run smoke:npm-bin` passed after fixes.
- `git check-ignore -v .env.local .npmrc` matched `.gitignore`.
- `git ls-files --error-unmatch .env.local` and `.npmrc` returned nonzero.
- Temporary-prefix tarball install passed after fixes.
- Actual npm publish remains pending OTP.

#### Reviewer Instructions
- Fresh internal subagent session.
- No inherited main-agent context.
- Read target files directly.
- Do not modify files.
- Cite evidence paths and line numbers when possible.
- Do not treat missing registry publish as part of this closure review; publish
  completion will be reviewed after OTP/publish succeeds.

### Reviewer Timeout Policy

| Complexity | Initial Wait | Extension | Max Attempts Per Role | Blocking Closure Behavior |
|---|---:|---:|---:|---|
| normal | 10 minutes | none | 2 | cannot pass if review is unavailable |

### Reviewer Selection

| Reviewer | Reason Selected | Risk Area |
|---|---|---|
| security-adversary | Accepted blocking finding involved npm token handling. | secret handling, supply chain, docs |

### Reviewer Launch Records

| Reviewer | Internal Mechanism | Session / Job ID | Trace Source | Context Forked | Input Packet | Context Explicitly Excluded | Read-only |
|---|---|---|---|---|---|---|---|
| security-adversary | multi_agent_v1.spawn_agent security-reviewer | 019e89e0-e1e3-71a0-b431-cd99f1c5d2b6 | spawn_agent tool call | fork_context=false | Round 2 Review Input | main-agent history, reasoning, drafts, conclusions, full diff unless read from repo | yes |

### Reviewer Timeout Records

| Reviewer Output Key | Reviewer Role | Attempt | Session / Job ID | Waited | Status | Reason | Action |
|---|---|---:|---|---:|---|---|---|
| security-adversary-r2 | security-adversary | 1 | 019e89e0-e1e3-71a0-b431-cd99f1c5d2b6 | <10 minutes | completed | reviewer returned output | completed |

### Reviewer Outputs

#### security-adversary-r2

##### Summary
Risk level for the accepted fix set is `LOW to MEDIUM`. No new blocking issue
should stop the next OTP-gated publish attempt. The unsafe literal token command
is gone, `.gitignore` covers `.env`, `.env.local`, and `.npmrc`, the shim reads
version from `package.json`, and `npm run smoke:npm-bin` is wired to the
verification script.

##### Blocking Findings
- none confirmed.

##### Non-blocking Risks
- The playbook still normalizes storing an npm publish token in `.env.local` and
  sourcing that file.
  - Broken assumption: every local token file load is low risk.
  - Failure scenario: broad-lived token remains on disk longer than intended, or
    `source .env.local` executes unexpected shell content.
  - Trigger condition: future maintainer uses the token fallback rather than OTP.
  - Impact: elevated credential exposure compared with OTP.
  - Proof needed: OTP-first wording, last-resort wording for `.env.local`, and
    no committed token values.
- The smoke script verifies `.env.local` and `.npmrc`, but initially not `.env`.
  - Broken assumption: all ignored secret files are covered by regression checks.
  - Failure scenario: `.env` is tracked despite being an ignored secret file.
  - Trigger condition: future maintainer adds `.env`.
  - Impact: credential leakage.
  - Proof needed: include `.env` in ignore/tracked assertions.
- No lockfile means `npm audit` cannot run.
  - Broken assumption: dependency-free package has no audit need.
  - Failure scenario: future dependency drift has no audit artifact.
  - Trigger condition: dependencies are added later.
  - Impact: supply-chain risk.
  - Proof needed: dependency-free state and package surface guard.

##### Required Fixes
- none required for closure on the stated accepted fixes.

##### Missing Tests
- Add `.env` to smoke script ignore/untracked assertions.
- Add explicit tarball assertion against dotfiles/auth material/docs, even though
  pack-list equality already covers this.

##### Missing Logs / Observability
- Document sanitized artifact capture for pre-publish and post-publish checks.

##### Evidence
- `.gitignore:8`-`.gitignore:10` ignore `.env`, `.env.local`, and `.npmrc`.
- `bin/harnesslab.js:3` reads package metadata version.
- `package.json:25` wires `npm run smoke:npm-bin` to the verification script.
- `scripts/verify-npm-reservation-package.sh` checks syntax, version, help,
  unsupported args, secret ignore/tracking, and pack list.
- `docs/playbooks/npm-package-reservation.md:58` documents OTP-first publish.
- Reviewer independently ran `npm run smoke:npm-bin` and `npm pack --dry-run --json`.

### Main Agent Response

| Reviewer | Finding | Broken Assumption / Failure Scenario | Severity | Decision | Evidence / Reason | Action Taken | Follow-up |
|---|---|---|---|---|---|---|---|
| security-adversary | No blocking findings | Accepted security fixes are sufficient for next OTP-gated publish attempt | n/a | accept | Reviewer reported no blocking findings | Proceed toward publish once OTP is available | Final publish verification |
| security-adversary | `.env.local` token fallback remains broader risk than OTP | Token file can remain on disk or source unexpected shell | non-blocking | accept | OTP-first flow is preferred; token fallback exists for current user-provided token path | Reworded `.env.local` as last resort and preserved no-print loading; no token values committed | None |
| security-adversary | `.env` missing from smoke assertions | New ignore entry lacked matching regression check | non-blocking | accept | `.gitignore` includes `.env` | Added `.env` to smoke script ignore/tracked loop | None |
| security-adversary | Tarball guard intent could be more explicit | Pack-list equality covers this but intent is implicit | non-blocking | accept | Reviewer requested explicit forbidden-file check | Added explicit forbidden file checks for `.env`, `.env.local`, `.npmrc`, and `docs/` | None |
| security-adversary | Missing sanitized evidence capture | Release forensics weaker without preserved non-secret outputs | non-blocking | accept | Playbook listed commands but not evidence capture | Added sanitized evidence capture commands and warning not to store credential material | None |

### Closure Status

- Blocking findings found: no
- Accepted blocking findings fixed: yes for local code/docs; real publish still pending OTP
- Blocking re-review completed: yes
- Blocking re-review passed: yes
- Blocking re-review round links:
  - Round 2
- Blocking re-review launch records:
  - Round 2 Reviewer Launch Records
- Rejected findings backed by evidence: n/a
- Deferred findings documented: n/a
- Blocked reason: pending OTP or bypass-2FA token for npm publish
- Allowed to proceed: yes for next publish attempt; final closure requires npm publish

## Final Conclusion

Local package, docs, and review fixes are ready for the next npm publish
attempt. Final task closure remains blocked until npm 2FA is satisfied and
registry-backed post-publish verification passes.
