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
- Preferred new product data home: `~/.ornnlab/data`
- Preferred npm launcher root: `~/.ornnlab/launcher`

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
- `~/.harnesslab` -> `~/.ornnlab/data`.
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

## Data Migration Contract

Do not treat the local home rename as a cosmetic path change. The first OrnnLab
runtime release must define deterministic behavior for users who already have
`~/.harnesslab`.

First-run discovery order:

1. If `ORNNLAB_HOME` is set, use it as the primary home.
2. If both `ORNNLAB_HOME` and `HARNESSLAB_HOME` are set, use `ORNNLAB_HOME` and
   emit a warning that `HARNESSLAB_HOME` was ignored.
3. If only `HARNESSLAB_HOME` is set, use that legacy path for one transition
   period and emit a deprecation warning.
4. If no env var is set and `~/.ornnlab/data/.ornnlab-home.json` exists, use
   `~/.ornnlab/data`.
5. If no env var is set, `~/.ornnlab` exists only because the npm launcher
   created a source checkout, and `~/.harnesslab` exists, run the explicit
   legacy-home migration path.
6. If neither product home exists, initialize `~/.ornnlab/data` and write
   `.ornnlab-home.json`.

Migration rules:

- Treat `~/.ornnlab/launcher` and `~/.ornnlab/data` as separate roots. The
  launcher may manage source checkouts; the runtime must never infer product
  data state from plain `~/.ornnlab` existence.
- Product-home discovery must use a marker file such as
  `.ornnlab-home.json`, not raw directory existence.
- Migration must be recoverable. Never delete or move `~/.harnesslab` during
  automatic migration.
- Before copying user data, create
  `~/.ornnlab/data/migration/ornnlab-home-migration.json` recording the legacy
  path, destination path, OrnnLab version, source file count, and action taken.
- Prefer copy-then-verify over in-place mutation. Only rewrite SQLite rows or
  artifact paths after a backup/export has succeeded.
- If SQLite or artifact records contain absolute paths under `~/.harnesslab`,
  rewrite them transactionally to `~/.ornnlab/data`. If rewrite validation
  fails, keep the old home readable and report the failed migration state.
- If migration fails, leave both homes readable and make `doctor` report the
  failed migration state and next command.
- Backup import must accept both old `harnesslab-backup-manifest.json` with
  `harnesslab_version` and new OrnnLab manifests.

Required migration observability:

- `doctor` / `/api/system/status` should expose `using_legacy_home`,
  `migrated_home`, `legacy_env_in_use`, `legacy_backup_manifest_imported`, and
  `migration_error` when relevant.
- Write a one-shot migration event under the new home so support can inspect
  what was detected and what action was taken.
- Emit warnings when old env vars, old Docker labels, or old backup manifests
  are used through compatibility paths.

## CLI And Packaging Compatibility Matrix

Decide and document every command surface before implementation.

| Surface | Target behavior | Compatibility requirement |
|---|---|---|
| `ornnlab` npm command | Primary user install/launch command | Must work from packed tarball and npm registry install |
| `ornnlab web` | Starts the OrnnLab backend | Must call `uv run ornnlab web` after Python CLI rename |
| `ornnlab doctor` | Runs OrnnLab diagnostics | Must report legacy env/home usage |
| `uv run ornnlab` | Primary Python console script | Must work after clean wheel/sdist install |
| `python -m ornnlab` | Primary module invocation | Requires `ornnlab/__main__.py` smoke coverage |
| `uv run harnesslab` | Optional compatibility alias | Keep only if explicitly supported for one transition period |
| `python -m harnesslab` | Optional compatibility alias | If retained, should warn and dispatch to OrnnLab |
| `npx @ceasarxuu/harnesslab` | Old scoped npm package path | Publish a scoped deprecation/redirect release that points to `ornnlab` |
| `bin/harnesslab.js` | Old shim | Exclude from root `ornnlab`; include only in scoped transition release |

Packaging proof must include `uv build`, clean venv install from wheel/sdist,
`python -m ornnlab`, `uv run ornnlab --version`, packed npm tarball install,
and `npx`/local bin smoke for `ornnlab`.

Old scoped npm package policy:

- Publish one transition release for `@ceasarxuu/harnesslab` from a separate
  staging manifest, not from the root `ornnlab` package.
- The transition release should keep the `harnesslab` bin and print a clear
  migration message with `npx ornnlab --help` and `npm install -g ornnlab`.
