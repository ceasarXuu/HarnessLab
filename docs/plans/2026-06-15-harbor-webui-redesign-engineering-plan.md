# HarnessLab Harbor WebUI Redesign Engineering Plan

- Created: 2026-06-15
- Updated: 2026-06-15
- Version: 3.0
- Status: Phase 1-4 foundation landed; real Harbor execution and hardening remain phased
- Owner: HarnessLab team
- Source PRD: `prd/2026-06-15-harnesslab-webui-prd.md`
- Harbor lifecycle spike: `docs/spikes/2026-06-15-harbor-lifecycle-spike.md`
- Supersedes:
  - `docs/plans/2026-06-15-harbor-integration-engineering-plan.md`
  - `docs/plans/2026-06-15-harnesslab-webui-engineering-plan.md`
  - archived legacy architecture/spec documents under `docs/archive/2026-06-15-pre-harbor-webui-redesign/`
- Related systems: Harbor 0.13.2, FastAPI, Vue 3, Docker, local filesystem, SQLite
- Risk level: High

## 1. Executive Decision

HarnessLab should stop investing in a self-owned benchmark runtime. Harbor becomes the execution engine. HarnessLab becomes a local product layer that makes agent registration, experiment setup, run monitoring, result reuse, and leaderboard review easier than using Harbor directly.

The target architecture is:

- Python backend, because Harbor is Python and exposes usable `JobConfig`, `AgentConfig`, `DatasetConfig`, `EnvironmentConfig`, and `Job.create(...).run()` APIs.
- FastAPI service, because the product is a local WebUI with REST commands and one-way live run/log streams.
- Vue 3 + TypeScript frontend, because the existing PRD already selected Vue and the app is an operations console rather than a content site.
- SQLite metadata index plus file-based artifacts, because experiments, queues, dashboards, and leaderboards need queryable local state while Harbor job outputs must remain inspectable as files.
- Declarative `AgentProfile` compiler, not a broad custom runtime. Built-in Harbor agents map directly to Harbor `AgentConfig`; custom command agents materialize only the minimal import-path Python class needed by Harbor.

Rust is no longer the default implementation language for the new product. The current Rust workspace is treated as a reference implementation for requirements, tests, redaction ideas, and prior adapter lessons. It should not be extended as the main runtime or kept as a Rust-to-Python bridge unless a later packaging phase proves a single binary is worth the added complexity.

## 2. Evidence From Current Project And Harbor

### 2.1 Current HarnessLab State

The repository currently contains a Rust workspace with `harnesslab-cli`, `harnesslab-core`, `harnesslab-adapters`, `harnesslab-infra`, `harnesslab-report`, and `xtask`. It has strong investment in adapter contracts, runtime snapshots, replay, redaction, official-runner preservation, and contract tests.

That investment solves a previous product: a CLI benchmark harness that owns run orchestration and calls external runners. It is not aligned with the new product goal because:

- current generic architecture says the center is `Run Orchestrator + Adapter Contracts + Artifact Store`, not WebUI;
- current technology decisions still say Rust single-binary first and reject Python;
- current plans split between Rust CLI + Python Bridge and full WebUI rewrite;
- current runtime work keeps adding compatibility logic around external runners that Harbor already owns;
- several Rust files already exceed the repository's 500-line code-file guideline, so a large Rust continuation would require structural cleanup before new features.

The reusable assets are requirements and test discipline, not the runtime implementation:

- agent profile schema ideas and redaction policy;
- test registry and traceability discipline;
- failure-class taxonomy and diagnostics expectations;
- run artifact expectations;
- lessons from Terminal-Bench and SWE-bench Pro failures;
- playbooks for Docker, Colima, real smoke checks, and artifact diagnosis.

### 2.2 Harbor Facts

Local verification found Harbor `0.13.2` installed as a uv tool. Its CLI supports:

- remote datasets with `--dataset name@version`;
- local task or dataset paths with `--path`;
- `--agent`, `--agent-import-path`, `--agent-env`, `--agent-kwarg`, `--skill`, and `--mcp-config`;
- environment backends including Docker, Daytona, E2B, Modal, GKE, and others;
- job result directories under `jobs/`;
- a viewer command for trajectories and artifacts.

Local Python API inspection confirms:

- `harbor.job.Job.create(config) -> Job`;
- `Job.run() -> JobResult`;
- `JobConfig` includes agents, datasets, tasks, artifacts, plugins, retry, environment, verifier, and concurrency;
- `AgentConfig` includes `name`, `import_path`, `model_name`, `skills`, `env`, `kwargs`, `mcp_servers`, and timeout fields;
- `AgentFactory` already registers Harbor built-ins such as `oracle`, `nop`, `claude-code`, `codex`, `aider`, `opencode`, `openhands`, `pi`, `gemini-cli`, `qwen-coder`, `cursor-cli`, `goose`, `kimi-cli`, and others.

Local run artifacts show:

- `terminal-bench@2.0` with `oracle` and `n_tasks=1` completed with mean `1.0`;
- a `swebench-verified` sample failed in Docker compose build/start with return code `-9`, which proves the new product needs first-class resource diagnostics, Harbor exception classification, and Docker log surfacing.

## 3. Product Scope

### 3.1 In Scope

- Local single-user WebUI launched by `harnesslab web`.
- Agent registration and editing through templates and structured forms.
- Declarative profiles persisted as TOML for user readability.
- Direct mapping to Harbor built-in agents where possible.
- Safe custom-command agent materialization for agents not built into Harbor.
- Experiment creation, cloning, queueing, running, cancellation, and retry.
- One active experiment at a time in MVP, with a persisted FIFO queue for additional runs.
- Real-time status and log tailing through Server-Sent Events.
- HarnessLab report shell with summary, trial table, failure explanation, raw Harbor artifact links, and optional Harbor viewer launch.
- Leaderboard computed from completed local experiments.
- Doctor/status surfaces for Harbor, Docker, disk, Python, generated agents, profile validity, and stale running jobs.
- Structured logs and audit events for agent changes, experiment lifecycle, Harbor calls, failure classification, and cleanup.

### 3.2 Out Of Scope

- Multi-user auth, RBAC, teams, or remote deployment.
- Reimplementing Harbor environments, task format, verifier lifecycle, or dataset registry.
- Uploading to Harbor Hub or sharing public links.
- Custom benchmark authoring UI.
- Cloud execution UX beyond exposing Harbor environment selection for advanced users.
- Rust CLI parity in the first rewrite.
- Mobile UI.
- Internationalization beyond Chinese MVP copy.

