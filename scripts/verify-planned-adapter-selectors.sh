#!/usr/bin/env bash
set -euo pipefail

selector_rows="$(cargo run -p xtask -- list-adapter-proof-selectors)"
planned_ids=()
active_ids=()
expected_active_ids=(
  ADAPT-DATA-001
  ADAPT-DATA-002
  ADAPT-DATA-003
  ADAPT-DATA-004
  ADAPT-DATA-005
  ADAPT-RUNTIME-001
  ADAPT-RUNTIME-002
  ADAPT-RUNTIME-003
  ADAPT-RUNTIME-004
  ADAPT-RUNTIME-005
  ADAPT-RUNTIME-006
  SWEPRO-001
  SWEPRO-002
  SWEPRO-003
  SWEPRO-004
  SWEPRO-005
)
expected_planned_ids=(ADAPT-DATA-000)

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

if [[ "${active_ids[*]}" != "${expected_active_ids[*]}" ]]; then
  echo "active adapter selector inventory drifted" >&2
  echo "expected: ${expected_active_ids[*]}" >&2
  echo "actual:   ${active_ids[*]}" >&2
  exit 1
fi

if [[ "${planned_ids[*]}" != "${expected_planned_ids[*]}" ]]; then
  echo "planned adapter selector inventory drifted" >&2
  echo "expected: ${expected_planned_ids[*]}" >&2
  echo "actual:   ${planned_ids[*]}" >&2
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
