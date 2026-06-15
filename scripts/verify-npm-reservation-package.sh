#!/usr/bin/env bash
set -euo pipefail

node --check bin/ornnlab.js

expected_version="$(node -p "require('./package.json').version")"
actual_version="$(node bin/ornnlab.js --version)"
if [[ "$actual_version" != "$expected_version" ]]; then
  printf 'version mismatch: expected %s, got %s\n' "$expected_version" "$actual_version" >&2
  exit 1
fi

node bin/ornnlab.js --help | grep -q 'ornnlab                    Set up if needed'
node bin/ornnlab.js --help | grep -q 'ORNNLAB_FRONTEND_PORT'
node bin/ornnlab.js --help | grep -q 'Frontend: http://'
node bin/ornnlab.js path >/dev/null

if node bin/ornnlab.js unexpected-command >/dev/null 2>&1; then
  printf 'expected unsupported command to fail\n' >&2
  exit 1
fi

for secret_file in .env .env.local .npmrc; do
  git check-ignore -q "$secret_file"
  if git ls-files --error-unmatch "$secret_file" >/dev/null 2>&1; then
    printf 'secret-bearing local config file must not be tracked: %s\n' "$secret_file" >&2
    exit 1
  fi
done

pack_json="$(npm pack --dry-run --json)"
PACK_JSON="$pack_json" node <<'NODE'
const packs = JSON.parse(process.env.PACK_JSON);
const files = packs[0].files.map((file) => file.path).sort();
const expected = ["LICENSE", "README.md", "bin/ornnlab.js", "package.json"];
const forbidden = files.filter((file) =>
  file === ".env" ||
  file === ".env.local" ||
  file === ".npmrc" ||
  file.startsWith("docs/")
);
if (forbidden.length > 0) {
  console.error(`forbidden files in npm package: ${forbidden.join(", ")}`);
  process.exit(1);
}
if (JSON.stringify(files) !== JSON.stringify(expected)) {
  console.error(`unexpected npm pack files: ${files.join(", ")}`);
  process.exit(1);
}
NODE