## 4. Architecture

### 4.1 System Boundary

```text
Browser Vue App
  | REST commands
  | SSE status/log streams
  v
FastAPI Local Service
  | application services
  | SQLite metadata
  | TOML/JSON/artifact files
  v
Harbor Engine Adapter
  | JobConfig / Job.create / Job.run
  v
Harbor Framework
  | Docker / tasks / agents / verifiers / artifacts
  v
Local Harbor job directory
```

HarnessLab owns product state, profile compilation, UX-level validation, status recovery, and summaries. Harbor owns environment lifecycle, task execution, agent execution, verifier execution, and raw job artifacts.

### 4.1.1 Harbor Lifecycle Contract

Harbor `0.13.2` exposes an async Python surface: `await Job.create(config)` and
`await job.run()`. `Job.run()` writes `config.json`, `result.json`, `lock.json`,
`job.log`, and per-trial directories under the configured `jobs_dir`. The
current Harbor API has progress hooks and trial cancel events, but no stable
top-level `Job.cancel()` method that HarnessLab can treat as a complete product
contract.

Implementation rule: Phase 1 must produce a Harbor lifecycle spike before any
user-facing cancel/restart promise is considered implemented. The spike must
verify these facts against the pinned Harbor version:

- exact async call shape and whether calls can run safely inside FastAPI
  background tasks;
- durable job identity fields available before and after `Job.run()`;
- whether `lock.json` is sufficient to identify running/resumable jobs;
- how `Job` resumes when `config.json` and partial trial result directories
  already exist;
- what happens when the asyncio task running `job.run()` is cancelled;
- whether Docker environments reliably receive `stop(delete=true)` during
  cancellation and exception paths;
- which Docker compose labels/project names can be used for cleanup when the
  Python process dies.

Until that spike passes, product cancellation is specified as a best-effort
state transition:

1. write `experiment.cancel_requested`;
2. cancel the backend asyncio task running Harbor;
3. wait a bounded grace period;
4. run HarnessLab cleanup using recorded `harbor_job_dir`, Harbor job name,
   known Docker compose project names when available, and Docker label scans;
5. mark the run `cancelled` only after cleanup evidence is written;
6. otherwise mark it `interrupted` with `cancel_escalation_failed`.

If the spike proves Harbor's Python API cannot satisfy these lifecycle
requirements, Phase 1 must stop and choose one of two fallbacks before Phase 3:

- wrap `harbor run --config <file>` as a managed subprocess with process-group
  cancellation and log tailing;
- keep the Python API for creation/result parsing but launch execution in a
  separate worker process so HarnessLab owns kill and restart boundaries.

This lifecycle decision is a release blocker, not an implementation detail.

### 4.2 Proposed Repository Shape

```text
HarnessLab/
  pyproject.toml
  harnesslab/
    __init__.py
    __main__.py
    cli.py
    app.py
    api/
      agents.py
      experiments.py
      reports.py
      leaderboard.py
      system.py
      events.py
    services/
      agent_service.py
      profile_compiler.py
      experiment_service.py
      queue_service.py
      harbor_engine.py
      report_service.py
      leaderboard_service.py
      doctor_service.py
      log_service.py
    models/
      agent.py
      experiment.py
      harbor.py
      report.py
      events.py
    storage/
      sqlite.py
      migrations/
        001_initial.sql
      file_store.py
      paths.py
      locks.py
    generated_agents/
      templates/
        command_agent.py.j2
    observability/
      logging.py
      audit.py
      failure_classifier.py
  frontend/
    package.json
    vite.config.ts
    .storybook/
    src/
      app/
      api/
      components/
      views/
      stores/
      test/
  tests/
    python/
    e2e/
    fixtures/
```

No production code file should exceed 500 lines. Large services must be split by responsibility before crossing that limit.

### 4.3 Local Data Layout

```text
~/.harnesslab/
  config.toml
  harnesslab.sqlite
  agents/
    <agent-id>.toml
  generated-agents/
    <agent-id>/
      agent.py
      manifest.json
  experiments/
    <experiment-id>/
      config.snapshot.json
      agent.snapshot.toml
      harbor.config.json
      harnesslab-events.jsonl
      app.log.jsonl
      harbor-job/
        config.json
        result.json
        job.log
        <trial>/
      report/
        summary.json
        index.html
  templates/
    <template-id>.json
  exports/
  logs/
```

SQLite stores indexes and state transitions. Files store source-of-truth profile text, snapshots, Harbor artifacts, logs, and generated reports. SQLite rows must always point to file artifacts by relative paths under `~/.harnesslab`.

### 4.4 SQLite Tables

Minimum schema:

```text
agents(
  id text primary key,
  name text unique not null,
  kind text not null,
  harbor_agent_name text,
  harbor_import_path text,
  model_name text,
  status text not null,
  profile_path text not null,
  created_at text not null,
  updated_at text not null
)

experiments(
  id text primary key,
  name text not null,
  kind text not null,
  status text not null,
  requested_run_count integer not null,
  mode text not null,
  created_at text not null,
  updated_at text not null
)

runs(
  id text primary key,
  experiment_id text not null,
  status text not null,
  run_order integer not null,
  agent_id text not null,
  agent_snapshot_hash text not null,
  benchmark_name text not null,
  benchmark_version text,
  split text,
  task_filter_hash text,
  n_tasks integer,
  n_attempts integer not null,
  n_concurrent integer not null,
  harbor_job_name text,
  harbor_job_id text,
  created_at text not null,
  updated_at text not null,
  started_at text,
  finished_at text,
  job_dir text,
  result_path text,
  report_path text,
  failure_class text,
  failure_code text,
  failure_summary text
  leaderboard_eligible integer not null,
  comparability_key text
)

queue_items(
  run_id text primary key,
  queue_position integer not null,
  state text not null,
  enqueued_at text not null,
  dequeued_at text,
  finished_at text
)

experiment_events(
  id integer primary key autoincrement,
  aggregate_type text not null,
  aggregate_id text not null,
  ts text not null,
  event_type text not null,
  severity text not null,
  payload_json text not null,
  mirror_file text,
  mirror_offset integer
)

templates(
  id text primary key,
  name text unique not null,
  config_json text not null,
  created_at text not null,
  updated_at text not null
)
```

Migrations must be explicit SQL files with a `schema_migrations` table and idempotent upgrade tests.

### 4.5 Entity Semantics

HarnessLab must not overload "experiment" to mean every product and runtime
object.

