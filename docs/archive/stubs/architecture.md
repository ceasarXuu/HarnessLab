# OrnnLab Architecture

The previous Rust runtime architecture was archived on 2026-06-15 because the
product direction changed to a Harbor-powered local WebUI.

- Archived copy: `docs/archive/2026-06-15-pre-harbor-webui-redesign/architecture.md`
- Canonical engineering plan: `docs/plans/2026-06-15-harbor-webui-redesign-engineering-plan.md`

## System Boundary

OrnnLab is a local product layer around Harbor. It does not reimplement
benchmark execution, sandbox orchestration, agent execution, verifier execution,
or raw Harbor job artifacts.

```text
Vue WebUI
  -> FastAPI REST and event routes
  -> application services
  -> SQLite metadata and file artifacts
  -> Harbor engine adapter
  -> Harbor framework and Docker-backed environments
```

## Ownership

OrnnLab owns:

- local agent profiles and generated custom-command agent manifests;
- experiment, run, queue, event, report, template, and leaderboard indexes;
- WebUI workflows for creating agents, starting/cancelling runs, viewing reports,
  and reviewing local leaderboard results;
- doctor diagnostics for Harbor, Docker, SQLite, stale running rows, and
  OrnnLab-labelled Docker orphan scans;
- restart reconciliation from SQLite state and Harbor artifacts.

Harbor owns:

- benchmark dataset/task loading;
- environment lifecycle;
- agent and verifier execution;
- raw `config.json`, `result.json`, `job.log`, and trial artifacts.

## Runtime Modes

`HarborEngine` supports three execution boundaries:

| Mode | Purpose | Cancellation boundary |
|---|---|---|
| `fake` | deterministic local tests and frontend workflows | async task only |
| `python-api` | direct Harbor API integration | best effort, no public `Job.cancel()` in inspected Harbor 0.13.x |
| `subprocess` | managed `harbor run --config <harbor.config.json>` execution | process-group terminate/kill plus `harbor.cleanup.json` evidence |

The subprocess mode is the current reliable hard-cancellation boundary. Real
subprocess smoke and cancel-recovery tests are opt-in because they require
Docker and Harbor execution.

## State And Recovery

SQLite is the authority for queryable product state. Files are the authority for
profile source text, immutable run snapshots, Harbor config/result artifacts,
and generated reports.

Startup recovery reconciles persisted `running` rows before serving routes:

- if `result.json` exists, OrnnLab marks the run terminal from that artifact
  and regenerates its report;
- if no result artifact exists, the run becomes `interrupted` with
  `stale_running_without_result`;
- doctor exposes `stale_running_runs` so operators can see unreconciled state.

## API Surface

The active API is local and single-user:

- `/api/system/status`, `/api/system/doctor`, `/api/system/docker-orphans`;
- `/api/agents` and compile/validate routes;
- `/api/experiments`, run/cancel/clone/template routes;
- `/api/runs/{run_id}` report, event, log, and cancel routes;
- `/api/templates`;
- `/api/leaderboard`.

SSE/log streaming is still governed by the canonical plan until the frontend
live-log implementation closes that phase.

## Artifact Layout

The default home is `~/.ornnlab/data`:

```text
ornnlab.sqlite
agents/<agent-id>.toml
generated-agents/<agent-id>/manifest.json
experiments/<experiment-id>/
  config.snapshot.json
  agent.snapshot.toml
  harbor.config.json
  ornnlab-events.jsonl
  harbor-job/
    job.log
    result.json
    harbor.cleanup.json
  report/
    summary.json
    index.html
```

Paths are indexed in SQLite but remain inspectable on disk for debugging and
manual recovery.
