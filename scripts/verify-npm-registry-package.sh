#!/usr/bin/env bash
set -euo pipefail

tmpdir="$(mktemp -d)"
trap 'rm -rf "$tmpdir"' EXIT

repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
expected_version="$(cd "$repo_root" && node -p "require('./package.json').version")"

cd "$tmpdir"

actual_version="$(npx --yes @ceasarxuu/harnesslab --version)"
if [[ "$actual_version" != "$expected_version" ]]; then
  printf 'registry version mismatch: expected %s, got %s\n' "$expected_version" "$actual_version" >&2
  exit 1
fi

npx --yes @ceasarxuu/harnesslab --help | grep -q '@ceasarxuu/harnesslab'

unscoped_status="$(
  curl -s -o /dev/null -w "%{http_code}" https://registry.npmjs.org/harnesslab
)"
if [[ "$unscoped_status" != "404" ]]; then
  printf 'unexpected unscoped harnesslab registry status: %s\n' "$unscoped_status" >&2
  exit 1
fi