| Entity | Meaning | Maps to Harbor | User-visible |
|---|---|---|---|
| `Experiment` | User intent and grouping container. Can be a single run, comparison, or batch. | No direct Harbor object. | Yes |
| `Run` | One immutable execution unit: one agent snapshot + one benchmark/dataset/version/filter + one Harbor job config. | Exactly one Harbor `JobConfig` and job directory. | Yes |
| `Comparison` | Experiment kind containing multiple runs intended for side-by-side review. | Multiple Harbor jobs, one per run. | Yes |
| `Batch` | Experiment kind created by selecting multiple benchmarks or filters. | Multiple Harbor jobs, one per run. | Yes |
| `Template` | Saved experiment creation spec. It creates new experiments/runs with new snapshots. | No direct Harbor object. | Yes |
| `Report` | A run report plus optional experiment summary over its child runs. | Reads one or more Harbor result dirs. | Yes |

Fan-out rules:

- single agent + single benchmark creates one experiment with one run;
- multi-agent comparison creates one experiment with one run per agent;
- multi-benchmark batch creates one experiment with one run per benchmark;
- multi-agent + multi-benchmark creates one experiment with `agent_count *
  benchmark_count` runs and must warn the user before enqueueing;
- queueing happens at run level, not experiment level;
- experiment status is derived from child runs: `draft`, `queued`,
  `running`, `completed`, `failed`, `partially_failed`, `cancelled`, or
  `interrupted`.

Report rules:

- every run gets a run report;
- comparison and batch experiments get an experiment summary that links to each
  run report;
- cloning from a report clones the experiment spec but creates fresh agent and
  benchmark snapshots.

### 4.5.1 Leaderboard Eligibility

Leaderboards rank runs, not experiments. A run is eligible for the default
leaderboard only when all comparability fields match the selected leaderboard
scope:

- benchmark name;
- benchmark version or Harbor dataset ref;
- split;
- task filter set;
- task count, unless the leaderboard explicitly chooses "sampled";
- agent profile snapshot hash;
- Harbor major/minor version;
- verifier enabled/disabled state;
- environment backend.

Default leaderboard behavior:

- exclude smoke runs such as `n_tasks=1`;
- exclude partial task filters unless the user selects that exact filter scope;
- exclude runs whose agent profile changed after the ranked run;
- show an "excluded runs" drawer with `leaderboard.entry_excluded` reasons;
- permit an explicit "include non-comparable runs" mode only in exploratory
  views, never in the default ranking.

The `comparability_key` stored on `runs` is the stable hash of those fields.

### 4.6 Crash Consistency And Recovery

SQLite is the authority for queue order and state transitions. Files are the
authority for user-authored profiles, immutable snapshots, Harbor raw artifacts,
and generated reports. `harnesslab-events.jsonl` is an append-only human/debug
mirror of `experiment_events`, not an independent state source.

Write order:

1. create the experiment/run directory with a temporary marker;
2. atomically write config/profile snapshots;
3. in one SQLite transaction, insert experiment/run rows, queue row, and
   `state.snapshot_written`;
4. append the mirrored JSONL event with the SQLite event id and record the file
   offset back to SQLite when available;
5. before Harbor starts, write `harbor.config.json` atomically and then set run
   status to `running`;
6. after Harbor finishes, read Harbor `result.json`, generate
   `summary.json`/report into temporary files, then atomically rename them;
7. in one SQLite transaction, mark the run terminal and record report/result
   paths;
8. if any mirror write fails after the SQLite commit, keep the state transition
   and emit `sqlite_file_drift_detected` on next startup.

Startup recovery:

- rebuild missing JSONL mirrors from SQLite when possible;
- if SQLite says `running` and Harbor `result.json` exists, classify from
  Harbor artifacts and mark terminal with `experiment.reconciled`;
- if SQLite says `running` and only `lock.json` exists, mark `interrupted`
  unless the lifecycle spike proves a safe resume path;
- if files exist without SQLite rows, import them only through an explicit
  repair command, never silently into the queue;
- if queue rows and run statuses disagree, queue order is rebuilt from
  `queue_items.queue_position` and a `queue.rebuilt_from_store` event is
  emitted.

Required crash tests must kill the backend after each numbered write-order
step and prove the next startup reaches a deterministic state.

## 5. Core Domain Design

### 5.1 AgentProfile v2

The user-facing TOML remains declarative:

```toml
schema_version = 2
id = "codex-default"
name = "Codex Default"
kind = "codex"
description = "Default Codex CLI profile"

[harbor]
agent = "codex"
model = "openai/gpt-5.1"
kwargs = { reasoning_effort = "medium" }

[auth]
inherit_env = ["OPENAI_API_KEY"]

[skills]
paths = ["~/skills"]

[mcp]
config_paths = []

[runtime]
agent_timeout_sec = 3600
setup_timeout_sec = 600
```

Custom command profile:

```toml
schema_version = 2
id = "claude-ds"
name = "Claude DS"
kind = "custom-command"

[command_agent]
install = []
run = "claude-ds -p {{instruction}}"
working_dir = "workspace"
shell = "bash"

[auth]
inherit_env = ["ANTHROPIC_API_KEY"]
include_paths = ["~/.claude"]
```

Compiler rules:

- Built-in profiles compile to Harbor `AgentConfig(name=..., model_name=..., env=..., skills=..., mcp_servers=..., kwargs=...)`.
- Custom command profiles compile to a generated `BaseInstalledAgent` subclass and Harbor `AgentConfig(import_path=..., env=..., kwargs=...)`.
- Only `{{instruction}}` is allowed in command templates for MVP.
- Environment inheritance is allowlisted by name; parent process env is never passed wholesale.
- `include_paths` must become explicit Harbor environment mounts or be blocked if the selected Harbor backend cannot support them.
- Unsupported profile fields are blocker diagnostics, not silent no-ops.
- Every compile writes `generated-agents/<agent-id>/manifest.json` with profile hash, generated file hash, Harbor version, and compiler version.
- Custom-command profiles are trusted-local-operator configuration. They are not
  safe templates to import from untrusted sources. The UI must show a command
  preview, inherited env list, mounted paths, selected Harbor backend support,
  and a warning before first run.
- `include_paths` is denied by default for non-Docker Harbor backends until
  backend-specific mount support is proven by a test.

### 5.2 Experiment Model

Status lifecycle:

```text
draft -> queued -> running -> completed
                      |        -> failed
                      |        -> cancelled
                      |        -> interrupted
draft -> invalid
```

