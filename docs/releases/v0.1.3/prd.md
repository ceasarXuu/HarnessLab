# Version PRD: OrnnLab v0.1.3

- App/Product Version: `v0.1.3`
- PRD Document Version: `1.0`
- Status: Ready for implementation
- Created: 2026-06-16
- Updated: 2026-06-23
- Owner / Requester: project maintainer
- Technical Design: `docs/releases/v0.1.3/technical-design.md`
- Engineering Plan: `docs/releases/v0.1.3/engineering-plan.md`
- Version Source: `package.json`

## Document Control

| Document Version | Engineering Version(s) | Updated | Change |
|---|---|---|---|
| 1.0 | OrnnLab Build Set `2026.06.16`; `ornnlab` npm `0.1.3`; Python app `0.2.0` | 2026-06-16 | Established version-scoped PRD for the npm bootstrap and documentation-governance release. |
| 1.1 | OrnnLab Build Set `2026.06.16`; `ornnlab` npm `0.1.3`; Python app `0.2.0` | 2026-06-23 | Merged version trio into `docs/releases/v0.1.3/` to consolidate all version documents and avoid documentation forking. |

## PRD Document Version History

| Document Version | Updated | Change |
|---|---|---|
| 1.0 | 2026-06-16 | Created `v0.1.3` version PRD. |
| 1.1 | 2026-06-23 | Relocated PRD into `docs/releases/v0.1.3/` as part of version-folder consolidation. |

This version PRD is the source of truth for `v0.1.3` product scope, version
goal, completion definition, rules, and acceptance criteria. The technical
design and engineering plan must derive from this PRD and must not redefine the
product scope.

## 1. Product Intent

OrnnLab `v0.1.3` should make the public npm launcher credible for a new user on
a mostly blank computer. The first usable command surface is the npm package
`ornnlab`, with setup work concentrated in explicit lifecycle commands rather
than hidden inside every invocation.

The version also introduces the documentation model for future releases: each
product version owns its own PRD, technical design, and engineering plan inside
one version folder.

## 2. Goals

- Provide a clear `ornnlab install` path that installs or verifies required
  runtime dependencies for the local WebUI.
- Keep Docker optional and light: detect existing Docker capability, ask before
  installing a core Docker runtime, allow skip, and do not install Docker
  Desktop.
- Make update and uninstall flows first-class planned commands:
  `ornnlab update` and `ornnlab uninstall`.
- Record npm publish operations in the playbook as a user-login plus local
  WebAuthn security-key flow, not a TOTP or access-token flow.
- Replace the active total-PRD model with version-scoped documents under
  `docs/releases/v0.1.3/`.

## 3. Users And Usage Context

- Primary user: a developer or evaluator installing OrnnLab from npm.
- Assumed machine state: Node/npm may exist because npm is the install surface;
  other tools such as `uv`, Harbor, frontend dependencies, and Docker may be
  missing.
- Desired first-run outcome: the user can run setup, understand what will be
  installed, and avoid heavyweight optional dependencies.

## 4. Scope

In scope:

- npm launcher bootstrap policy and user-visible install/update/uninstall
  command plan.
- Backend dependency readiness through `uv sync`.
- Harbor import/readiness verification.
- Frontend dependency readiness through `npm ci` and build checks.
- Docker detection, optional install prompt, skip state, and later recovery.
- Version document folder governance.

Out of scope:

- Installing Docker Desktop.
- Publishing private frontend packages independently.
- Merging all historical PRDs into one maintained total PRD.
- Removing historical documents that still provide audit context.

## 5. User Journey

1. User installs the launcher with `npm install -g ornnlab`.
2. User runs `ornnlab install`.
3. The launcher prints the concrete commands it will run before execution.
4. The launcher installs or verifies required core dependencies.
5. If Docker is missing, the launcher explains why Docker is optional and asks
   whether to install a lightweight core runtime or skip.
6. The launcher records bootstrap state so future commands can explain what is
   ready, skipped, or needs repair.
7. User later runs `ornnlab update` to update the launcher and managed
   dependencies.
8. User later runs `ornnlab uninstall` to remove launcher-managed state through
   recoverable cleanup, not irreversible deletion.

## 6. Acceptance Criteria

- `ornnlab install` has an explicit design for `uv`, Harbor, frontend
  dependencies, and optional Docker handling.
- Docker Desktop is excluded from the default install path.
- `ornnlab update` and `ornnlab uninstall` are documented as planned lifecycle
  commands.
- npm publishing instructions state that the maintainer must log in and use the
  local machine WebAuthn key.
- Active version documents live under `docs/releases/v0.1.3/` as the exact
  three-file package `prd.md`, `technical-design.md`, and `engineering-plan.md`
  alongside the release ledger and governance documents.
- No new total PRD is introduced for the product as a whole.

## 7. Open Risks

- Fully blank computers without Node/npm still need an external npm bootstrap
  path; this release treats npm as the distribution prerequisite.
- Lightweight Docker runtime installation varies by operating system and must
  remain transparent and skippable.
- `ornnlab update` and `ornnlab uninstall` still need implementation detail in
  follow-up engineering work.
