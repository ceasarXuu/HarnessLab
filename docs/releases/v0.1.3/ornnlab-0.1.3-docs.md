# OrnnLab v0.1.3 Document Index

## Document Control

| Document Version | Engineering Version(s) | Updated | Change |
|---|---|---|---|
| 1.0 | OrnnLab Build Set `2026.06.16`; `ornnlab` npm `0.1.3`; Python app `0.2.0` | 2026-06-16 | Created the canonical version document folder for `v0.1.3`. |

This file indexes the active documentation bundle for OrnnLab `v0.1.3`.

## Document Set

| Document | Purpose |
|---|---|
| `../v0.1.3/prd.md` | Product requirements for this version only. |
| `../v0.1.3/technical-design.md` | Technical design derived from the version PRD. |
| `../v0.1.3/engineering-plan.md` | Actual implementation and validation plan for this version. |
| `ornnlab-0.1.3.md` | Release evidence, artifact versions, publish state, and rollback notes. |

## Build Set Composition

| Component | Version / Range | Authority | Role |
|---|---:|---|---|
| npm launcher | `0.1.3` | `package.json` | Public install/update entrypoint |
| Python app/backend | `0.2.0` | `pyproject.toml` | Local API, CLI, data model, diagnostics |
| Frontend private package | `0.1.0` | `frontend/package.json` | WebUI implementation package |
| Transition npm package | `0.1.2` | `npm/harnesslab-transition/package.json` | Old scoped command migration notice |
| Harbor dependency | `>=0.13,<0.14` | `pyproject.toml` | Execution engine compatibility range |
| Source commit | `76f754f` | git | Prepared bootstrap policy implementation |

## Status

- Version status: Prepared, not yet published.
- Canonical release evidence: `ornnlab-0.1.3.md`.
- Historical release index entry: `../releases/2026-06-16-ornnlab-0.1.3.md`.
