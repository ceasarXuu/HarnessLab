# OrnnLab WebUI Test Engineering

## Document Control

| Document Version | Engineering Version(s) | Updated | Change |
|---|---|---|---|
| 1.0 | Python app `0.2.0`; Harbor `0.13.x` | 2026-06-15 | Defined Python/Web-first test strategy for the Harbor WebUI rewrite. |
| 1.1 | `ornnlab` npm `0.1.3`; Python app `0.2.0` | 2026-06-16 | Linked test strategy to document version governance. |
| 1.2 | `ornnlab` npm `0.1.3`; Python app `0.2.0` | 2026-06-28 | Replaced Vue frontend gates with the Harbor Viewer-aligned React/Vite demo gates. |
| 1.3 | `ornnlab` npm `0.1.3`; Python app `0.3.0` | 2026-07-10 | Replaced retired product API references with the v1.0.5 WebUI contract and Operation tests. |

The Rust CLI test-engineering document was archived on 2026-06-15.

- Archived copy: `../archive/2026-06-15-pre-harbor-webui-redesign/test-engineering.md`
- Current test strategy: `../releases/v1.0.5/engineering-plan.md`

Current rewrite gates are Python/Web first:

- pytest for backend units and integration tests;
- Harbor subprocess-boundary tests for deterministic queue, recovery,
  config-artifact, and failure paths;
- app-level worker tests that enqueue runs, call `QueueWorkerService.start()`,
  and wait for idle without coupling execution to a request handler;
- cancellation tests that cancel a subprocess-backed worker job and verify the worker
  cannot overwrite `cancelled` with a late engine result;
- startup recovery tests that recreate the app with persisted `running` rows and
  verify deterministic `completed` or `interrupted` outcomes;
- optional Docker-marked Harbor Python API smoke tests gated by
  `ORNNLAB_REAL_HARBOR=1`;
- managed Harbor subprocess tests that verify `harbor.config.json` execution,
  `job.log` capture, and `harbor.cleanup.json` after task cancellation;
- Docker orphan doctor tests that use a fake Docker CLI to verify
  `ornnlab.run_id` label scans, scan failure diagnostics, and dry-run
  cleanup plans;
- system diagnostics tests that verify `WebUiSystemService` reports health and
  that `/api/webui/v1/system/health` returns a contract envelope;
- backup tests that verify exports exclude nested backups, imports restore into
  an empty home, non-empty targets are rejected, and unsafe tar members are
  blocked;
- cleanup tests that verify only unreferenced generated-agent and artifact
  directories are selected and archived into a recoverable location;
- opt-in real Harbor subprocess smoke and cancel-recovery tests in
  `tests/python/test_real_harbor_cancel_recovery.py`, gated by
  `ORNNLAB_REAL_HARBOR=1` and Docker availability;
- ruff and pyright for Python static gates;
- React typecheck, lint, unit tests, Storybook smoke/static-build tests, and
  Codex Web Preview visual acceptance for the frontend;
- GitHub Actions default jobs `python-web` and `frontend-web`, plus an opt-in
  `real-harbor-docker-smoke` workflow dispatch job;
- a line-count gate that fails when production source files exceed 500 lines.

The old Cargo registry remains a legacy reference. Current rewrite traceability
lives in `tests/WEB_REQUIREMENTS.toml`, `tests/WEB_TEST_REGISTRY.toml`, and
`scripts/test-after-change-web.sh`.

Operational note: backend restart tests should build state through public APIs,
then mutate only the persisted crash boundary under test. Do not trust in-memory
worker state after restart; SQLite run status plus Harbor artifacts are the
authoritative recovery inputs.

Operational note: API tests that need deterministic terminal results should create
a Job with `runImmediately=false`, or test the underlying queue service directly.
Product-style tests submit a WebUI write, poll `GET /api/webui/v1/operations/{id}`
until terminal, then read the affected Job, Dataset, event or trial resource.

Operational note: TestClient-based tests must use the context-manager fixture so
FastAPI lifespan and app-level worker tasks share a stable event loop. Creating a
bare TestClient can let request-scoped event loop cleanup cancel background work
and produce false `interrupted` states.

Operational note: Real Harbor subprocess validation is intentionally opt-in.
Run it on a Docker-capable machine with
`ORNNLAB_REAL_HARBOR=1 uv run pytest -m docker tests/python/test_real_harbor_cancel_recovery.py`.

Operational note: Docker orphan cleanup starts as a doctor/reporting gate, not
an automatic remover. The WebUI scans labelled `ornnlab.run_id` containers
and returns dry-run `docker rm -f` cleanup plans for manual review; execution
needs a separate product decision because container removal is not recoverable.

Operational note: the old Vue frontend suite is intentionally gone. Frontend
tests now target the Harbor Viewer-aligned React/Vite demo under `frontend/`.

Operational note: CI keeps real Harbor Docker smoke out of the default required
jobs. Use workflow dispatch with `real_harbor_smoke=true` when Docker-backed
Harbor behavior must be revalidated.
