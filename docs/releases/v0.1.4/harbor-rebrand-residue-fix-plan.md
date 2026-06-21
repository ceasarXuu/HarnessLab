# v0.1.4 Work Item: Harbor Rebrand Residue Fix Plan

## Document Control

| Document Version | Engineering Version(s) | Updated | Change |
|---|---|---|---|
| 1.0 | OrnnLab Build Set (planned `2026.06.22`); `ornnlab` npm `0.1.4` (planned); Python app `0.2.0` | 2026-06-22 | Created the Harbor rebrand residue fix plan as a v0.1.4 work item. |

- Status: Drafted, awaiting v0.1.4 PRD/technical-design/engineering-plan to bind this work item.
- Parent version folder: `docs/releases/v0.1.4/`
- Source audit: see "Audit Evidence" section below. The audit was produced on
  2026-06-22 against the working tree after the Harbor + Python/Vue rewrite
  superseded the self-built Rust runtime.

## Goal

Finish the Harbor rebrand / re-architecture so that the repository no longer
contains residual references to the previous `docs/release/` documentation root
or to the legacy `HarnessLab` product brand in user-visible surfaces.

Out of scope for this work item:

- The Rust legacy workspace under `crates/`, `xtask/`, `integrations/`,
  `Cargo.toml`. Its fate is governed by
  `docs/archive/stubs/rust-legacy-fate.md` and must not change here.
- The intentional `harnesslab` compatibility shims listed in
  `docs/plans/2026-06-15-ornnlab-rebrand-checklist.md` under
  "Rename With Compatibility". They are kept on purpose during the migration
  window.
- A general docs rewrite. This plan only fixes residue created by the
  `docs/release/` -> `docs/releases/v<version>/` migration and the
  HarnessLab -> OrnnLab UI/brand pass.

## Background

OrnnLab finished a major refactor: the self-built Rust runtime was retired and
Harbor became the execution engine. The product is now Python/FastAPI +
Vue3 + Harbor. The Rust workspace is preserved as a legacy reference per
`docs/archive/stubs/rust-legacy-fate.md`.

During v0.1.3, version governance (`docs/releases/v0.1.3/version-governance.md`)
moved release / version files from `docs/release/` (singular, flat) to
`docs/releases/v<version>/` (per-version folder, plural root). The migration
moved the files but did not finish updating links, scripts, and README
references. Several "must rename for the product" UI surfaces also still show
the old brand.

## Audit Evidence

### A. Path drift residue (`docs/release/` -> `docs/releases/v0.1.3/`)

The following files still reference `docs/release/...` which no longer exists.
Verified on 2026-06-22 via `rg "docs/release/"` and confirmed against the
actual filesystem `docs/releases/v0.1.3/`:

- `README.md` lines 8, 13, 49 (Version document index, Release and rollback
  checklist, version governance pointer).
- `scripts/verify-version-governance.py` `ALLOWED_LITERAL_FILES` entries.
- `scripts/verify-ornnlab-rebrand.py` `DOC_INVENTORY` and
  `DOC_CONTROL_REQUIRED` entries.
- `docs/v0.1.3/engineering-plan.md` Phase 4 acceptance text.
- `docs/v0.1.3/technical-design.md` release evidence pointer.
- `docs/releases/v0.1.3/ornnlab-0.1.3.md` "Documentation Updated" section.
- `docs/releases/v0.1.3/checklist.md` pre-release gate references.
- `docs/releases/v0.1.3/2026-06-16-ornnlab-0.1.3.md` documentation list.
- `docs/releases/v0.1.3/version-governance.md` references to old release root.
- `docs/playbooks/install-quickstart.md` release status pointer.

Impact: `scripts/verify-version-governance.py` and
`scripts/verify-ornnlab-rebrand.py` exercise stale paths, so the rebrand /
governance gate cannot detect real drift. README links 404 for users.

### B. User-visible brand residue (HarnessLab still visible in WebUI)

