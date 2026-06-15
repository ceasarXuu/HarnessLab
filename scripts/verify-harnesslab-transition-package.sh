#!/bin/bash
set -euo pipefail

pkg_dir="npm/harnesslab-transition"
tmpdir="$(mktemp -d)"

(cd "$pkg_dir" && npm pack --dry-run --json >/dev/null)
tarball="$(cd "$pkg_dir" && npm pack --pack-destination "$tmpdir" --silent)"
npm install --prefix "$tmpdir/install" "$tmpdir/$tarball" >/dev/null

"$tmpdir/install/node_modules/.bin/harnesslab" --version >/dev/null
"$tmpdir/install/node_modules/.bin/harnesslab" --help | grep -q 'ornnlab --help'

pack_json="$(cd "$pkg_dir" && npm pack --dry-run --json)"
if [[ "$pack_json" == *'"path": "bin/ornnlab.js"'* ]]; then
  echo "transition package must not ship the active ornnlab bin" >&2
  exit 1
fi
