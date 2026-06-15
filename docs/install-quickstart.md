# HarnessLab Install And Quickstart

This is the current developer quickstart for the Harbor WebUI rewrite.

## npm Launcher Install

After `ornnlab@0.1.1` is published:

```bash
npm install -g ornnlab
ornnlab setup
ornnlab dev
```

The launcher checks out the source repository under `~/.ornnlab/HarnessLab` by
default, installs backend and frontend dependencies, then starts the current
FastAPI backend and Vue frontend development servers.

Prerequisites: `git`, `uv`, Node.js, and npm must be available on `PATH`.

If the npm registry still serves `ornnlab@0.1.0`, that is the older reservation
package and does not yet contain the WebUI launcher.

## Requirements

- Python 3.12 available through `uv`
- Node.js 22 for the frontend gate and CI parity
- Docker for opt-in real Harbor smoke only
- Harbor resolved from `pyproject.toml` as `harbor>=0.13,<0.14`

## Fresh Checkout

From the repository root:

```bash
uv sync --group dev
npm --prefix frontend ci
```

Verify the backend CLI:

```bash
uv run harnesslab --version
uv run harnesslab doctor
```

Start the local backend:

```bash
uv run harnesslab web --host 127.0.0.1 --port 8765
```

In another shell, start the frontend development server:

```bash
npm --prefix frontend run dev -- --host 127.0.0.1
```

The backend API listens on `http://127.0.0.1:8765`. The Vite frontend prints
its selected local URL.

## Quality Gate

For code changes, run:

```bash
scripts/test-after-change-web.sh
```

For docs-only changes, `git diff --check` is the minimum gate.

## Optional Real Harbor Smoke

Run this only on a Docker-capable machine:

```bash
HARNESSLAB_REAL_HARBOR=1 uv run pytest -m docker tests/python/test_real_harbor_cancel_recovery.py
```

The default local and CI gates intentionally skip real Docker execution.

## Local Data

HarnessLab stores local product state under `~/.harnesslab` by default. Before
manual migration or destructive local experiments, create a recoverable backup:

```bash
uv run harnesslab backup export
```

Stale generated-agent and run artifact directories should be archived, not
deleted:

```bash
uv run harnesslab cleanup plan
uv run harnesslab cleanup archive
```
