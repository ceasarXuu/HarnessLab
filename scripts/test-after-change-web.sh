#!/bin/bash
set -euo pipefail

uv run ruff check ornnlab tests/python
uv run pyright
uv run pytest tests/python
uv run python scripts/verify-ornnlab-rebrand.py

uv run python - <<'PY'
from pathlib import Path

roots = [Path("ornnlab")]
violations = []
for root in roots:
    for path in root.rglob("*.py"):
        line_count = len(path.read_text(encoding="utf-8").splitlines())
        if line_count > 500:
            violations.append(f"{path}:{line_count}")
if violations:
    raise SystemExit("Files exceed 500 lines:\n" + "\n".join(violations))
PY

bash scripts/verify-harnesslab-transition-package.sh
bash scripts/verify-npm-reservation-package.sh

if [ -f frontend/package.json ]; then
  npm --prefix frontend run typecheck
  npm --prefix frontend run lint
  npm --prefix frontend run test
  npm --prefix frontend run storybook:test
  npm --prefix frontend run e2e
else
  echo "frontend/package.json absent: legacy Vue frontend removed; v1.0.5 rebuild pending."
fi

git diff --check