- Required proof:

```bash
npm view @ceasarxuu/harnesslab name version bin --json
npx --yes @ceasarxuu/harnesslab --help
npx --yes ornnlab --help
```

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
  - Change default checkout path from `~/.ornnlab/HarnessLab` to a launcher
    root such as `~/.ornnlab/launcher/source`.
  - Replace help text that says "HarnessLab source" or "HarnessLab doctor".
  - After Python CLI rename, run `uv run ornnlab web` and
    `uv run ornnlab doctor`.
- `bin/harnesslab.js`
  - Do not ship this file in the root `ornnlab` package.
  - Use it only from the separate `@ceasarxuu/harnesslab` transition-release
    staging manifest, with help text pointing to `ornnlab`.

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
  - Add or move `__main__.py` so `python -m ornnlab` works.
  - If `python -m harnesslab` is retained, keep a thin compatibility module
    that warns and delegates instead of duplicating runtime logic.
  - Keep migration helpers small and explicit instead of hiding compatibility
    behind silent fallbacks.
- `scripts/test-after-change-web.sh`
  - Update lint/typecheck roots from `harnesslab` to `ornnlab`.
  - Keep an explicit compatibility selector if the old package alias is tested.
- `tests/python/`
  - Rewrite imports to `ornnlab.*`.
  - Add coverage for old env var compatibility and old home migration.

### Runtime Data

- `Settings`
  - Default product home should become `~/.ornnlab/data`.
  - `ORNNLAB_HOME` should be primary.
  - `HARNESSLAB_HOME` can be accepted with a warning during transition.
  - Add deterministic first-run migration behavior from the Data Migration
    Contract above.
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
  - New exports should define whether legacy fields are retained, aliased, or
    replaced. Dual-read is required; dual-write should be time-bounded.
- Cleanup
  - Cleanup archive should move data under the OrnnLab home.
  - No irreversible deletion.

### Environment Variables

Classify env vars before renaming them.

Product/runtime vars to dual-read during transition:

- `HARNESSLAB_HOME` -> `ORNNLAB_HOME`
- `HARNESSLAB_HARBOR_ENGINE` -> `ORNNLAB_HARBOR_ENGINE`
- `HARNESSLAB_HARBOR_SUBPROCESS_COMMAND` -> `ORNNLAB_HARBOR_SUBPROCESS_COMMAND`
- `HARNESSLAB_DOCKER_COMMAND` -> `ORNNLAB_DOCKER_COMMAND`
- `HARNESSLAB_REAL_HARBOR` -> `ORNNLAB_REAL_HARBOR`
- `HARNESSLAB_REAL_HARBOR_AGENT` -> `ORNNLAB_REAL_HARBOR_AGENT`
- `HARNESSLAB_REAL_HARBOR_BENCHMARK` -> `ORNNLAB_REAL_HARBOR_BENCHMARK`
- `HARNESSLAB_REAL_HARBOR_BENCHMARK_VERSION` -> `ORNNLAB_REAL_HARBOR_BENCHMARK_VERSION`
- `HARNESSLAB_REAL_HARBOR_N_TASKS` -> `ORNNLAB_REAL_HARBOR_N_TASKS`
- `HARNESSLAB_REAL_HARBOR_CANCEL_DELAY` -> `ORNNLAB_REAL_HARBOR_CANCEL_DELAY`

Rules:

- New `ORNNLAB_*` vars win over old `HARNESSLAB_*` vars.
- Old vars must emit a warning in doctor/status when exercised.
- Internal Rust/test-only vars may remain `HARNESSLAB_*` while Rust legacy is
  frozen, but they must be explicitly labeled as legacy/test-only.
- Add tests for precedence and warning behavior for every product/runtime var.

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

### CI And Automation

- `.github/workflows/ci.yml`
  - Update Python roots from `harnesslab` to `ornnlab`.
  - Update opt-in real Harbor env vars when the runtime vars are renamed.
  - Keep compatibility CI coverage for old env vars if compatibility is
    supported.
- `scripts/test-after-change-web.sh`
  - Must match CI roots and package names.