Rules:

- MVP runs at most one Harbor job at a time.
- Creating or starting while another run is active enqueues runs unless the user explicitly chooses "save draft".
- Cancellation requests are idempotent.
- If the backend restarts and finds `running` runs, it reconciles against Harbor job artifacts and marks them `completed`, `failed`, or `interrupted`.
- Every run uses immutable snapshots of agent profile and experiment config.
- Editing an agent does not affect already queued/running runs unless the user regenerates the snapshot.

### 5.3 HarborEngine

Responsibilities:

- build Harbor `JobConfig` from experiment snapshot;
- call `await Job.create(config)` and `await job.run()`;
- write `harbor.config.json` before execution;
- stream structured app events around Harbor calls;
- classify Harbor exceptions into HarnessLab failure classes;
- detect job artifact paths;
- expose cancellation hooks best-effort through task cancellation and process cleanup if Harbor API lacks direct cancellation for a specific state.

The implementation must record a Harbor capability snapshot for every run:
Harbor package version, imported API symbols used, `JobConfig` schema hash,
detected cancellation mode, and environment backend. Harbor version upgrades
must update this snapshot test before the dependency range changes.

Failure classes:

| Class | Examples | User message target |
|---|---|---|
| `profile_invalid` | bad TOML, unsupported kind, unsafe env/path | Agent editor field |
| `harbor_config_invalid` | invalid dataset, invalid agent name, bad kwargs | Experiment form |
| `dataset_unavailable` | registry unavailable, auth needed, no tasks matched | Benchmark selector |
| `docker_unavailable` | no Docker CLI/daemon/context | System status |
| `docker_resource_failure` | compose killed, OOM, disk exhaustion, image build killed | Experiment detail diagnostics |
| `agent_setup_failed` | install command failed, missing auth | Agent detail diagnostics |
| `agent_run_failed` | agent non-zero, timeout, parse issue | Trial detail |
| `verifier_failed` | verifier exception, reward file missing | Trial detail |
| `harbor_internal_error` | uncategorized Harbor exception | Developer diagnostics |
| `cancelled` | user cancellation | Experiment status |

### 5.4 Logs And Observability

Required event families:

- `app.started`, `app.stopped`
- `system.status.checked`
- `agent.created`, `agent.updated`, `agent.deleted`, `agent.compiled`, `agent.compile_failed`
- `experiment.created`, `experiment.queued`, `experiment.started`, `experiment.progress`, `experiment.completed`, `experiment.failed`, `experiment.cancelled`
- `experiment.cancel_requested`, `experiment.cancel_escalated`, `experiment.interrupted`, `experiment.reconciled`, `experiment.reconcile_decision`
- `queue.position_assigned`, `queue.dequeued`, `queue.rebuilt_from_store`
- `harbor.job.configured`, `harbor.job.created`, `harbor.job.running`, `harbor.job.completed`, `harbor.job.failed`
- `harbor.capability_snapshot`
- `state.authority_selected`, `sqlite_file_drift_detected`
- `leaderboard.entry_included`, `leaderboard.entry_excluded`
- `docker.diagnostic.collected`
- `report.generated`

Every log line must include `ts`, `level`, `event`, `run_id` or `experiment_id` when applicable, `correlation_id`, and redacted payload.

`harnesslab doctor --logs` must print the latest failed experiment, failure class/code, relevant log paths, Docker status, Harbor version, generated agent manifest path, and next remediation.

## 6. API Design

REST endpoints:

```text
GET    /api/system/status
POST   /api/system/doctor

GET    /api/agent-templates
GET    /api/agents
POST   /api/agents
GET    /api/agents/{agent_id}
PUT    /api/agents/{agent_id}
DELETE /api/agents/{agent_id}
POST   /api/agents/{agent_id}/validate
POST   /api/agents/{agent_id}/compile

GET    /api/benchmarks

GET    /api/experiments
POST   /api/experiments
GET    /api/experiments/{experiment_id}
PUT    /api/experiments/{experiment_id}
DELETE /api/experiments/{experiment_id}
POST   /api/experiments/{experiment_id}/run
POST   /api/experiments/{experiment_id}/cancel
POST   /api/experiments/{experiment_id}/clone
POST   /api/experiments/{experiment_id}/save-template
GET    /api/experiments/{experiment_id}/runs

GET    /api/experiments/{experiment_id}/events
GET    /api/experiments/{experiment_id}/events/stream
GET    /api/experiments/{experiment_id}/report

GET    /api/runs/{run_id}
POST   /api/runs/{run_id}/cancel
GET    /api/runs/{run_id}/events
GET    /api/runs/{run_id}/events/stream
GET    /api/runs/{run_id}/logs/{log_name}
GET    /api/runs/{run_id}/report

GET    /api/templates
POST   /api/templates
DELETE /api/templates/{template_id}

GET    /api/leaderboard
```

SSE stream contract:

```text
event: experiment.progress
data: {"experiment_id":"...","status":"running","completed_trials":1,"total_trials":10}

event: log.line
data: {"source":"harbor-job.log","offset":1234,"line":"...redacted..."}

event: experiment.failed
data: {"failure_class":"docker_resource_failure","failure_code":"compose_return_code_negative_9"}
```

SSE is preferred over WebSocket for MVP because logs and status are one-way. Commands remain normal HTTP requests.

SSE recovery contract:

- every SSE event has a monotonically increasing `id` matching
  `experiment_events.id`;
- clients send `Last-Event-ID` on reconnect;
- `/api/experiments/{experiment_id}/events?after=<id>` returns missed events
  before the live stream resumes;
- log line events include `source`, `offset`, and `line`;
- if the requested offset has been compacted, the server emits
  `log.replay_unavailable` and gives the current tail boundary instead of
  pretending the stream is continuous.

## 7. Frontend Design

### 7.1 Stack

- Vue 3
- TypeScript
- Vite
- Vue Router
- Pinia
- `@tanstack/vue-query` for server state
- Storybook for component development and interaction checks
- Vitest for component/unit tests
- Playwright for browser smoke and e2e
- `lucide-vue-next` for icons

### 7.2 Main Views

- Dashboard: system status, active/queued experiment, recent results, quick actions.
- Agents: list, create wizard, edit, detail, compile/validate diagnostics.
- Experiments: list, create wizard, detail, live logs, cancel/retry/clone.
- Reports: HarnessLab summary shell plus raw Harbor artifacts.
- Leaderboard: benchmark filter, rank table, score trend, link to source experiment.
- Settings/Doctor: Harbor version, Docker status, data directory, logs, generated agents, repair actions.

