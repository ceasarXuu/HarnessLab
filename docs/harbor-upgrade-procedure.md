# Harbor Upgrade Procedure

HarnessLab pins Harbor as `harbor>=0.13,<0.14`. Any Harbor dependency bump must
go through this procedure before merging.

## Scope

This procedure applies to changes in:

- `pyproject.toml`
- `uv.lock`
- `harnesslab/services/harbor_engine.py`
- `harnesslab/services/profile_compiler.py`
- real Harbor smoke tests
- docs or scripts that describe Harbor CLI/API behavior

## Required Checks

1. Inspect the target Harbor release notes and identify changes to `JobConfig`,
   `AgentConfig`, `DatasetConfig`, `EnvironmentConfig`, `Job.create`,
   `Job.run`, CLI `harbor run`, result layout, and job log layout.
2. Update the version constraint in `pyproject.toml`.
3. Run `uv lock`.
4. Re-run Harbor API compatibility tests:

   ```bash
   uv run pytest tests/python/test_harbor_engine.py tests/python/test_profile_compiler.py -vv
   ```

5. Run the full local WebUI gate:

   ```bash
   scripts/test-after-change-web.sh
   ```

6. On a Docker-capable machine, run:

   ```bash
   HARNESSLAB_REAL_HARBOR=1 uv run pytest -m docker tests/python/test_real_harbor_cancel_recovery.py
   ```

7. Inspect a produced Harbor job directory and confirm these files still exist
   or the parser/tests were updated intentionally:
   - `config.json`
   - `result.json`
   - `job.log`
   - trial artifact directories
8. Update `docs/technology-decisions.md`, this procedure, and the engineering
   plan ledger with the new Harbor version and evidence.

## Failure Policy

If Harbor changes cancellation, result parsing, job directory layout, or
generated-agent import behavior, stop the upgrade and keep the existing Harbor
pin until HarnessLab has a dedicated compatibility patch.

Do not add silent compatibility fallbacks. Add explicit version-aware behavior,
tests, and doctor diagnostics.
