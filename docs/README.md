# OrnnLab Documentation

Start here for the active Harbor WebUI redesign.

- Current PRD: `../prd/2026-06-15-ornnlab-webui-prd.md`
- Current engineering plan: `plans/2026-06-15-harbor-webui-redesign-engineering-plan.md`
- Install quickstart: `install-quickstart.md`
- Release and rollback checklist: `release-checklist.md`
- Version and documentation governance: `version-governance.md`
- Harbor upgrade procedure: `harbor-upgrade-procedure.md`
- Rust legacy workspace decision: `rust-legacy-fate.md`
- Harbor lifecycle spike: `spikes/2026-06-15-harbor-lifecycle-spike.md`
- Legacy archive: `archive/2026-06-15-pre-harbor-webui-redesign/README.md`

## Current Direction

OrnnLab is a local WebUI over Harbor. Harbor owns benchmark execution and raw
job artifacts. OrnnLab owns agent registration, experiment/run management,
diagnostics, report summaries, and leaderboard views.

## Superseded Documents

Legacy Rust CLI/runtime guides, adapter plans, old playbooks, and historical
reviews have been moved to `archive/2026-06-15-pre-harbor-webui-redesign/`.
Old paths that remain under `docs/` are short supersession stubs only. They
exist to keep old references resolvable while preventing stale Rust-runtime
guidance from being read as current direction.

## Current Implementation Entrypoints

- Backend package: `../ornnlab/`
- Frontend package: `../frontend/`
- Web gate: `../scripts/test-after-change-web.sh`
- Web test registry: `../tests/WEB_TEST_REGISTRY.toml`
- Release gate: `release-checklist.md`