### 7.3 UI Rules

- No marketing landing page. First screen is the dashboard.
- Use compact operations-console layout, not a hero site.
- Use tables for scan/comparison workflows.
- Use wizards only for creation flows that need staged validation.
- Buttons use icons where the command is common and tooltip text for icon-only actions.
- Long logs use virtualization and fixed-height containers.
- All tables and controls must have stable responsive dimensions; no text overlap at desktop or narrow browser widths.
- Agent deletion, experiment cancellation, and artifact cleanup require confirmation with clear consequence text.

## 8. Testing Strategy

### 8.1 Test Pyramid

| Layer | Tool | Required coverage |
|---|---|---|
| Python unit | pytest | profile compiler, schema validation, storage migrations, failure classifier |
| Python integration | pytest + fake HarborEngine | API routes, queue transitions, recovery, report generation |
| Real Harbor smoke | pytest mark `docker` | `oracle` on `terminal-bench@2.0` with `n_tasks=1` |
| Frontend unit | Vitest | stores, API clients, form validation |
| Storybook | Storybook test runner | agent wizard states, experiment states, logs, empty/error states |
| Browser e2e | Playwright | first-run flow, create agent, run smoke, report, leaderboard |
| Static gates | ruff, pyright, eslint, vue-tsc | type/lint/style |
| Docs/gates | custom scripts | no code file over 500 lines, API schema snapshots, test registry sync |

### 8.2 Required Smoke Paths

1. `harnesslab web --port 0` starts backend and serves frontend.
2. System status returns Harbor version, Docker status, data directory, and schema version.
3. Create built-in `oracle` profile through API and UI.
4. Compile profile to Harbor `AgentConfig(name="oracle")`.
5. Create experiment for `terminal-bench@2.0`, `n_tasks=1`, `n_concurrent=1`.
6. Run experiment to completion with real Harbor when Docker is available.
7. Generate HarnessLab report summary.
8. Leaderboard includes the completed experiment.
9. Cancel a fake long-running experiment and verify no running state remains after backend restart.
10. Force a fake Docker resource failure and verify user-facing diagnostics include log paths and remediation.

### 8.3 Test IDs

New Python/Web test IDs should extend the current test registry rather than discard the discipline:

- `WEB-SYS-*` system status and doctor.
- `WEB-AGT-*` agent forms, validation, compilation, generated agent manifests.
- `WEB-EXP-*` experiment CRUD, state machine, queue, cancellation.
- `WEB-HARBOR-*` Harbor config building and failure classification.
- `WEB-RPT-*` report summary and artifact links.
- `WEB-LB-*` leaderboard aggregation.
- `WEB-LOG-*` structured logs and SSE streams.
- `WEB-E2E-*` full browser flows.

### 8.4 Test Engineering Migration

The existing Rust/Cargo registry and scripts are legacy references. The rewrite
must create Python/Web gates instead of pretending the current Cargo gates still
cover the product.

Required migration tasks:

- introduce `tests/WEB_REQUIREMENTS.toml` and `tests/WEB_TEST_REGISTRY.toml`
  or extend the existing manifests with explicit `toolchain = "python-web"`;
- create `scripts/test-after-change-web.sh` that runs pytest, ruff, pyright,
  frontend typecheck/lint/unit tests, Storybook tests, and Playwright smoke;
- keep Docker/real Harbor smoke as an optional marked gate, not the default
  local unit gate;
- update traceability generation so `WEB-*` requirements do not depend on
  Rust crate file patterns;
- add a meta-test proving the web gate fails when a registered `WEB-*` test is
  removed.

## 9. Phased Execution Plan

### Phase 0: Decision Freeze And Documentation Convergence

Objective: make the rewrite direction unambiguous before code churn.

Tasks:

1. Mark or archive the Rust CLI + Python Bridge plan as superseded.
2. Mark or archive the older WebUI plan as superseded by this plan.
3. Archive legacy Rust/self-runtime source documents that would mislead implementation.
4. Update PRD decisions: SQLite metadata, report shell, persisted queue, SSE, Python backend.
5. Add ADR in this document as the source of truth.
6. Add a short Harbor API spike note with local `0.13.2` evidence.
7. Keep original paths as stubs when tests, review reports, or old links still reference them.
8. Update repository root `README.md` and any docs index so first-contact readers see the Harbor WebUI direction.

Acceptance criteria:

- A fresh agent can identify this file as the canonical engineering plan.
- No 2026-06-15 plan claims conflicting implementation direction without a supersession notice or archive stub.
- Legacy root architecture/spec/technology docs point to this plan instead of showing stale Rust runtime content.
- Repository root `README.md` no longer describes the Rust CLI as the active product direction.
- PRD has no blocking open question for MVP architecture.
- VS review report exists and every finding is triaged.

Verification:

- `git diff --check`
- Manual link check for referenced local files.
- Adversarial review recorded under `vs_review/`.

### Phase 1: Python Backend Foundation

Objective: create an installable backend with status, storage, migrations, and Harbor config construction.

Tasks:

1. Create `pyproject.toml` with FastAPI, uvicorn, pydantic, ruff, pyright, pytest, and Harbor dependency pinned to `>=0.13,<0.14`.
2. Implement `harnesslab web`, `harnesslab doctor`, and `harnesslab version`.
3. Implement app startup, settings, path initialization, structured logging, and SQLite migrations.
4. Implement `/api/system/status` and `/api/benchmarks`.
5. Implement Harbor model wrappers and config-builder tests without running Docker.
6. Complete the Harbor lifecycle spike from section 4.1.1 and record the decision.
7. Add generated OpenAPI snapshot.
8. Add code-file line-count gate for Python and frontend source.
9. Create the Python/Web test registry and `scripts/test-after-change-web.sh` skeleton.

Acceptance criteria:

- `uv run harnesslab web --port 0` starts and reports the selected port.
- `/api/system/status` returns Harbor version, Docker status, data dir, DB schema version, and warnings.
- SQLite initializes idempotently.
- Harbor `JobConfig` can be built for `oracle + terminal-bench@2.0 + n_tasks=1`.
- Harbor lifecycle spike explicitly chooses Python API, managed subprocess, or worker-process execution for Phase 3.
- No new production code file exceeds 500 lines.

Testing:

- `uv run pytest tests/python/test_storage.py tests/python/test_system_api.py tests/python/test_harbor_config.py`
- `uv run ruff check harnesslab tests`
- `uv run pyright`
- `git diff --check`