- `frontend/src/components/AppShell.vue` line 40 hardcodes the sidebar
  eyebrow as `HarnessLab` while the product is now OrnnLab.
- `frontend/src/api/client.ts` line 134 exports `harnessLabApi`. This is not
  user visible but is inconsistent with the active brand and other OrnnLab
  symbol names.

Impact: Brand drift between user-visible WebUI and README. The rebrand
checklist marks visible UI copy as "must rename for the product".

### C. Intentionally retained (do NOT change in this work item)

These are confirmed compatibility shims per
`docs/plans/2026-06-15-ornnlab-rebrand-checklist.md` "Rename With
Compatibility":

- `ornnlab/settings.py` (`LEGACY_HOME`, `HARNESSLAB_HOME`, legacy SQLite name,
  first-run migration).
- `ornnlab/services/doctor_service.py` (`RUNTIME_ENV_PAIRS` mapping,
  `harnesslab_orphans` dual-field).
- `ornnlab/services/harbor_subprocess.py`,
  `ornnlab/services/harbor_engine.py` (legacy `HARNESSLAB_HARBOR_*` envs).
- `ornnlab/services/backup_service.py`,
  `ornnlab/services/docker_orphan_service.py` (legacy manifest, run label,
  alias method).
- `harnesslab/` Python package as deprecated import alias.
- `bin/harnesslab.js`, `npm/harnesslab-transition/` scoped reservation.
- `pyproject.toml` `harnesslab` console script alias.
- Rust legacy workspace (`crates/`, `xtask/`, `Cargo.toml`,
  `integrations/terminal_bench/`).
- `docs/archive/**`, `coe/**`, `vs_review/**` history.

## Fix Plan

### Phase 1: Path drift gate first

Rationale: fix the validation scripts before touching the prose so that the
prose changes can be verified by `scripts/verify-version-governance.py` and
`scripts/verify-ornnlab-rebrand.py`.

Tasks:

- Update `scripts/verify-version-governance.py` `ALLOWED_LITERAL_FILES` so
  every `docs/release/...` entry becomes `docs/releases/v0.1.3/...` (or the
  active version folder if v0.1.4 has its own release ledger).
- Update `scripts/verify-ornnlab-rebrand.py` `DOC_INVENTORY` and
  `DOC_CONTROL_REQUIRED` sets the same way.
- Run `uv run python -m py_compile scripts/verify-version-governance.py` and
  `uv run python -m py_compile scripts/verify-ornnlab-rebrand.py`.
- Run `uv run python scripts/verify-ornnlab-rebrand.py` and
  `uv run python scripts/verify-version-governance.py` and expect them to
  pass on the new paths.

Acceptance:

- Both verify scripts exit 0.
- Both scripts contain zero remaining `docs/release/` literals.

### Phase 2: Active docs and README link repair

Tasks:

- `README.md`: replace each `docs/release/...` with the actual
  `docs/releases/v0.1.3/...` path (or `docs/releases/v<active>/...` when the
  active release ledger advances to v0.1.4). Keep wording minimal.
- `docs/v0.1.3/engineering-plan.md`: replace the Phase 4 reference.
- `docs/v0.1.3/technical-design.md`: replace the release-evidence pointer.
- `docs/playbooks/install-quickstart.md`: replace the release status
  pointer.
- `docs/releases/v0.1.3/checklist.md`: replace gate references.
- `docs/releases/v0.1.3/ornnlab-0.1.3.md`: replace documentation list
  entries.
- `docs/releases/v0.1.3/2026-06-16-ornnlab-0.1.3.md`: replace documentation
  list entries.
- `docs/releases/v0.1.3/version-governance.md`: rewrite the "release ledgers
  live in `docs/release/`" passage to describe the per-version folder rule
  that v1.5 already introduced, so the prose matches the on-disk reality.

Acceptance:

- `rg "docs/release/" -g '!docs/archive/**' -g '!docs/plans/**' -g '!vs_review/**' -g '!coe/**'`
  returns no results in active surfaces.