- Add a rebrand verification script that writes
  `artifacts/rebrand/ornnlab-rebrand-verification.json` with command, status,
  version, package artifact path, migration fixture path, and timestamp fields.

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
- `prd/2026-06-15-ornnlab-npm-distribution.md`
- `docs/README.md`
- `docs/install-quickstart.md`
- `docs/release-checklist.md`
- `docs/development-operations.md`
- `docs/technology-decisions.md`
- `docs/harbor-upgrade-procedure.md`
- `docs/test-engineering.md`
- `docs/playbooks/npm-package-reservation.md`
- current engineering plans under `docs/plans/`

Active non-archive documentation inventory:

| Path | Action |
|---|---|
| `README.md` | rename now |
| `docs/README.md` | rename now |
| `docs/install-quickstart.md` | rename now |
| `docs/release-checklist.md` | rename now |
| `docs/development-operations.md` | rename now |
| `docs/technology-decisions.md` | rename now |
| `docs/harbor-upgrade-procedure.md` | rename now |
| `docs/test-engineering.md` | rename now |
| `docs/prd.md` | superseded stub |
| `docs/architecture.md` | superseded stub |
| `docs/mvp-development-spec.md` | superseded stub |
| `docs/rust-legacy-fate.md` | historical |
| `docs/adapter-protocol.md` | historical |
| `docs/agent-profile-reference.md` | historical |
| `docs/agent-registration-guide.md` | historical |
| `docs/architecture/benchmark-compatibility-strategy.md` | historical |
| `docs/architecture/harnesslab-vs-harbor.md` | historical |
| `docs/playbooks/npm-package-reservation.md` | rename now |
| `docs/playbooks/terminal-bench-claude-ds.md` | historical |
| `docs/reviews/2026-05-27-docker-runner-review-3.md` | historical |
| `docs/spikes/2026-06-15-harbor-lifecycle-spike.md` | rename now |
| `prd/2026-06-15-harnesslab-webui-prd.md` | rename now |
| `prd/2026-06-15-ornnlab-npm-distribution.md` | rename now |
| `prd/2026-06-07-universal-benchmark-adapter-protocol.md` | historical |
| `docs/plans/2026-06-15-ornnlab-rebrand-checklist.md` | rename now |
| `docs/plans/*` older than 2026-06-15 OrnnLab plan | historical unless linked from current README |

Add a doc-inventory guard that fails when any non-archive `docs/**/*.md` or
`prd/**/*.md` path is not represented in this table or a successor inventory.

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
uv build
uv run ruff check ornnlab tests/python
uv run pyright ornnlab tests/python
uv run pytest tests/python
uv run ornnlab --version
uv run ornnlab doctor
python -m ornnlab --version
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

For packaging release proof:

```bash
tmpdir="$(mktemp -d)"
npm pack --pack-destination "$tmpdir"
! npm pack --dry-run --json | rg 'bin/harnesslab.js'
npm --prefix "$tmpdir/install" install "$tmpdir"/ornnlab-*.tgz
"$tmpdir/install/node_modules/.bin/ornnlab" --version
uv build --out-dir "$tmpdir/dist"
python -m venv "$tmpdir/venv"
"$tmpdir/venv/bin/python" -m pip install "$tmpdir"/dist/*.whl
"$tmpdir/venv/bin/ornnlab" --version
"$tmpdir/venv/bin/python" -m ornnlab --version
```

Run the existing CI-equivalent gates:

```bash
scripts/test-after-change-web.sh
npm --prefix frontend run storybook:test
npm --prefix frontend run e2e
```

Add or update tests for:

- old `HARNESSLAB_HOME` warning path
- new `ORNNLAB_HOME` primary path
- old/new env var precedence for Harbor and Docker command vars
- old backup manifest import
- old Docker label orphan scan
- new Docker label orphan scan
- generated Harbor job name prefix
- `python -m ornnlab`
- optional `python -m harnesslab` and `uv run harnesslab` compatibility paths
- old scoped npm package help/deprecation path if a compatibility release is
  published
- launcher-root-only `~/.ornnlab/launcher` plus populated `~/.harnesslab`
  migration fixture
- migration rollback from a populated legacy home

## Suggested Execution Order

1. Freeze compatibility policy for old Python and npm entrypoints.
2. Product-visible docs and npm metadata.
3. Python package, `__main__.py`, and CLI alias migration.
4. Runtime data migration contract and environment variable compatibility.
5. Docker label and Harbor job prefix migration.
6. CI, scripts, and release proof updates.
7. Frontend package and visible copy.
8. Optional Rust legacy rename only if the Rust binary becomes a shipping target.

Each step should include focused tests and a small commit.
