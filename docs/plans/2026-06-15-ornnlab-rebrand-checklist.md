# OrnnLab Rebrand Checklist

Date: 2026-06-15

## Purpose

This checklist gathers every known HarnessLab to OrnnLab rename surface into one
reviewable document. The goal is to make the rebrand fast to audit without
losing compatibility for existing local data, published npm packages, old
artifacts, and legacy Rust benchmark work.

Target naming:

- Product brand: OrnnLab
- npm package: `ornnlab`
- npm command: `ornnlab`
- Preferred new Python command: `ornnlab`
- Preferred new local home: `~/.ornnlab`

## Current State

The repository is already partially renamed:

- Root npm package name is `ornnlab`.
- Root npm bin maps `ornnlab` to `bin/ornnlab.js`.
- `bin/ornnlab.js` exists and uses `ORNNLAB_*` launcher variables.

The main product implementation still uses HarnessLab names:

- Python package and imports are under `harnesslab/`.
- Python project name and console script are `harnesslab`.
- The default local data directory is `~/.harnesslab`.
- FastAPI app title, report title, README, current PRD, release docs, and tests
  still use HarnessLab wording.
- Docker labels, event files, backup artifacts, and some Harbor job names still
  use `harnesslab`.
- The Rust workspace is legacy/reference and still uses `harnesslab-*` crate,
  binary, adapter, and test names.

## Rename Boundaries

Use three separate buckets instead of a global search-and-replace.

### Must Rename For The Product

These are user-visible or publish-visible and should move to OrnnLab in the
first rebrand pass:

- Root README title, first paragraph, install instructions, npm instructions,
  and local data references.
- Current WebUI PRD and current engineering plan headings/body references.
- npm metadata description, repository URL, homepage, and bugs URL when the
  GitHub repository is renamed.
- `bin/ornnlab.js` help text, default source checkout path, and internal command
  calls once the Python console script is renamed.
- FastAPI app title.
- CLI `prog`, help text, version text, and command examples.
- Generated report HTML title and heading.
- Frontend package name and any visible UI copy.
- Install quickstart, release checklist, development operations, and npm
  publish playbook.

### Rename With Compatibility

These affect user data, automation, or cleanup logic. Add new OrnnLab names, but
keep old HarnessLab names readable during a transition period.

- `HARNESSLAB_HOME` -> `ORNNLAB_HOME`.
- `~/.harnesslab` -> `~/.ornnlab`.
- `harnesslab.sqlite` -> `ornnlab.sqlite`.
- `harnesslab-events.jsonl` -> `ornnlab-events.jsonl`.
- `harnesslab-backup-*` and `harnesslab-backup-manifest.json` -> OrnnLab names.
- `harnesslab.run_id` Docker label -> `ornnlab.run_id`.
- Docker orphan scan response field `harnesslab_orphans` -> `ornnlab_orphans`.
- Harbor generated job names such as `harnesslab-terminal-bench-2-0`.
- `HARNESSLAB_HARBOR_ENGINE`, `HARNESSLAB_HARBOR_SUBPROCESS_COMMAND`,
  `HARNESSLAB_REAL_HARBOR`, and related test/runtime environment variables.

Compatibility requirements:

- New variables should win when both old and new are set.
- Doctor/status output should warn when only old variables or old home paths are
  used.
- Cleanup and orphan scan must continue to detect old Docker labels.
- Backup import should accept old HarnessLab backup manifests.
- Migration should be recoverable and should not delete old data.

### Keep Or Archive

These should not be renamed in the first product rebrand unless there is a
separate legacy cleanup plan:

- `docs/archive/**`
- `vs_review/**`
- `coe/**`
- historical benchmark artifacts under `.benchmarks/**`
- historical job outputs under `jobs/**`
- old review file names
- old PRD file names that are explicitly archived

For historical material, prefer a short note that HarnessLab was the previous
project name. Do not rewrite evidence logs or adversarial review records.

## File And Directory Checklist

### npm Launcher

- `package.json`
  - Confirm `"name": "ornnlab"`.
  - Keep `bin.ornnlab = "bin/ornnlab.js"`.
  - Update description from "OrnnLab npm launcher for the HarnessLab Harbor
    WebUI" to OrnnLab-only wording.
  - Update repository/homepage/bugs after GitHub repository rename.
- `bin/ornnlab.js`
  - Change default repository URL after repository rename.
  - Change default checkout path from `~/.ornnlab/HarnessLab` to a stable
    OrnnLab path such as `~/.ornnlab/OrnnLab` or `~/.ornnlab/source`.
  - Replace help text that says "HarnessLab source" or "HarnessLab doctor".
  - After Python CLI rename, run `uv run ornnlab web` and
    `uv run ornnlab doctor`.
- `bin/harnesslab.js`
  - Decide whether this remains a compatibility shim for the old scoped package
    or is excluded from future npm publishes.

