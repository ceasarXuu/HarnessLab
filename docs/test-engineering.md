# HarnessLab WebUI Test Engineering

The Rust CLI test-engineering document was archived on 2026-06-15.

- Archived copy: `archive/2026-06-15-pre-harbor-webui-redesign/test-engineering.md`
- Current test strategy: `plans/2026-06-15-harbor-webui-redesign-engineering-plan.md#8-testing-strategy`

Current rewrite gates are Python/Web first:

- pytest for backend units and integration tests;
- fake HarborEngine tests for deterministic queue, recovery, and failure paths;
- optional Docker-marked Harbor smoke tests;
- ruff and pyright for Python static gates;
- Vue typecheck, lint, unit tests, Storybook interaction tests, and Playwright
  smoke tests for the frontend;
- a line-count gate that fails when production source files exceed 500 lines.

The old Cargo registry remains a legacy reference until Phase 1 creates the
`WEB-*` registry and `scripts/test-after-change-web.sh`.
