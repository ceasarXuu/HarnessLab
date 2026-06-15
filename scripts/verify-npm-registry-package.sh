#!/usr/bin/env bash
set -euo pipefail

tmpdir="$(mktemp -d)"
trap 'rm -rf "$tmpdir"' EXIT

repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
expected_version="$(cd "$repo_root" && node -p "require('./package.json').version")"

cd "$tmpdir"

actual_version="$(npx --yes ornnlab --version)"
if [[ "$actual_version" != "$expected_version" ]]; then
  printf 'registry version mismatch: expected %s, got %s\n' "$expected_version" "$actual_version" >&2
  exit 1
fi

npx --yes ornnlab --help | grep -q 'ornnlab setup'

ornnlab_status="$(
  curl -s -o /dev/null -w "%{http_code}" https://registry.npmjs.org/ornnlab
)"
if [[ "$ornnlab_status" != "200" ]]; then
  printf 'unexpected ornnlab registry status: %s\n' "$ornnlab_status" >&2
  exit 1
fi
