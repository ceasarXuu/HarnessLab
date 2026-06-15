# HarnessLab

HarnessLab is being redesigned as a Harbor-powered local WebUI for agent
registration, experiment management, reports, and leaderboard review.

Current source of truth:

- PRD: `prd/2026-06-15-harnesslab-webui-prd.md`
- Engineering plan: `docs/plans/2026-06-15-harbor-webui-redesign-engineering-plan.md`
- Documentation archive: `docs/archive/2026-06-15-pre-harbor-webui-redesign/README.md`

The active product direction is no longer a self-owned Rust benchmark runtime.
Harbor owns benchmark execution, environment lifecycle, agent execution,
verification, and raw job artifacts. HarnessLab owns the local product layer:
declarative agent registration, experiment/run management, diagnostics, report
summaries, and leaderboard views.

## Current Status

This repository still contains the previous Rust workspace and npm reservation
package materials. They are legacy/reference assets. The active implementation
path is the new Python/FastAPI backend and Vue frontend.

Implemented rewrite foundation:

- `harnesslab web` / `python -m harnesslab web` backend entrypoint
- `/api/system/status`, agents, experiments, events, benchmarks, leaderboard
- SQLite migration and local `~/.harnesslab` data directory initialization
- AgentProfile v2 validation and Harbor agent config compilation
- fake HarborEngine path for deterministic local tests
- managed Harbor subprocess execution with cancellation cleanup evidence
- Docker orphan doctor scan with dry-run cleanup plans
- local `harnesslab backup export` / `harnesslab backup import` archives
- safe `harnesslab cleanup plan` / `harnesslab cleanup archive` for stale local artifacts
- Vue operations-console scaffold under `frontend/`
- Python/Web gate script: `scripts/test-after-change-web.sh`
- GitHub Actions CI for Python Web, frontend Web, and opt-in real Harbor Docker smoke

## Planned Local App

The intended MVP stack is:

- Python + FastAPI backend
- Vue 3 + TypeScript frontend
- SQLite metadata index
- File-based artifacts under `~/.harnesslab`
- Harbor `0.13.x` as the execution engine
- Server-Sent Events for status and log streams

Development launch command:

```bash
uv sync --group dev
uv run harnesslab web
```

## Existing npm Reservation Package

The scoped npm package currently reserves the `harnesslab` command name:

```bash
npx @ceasarxuu/harnesslab --help
npx @ceasarxuu/harnesslab --version
```

This package is not the active WebUI implementation.