### Python Package

- `pyproject.toml`
  - Change project name to `ornnlab`.
  - Add `ornnlab = "ornnlab.cli:main"` under `[project.scripts]`.
  - Decide whether to temporarily keep
    `harnesslab = "ornnlab.cli:main"` as a compatibility alias.
  - Update pyright include from `harnesslab` to `ornnlab`.
- `harnesslab/`
  - Rename package directory to `ornnlab/`.
  - Rewrite imports from `harnesslab.*` to `ornnlab.*`.
  - Keep migration helpers small and explicit instead of hiding compatibility
    behind silent fallbacks.
- `tests/python/`
  - Rewrite imports to `ornnlab.*`.
  - Add coverage for old env var compatibility and old home migration.

### Runtime Data

- `Settings`
  - Default home should become `~/.ornnlab`.
  - `ORNNLAB_HOME` should be primary.
  - `HARNESSLAB_HOME` can be accepted with a warning during transition.
- SQLite
  - New default file should be `ornnlab.sqlite`.
  - Existing `harnesslab.sqlite` should be detected and migrated or reused
    through an explicit compatibility path.
- Events
  - New event file should be `ornnlab-events.jsonl`.
  - Old `harnesslab-events.jsonl` should remain readable.
- Backup
  - New backup prefix and manifest should use OrnnLab.
  - Import should accept old HarnessLab backup manifests.
- Cleanup
  - Cleanup archive should move data under the OrnnLab home.
  - No irreversible deletion.

### Docker And Harbor

- Docker labels
  - New label: `ornnlab.run_id`.
  - Old label: `harnesslab.run_id`, scanned for compatibility.
- Docker orphan API/status
  - Rename visible field to `ornnlab_orphans`.
  - Consider keeping old field for one compatibility release only if clients
    already consume it.
- Harbor job names
  - New generated job names should use `ornnlab-*`.
  - Tests should assert the new prefix.
  - Old job directories should remain visible in history.

### Frontend

- `frontend/package.json`
  - Rename package from `@ceasarxuu/harnesslab-frontend` to an OrnnLab name.
- UI copy
  - Replace visible HarnessLab brand text with OrnnLab.
  - Keep Harbor references when describing the execution engine.
- Storybook
  - Update stories if they show product names or package names.

### Rust Legacy Workspace

Current Rust code is treated as a legacy/reference asset for the Harbor WebUI
direction. Do not include Rust crate renames in the first pass unless the Rust
binary is being reactivated as a shipping product.

If a later Rust rename is approved, handle it as a separate migration:

- `crates/harnesslab-*` directory names.
- Cargo package names.
- Rust imports such as `harnesslab_core`.
- CLI binary `harnesslab`.
- Adapter IDs such as `harnesslab.terminal-bench.runtime`.
- Docker labels and test fixtures in Rust tests.
- `scripts/test-after-change.sh` package selectors.

This pass will be high risk because many tests and artifact contracts encode
these names.

## Documentation Checklist

Update current docs:

- `README.md`
- `prd/2026-06-15-harnesslab-webui-prd.md`
- `docs/install-quickstart.md`
- `docs/release-checklist.md`
- `docs/development-operations.md`
- `docs/playbooks/npm-package-reservation.md`
- current engineering plans under `docs/plans/`

Do not bulk rewrite:

- archived docs
- review records
- COE/debug records
- generated benchmark artifacts
- historical job outputs

## Validation Checklist

Run targeted checks after each rename phase.

For npm launcher changes:

```bash
npm run smoke:npm-bin
npm pack --dry-run
node bin/ornnlab.js --help
node bin/ornnlab.js --version
```

For Python package and CLI changes:

```bash
uv run ruff check ornnlab tests/python
uv run pyright ornnlab tests/python
uv run pytest tests/python
uv run ornnlab --version
uv run ornnlab doctor
```

For frontend changes:

```bash
npm --prefix frontend run lint
npm --prefix frontend run typecheck
npm --prefix frontend run test
npm --prefix frontend run build
```

For compatibility:

```bash
HARNESSLAB_HOME=/tmp/old-harnesslab-home uv run ornnlab doctor
ORNNLAB_HOME=/tmp/new-ornnlab-home uv run ornnlab doctor
```

Add or update tests for:

- old `HARNESSLAB_HOME` warning path
- new `ORNNLAB_HOME` primary path
- old backup manifest import
- old Docker label orphan scan
- new Docker label orphan scan
- generated Harbor job name prefix

## Suggested Execution Order

1. Product-visible docs and npm metadata.
2. Python package and CLI alias migration.
3. Runtime data path and environment variable compatibility.
4. Docker label and Harbor job prefix migration.
5. Frontend package and visible copy.
6. Optional Rust legacy rename only if the Rust binary becomes a shipping target.

Each step should include focused tests and a small commit.
