# OrnnLab Documentation

Start here for the active OrnnLab version documentation.

- Current version docs index: `../releases/v1.0.5/README.md`
- Version PRD: `../releases/v1.0.5/prd.md`
- Version technical design: `../releases/v1.0.5/technical-design.md`
- Version engineering plan: `../releases/v1.0.5/engineering-plan.md`
- Install quickstart: `../playbooks/install-quickstart.md`
- Release and rollback checklist: `../releases/v0.1.3/checklist.md`
- Harbor upgrade procedure: `../playbooks/harbor-upgrade-procedure.md`
- Rust legacy workspace decision: `../archive/stubs/rust-legacy-fate.md`
- Harbor lifecycle spike: `../spikes/2026-06-15-harbor-lifecycle-spike.md`
- Harbor WebUI frontend governance: `frontend-webui-governance.md`
- Harbor WebUI API contract: `frontend-api-contract.md`
- Legacy archive: `../archive/2026-06-15-pre-harbor-webui-redesign/README.md`

## Current Direction

OrnnLab is a local WebUI over Harbor. Harbor owns benchmark execution and raw
job artifacts. OrnnLab owns agent registration, experiment/run management,
diagnostics, report summaries, and leaderboard views.

The next product direction is v1.0.5 Harbor WebUI productization: OrnnLab Web
should cover Harbor's day-to-day job configuration, execution, observation,
artifact review, diagnostics, and recovery workflows.

Active product requirements are version-scoped. Do not create or maintain a
single total PRD as the source of truth; create the next version folder under
`docs/releases/v<version>/` instead.

## Superseded Documents

Legacy Rust CLI/runtime guides, adapter plans, old playbooks, and historical
reviews have been moved to `../archive/2026-06-15-pre-harbor-webui-redesign/`.
Old paths that remain under `docs/` are short supersession stubs only. They
exist to keep old references resolvable while preventing stale Rust-runtime
guidance from being read as current direction.

## Current Implementation Entrypoints

- Architecture diagram (v0.1.4): `architecture-v0.1.4.mmd` / `architecture-v0.1.4.png`
- Backend package: `../../ornnlab/`
- Frontend package: `../../frontend/`
- Web gate: `../../scripts/test-after-change-web.sh`
- Web test registry: `../../tests/WEB_TEST_REGISTRY.toml`
- Release gate: `../releases/v1.0.5/engineering-plan.md`
