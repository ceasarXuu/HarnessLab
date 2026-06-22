# Engineering Plan: OrnnLab v0.1.3

## Document Control

| Document Version | Engineering Version(s) | Updated | Change |
|---|---|---|---|
| 1.0 | OrnnLab Build Set `2026.06.16`; `ornnlab` npm `0.1.3`; Python app `0.2.0` | 2026-06-16 | Created the actual implementation and validation plan for the `v0.1.3` version bundle. |
| 1.1 | OrnnLab Build Set `2026.06.16`; `ornnlab` npm `0.1.3`; Python app `0.2.0` | 2026-06-21 | Updated remaining work: update and uninstall commands implemented; added version governance guard and publish script. |
| 1.2 | OrnnLab Build Set `2026.06.16`; `ornnlab` npm `0.1.3`; Python app `0.2.0` | 2026-06-23 | Relocated into `docs/releases/v0.1.3/` as part of version-folder consolidation; updated cross-references. |

- Source PRD: `prd.md`
- Technical design: `technical-design.md`
- Release ledger: `ornnlab-0.1.3.md`
- Status: Prepared, not yet published.

This engineering plan derives from `docs/releases/v0.1.3/prd.md` and
`docs/releases/v0.1.3/technical-design.md`; it must not redefine the `v0.1.3`
product scope or completion definition.

## Phase 1: Bootstrap Command Surface

Scope:

- Keep `ornnlab install` as the explicit setup command.
- Print required commands before execution.
- Install or verify `uv`, backend dependencies, Harbor importability, frontend
  dependencies, and frontend build readiness.
- Detect Docker and record present/missing/skipped/install-intended state.

Acceptance:

- `npm run smoke:npm-bin` passes.
- `uv run python scripts/verify-ornnlab-rebrand.py` passes.
- Bootstrap state records launcher and schema versions.

## Phase 2: Optional Docker Policy

Scope:

- Do not install Docker Desktop.
- If Docker is present, record capability.
- If Docker is missing, ask whether to install a lightweight core runtime or
  skip.
- If skipped, keep later repair guidance available.

Acceptance:

- Documentation states Docker is optional and lightweight.
- Release checklist blocks any Docker Desktop default install.
- Future implementation tests cover present, missing, skipped, and repair flows.

## Phase 3: Lifecycle Commands

Scope:

- Add the command plan for `ornnlab update`.
- Add the command plan for `ornnlab uninstall`.
- Preserve data and prefer recoverable cleanup for uninstall.

Acceptance:

- Update and uninstall semantics are documented in the version PRD and technical
  design.
- Follow-up implementation work has clear command ownership and safety rules.

## Phase 4: Version Documentation Governance

Scope:

- Consolidate version documents under `docs/releases/v0.1.3/`.
- Keep version-scoped PRD, technical design, and engineering plan together with
  release evidence and governance in one folder.
- Stop treating a total PRD as the active product authority.
- Update validation to check the consolidated version-folder contract.

Acceptance:

- `docs/architecture/docs-index.md` points to
  `docs/releases/v0.1.3/prd.md` as the active version PRD.
- `docs/releases/v0.1.3/version-governance.md` defines the consolidated
  version-folder contract.
- `scripts/verify-ornnlab-rebrand.py` checks the required version documents.

## Validation Commands

```bash
uv run python scripts/verify-ornnlab-rebrand.py
uv run python -m py_compile scripts/verify-ornnlab-rebrand.py
git diff --check
npm run smoke:npm-bin
```

## Remaining Work

- Add platform-specific lightweight Docker installation paths after confirming
  supported operating-system behavior.
