# HarnessLab Technology Decisions

The previous Rust single-binary technology decision record was archived on
2026-06-15.

- Archived copy: `docs/archive/2026-06-15-pre-harbor-webui-redesign/technology-decisions.md`
- Canonical engineering plan: `docs/plans/2026-06-15-harbor-webui-redesign-engineering-plan.md`

## Active Decisions

| Area | Decision |
|---|---|
| Execution engine | Harbor 0.13.x |
| Backend | Python + FastAPI |
| Frontend | Vue 3 + TypeScript + Vite |
| Metadata | Local SQLite |
| Artifacts | TOML/JSON/JSONL/HTML files under `~/.harnesslab` |
| Live updates | Server-Sent Events for status/log streams |
| Default local test engine | Fake Harbor engine |
| Real execution boundary | Harbor Python API or managed Harbor subprocess |
| Hard cancellation boundary | Managed Harbor subprocess process group |
| Frontend component workflow | Storybook smoke plus Vue/Vitest/Playwright gates |
| Packaging | Python package first; Rust binary is not the MVP path |

## Rationale

Python/FastAPI is the backend because Harbor is a Python framework and its
`JobConfig`, `AgentConfig`, dataset, environment, and result models are the
integration surface. A Rust-to-Python bridge would add a second runtime boundary
without improving the local WebUI workflows.

SQLite is used because experiments, queues, reports, and leaderboards need
queryable local state. File artifacts remain first-class because Harbor outputs
must be inspectable, recoverable, and linkable from reports.

Vue 3 + TypeScript is used for the local operations console. The frontend is
tested through typecheck, lint, Vitest, Storybook smoke, and Playwright e2e.

## Execution Mode Policy

`HARNESSLAB_HARBOR_ENGINE` selects execution:

- unset or `fake`: deterministic development and CI default;
- `python-api`: direct `Job.create(...).run()` integration;
- `subprocess` or `cli`: managed `harbor run --config <file>` execution.

`HARNESSLAB_HARBOR_SUBPROCESS_COMMAND` can override the subprocess command. The
default is `harbor run`.

The subprocess boundary is preferred for real cancellation because the app owns
the process group and can write `harbor.cleanup.json` after termination. Direct
Python API execution remains useful for compatibility checks but is not treated
as a complete hard-cancel contract until Harbor exposes one.

## Diagnostics Policy

Doctor/status output should prefer actionable structured diagnostics over
silent fallbacks:

- Docker CLI absence reports `docker_cli_missing`;
- Docker scan failure reports `docker_orphan_scan_failed`;
- labelled HarnessLab container survivors report `docker_orphans_detected` and a
  dry-run cleanup plan;
- stale SQLite `running` rows report `stale_running_runs`.
- `harnesslab doctor --logs` includes the latest failed or interrupted run,
  relevant result/report/job log paths, and remediation actions.

Cleanup plans are not executed automatically because container removal is not
recoverable. Any automatic cleanup command needs a product decision and tests.

Local filesystem cleanup uses archive moves, not deletion. `harnesslab cleanup
plan` reports generated-agent directories and experiment artifact directories
that SQLite no longer references. `harnesslab cleanup archive` moves those
candidates under `~/.harnesslab/archive/cleanup-*` so they remain recoverable.

## Backup Policy

`harnesslab backup export` writes a local `.tar.gz` archive of `~/.harnesslab`.
The export excludes `exports/` so backups do not recursively include earlier
backups, checkpoints SQLite before archiving, and includes a manifest with the
HarnessLab version and file count.

`harnesslab backup import <archive>` restores only into an empty HarnessLab home.
It rejects absolute paths, `..` path traversal, links, and device files. The
import command does not delete or overwrite existing user data.

## Quality Gate Policy

Every code change should pass `scripts/test-after-change-web.sh` unless the
change is intentionally docs-only and the narrower evidence is stated. The gate
uses `uv run` for Python commands so it does not depend on a system `python`
alias, and it runs Vitest with `--pool threads --maxWorkers=1` to avoid local
fork-worker startup flakes.

GitHub Actions mirrors the WebUI gate as two required default jobs:

- `python-web`: `uv sync --group dev`, ruff, pyright, pytest, line-count, and
  `git diff --check`;
- `frontend-web`: Node 22, `npm ci`, Playwright Chromium install, Vue
  typecheck, ESLint, Vitest, Storybook smoke, and Playwright e2e.

The workflow sets `FORCE_JAVASCRIPT_ACTIONS_TO_NODE24=true` so reusable GitHub
JavaScript actions run on the Node 24 runtime while the frontend application
continues to test against Node 22.

Real Harbor Docker smoke remains opt-in through `workflow_dispatch` with
`real_harbor_smoke=true`, because it requires Docker and real benchmark runtime
resources that should not block every PR by default.
