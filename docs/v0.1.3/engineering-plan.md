# Engineering Plan: OrnnLab v0.1.3

## Document Control

| Document Version | Engineering Version(s) | Updated | Change |
|---|---|---|---|
| 1.0 | OrnnLab Build Set `2026.06.16`; `ornnlab` npm `0.1.3`; Python app `0.2.0` | 2026-06-16 | Created the actual implementation and validation plan for the `v0.1.3` version bundle. |

- Source PRD: `version-prd.md`
- Technical design: `technical-design.md`
- Release ledger: `release-ledger.md`
- Status: Prepared, not yet published.

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

- Create `docs/v0.1.3/`.
- Move current release thinking into version-scoped PRD, technical design,
  engineering plan, and release ledger.
- Stop treating a total PRD as the active product authority.
- Update validation to check the version folder contract.

Acceptance:

- `docs/index/README.md` points to `docs/v0.1.3/README.md` as the active version
  documentation entrypoint.
- `docs/current/version-governance.md` defines the version-folder contract.
- `scripts/verify-ornnlab-rebrand.py` checks the required version documents.

## Validation Commands

```bash
uv run python scripts/verify-ornnlab-rebrand.py
uv run python -m py_compile scripts/verify-ornnlab-rebrand.py
git diff --check
npm run smoke:npm-bin
```

## Remaining Work

- Implement `ornnlab update`.
- Implement `ornnlab uninstall`.
- Add platform-specific lightweight Docker installation paths after confirming
  supported operating-system behavior.