- Manually open every README link and confirm it resolves on disk.

### Phase 3: User-visible brand residue

Tasks:

- `frontend/src/components/AppShell.vue` line 40: change the eyebrow text
  from `HarnessLab` to `OrnnLab`. No CSS or layout change.
- `frontend/src/api/client.ts`: rename the exported symbol `harnessLabApi`
  to `ornnLabApi`. Update each call site inside `frontend/src/**`.
- Re-run frontend gates:
  - `npm --prefix frontend run typecheck`
  - `npm --prefix frontend run lint`
  - `npm --prefix frontend run test`
  - `npm --prefix frontend run build`
- Optionally re-run `npm --prefix frontend run storybook:test` since the
  Storybook KPI story imports nothing renamed but the smoke test is cheap.

Acceptance:

- WebUI sidebar shows `OrnnLab`.
- `rg "HarnessLab|harnessLab|harnesslabApi" frontend/src` returns no
  results.
- All frontend gates pass.

### Phase 4: Bind to v0.1.4 release plan

Tasks:

- Add a short "Harbor rebrand residue closed" bullet in the future
  `docs/releases/v0.1.4/ornnlab-0.1.4.md` release ledger when v0.1.4 is
  cut. Do not pre-create the ledger here.
- Update `docs/releases/v0.1.4/checklist.md` (when created) so the pre-
  release gate also greps for `docs/release/` and `HarnessLab` to prevent
  the same drift returning.
- Note this work item under the v0.1.4 engineering plan once the v0.1.4
  PRD / technical-design / engineering-plan trio is approved separately.

Acceptance:

- v0.1.4 release artifacts, once created, point to this plan.
- The Build Set composition table records the commit that closes residue.

## Validation Commands

```bash
uv run python scripts/verify-version-governance.py
uv run python scripts/verify-ornnlab-rebrand.py
uv run python -m py_compile scripts/verify-version-governance.py
uv run python -m py_compile scripts/verify-ornnlab-rebrand.py
npm --prefix frontend run typecheck
npm --prefix frontend run lint
npm --prefix frontend run test
npm --prefix frontend run build
git diff --check
```

## Rollback

Each phase is a separate commit on a v0.1.4 release branch (see
`docs/releases/v0.1.3/version-governance.md` Release Branch policy):

- Phase 1: revert script changes only; no user-visible impact.
- Phase 2: revert README/docs link changes; restores broken links but
  preserves on-disk files (files were not moved by this work item).
- Phase 3: revert two frontend edits; UI returns to `HarnessLab` eyebrow
  and `harnessLabApi` export.

No data, no SQLite schema, no Harbor config, no Docker label is touched by
this plan, so rollback is purely a `git revert` per phase.

## Risk Notes

- Verify scripts must be fixed before the docs sweep, otherwise the docs
  sweep cannot be gate-checked.
- `docs/archive/**`, `docs/plans/**`, `vs_review/**`, and `coe/**` keep
  historical `HarnessLab` and `docs/release/` literals on purpose. The
  audit greps must always exclude these roots.
- Do NOT rename, move, or delete anything under `crates/`, `xtask/`,
  `integrations/terminal_bench/`, or `Cargo.toml`. The Rust workspace is
  legacy reference and is governed by a separate decision document.
- Do NOT delete the `harnesslab/` Python package, the `harnesslab` console
  script, the `bin/harnesslab.js` reservation, the
  `npm/harnesslab-transition/` package, or any `HARNESSLAB_*` env shim.
  These are migration-window compatibility shims and have their own
  retirement schedule (currently unscheduled).

## Open Decisions

- Whether to schedule retirement of the HarnessLab compatibility shims in
  v0.1.4 or defer them. This plan does not decide it.
- Whether v0.1.4 will publish a release ledger
  `docs/releases/v0.1.4/ornnlab-0.1.4.md`. If yes, Phase 4 binds this work
  to that ledger; if no, this plan remains a standalone work-item record.
