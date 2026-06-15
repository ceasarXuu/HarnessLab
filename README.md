# HarnessLab

HarnessLab is being redesigned as a Harbor-powered local WebUI for agent
registration, experiment management, reports, and leaderboard review.

Current source of truth:

- PRD: `prd/2026-06-15-harnesslab-webui-prd.md`
- Engineering plan: `docs/plans/2026-06-15-harbor-webui-redesign-engineering-plan.md`
- Install quickstart: `docs/install-quickstart.md`
- Release and rollback checklist: `docs/release-checklist.md`
- Harbor upgrade procedure: `docs/harbor-upgrade-procedure.md`
- Rust legacy workspace decision: `docs/rust-legacy-fate.md`
- Documentation archive: `docs/archive/2026-06-15-pre-harbor-webui-redesign/README.md`

The active product direction is no longer a self-owned Rust benchmark runtime.
Harbor owns benchmark execution, environment lifecycle, agent execution,
verification, and raw job artifacts. HarnessLab owns the local product layer:
declarative agent registration, experiment/run management, diagnostics, report
summaries, and leaderboard views.

## Install With npm

The npm launcher is prepared under the `ornnlab` package name. After
`ornnlab@0.1.1` is published, it is the install path for the active
source-based WebUI workflow.

```bash
npm install -g ornnlab
ornnlab
```

Prerequisites: `git`, `uv`, Node.js, and npm must be available on `PATH`.

The launcher stores its managed source checkout under `~/.ornnlab/HarnessLab` by
default. HarnessLab product data remains under `~/.harnesslab`.
When the app starts, the terminal prints the frontend URL:

```text
Frontend: http://127.0.0.1:5173/
```

Until `ornnlab@0.1.1` is live on npm, the registry may still serve the older
`ornnlab@0.1.0` reservation package.

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

Frontend development command:

```bash
npm --prefix frontend ci
npm --prefix frontend run dev -- --host 127.0.0.1
```

See `docs/install-quickstart.md` for the full fresh-checkout flow.

## Existing npm Package History

The previous scoped npm package reserved the `harnesslab` command name:

```bash
npx @ceasarxuu/harnesslab --help
npx @ceasarxuu/harnesslab --version
```

The active npm install path is now `ornnlab`.
