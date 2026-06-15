# OrnnLab Documentation

Start here for the active OrnnLab version documentation.

- Current version bundle: `../v0.1.3/README.md`
- Version PRD: `../v0.1.3/version-prd.md`
- Version technical design: `../v0.1.3/technical-design.md`
- Version engineering plan: `../v0.1.3/engineering-plan.md`
- Version release ledger: `../v0.1.3/release-ledger.md`
- Install quickstart: `../current/install-quickstart.md`
- Release and rollback checklist: `../current/release-checklist.md`
- Version and documentation governance: `../current/version-governance.md`
- Harbor upgrade procedure: `../current/harbor-upgrade-procedure.md`
- Rust legacy workspace decision: `../legacy/rust-legacy-fate.md`
- Harbor lifecycle spike: `../spikes/2026-06-15-harbor-lifecycle-spike.md`
- Legacy archive: `../archive/2026-06-15-pre-harbor-webui-redesign/README.md`

## Current Direction

OrnnLab is a local WebUI over Harbor. Harbor owns benchmark execution and raw
job artifacts. OrnnLab owns agent registration, experiment/run management,
diagnostics, report summaries, and leaderboard views.

Active product requirements are version-scoped. Do not create or maintain a
single total PRD as the source of truth; create the next version folder under
`docs/v<version>/` instead.

## Superseded Documents

Legacy Rust CLI/runtime guides, adapter plans, old playbooks, and historical
reviews have been moved to `../archive/2026-06-15-pre-harbor-webui-redesign/`.
Old paths that remain under `docs/` are short supersession stubs only. They
exist to keep old references resolvable while preventing stale Rust-runtime
guidance from being read as current direction.

## Current Implementation Entrypoints

- Backend package: `../../ornnlab/`
- Frontend package: `../../frontend/`
- Web gate: `../../scripts/test-after-change-web.sh`
- Web test registry: `../../tests/WEB_TEST_REGISTRY.toml`
- Release gate: `../current/release-checklist.md`
