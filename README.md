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
package materials. They are legacy/reference assets while the Harbor WebUI
redesign is planned and implemented. Do not use the Rust CLI architecture docs
or archived 2026-06-15 drafts as implementation source of truth.

## Planned Local App

The intended MVP stack is:

- Python + FastAPI backend
- Vue 3 + TypeScript frontend
- SQLite metadata index
- File-based artifacts under `~/.harnesslab`
- Harbor `0.13.x` as the execution engine
- Server-Sent Events for status and log streams

Planned launch command:

```bash
harnesslab web
```

## Existing npm Reservation Package

The scoped npm package currently reserves the `harnesslab` command name:

```bash
npx @ceasarxuu/harnesslab --help
npx @ceasarxuu/harnesslab --version
```

This package is not the active WebUI implementation.
