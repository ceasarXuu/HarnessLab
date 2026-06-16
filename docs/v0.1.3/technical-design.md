# Technical Design: OrnnLab v0.1.3

## Document Control

| Document Version | Engineering Version(s) | Updated | Change |
|---|---|---|---|
| 1.0 | OrnnLab Build Set `2026.06.16`; `ornnlab` npm `0.1.3`; Python app `0.2.0` | 2026-06-16 | Derived the `v0.1.3` bootstrap and document-governance design from the version PRD. |

- Source PRD: `prd.md`
- Implementation plan: `engineering-plan.md`
- Release ledger: `../release/ornnlab-0.1.3.md`

This technical design derives from `docs/v0.1.3/prd.md` and must not redefine
the `v0.1.3` product scope or completion definition.

## 1. Architecture Boundary

The npm launcher is the public lifecycle controller. The Python package remains
the application/backend authority. Harbor remains the execution engine.

| Layer | Responsibility |
|---|---|
| npm launcher | Install/update/uninstall orchestration, command explanation, bootstrap state |
| Python app | CLI, API server, local data model, diagnostics |
| Frontend | WebUI build and static assets |
| Harbor | Benchmark execution, raw job artifacts, container-facing execution |

## 2. Command Model

| Command | Design Status | Responsibility |
|---|---|---|
| `ornnlab install` | Prepared design | Verify/install required dependencies and record bootstrap state. |
| `ornnlab update` | Planned | Update public launcher and managed dependency set with preflight checks. |
| `ornnlab uninstall` | Planned | Remove launcher-managed artifacts through recoverable cleanup. |
| `ornnlab --version` | Existing | Report public npm launcher version. |

Setup work belongs in `ornnlab install`. Regular command invocation should read
bootstrap state and provide repair guidance instead of silently performing large
installs.

## 3. Bootstrap State

State is persisted under the OrnnLab launcher home with a schema version and
launcher version. Required state categories:

- npm launcher version
- backend readiness
- Harbor readiness
- frontend dependency/build readiness
- Docker capability and user decision
- skipped optional dependencies
- last install command evidence

The state file is diagnostic evidence, not the sole source of truth. Readiness
checks must still probe installed tools when a command depends on them.

## 4. Dependency Policy

| Dependency | Install Policy | Verification |
|---|---|---|
| `uv` | Required for backend dependency sync; install or repair during `ornnlab install`. | `uv --version`, then backend sync/import smoke. |
| Harbor | Required Python dependency through `pyproject.toml`. | `uv run python -c "import harbor; import ornnlab"`. |
| Frontend dependencies | Required for local WebUI build. | `npm --prefix frontend ci`, then frontend build. |
| Docker | Optional at bootstrap. Detect existing Docker, offer lightweight core install, or record skip. | Docker CLI/daemon probe plus explicit skip/install state. |

Docker Desktop is not part of the install policy. If a platform requires manual
Docker setup, the installer should say so and record a recoverable skipped
state.

## 5. Update Policy

`ornnlab update` should:

- update the global npm launcher through npm;
- rerun dependency readiness checks;
- repair managed dependencies only after showing planned commands;
- preserve user data and local run artifacts;
- update bootstrap state with the old and new launcher versions.

The command must not rewrite application data schemas without backup/export
support.

## 6. Uninstall Policy

`ornnlab uninstall` should:

- remove launcher-managed files and caches only after a plan is shown;
- move state/data to a dated backup by default;
- avoid irreversible deletion;
- print remaining manual cleanup items that are outside launcher ownership;
- preserve user experiment data unless the user explicitly chooses an archived
  data removal path.

## 7. Documentation Design

Each product version owns a folder:

```text
docs/v<version>/
  prd.md
  technical-design.md
  engineering-plan.md
```

The version PRD describes only that version. Technical design derives from the
PRD. Engineering plan records how the release is implemented and verified.
Release evidence lives under `docs/release/`.

## 8. Validation Design

The release guard should verify:

- current version folder exists;
- required version documents exist;
- governed active documents have `Document Control` tables;
- active docs do not point readers to a superseded total PRD as current truth;
- npm package metadata and transition package metadata remain aligned.
