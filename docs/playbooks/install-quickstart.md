# OrnnLab Install And Quickstart

## Document Control

| Document Version | Engineering Version(s) | Updated | Change |
|---|---|---|---|
| 1.0 | `ornnlab` npm `0.1.1`; Python app `0.2.0` | 2026-06-15 | Documented npm launcher install and local WebUI startup. |
| 1.1 | `ornnlab` npm `0.1.3`; Python app `0.2.0` | 2026-06-16 | Removed stale literal versions and linked install behavior to version governance. |
| 1.2 | `ornnlab` npm `0.1.3`; Python app `0.2.0` | 2026-06-28 | Removed old Vue frontend startup guidance after the v1.0.5 official-architecture rebuild decision. |

This is the current developer quickstart for the Harbor WebUI rewrite.

## npm Launcher Install

Install the latest OrnnLab npm launcher:

```bash
npm install -g ornnlab
ornnlab
```

The launcher bootstraps the local machine, checks out the source repository
under `~/.ornnlab/launcher/source` by default, installs backend dependencies,
then starts the current FastAPI backend. The old Vue frontend has been removed;
v1.0.5 will rebuild the UI against Harbor's official Viewer architecture.

The terminal prints the backend API URL before starting the server:

```text
Backend API: http://127.0.0.1:8765/
```

The npm install command still requires an existing Node/npm entrypoint. After
that, the launcher checks required runtime tools (`git`, `uv`, Node.js, and
npm) and attempts to install missing tools on macOS, Linux, and Windows when a
supported system package manager or installer path is available.

Docker is optional for first launch. If Docker is already present, the launcher
records that capability. If it is missing, the launcher asks whether to install
lightweight core Docker tooling; choosing no continues WebUI setup and lets you
retry later:

```bash
ORNNLAB_INSTALL_DOCKER=1 ornnlab install
```

Bootstrap state is written under `~/.ornnlab/launcher/bootstrap-state.json` for
diagnostics. The state includes a schema version and launcher version. Rerunning
`ornnlab` retries incomplete setup phases.

OrnnLab does not install Docker Desktop. On macOS, the lightweight path is Docker
CLI plus Colima. On Windows, OrnnLab does not silently install Docker Desktop; it
will guide users toward a core WSL/Docker Engine path instead.

For explicit bootstrap without starting the WebUI, run:

```bash
ornnlab install
```

`ornnlab setup` remains a compatibility alias.

If the npm registry serves an older launcher, check `docs/releases/` and
`docs/playbooks/npm-package-reservation.md` for the current release status.

## Requirements

- Python 3.12 available through `uv`
- Node.js 22 for the npm launcher and future frontend rebuild
- Docker for Harbor benchmark execution through the default subprocess path
- Harbor resolved from `pyproject.toml` as `harbor>=0.13,<0.14`

## Fresh Checkout

From the repository root:

```bash
uv sync --group dev
```

Verify the backend CLI:

```bash
uv run ornnlab --version
uv run ornnlab doctor
```

Start the local backend:

```bash
uv run ornnlab web --host 127.0.0.1 --port 8765
```

The backend API listens on `http://127.0.0.1:8765`. Frontend development will
resume after the v1.0.5 React/Vite package is rebuilt.

## Quality Gate

For code changes, run:

```bash
scripts/test-after-change-web.sh
```

For docs-only changes, `git diff --check` is the minimum gate.

## Real Harbor Smoke

Run this when changing Harbor execution, cancellation, or recovery behavior on a
Docker-capable machine:

```bash
ORNNLAB_REAL_HARBOR=1 uv run pytest -m docker tests/python/test_real_harbor_cancel_recovery.py
```

The standard local and CI quality gates use a subprocess-boundary simulator for
deterministic coverage. Real benchmark execution still uses Harbor and Docker by
default at runtime.

## Local Data

OrnnLab stores local product state under `~/.ornnlab/data` by default. Before
manual migration or destructive local experiments, create a recoverable backup:

```bash
uv run ornnlab backup export
```

Stale generated-agent and run artifact directories should be archived, not
deleted:

```bash
uv run ornnlab cleanup plan
uv run ornnlab cleanup archive
```
