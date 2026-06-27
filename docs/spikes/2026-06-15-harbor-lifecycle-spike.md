# Harbor Lifecycle Spike

## Document Control

| Document Version | Engineering Version(s) | Updated | Change |
|---|---|---|---|
| 1.0 | Python app `0.2.0`; Harbor `0.13.2` | 2026-06-15 | Recorded Harbor API and CLI lifecycle evidence for WebUI execution design. |
| 1.1 | `ornnlab` npm `0.1.3`; Python app `0.2.0` | 2026-06-16 | Linked spike evidence to document version governance. |

- Date: 2026-06-15
- Harbor version inspected: `0.13.2`
- Related plan: `../plans/2026-06-15-harbor-webui-redesign-engineering-plan.md`

## Summary

Local API introspection confirms Harbor `Job.create(config)` and `Job.run()` are
async coroutine functions. The inspected `Job` class does not expose a stable
top-level `cancel()` method in Harbor `0.13.2`.

`harbor run --help` also confirms `--config PATH` accepts JSON/YAML matching
`harbor.models.job.config:JobConfig`, so OrnnLab can persist one canonical
`harbor.config.json` artifact and use it for both Python API and future CLI or
subprocess execution boundaries.

This closes the Phase 1 API-shape question but keeps the product cancellation
contract in best-effort mode until real Docker cancellation and restart tests
prove cleanup behavior.

## Evidence

Command:

```bash
uv run python - <<'PY'
import inspect, json
from importlib import metadata
from harbor.job import Job

print(json.dumps({
    "harbor_version": metadata.version("harbor"),
    "Job.create_is_coroutine": inspect.iscoroutinefunction(Job.create),
    "Job.run_is_coroutine": inspect.iscoroutinefunction(Job.run),
    "Job_has_cancel": hasattr(Job, "cancel"),
    "Job_create_signature": str(inspect.signature(Job.create)),
    "Job_run_signature": str(inspect.signature(Job.run)),
}, indent=2, sort_keys=True))
PY
```

Observed:

```json
{
  "Job.create_is_coroutine": true,
  "Job.run_is_coroutine": true,
  "Job_create_signature": "(config: harbor.models.job.config.JobConfig) -> 'Job'",
  "Job_has_cancel": false,
  "Job_run_signature": "(self) -> harbor.models.job.result.JobResult",
  "harbor_version": "0.13.2"
}
```

## Decision

Phase 3 must keep `HarborEngine` isolated behind a lifecycle adapter and must
persist Harbor `JobConfig` before execution. The landed first adapter pass used:

- a deterministic local adapter for early tests, now superseded by managed
  subprocess execution as the default path;
- opt-in `python-api` adapter via `ORNNLAB_HARBOR_ENGINE=python-api`;
- opt-in real Docker smoke via `ORNNLAB_REAL_HARBOR=1`;
- explicit `supports_cancel=false` in capability snapshots.

Current runtime policy uses the managed Harbor subprocess path by default
because it owns the process group and can record cancellation cleanup evidence.

Phase 3 must not promise hard cancellation until
`tests/python/test_real_harbor_cancel_recovery.py` proves the following against
Docker:

- cancelling the backend task stops Harbor work or marks it `interrupted`;
- orphan Docker resources are discovered and cleaned up after restart;
- terminal status is written only after cleanup evidence exists;
- stale Harbor job directories reconcile deterministically.

If real cleanup cannot be proven, use a managed Harbor subprocess or worker
process boundary for execution while keeping Python API config construction.
