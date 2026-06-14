# HarnessLab Technology Decisions

The previous Rust single-binary technology decision record was archived on
2026-06-15.

- Archived copy: `docs/archive/2026-06-15-pre-harbor-webui-redesign/technology-decisions.md`
- Canonical engineering plan: `docs/plans/2026-06-15-harbor-webui-redesign-engineering-plan.md`

Current MVP decisions:

| Area | Decision |
|---|---|
| Execution engine | Harbor 0.13.x |
| Backend | Python + FastAPI |
| Frontend | Vue 3 + TypeScript + Vite |
| Metadata | Local SQLite |
| Artifacts | TOML/JSON/JSONL/HTML files under `~/.harnesslab` |
| Live updates | Server-Sent Events for status/log streams |
| Packaging | Python package first; Rust binary is not the MVP path |

Expand this into a full decision record after Phase 1 validates the backend skeleton.
