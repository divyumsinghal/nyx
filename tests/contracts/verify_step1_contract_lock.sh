#!/usr/bin/env bash
set -euo pipefail

if [ "$#" -ne 1 ]; then
  printf 'usage: %s <contract-lock-file>\n' "$0" >&2
  exit 2
fi

contract_lock="$1"

if [ ! -f "$contract_lock" ]; then
  printf 'Contract lock file not found: %s\n' "$contract_lock" >&2
  exit 2
fi

total=0
failures=0

while IFS='|' read -r file match_type pattern invariant_id; do
  if [ -z "${file:-}" ] || [ "${file#\#}" != "$file" ]; then
    continue
  fi

  total=$((total + 1))

  if [ ! -f "$file" ]; then
    printf 'DRIFT: [%s] file missing: %s\n' "$invariant_id" "$file" >&2
    failures=$((failures + 1))
    continue
  fi

  case "$match_type" in
    fixed)
      if ! grep -Fq -- "$pattern" "$file"; then
        printf 'DRIFT: [%s] missing fixed pattern in %s :: %s\n' "$invariant_id" "$file" "$pattern" >&2
        failures=$((failures + 1))
      fi
      ;;
    regex)
      if ! grep -Eq -- "$pattern" "$file"; then
        printf 'DRIFT: [%s] missing regex pattern in %s :: %s\n' "$invariant_id" "$file" "$pattern" >&2
        failures=$((failures + 1))
      fi
      ;;
    *)
      printf 'Invalid lock entry match type for [%s]: %s\n' "$invariant_id" "$match_type" >&2
      failures=$((failures + 1))
      ;;
  esac
done < "$contract_lock"

if [ "$total" -eq 0 ]; then
  printf 'Contract lock has no invariants: %s\n' "$contract_lock" >&2
  exit 2
fi

if [ "$failures" -gt 0 ]; then
  printf 'Step 1 compatibility contract FAILED (%s/%s invariants drifted).\n' "$failures" "$total" >&2
  exit 1
fi

printf 'Step 1 compatibility contract PASSED (%s invariants checked).\n' "$total"
