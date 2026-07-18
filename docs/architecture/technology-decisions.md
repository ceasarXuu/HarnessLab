# OrnnLab Technology Decisions

## Document Control

| Document Version | Engineering Version(s) | Updated | Change |
|---|---|---|---|
| 1.0 | Python app `0.2.0`; Harbor `0.13.x` | 2026-06-15 | Recorded active Harbor WebUI technology decisions after Rust runtime archive. |
| 1.1 | `ornnlab` npm `0.1.3`; Python app `0.2.0` | 2026-06-16 | Linked technology decisions to document version governance. |
| 1.2 | `ornnlab` npm `0.1.3`; Python app `0.2.0` | 2026-06-28 | Replaced Vue demo frontend decision with the v1.0.5 Harbor official Viewer-aligned React/Vite demo. |
| 1.3 | v1.0.5 | 2026-07-10 | Upgraded the backend directly to the WebUI contract and retired Playwright from the active gate. |
| 1.4 | Python app `0.2.0`; Harbor `0.13.x` | 2026-07-19 | 统一新建与恢复 Job 的 Harbor CLI 可执行文件解析。 |

The previous Rust single-binary technology decision record was archived on
2026-06-15.

- Archived copy: `../archive/2026-06-15-pre-harbor-webui-redesign/technology-decisions.md`
- Canonical engineering plan: `../releases/v1.0.5/engineering-plan.md`

## Active Decisions

| Area | Decision |
|---|---|
| Execution engine | Harbor 0.13.x |
| Backend | Python + FastAPI |
| Frontend | React + Vite, aligned with Harbor official Viewer |
| Metadata | Local SQLite |
| Artifacts | TOML/JSON/JSONL/HTML files under `~/.ornnlab/data` |
| Live updates | Operation polling and Job event reads |
| Default local test engine | Fake Harbor engine |
| Real execution boundary | Harbor Python API or managed Harbor subprocess |
| Hard cancellation boundary | Managed Harbor subprocess process group |
| Frontend component workflow | Storybook static build plus React/Vitest gates; visual acceptance in Codex Web Preview |
| Packaging | Python package first; Rust binary is not the MVP path |

## Rationale

Python/FastAPI is the backend because Harbor is a Python framework and its
`JobConfig`, `AgentConfig`, dataset, environment, and result models are the
integration surface. A Rust-to-Python bridge would add a second runtime boundary
without improving the local WebUI workflows.

SQLite is used because experiments, queues, reports, and leaderboards need
queryable local state. File artifacts remain first-class because Harbor outputs
must be inspectable, recoverable, and linkable from reports.

The old Vue operations-console demo has been removed. v1.0.5 frontend work now
starts from a React/Vite demo aligned with Harbor's official Viewer architecture:
React, Vite, Tailwind, shadcn-style primitives, Storybook, and lucide-react.
The public Harbor Hub remains a visual reference, but the local
product architecture tracks Harbor `apps/viewer`.

## Execution Mode Policy

`ORNNLAB_HARBOR_ENGINE` selects execution:

- unset, `real`, `subprocess`, or `cli`: managed `harbor run --config <file>`
  execution. This is the official default path;
- `python-api`: direct `Job.create(...).run()` integration;

`ORNNLAB_HARBOR_SUBPROCESS_COMMAND` can override the subprocess command. The
default arguments are `run`, while the `harbor` executable is resolved through
the shared CLI resolver: `ORNNLAB_HARBOR_CLI`, then `PATH`, then the executable
next to the active Python interpreter. New Job and resume flows must use the
same resolver so a project-local virtual environment does not depend on the
daemon's inherited `PATH`.

The subprocess boundary is preferred for real cancellation because the app owns
the process group and can write `harbor.cleanup.json` after termination. Direct
Python API execution remains useful for compatibility checks but is not treated
as a complete hard-cancel contract until Harbor exposes one.

## Diagnostics Policy

Doctor/status output should prefer actionable structured diagnostics over
silent fallbacks:

- Docker CLI absence reports `docker_cli_missing`;
- Docker scan failure reports `docker_orphan_scan_failed`;
- labelled OrnnLab container survivors report `docker_orphans_detected` and a
  dry-run cleanup plan;
- stale SQLite `running` rows report `stale_running_runs`.
- `ornnlab doctor --logs` includes the latest failed or interrupted run,
  relevant result/report/job log paths, and remediation actions.

Cleanup plans are not executed automatically because container removal is not
recoverable. Any automatic cleanup command needs a product decision and tests.

Local filesystem cleanup uses archive moves, not deletion. `ornnlab cleanup
plan` reports generated-agent directories and experiment artifact directories
that SQLite no longer references. `ornnlab cleanup archive` moves those
candidates under `~/.ornnlab/data/archive/cleanup-*` so they remain recoverable.

## Backup Policy

`ornnlab backup export` writes a local `.tar.gz` archive of `~/.ornnlab/data`.
The export excludes `exports/` so backups do not recursively include earlier
backups, checkpoints SQLite before archiving, and includes a manifest with the
OrnnLab version and file count.

`ornnlab backup import <archive>` restores only into an empty OrnnLab home.
It rejects absolute paths, `..` path traversal, links, and device files. The
import command does not delete or overwrite existing user data.

## Quality Gate Policy

Every code change should pass `scripts/test-after-change-web.sh` unless the
change is intentionally docs-only and the narrower evidence is stated. The gate
uses `uv run` for Python commands so it does not depend on a system `python`
alias, and it runs the React frontend checks when `frontend/package.json` exists.

GitHub Actions mirrors the WebUI gate as two required default jobs plus optional
real Harbor smoke:

- `python-web`: `uv sync --group dev`, ruff, pyright, pytest, line-count, and
  `git diff --check`;
- `frontend-web`: Node 22, `npm ci`, React typecheck, ESLint, Vitest, Storybook
  smoke and Storybook static build. Visual acceptance runs in Codex Web Preview.

Real Harbor Docker smoke remains opt-in through `workflow_dispatch` with
`real_harbor_smoke=true`, because it requires Docker and real benchmark runtime
resources that should not block every PR by default.
