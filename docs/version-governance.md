# OrnnLab Version And Documentation Governance

## Document Control

| Document Version | Engineering Version(s) | Updated | Change |
|---|---|---|---|
| 1.0 | `ornnlab` npm `0.1.3`; Python app `0.2.0` | 2026-06-16 | Defined version authorities, release ledger, and document/version drift rules. |
| 1.1 | `ornnlab` npm `0.1.3`; Python app `0.2.0` | 2026-06-16 | Added required document version tables and engineering-version linkage for active PRD and technical docs. |
| 1.2 | `ornnlab` npm `0.1.3`; Python app `0.2.0` | 2026-06-16 | Expanded document-control coverage to operations, test strategy, and Harbor spike docs. |

This document defines how engineering versions, npm package versions, and
documentation references stay aligned.

## Problem

OrnnLab currently has several valid version surfaces:

- npm launcher: `package.json`
- Python backend/application package: `pyproject.toml`
- private frontend package: `frontend/package.json`
- scoped transition npm package: `npm/harnesslab-transition/package.json`
- bootstrap state schema: `~/.ornnlab/launcher/bootstrap-state.json`
- external runtime dependency constraints, especially Harbor in `pyproject.toml`

The project previously allowed user-facing docs to mention concrete versions
directly. That made drift easy: one file could move to `ornnlab@0.1.3` while
another still said `ornnlab@0.1.2`.

## Goals

- Make one file authoritative for each versioned artifact.
- Keep user docs stable by avoiding unnecessary concrete version strings.
- Require release notes or a release ledger when a version is changed.
- Make version/document drift detectable by local gates.
- Keep historical docs readable without treating old versions as current truth.

## Version Authority Map

| Versioned thing | Authority | Consumers | Notes |
|---|---|---|---|
| Public npm launcher version | `package.json` `version` | `ornnlab --version`, npm publish, npm smoke | This is the public install/update surface. |
| Python app/backend version | `pyproject.toml` `project.version` | `uv run ornnlab --version`, app status, backend diagnostics | This can differ from npm launcher version. |
| Frontend package version | `frontend/package.json` `version` | frontend build metadata if needed | Private implementation version; do not mention in user docs unless needed. |
| Scoped transition package version | `npm/harnesslab-transition/package.json` `version` | old `@ceasarxuu/harnesslab` compatibility package | Only changes for transition/deprecation releases. |
| Bootstrap state schema | launcher source constant and persisted `schemaVersion` | `~/.ornnlab/launcher/bootstrap-state.json`, bootstrap migrations | Schema version is not a product release version. |
| Harbor dependency range | `pyproject.toml` dependency constraint | backend install, doctor, Harbor upgrade procedure | Upgrade through `docs/harbor-upgrade-procedure.md`. |

## Documentation Rules

- User-facing installation docs should prefer `latest` or unversioned commands:
  ```bash
  npm install -g ornnlab
  npm install -g ornnlab@latest
  ```
- Do not write concrete npm versions such as `ornnlab@0.1.3` in quickstarts or
  README unless the sentence is explicitly about release history.
- PRDs may mention planned versions only in a "Release Intent" or "History"
  section. They must not imply that a future version is already live.
- Playbooks may record historical observed versions, but must label them as
  historical or prepared.
- Active docs must link to authoritative files rather than duplicating version
  facts when possible.
- Archived docs may retain old version references; active gates should ignore
  `docs/archive/`.

## Document Version Tables

Every active PRD, technical design, release, bootstrap, install, and packaging
document must keep a `Document Control` section immediately below the top-level
title.

Required table:

```markdown
## Document Control

| Document Version | Engineering Version(s) | Updated | Change |
|---|---|---|---|
| 1.0 | `ornnlab` npm `0.1.3`; Python app `0.2.0` | 2026-06-16 | Initial governed version. |
```

Rules:

- `Document Version` tracks document content, not package release versions.
- Increment minor document versions for clarifications, policy additions, or
  scope changes.
- Increment major document versions when the document changes direction or
  invalidates previous acceptance criteria.
- `Engineering Version(s)` must name the affected artifact versions or say
  `policy only` when no artifact version is affected.
- `Updated` must be an exact date.
- `Change` must summarize the reason a reviewer should care.
- Historical release facts remain in `docs/releases/`; active PRDs and technical
  docs should link to release ledgers rather than duplicating long histories.

## Release Ledger

Create or update `docs/releases/` for each public release that changes a
published artifact.

Recommended file name:

```text
docs/releases/YYYY-MM-DD-ornnlab-<version>.md
```

Minimum content:

- artifact name and version
- commit SHA
- changed user-visible behavior
- docs updated
- local validation commands
- publish command and registry verification
- rollback notes

For multi-artifact releases, record every artifact:

```markdown
| Artifact | Version | Authority | Published? |
|---|---:|---|---|
| `ornnlab` npm launcher | `0.1.3` | `package.json` | yes/no |
| Python app package | `0.2.0` | `pyproject.toml` | yes/no |
```

## Required Change Flow

When changing a version authority file:

1. Update the authority file.
2. Update or create the release ledger entry.
3. Update only docs that describe behavior affected by the version.
4. Avoid scattering the literal version into README/quickstart.
5. Run version/document guards before commit.
6. Publish only after the commit containing version and docs is merged/pushed.
7. After publishing, update the release ledger with registry verification.

## Guard Requirements

Local gates should detect:

- active docs containing stale literal `ornnlab@<version>` values outside
  approved historical/release-ledger contexts.
- README or quickstart claiming a prepared version is live before registry proof.
- npm launcher package version mismatching `ornnlab --version`.
- Python package version mismatching `uv run ornnlab --version`.
- package tarball contents drifting from the intended npm surface.
- required active PRD and technical docs missing a top-level `Document Control`
  table.

Recommended commands:

```bash
npm run smoke:npm-bin
uv run ornnlab --version
uv run python scripts/verify-ornnlab-rebrand.py
git diff --check
```

Future dedicated guard:

```bash
uv run python scripts/verify-version-governance.py
```

The current `scripts/verify-ornnlab-rebrand.py` already checks the required
`Document Control` table presence. A future dedicated guard should additionally
parse authoritative version files and scan active docs for unapproved literal
version references.

## Active Documentation Index

The active version-governed documents are:

- `README.md`
- `docs/version-governance.md`
- `docs/releases/*.md`
- `docs/install-quickstart.md`
- `docs/release-checklist.md`
- `docs/development-operations.md`
- `docs/technology-decisions.md`
- `docs/harbor-upgrade-procedure.md`
- `docs/test-engineering.md`
- `docs/spikes/2026-06-15-harbor-lifecycle-spike.md`
- `docs/playbooks/npm-package-reservation.md`
- `docs/plans/2026-06-15-harbor-webui-redesign-engineering-plan.md`
- `prd/2026-06-15-ornnlab-webui-prd.md`
- `prd/2026-06-15-ornnlab-npm-distribution.md`
- `prd/2026-06-16-ornnlab-zero-friction-bootstrap.md`

When a new active install, release, bootstrap, or packaging document is added,
it must either follow this governance policy or explicitly explain why it is
historical.

## Decisions

| Topic | Decision | Rationale |
|---|---|---|
| npm launcher version source | `package.json` only | It is what npm publishes and what `ornnlab --version` reads. |
| Python app version source | `pyproject.toml` only | It is what Python packaging and backend diagnostics read. |
| Quickstart versions | Avoid literal versions | Prevents stale install docs after release bumps. |
| Release history | Use `docs/releases/` | Keeps version facts in one reviewable ledger. |
| Historical docs | Preserve old versions under archive/release notes | Historical evidence should not be rewritten as current guidance. |
