#!/bin/bash
set -euo pipefail

uv run ruff check ornnlab tests/python
uv run pyright
uv run pytest tests/python
uv run python scripts/verify-ornnlab-rebrand.py

uv run python scripts/check-python-file-length.py ornnlab

bash scripts/verify-harnesslab-transition-package.sh
bash scripts/verify-npm-reservation-package.sh

if [ -f frontend/package.json ]; then
  npm --prefix frontend run typecheck
  npm --prefix frontend run lint
  npm --prefix frontend run test
  npm --prefix frontend run build
  npm run check:webui-bundle
  node scripts/test-webui-build-config.js
  npm --prefix frontend run storybook:test
  npm --prefix frontend run storybook:build
  npm run test:launcher
  bash scripts/test-run-dev-api.sh
else
  echo "frontend/package.json absent: legacy Vue frontend removed; v1.0.5 rebuild pending."
fi

git diff --check
