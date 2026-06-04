#!/usr/bin/env bash
set -euo pipefail

selector_rows="$(cargo run -p xtask -- list-adapter-proof-selectors)"
planned_ids=()
active_ids=()

while IFS=$'\t' read -r status id; do
  [[ -z "${status:-}" || -z "${id:-}" ]] && continue
  case "$status" in
    active) active_ids+=("$id") ;;
    planned) planned_ids+=("$id") ;;
    *) echo "adapter selector $id has unsupported status $status" >&2; exit 1 ;;
  esac
done <<<"$selector_rows"

if [[ "${#active_ids[@]}" -eq 0 ]]; then
  echo "no active adapter proof selectors found" >&2
  exit 1
fi

for id in "${active_ids[@]}"; do
  echo "checking active adapter selector: $id"
  scripts/test-after-change.sh --select "$id"
done

for id in "${planned_ids[@]}"; do
  set +e
  output="$(scripts/test-after-change.sh --select "$id" 2>&1)"
  status=$?
  set -e
  printf '%s\n' "$output"
  if [[ "$status" -ne 64 ]]; then
    echo "planned adapter selector $id exited $status, expected 64" >&2
    exit 1
  fi
  if ! grep -q "planned adapter proof is registered but not implemented yet: $id" <<<"$output"; then
    echo "planned adapter selector $id did not print the planned-proof message" >&2
    exit 1
  fi
done

echo "adapter selectors ok: active=${#active_ids[@]} planned=${#planned_ids[@]}"
