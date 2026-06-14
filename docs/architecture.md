# HarnessLab Architecture

The previous Rust runtime architecture was archived on 2026-06-15 because the
product direction changed to a Harbor-powered local WebUI.

- Archived copy: `docs/archive/2026-06-15-pre-harbor-webui-redesign/architecture.md`
- Canonical engineering plan: `docs/plans/2026-06-15-harbor-webui-redesign-engineering-plan.md`

Current architectural source of truth:

1. Harbor owns benchmark execution, environment lifecycle, agent execution, verifier execution, and raw job artifacts.
2. HarnessLab owns local product state, declarative agent registration, experiment management, report summaries, leaderboard, diagnostics, and WebUI.
3. The target implementation is Python/FastAPI + Vue 3 + SQLite metadata + file artifacts.

Rewrite this document from the canonical plan after Phase 1 proves the backend skeleton.