### Phase 2: Agent Registry And Declarative Compiler

Objective: make agent registration usable before experiments depend on it.

Tasks:

1. Define `AgentProfile v2` Pydantic model and TOML serialization.
2. Implement preset templates for Harbor built-ins: `oracle`, `claude-code`, `codex`, `aider`, `opencode`, `openhands`, `pi`, `gemini-cli`, `qwen-coder`, and `custom-command`.
3. Implement profile validation with field-level diagnostics.
4. Implement built-in profile compiler to Harbor `AgentConfig`.
5. Implement custom-command materializer with deterministic generated Python and manifest hash.
6. Implement agent CRUD API.
7. Implement audit logs for create/update/delete/compile.
8. Add UI Storybook stories for agent list, empty state, create wizard, validation errors, and compile diagnostics.

Acceptance criteria:

- User can create, edit, validate, compile, and delete an agent through API.
- Built-in `oracle` compiles without generated Python.
- Custom command profile generates importable Python under `~/.harnesslab/generated-agents/`.
- Unsupported fields produce blocker diagnostics, not warnings.
- Deleting an agent referenced by a running/queued experiment is blocked.
- Custom-command UI/API exposes command preview, env inheritance, mount list, and trusted-local warning before first run.

Testing:

- `uv run pytest tests/python/test_agent_profile.py tests/python/test_profile_compiler.py tests/python/test_agent_api.py`
- Import test for generated custom-command agent in a temp home.
- Custom-command quoting normalization test for complex shell strings.
- Unsupported `include_paths` mount denial test for non-Docker Harbor backends.
- `npm --prefix frontend run storybook:test -- Agent`
- `npm --prefix frontend run test -- agents`

### Phase 3: Experiment Engine, Queue, Logs, And Failure Classification

Objective: run Harbor jobs through the backend with recoverable local state.

Tasks:

1. Define experiment, queue, event, and result-summary models.
2. Implement experiment CRUD API and immutable run snapshots.
3. Implement persisted FIFO queue with one active job.
4. Implement `HarborEngine` async runner.
5. Implement fake HarborEngine for deterministic tests.
6. Implement SSE event/log stream.
7. Implement cancellation and restart reconciliation.
8. Implement failure classifier for Harbor exceptions and Docker compose failures.
9. Implement Docker diagnostic collection for common local failures.
10. Implement crash-consistency recovery rules from section 4.6.

Acceptance criteria:

- API can create draft experiment, enqueue it, run it, cancel it, and recover it after backend restart.
- Queue order is stored in `queue_items` and survives backend restart.
- Real Harbor smoke with `oracle + terminal-bench@2.0 + n_tasks=1` completes when Docker is available.
- Fake failure cases map to stable failure classes and user remediation text.
- SSE stream emits status, progress, log lines, completion, and failure events.
- SSE reconnect with `Last-Event-ID` replays missed events or emits `log.replay_unavailable`.
- Running experiment artifacts contain `harbor.config.json`, `harnesslab-events.jsonl`, app logs, and Harbor job directory.

Testing:

- `uv run pytest tests/python/test_experiment_state.py tests/python/test_queue_service.py tests/python/test_sse_events.py tests/python/test_failure_classifier.py`
- `uv run pytest -m docker tests/python/test_real_harbor_smoke.py tests/python/test_real_harbor_cancel_recovery.py` when Docker is available.
- Restart recovery test with fake running rows.
- Cancellation test proving final status is not left as `running`.
- Kill-after-each-transition crash-consistency tests for enqueue/start/complete/report generation.

### Phase 4: WebUI Agent And Experiment Workflows

Objective: deliver the usable local app shell and core workflows.

Tasks:

1. Implement Vue app layout, route structure, API client, and Pinia stores.
2. Implement Dashboard with active experiment and system status.
3. Implement Agent list/create/edit/detail views.
4. Implement Experiment list/create/detail views.
5. Implement live logs panel with virtualization.
6. Implement confirmation flows for delete/cancel.
7. Implement Storybook coverage for all major states.
8. Implement Playwright flow using fake backend.

Acceptance criteria:

- Browser first screen is Dashboard, not a landing page.
- User can create an `oracle` agent, create experiment, start it, watch status/logs, cancel fake run, and view final state.
- All empty/loading/error states are represented.
- Narrow and desktop screenshots show no overlapping controls or overflowing button text.
- Storybook includes agent wizard, experiment wizard, running detail, failed detail, and dashboard states.

Testing:

- `npm --prefix frontend run typecheck`
- `npm --prefix frontend run lint`
- `npm --prefix frontend run test`
- `npm --prefix frontend run storybook:test`
- `npm --prefix frontend run e2e -- --project=chromium`

### Phase 5: Reports, Leaderboard, And Result Reuse

Objective: close the product loop after experiments finish.

Tasks:

1. Parse Harbor `result.json` into HarnessLab `summary.json`.
2. Generate report shell with score, failures, trials, usage, links, and raw artifact access.
3. Add optional Harbor viewer launch/open action.
4. Implement leaderboard aggregation by benchmark, agent, score, pass rate, duration, tokens, and run time.
5. Implement experiment clone and save-as-template from reports.
6. Implement trend data for repeated agent/benchmark runs.
7. Implement leaderboard eligibility and exclusion explanations.

Acceptance criteria:

- Completed experiments have a report page with summary and raw Harbor artifact links.
- Failed experiments have a report page that explains failure class and log locations.
- Leaderboard updates after each completed experiment.
- Default leaderboard excludes smoke runs, partial task filters, mismatched splits/task sets, mismatched agent snapshot hashes, and mismatched Harbor major/minor versions unless the user explicitly includes them.
- Users can clone an experiment or save it as a template from the report.
- Raw Harbor artifacts remain available without copying secrets into public summaries.

Testing:

- `uv run pytest tests/python/test_report_service.py tests/python/test_leaderboard_service.py tests/python/test_redaction.py`
- Leaderboard comparability tests proving smoke runs and mismatched snapshots are excluded by default.
- Frontend report and leaderboard component tests.
- Playwright e2e: run fake completed experiment, open report, clone, verify leaderboard.
- Artifact scan for known secret patterns.

### Phase 6: Hardening, Migration, And Release Packaging

Objective: make the rewrite shippable and retire stale architecture safely.

Tasks:

1. Add `harnesslab doctor --logs` and repair guidance.
2. Add stale job cleanup and generated-agent cleanup.
3. Add local backup/export/import for `~/.harnesslab`.
4. Add installer and README quickstart.
5. Expand `docs/architecture.md` and `docs/technology-decisions.md` from their current stubs into full documents matching this plan.
6. Decide whether to archive, keep, or remove Rust crates. If removal is chosen, move them to a backup/archive path or use a reversible git commit; do not use unrecoverable deletion outside git.
7. Add CI matrix for Python, frontend, and optional Docker smoke.
8. Add release checklist and rollback instructions.

Acceptance criteria:

- Fresh checkout can install and launch the WebUI from documented commands.
- CI passes Python tests, frontend tests, static gates, and non-Docker integration tests.
- Optional Docker smoke passes on a machine with Docker.
- Legacy docs no longer contradict the new architecture.
- Rust code fate is explicitly decided and documented.
- Working tree is clean after commit.

Testing:

- Full local gate script, for example `scripts/test-after-change-web.sh`.
- `uv run pytest`
- `npm --prefix frontend run test`
- `npm --prefix frontend run e2e`
- Docker smoke where available.
- `git diff --check`

## 10. Implementation Staffing Guidance

The work can be split into non-conflicting lanes:

- Backend foundation: storage, settings, system API, logging.
- Agent compiler: profile schema, compiler, generated agents, validation.
- Harbor execution: config builder, engine, queue, failure classifier.
- Frontend app: shell, components, Storybook, API client.
- Reporting: result parser, report shell, leaderboard.
- Test engineering: fixtures, fake HarborEngine, Playwright, Docker smoke, gates.

Do not parallelize two workers over the same service files. Keep write scopes disjoint and merge through the API/schema contracts.

## 11. Risks And Mitigations

| Risk | Impact | Mitigation |
|---|---|---|
| Harbor API changes before release | High | Pin `>=0.13,<0.14`, add API compatibility tests, isolate imports in `harbor_engine.py` |
| Generated custom agents become code-injection vector | High | Allow only structured fields and `{{instruction}}`, shell quote commands, hash generated files, block raw Python editing in UI |
| File artifacts and SQLite drift | Medium | Treat files as source artifacts, SQLite as index, add reconciliation on startup |
| Docker failures look like agent failures | High | Failure classifier and Docker diagnostics are Phase 3 acceptance criteria |
| Long logs freeze frontend | Medium | SSE backpressure, log tail offsets, virtualization, max line size |
| WebUI hides Harbor details users need | Medium | Link raw `config.json`, `result.json`, `job.log`, trial logs, and optional Harbor viewer |
| Existing Rust docs mislead future agents | Medium | Phase 0 supersession notices, Phase 6 documentation rewrite |
| Single active job frustrates users | Low for MVP | Persist queue and show position; add concurrency only after reliability is proven |

## 12. ADR

### Decision

Build HarnessLab v3 as a Python FastAPI + Vue 3 local WebUI that delegates benchmark execution to Harbor.

### Drivers

- Product goal is agent and experiment management, not runtime ownership.
- Harbor already owns the hard runtime boundary.
- Direct Python integration removes the Rust-Python bridge and subprocess protocol.
- WebUI needs queryable local metadata and operational recovery.
- Users need simple declarative agent registration over Harbor's SDK/import-path model.

### Alternatives Considered

| Alternative | Why rejected |
|---|---|
| Continue Rust runtime | Duplicates Harbor, prolongs external-runner compatibility work, delays WebUI |
| Rust CLI + Python Bridge | Keeps two runtimes and a fragile JSON/subprocess boundary without user-visible benefit |
| Pure Harbor CLI wrapper with no backend | Cannot deliver agent registry, experiment queue, dashboard, or report reuse UX |
| File-only metadata | Too fragile for queues, dashboards, recovery, and leaderboard queries |
| Full database/server deployment | Overkill for local single-user MVP |

### Consequences

- Packaging now depends on Python and Node build tooling during development.
- Harbor version compatibility is a core product risk.
- Some Rust test assets and architecture docs become legacy references.
- SQLite migrations and frontend e2e become part of the quality gate.

### Follow-ups

- Rewrite architecture and technology decision docs after Phase 1 proves the skeleton.
- Decide Rust crate archival in Phase 6.
- Add explicit Harbor upgrade procedure before any dependency bump.

## 13. Implementation Ledger

### 2026-06-15 Foundation Pass

Landed initial rewrite scaffolding:

- Python package with `harnesslab web`, `harnesslab doctor`, and `harnesslab version`.
- FastAPI app with system, agent, benchmark, experiment, event, and leaderboard endpoints.
- SQLite migration for agents, experiments, runs, queue items, events, and templates.
- AgentProfile v2 Pydantic model plus built-in Harbor agent config compilation and
  custom-command materialization with manifest hashes.
- Fake HarborEngine path, report summary writer, and leaderboard query service.
- Vue 3 + TypeScript operations-console scaffold under `frontend/`.
- Python/Web test registry skeleton and `scripts/test-after-change-web.sh`.

Validation evidence:

- `scripts/test-after-change-web.sh`
- `uv run harnesslab --version`
- `uv run harnesslab doctor`

### 2026-06-15 Queue And Result Pass

Landed additional Phase 3/5 backend behavior:

- persisted queue service with `queued -> running -> terminal` transitions;
- run-level API routes under `/api/runs/{run_id}`;
- idempotent queued/draft run cancellation;
- fake Docker resource failure classification into `docker_resource_failure`;
- report generation for failed and completed fake-engine runs;
- leaderboard score field and default smoke-run exclusion evidence;
- frontend API client types for experiments, runs, cancellation, and leaderboard.

Validation evidence:

- `uv run pytest tests/python`
- `uv run ruff check harnesslab tests/python`
- `uv run pyright`

### 2026-06-15 CRUD Template Report Pass

Landed more Phase 2/3/5 product-loop behavior:

- agent update and soft-delete with queued/running run protection;
- experiment cancel, soft-delete, clone, and save-as-template APIs;
- template create/list/soft-delete APIs;
- run and experiment report summary APIs;
- failed report summaries now retain failure class/code;
- frontend API client types for templates and report endpoints.

Validation evidence:

- `uv run pytest tests/python`
- `uv run ruff check harnesslab tests/python`
- `uv run pyright`

### 2026-06-15 Harbor Lifecycle Adapter Pass

Landed the first Phase 3 real-execution boundary:

- `HarborEngine` now selects a deterministic `fake` adapter by default or a real
  `python-api` adapter when `HARNESSLAB_HARBOR_ENGINE=python-api` is set.
