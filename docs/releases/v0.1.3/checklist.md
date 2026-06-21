# OrnnLab Release And Rollback Checklist

## Document Control

| Document Version | Engineering Version(s) | Updated | Change |
|---|---|---|---|
| 1.0 | Python app `0.2.0`; Harbor `0.13.x` | 2026-06-15 | Defined WebUI release and rollback gate. |
| 1.1 | `ornnlab` npm `0.1.3`; Python app `0.2.0` | 2026-06-16 | Added version-governance and release-ledger requirements. |
| 1.2 | OrnnLab Build Set `2026.06.16`; `ornnlab` npm `0.1.3`; Python app `0.2.0` | 2026-06-16 | Added Build Set confirmation to release gate. |
| 1.3 | OrnnLab Build Set `2026.06.16`; `ornnlab` npm `0.1.3`; Python app `0.2.0` | 2026-06-16 | Added release branch and worktree confirmation. |
| 1.4 | OrnnLab Build Set `2026.06.16`; `ornnlab` npm `0.1.3`; Python app `0.2.0` | 2026-06-16 | Added required version-folder document bundle checks. |
| 1.5 | OrnnLab Build Set `2026.06.16`; `ornnlab` npm `0.1.3`; Python app `0.2.0` | 2026-06-16 | Added docs-root convergence check. |
| 1.6 | OrnnLab Build Set `2026.06.16`; `ornnlab` npm `0.1.3`; Python app `0.2.0` | 2026-06-16 | Aligned release checklist with the `docs-manager` version-doc trio. |

Use this checklist for Harbor WebUI rewrite releases.

## Pre-Release Gate

- Confirm `docs/plans/2026-06-15-harbor-webui-redesign-engineering-plan.md`
  has current implementation ledger evidence.
- Confirm version changes follow `docs/release/version-governance.md`.
- Confirm `docs/` has no Markdown files directly under the root; active
  entrypoint, current docs, and legacy stubs must live in subdirectories.
- Confirm the matching `docs/v<version>/` folder contains exactly:
  `prd.md`, `technical-design.md`, and `engineering-plan.md`.
- Confirm the version PRD is scoped to this version only and is not a total
  product PRD.
- Confirm the version PRD has `PRD Document Version` metadata and a PRD version
  history table.
- Confirm the version technical design cites the version PRD as its source.
- Confirm the version engineering plan cites both the version PRD and technical
  design.
- Confirm Build Set development is on an approved release/hotfix branch, not
  directly on `main`.
- Confirm parallel release, hotfix, or publish-verification work uses a dedicated
  worktree.
- Create or update the matching `docs/release/ornnlab-<version>.md` entry for
  every public artifact version change.
- Confirm the release ledger includes a Build Set composition table binding npm,
  Python app, frontend, transition package, Harbor range, and source commit.
- Confirm affected PRDs and technical docs updated their `Document Control`
  table with document version, engineering version, update date, and change.
- Run `uv sync --group dev`.
- Run `npm --prefix frontend ci`.
- Run `scripts/test-after-change-web.sh`.
- Run `git diff --check`.
- Confirm GitHub Actions default jobs pass:
  - Python Web Gate
  - Frontend Web Gate
- Run the opt-in real Harbor Docker smoke when release scope touches Harbor
  execution, cancellation, Docker cleanup, or result parsing.
- Confirm `uv run ornnlab doctor --logs` returns structured status and does
  not hide failed-run log paths.
- Confirm `uv run ornnlab backup export` succeeds before any migration test.
- Confirm `uv run ornnlab cleanup plan` reports only recoverable archive
  candidates.
- Confirm no production code file exceeds 500 lines.
- Confirm active README/quickstart docs do not contain stale literal package
  versions; prefer `npm install -g ornnlab` or `ornnlab@latest`.
- Confirm `git status --short --branch` is clean and synchronized with
  `origin/main`.

## Packaging Smoke

From a fresh checkout:

```bash
uv sync --group dev
uv run ornnlab --version
uv run ornnlab doctor
uv run ornnlab web --host 127.0.0.1 --port 8765
```

In a second shell:

```bash
npm --prefix frontend ci
npm --prefix frontend run typecheck
npm --prefix frontend run lint
npm --prefix frontend run test
npm --prefix frontend run storybook:test
npm --prefix frontend run e2e
```

## Rollback

OrnnLab local state is file and SQLite based. Rollback should preserve user
data before changing versions:

1. Stop the backend process.
2. Export a backup with `uv run ornnlab backup export`.
3. Record the archive path printed by the command.
4. Check for stale local artifacts with `uv run ornnlab cleanup plan`.
5. Move stale candidates with `uv run ornnlab cleanup archive` only when the
   plan is understood.
6. Revert the application version through git or package manager controls.
7. Start the backend and run `uv run ornnlab doctor --logs`.
8. If local state cannot be read, restore the backup into an empty OrnnLab
   home with `uv run ornnlab backup import <archive>`.

Do not delete `~/.ornnlab/data` as a rollback step. Move it to a dated backup
location if manual intervention is required.

## Release Blockers

- Default GitHub Actions jobs are failing.
- Required real Harbor smoke was skipped for a Harbor execution change.
- `doctor --logs` cannot surface the latest failed/interrupted run paths.
- Backup export/import fails.
- Cleanup requires irreversible deletion.
- Active docs contradict the Harbor WebUI architecture.
- Rust legacy crates are treated as active release artifacts without a new
  explicit decision.
