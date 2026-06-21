#!/usr/bin/env bash
# OrnnLab npm publish script
#
# 自动读取 package.json 版本号，运行本地验证，然后发布到 npm registry。
# 用户只需在 npm login 和 npm publish 时完成 WebAuthn 安全密钥认证。
set -euo pipefail

repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
cd "$repo_root"

# 1. 读取版本号
version="$(node -p "require('./package.json').version")"
echo "==> Publishing ornnlab@$version"

# 2. 检查工作区干净
if ! git diff --check; then
  echo "ERROR: git diff --check failed. Commit or stash changes first." >&2
  exit 1
fi

if [ -n "$(git status --porcelain)" ]; then
  echo "ERROR: working tree is not clean. Commit or stash changes first." >&2
  exit 1
fi

# B4 修复：验证本地 commits 已推送到 origin/main，确保 npm 发布与 GitHub 源码一致
echo "==> Verifying origin/main sync..."
current_branch="$(git rev-parse --abbrev-ref HEAD)"
if [ "$current_branch" != "main" ]; then
  echo "ERROR: not on main branch (current: $current_branch). Publish must be from main." >&2
  exit 1
fi

git fetch origin main --quiet

ahead="$(git rev-list --count origin/main..HEAD)"
behind="$(git rev-list --count HEAD..origin/main)"

if [ "$ahead" != "0" ]; then
  echo "ERROR: local main is ahead of origin/main by $ahead commit(s)." >&2
  echo "       Push commits before publishing to keep npm/GitHub provenance aligned." >&2
  echo "       Run: git push" >&2
  exit 1
fi

if [ "$behind" != "0" ]; then
  echo "ERROR: local main is behind origin/main by $behind commit(s)." >&2
  echo "       Pull before publishing to ensure the tarball reflects the latest source." >&2
  echo "       Run: git pull --ff-only" >&2
  exit 1
fi

echo "  Branch in sync with origin/main ✓"

# 3. 运行本地验证
echo "==> Running local validation..."
npm run smoke:npm-bin
uv run python scripts/verify-ornnlab-rebrand.py
if [ -f scripts/verify-version-governance.py ]; then
  uv run python scripts/verify-version-governance.py
fi
echo "==> Local validation passed."

# 4. npm login（WebAuthn - 需要用户认证）
echo ""
echo "==> Refreshing npm web login..."
echo "    You will need to approve with your local security key."
echo ""
npm login --auth-type=web

# 5. npm publish（WebAuthn - 需要用户认证）
echo ""
echo "==> Publishing to npm registry..."
echo "    You will need to approve with your local security key again."
echo ""
npm publish --access public --auth-type=web

# 6. 发布后验证
echo ""
echo "==> Verifying publish..."
npm view ornnlab name version bin --json

actual_version="$(npm view ornnlab version)"
if [ "$actual_version" != "$version" ]; then
  echo "ERROR: registry version mismatch. Expected $version, got $actual_version." >&2
  exit 1
fi

echo "  Registry version: ornnlab@$actual_version ✓"

# 7. 清洁目录 npx 验证
echo "==> Clean-directory npx verification..."
tmpdir="$(mktemp -d)"
trap 'rm -rf "$tmpdir"' EXIT
cd "$tmpdir"
npx --yes ornnlab --version
npx --yes ornnlab --help | head -5

echo ""
echo "==> Publish complete: ornnlab@$version is live."
echo "    Registry: https://www.npmjs.com/package/ornnlab"
