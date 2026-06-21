# Rust Legacy Workspace Fate

Decision date: 2026-06-15
Retirement date: 2026-06-22

The Rust workspace has been **Retired in commit `699e23e` on 2026-06-22**.

## Final Status

- Cargo workspace (`Cargo.toml`, `Cargo.lock`, `crates/`, `xtask/`) — deleted.
- Rust toolchain files (`rust-toolchain.toml`, `rust-toolchain.coverage.toml`)
  — deleted.
- Rust coverage configuration (`coverage-critical.toml`) — deleted.
- Rust-only verification scripts (11 entries under `scripts/`) — deleted.
- See `docs/releases/v0.1.4/shim-retirement/harnesslab-shim-retirement-plan.md`
  Phase 3 for the deletion ledger.

The retirement is recoverable through git history. The original Rust workspace
content lives at the commits immediately preceding the Phase 3 commit on the
`main` branch.

## Why retired

The active product runtime is `ornnlab/` (Python/FastAPI + Vue) backed by
Harbor. The Rust workspace was kept as historical reference during the
HarnessLab → OrnnLab brand migration. Once the project owner confirmed
"the project has no released users and no real legacy data to preserve" in
2026-06-22, the legacy reference was retired together with the rest of the
HarnessLab compatibility layer to simplify the repository and remove search
noise.

## Historical Decision (2026-06-15) — superseded

The original 2026-06-15 decision was to "Keep `Cargo.toml`, `crates/`, and
`xtask/` in place for now" as legacy/reference. That stance was reversed
on 2026-06-22 by the shim-retirement PRD when the project owner approved
maximum-scope retirement.

## Guardrails (post-retirement)

- The OrnnLab Python codebase is the single source of truth.
- `pytest tests/python`, `npm run` gates in `frontend/`, and the verify
  scripts under `scripts/` form the only active CI gate.
- If Rust ever returns, it must come from a brand-new architecture decision,
  not from resurrecting this workspace.
