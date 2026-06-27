# OrnnLab

OrnnLab is being redesigned as a Harbor-powered local WebUI for agent
registration, experiment management, reports, and leaderboard review.

Current source of truth:

- Version document index: `docs/releases/v0.1.3/ornnlab-0.1.3-docs.md`
- Version PRD: `docs/releases/v0.1.3/prd.md`
- Version technical design: `docs/releases/v0.1.3/technical-design.md`
- Version engineering plan: `docs/releases/v0.1.3/engineering-plan.md`
- Install quickstart: `docs/playbooks/install-quickstart.md`
- Release and rollback checklist: `docs/releases/v0.1.3/checklist.md`
- Harbor upgrade procedure: `docs/playbooks/harbor-upgrade-procedure.md`
- Rust legacy workspace decision: `docs/archive/stubs/rust-legacy-fate.md`
- Documentation archive: `docs/archive/2026-06-15-pre-harbor-webui-redesign/README.md`

The active product direction is no longer a self-owned Rust benchmark runtime.
Harbor owns benchmark execution, environment lifecycle, agent execution,
verification, and raw job artifacts. OrnnLab owns the local product layer:
declarative agent registration, experiment/run management, diagnostics, report
summaries, and leaderboard views.

## Install With npm

The npm launcher is published under the `ornnlab` package name. It is the public
install path for the active source-based WebUI workflow.

```bash
npm install -g ornnlab
ornnlab
```

The npm install command still requires an existing Node/npm entrypoint. After
that, the launcher checks `git`, `uv`, Node.js, npm, and optional Docker
capability. Missing required tools are installed automatically when the platform
has a supported package manager or installer path. Docker is optional for first
launch and can be installed or skipped during setup.

The launcher stores its managed source checkout under `~/.ornnlab/launcher/source` by
default. OrnnLab product data remains under `~/.ornnlab/data`.
When the app starts, the terminal prints the frontend URL:

```text
Frontend: http://127.0.0.1:5173/
```

For version authority and release documentation rules, see
`docs/releases/v0.1.3/version-governance.md`.

## Current Status

This repository still contains the previous Rust workspace and npm reservation
package materials. They are legacy/reference assets. The active implementation
path is the new Python/FastAPI backend and Vue frontend.

Implemented rewrite foundation:

- `ornnlab web` / `python -m ornnlab web` backend entrypoint
- `/api/system/status`, agents, experiments, events, benchmarks, leaderboard
- SQLite migration and local `~/.ornnlab/data` data directory initialization
- AgentProfile v2 validation and Harbor agent config compilation
- managed Harbor subprocess execution as the default engine path
- cancellation cleanup evidence and doctor diagnostics for the Harbor engine
- Docker orphan doctor scan with dry-run cleanup plans
- local `ornnlab backup export` / `ornnlab backup import` archives
- safe `ornnlab cleanup plan` / `ornnlab cleanup archive` for stale local artifacts
- Vue operations-console scaffold under `frontend/`
- Python/Web gate script: `scripts/test-after-change-web.sh`
- GitHub Actions CI for Python Web, frontend Web, and opt-in real Harbor Docker smoke

## Planned Local App

The intended MVP stack is:

- Python + FastAPI backend
- Vue 3 + TypeScript frontend
- SQLite metadata index
- File-based artifacts under `~/.ornnlab/data`
- Harbor `0.13.x` as the execution engine
- Server-Sent Events for status and log streams

Development launch command:

```bash
uv sync --group dev
uv run ornnlab web
```

Frontend development command:

```bash
npm --prefix frontend ci
npm --prefix frontend run dev -- --host 127.0.0.1
```

See `docs/playbooks/install-quickstart.md` for the full fresh-checkout flow.

## Existing npm Package History

The previous scoped npm package reserved the `harnesslab` command name:

```bash
npx @ceasarxuu/harnesslab --help
npx @ceasarxuu/harnesslab --version
```

The active npm install path is now `ornnlab`. The `harnesslab` Python console
script remains as a compatibility alias during the transition.
