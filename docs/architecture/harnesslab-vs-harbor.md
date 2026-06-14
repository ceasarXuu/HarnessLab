# Superseded: HarnessLab vs Harbor Architecture Comparison

This architecture comparison draft was archived on 2026-06-15.

- Archived copy: `docs/archive/2026-06-15-pre-harbor-webui-redesign/architecture/harnesslab-vs-harbor.md`
- Canonical plan: `docs/plans/2026-06-15-harbor-webui-redesign-engineering-plan.md`

The active decision is simpler than this draft: HarnessLab does not build a
parallel runtime. Harbor owns benchmark execution; HarnessLab owns the local
product layer.