- Every run writes `harbor.config.json` in Harbor `JobConfig` shape before the
  run is marked `running`, plus `harbor.capability.json` for auditability.
- Experiment execution now resolves each agent through AgentProfile compilation
  instead of passing only `agent_id`.
- Fake runs write `result.json`, preserving the same artifact contract as the
  Python API runner.
- Real Harbor smoke coverage is available as an opt-in Docker test via
  `HARNESSLAB_REAL_HARBOR=1`.

Known remaining Phase 3 blockers:

- Harbor `Job` exposes `create` and `run` but no public cancel API in the
  inspected 0.13.x surface, so hard running-job cancellation remains
  unsupported until a process/plugin boundary is introduced.
- Background workers, restart recovery, and orphan cleanup remain separate
  hardening work.

Validation evidence:

- `uv run pytest tests/python`
- `uv run ruff check harnesslab tests/python`
- `uv run pyright`

### 2026-06-15 Startup Recovery Pass

Landed the first Phase 3 restart-recovery boundary:

- backend startup now reconciles persisted `running` runs before serving routes;
- if a Harbor `result.json` artifact exists, the run is marked terminal from
  that artifact and a report is generated;
- if no result artifact exists, the stale run is marked `interrupted` with
  `harbor_recovery/stale_running_without_result`;
- queue item state is moved to the same terminal status as the recovered run;
- experiment status is re-derived after recovery;
- `system/status` now exposes `stale_running_runs` as a doctor signal;
- recovery decisions emit durable run and experiment events.

Known remaining Phase 3 blockers:

- hard cancellation of actively running Harbor work still requires a managed
  process or plugin boundary because Harbor 0.13.x has no public `Job.cancel()`.
- orphan Docker/process cleanup still needs a real Harbor lifecycle proof.

Validation evidence:

- `uv run pytest tests/python`
- `uv run ruff check harnesslab tests/python`
- `uv run pyright`
- `npm --prefix frontend run typecheck`

### 2026-06-15 Queue Worker Pass

Landed the first Phase 3 background-worker boundary:

- `POST /api/experiments/{id}/run` now enqueues and starts the app-level worker
  by default instead of blocking the request until completion;
- deterministic tests and scripts can call
  `POST /api/experiments/{id}/run?wait=true`;
- `QueueWorkerService` drains persisted FIFO queue rows with one active worker
  task and one active run at a time;
- app startup starts the worker when persisted queued rows already exist;
- worker drain uses the same `ExperimentService.execute_dequeued_run` path as
  blocking tests, keeping result/report/failure behavior single-sourced;
- queue worker start/idle events are mirrored for diagnostics.

Known remaining Phase 3 blockers:

- hard cancellation of actively running Harbor work still requires a managed
  process or plugin boundary because Harbor 0.13.x has no public `Job.cancel()`;
- orphan Docker/process cleanup still needs a real Harbor lifecycle proof.

Validation evidence:

- `uv run pytest tests/python`
- `uv run ruff check harnesslab tests/python`
- `uv run pyright`

### 2026-06-15 Running Cancellation Guard Pass

Landed a Phase 3 cancellation correctness guard for the fake/worker path:

- fake Harbor runner supports `fake-slow-cancel` to create a deterministic
  running window for cancellation tests;
- cancelling a running run writes a cancellation report and emits
  `experiment.cancel_requested`;
- if the engine returns or fails after the run was cancelled, the worker now
  preserves the existing `cancelled` terminal state instead of overwriting it
  with a late completed/failed result;
- cancelled runs finalize their parent experiment when all child runs are
  terminal;
- worker cancellation tests prove the final status does not return to
  `running` or `completed`.

Known remaining Phase 3 blockers:

- hard cancellation of actively running real Harbor work still requires a
  managed process or plugin boundary because Harbor 0.13.x has no public
  `Job.cancel()`;
- orphan Docker/process cleanup still needs a real Harbor lifecycle proof.

Validation evidence:

- `uv run pytest tests/python`
- `uv run ruff check harnesslab tests/python`
- `uv run pyright`

### 2026-06-15 Active Worker Task Cancellation Pass

Landed the first app-level execution cancellation hook:

- `QueueWorkerService` now tracks the active asyncio task for each running run;
- `POST /api/runs/{run_id}/cancel` and experiment cancellation notify the worker
  to cancel the active task after SQLite records the terminal cancellation;
- `ExperimentService` catches `asyncio.CancelledError` from the execution
  boundary and preserves `cancelled` when the database already records user
  cancellation;
- unexpected worker task cancellation is marked `interrupted` with
  `worker_lifecycle/worker_task_cancelled` rather than leaving a run as
  `running`;
- TestClient fixtures now use context-manager lifespan semantics so background
  worker tests run on a stable event loop;
- fake slow-run tests prove active task cancellation prevents the fake engine
  from writing a late `result.json`.

Known remaining Phase 3 blockers:

- real Harbor hard cancellation still needs a managed subprocess/plugin boundary
  and Docker/process cleanup evidence;
- orphan Docker/process cleanup still needs a real Harbor lifecycle proof.

Validation evidence:

- `uv run pytest tests/python`
- `uv run ruff check harnesslab tests/python`
- `uv run pyright`

### 2026-06-15 Managed Subprocess Harbor Runner Pass

Landed the first process-owned Harbor execution boundary:

- `HARNESSLAB_HARBOR_ENGINE=subprocess` selects a managed subprocess runner;
- the runner invokes `harbor run --config <harbor.config.json>` by default and
  supports `HARNESSLAB_HARBOR_SUBPROCESS_COMMAND` for explicit command override;
- stdout/stderr are mirrored to `job.log`;
- successful subprocess execution reads or writes `result.json` so report
  generation has a stable artifact contract;
- task cancellation terminates the subprocess process group, escalates to kill
  after a grace period, and writes `harbor.cleanup.json`;
- capability snapshots now report `supports_cancel=true` for subprocess mode;
- subprocess tests prove config-file execution, log capture, and cleanup JSON on
  task cancellation without requiring Docker.

Known remaining Phase 3 blockers:

- real Harbor subprocess smoke must still prove `harbor run --config` produces
  expected Harbor artifacts for a real benchmark;
- Docker/process orphan cleanup still needs the real Harbor cancel-recovery
  proof described in `tests/python/test_real_harbor_cancel_recovery.py`.

Validation evidence:

- `uv run pytest tests/python`
- `uv run ruff check harnesslab tests/python`
- `uv run pyright`
