#!/usr/bin/env bash
set -euo pipefail

uv run ruff check harnesslab tests/python
uv run pyright
uv run pytest tests/python

uv run python - <<'PY'
from pathlib import Path

roots = [Path("harnesslab")]
violations = []
for root in roots:
    for path in root.rglob("*.py"):
        line_count = len(path.read_text(encoding="utf-8").splitlines())
        if line_count > 500:
            violations.append(f"{path}:{line_count}")
if violations:
    raise SystemExit("Files exceed 500 lines:\n" + "\n".join(violations))
PY

if [ -f frontend/package.json ]; then
  npm --prefix frontend run typecheck
  npm --prefix frontend run lint
  npm --prefix frontend run test
  npm --prefix frontend run storybook:test
  npm --prefix frontend run e2e
fi

git diff --check
