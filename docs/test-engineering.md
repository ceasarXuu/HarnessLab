# HarnessLab WebUI Test Engineering

The Rust CLI test-engineering document was archived on 2026-06-15.

- Archived copy: `archive/2026-06-15-pre-harbor-webui-redesign/test-engineering.md`
- Current test strategy: `plans/2026-06-15-harbor-webui-redesign-engineering-plan.md#8-testing-strategy`

Current rewrite gates are Python/Web first:

- pytest for backend units and integration tests;
- fake HarborEngine tests for deterministic queue, recovery, config-artifact, and
  failure paths;
- app-level worker tests that enqueue runs, call `QueueWorkerService.start()`,
  and wait for idle without coupling execution to a request handler;
- cancellation tests that cancel a fake running worker job and verify the worker
  cannot overwrite `cancelled` with a late fake-engine result;
- startup recovery tests that recreate the app with persisted `running` rows and
  verify deterministic `completed` or `interrupted` outcomes;
- optional Docker-marked Harbor Python API smoke tests gated by
  `HARNESSLAB_REAL_HARBOR=1`;
- managed Harbor subprocess tests that verify `harbor.config.json` execution,
  `job.log` capture, and `harbor.cleanup.json` after task cancellation;
- Docker orphan doctor tests that use a fake Docker CLI to verify
  `harnesslab.run_id` label scans, scan failure diagnostics, and dry-run
  cleanup plans;
- doctor logs tests that verify `harnesslab doctor --logs` and
  `/api/system/doctor?logs=true` expose failed-run paths and remediation;
- backup tests that verify exports exclude nested backups, imports restore into
  an empty home, non-empty targets are rejected, and unsafe tar members are
  blocked;
- cleanup tests that verify only unreferenced generated-agent and artifact
  directories are selected and archived into a recoverable location;
- opt-in real Harbor subprocess smoke and cancel-recovery tests in
  `tests/python/test_real_harbor_cancel_recovery.py`, gated by
  `HARNESSLAB_REAL_HARBOR=1` and Docker availability;
- ruff and pyright for Python static gates;
- Vue typecheck, lint, unit tests, Storybook interaction tests, and Playwright
  smoke tests for the frontend;
- a line-count gate that fails when production source files exceed 500 lines.

The old Cargo registry remains a legacy reference. Current rewrite traceability
lives in `tests/WEB_REQUIREMENTS.toml`, `tests/WEB_TEST_REGISTRY.toml`, and
`scripts/test-after-change-web.sh`.

Operational note: backend restart tests should build state through public APIs,
then mutate only the persisted crash boundary under test. Do not trust in-memory
worker state after restart; SQLite run status plus Harbor artifacts are the
authoritative recovery inputs.

Operational note: API tests that need deterministic terminal results should call
`POST /api/experiments/{id}/run?wait=true`. Product-style tests should use the
default `wait=false` path and then observe state through queue, run, event, or
SSE APIs.

Operational note: TestClient-based tests must use the context-manager fixture so
FastAPI lifespan and app-level worker tasks share a stable event loop. Creating a
bare TestClient can let request-scoped event loop cleanup cancel background work
and produce false `interrupted` states.

Operational note: Real Harbor subprocess validation is intentionally opt-in.
Run it on a Docker-capable machine with
`HARNESSLAB_REAL_HARBOR=1 uv run pytest -m docker tests/python/test_real_harbor_cancel_recovery.py`.

Operational note: Docker orphan cleanup starts as a doctor/reporting gate, not
an automatic remover. The WebUI scans labelled `harnesslab.run_id` containers
and returns dry-run `docker rm -f` cleanup plans for manual review; execution
needs a separate product decision because container removal is not recoverable.

Operational note: Playwright e2e requires the browser cache for the installed
frontend Playwright version. If `npm --prefix frontend run e2e` fails with a
missing `chromium_headless_shell-*` executable, refresh the local browser cache
with `npm --prefix frontend exec playwright install chromium` and rerun the
full gate.

Operational note: Vitest should run with `--pool threads --maxWorkers=1` for the
current small frontend suite. The default fork pool can time out while starting
workers on this local environment before any test file runs, which makes the
gate flaky without increasing coverage.
