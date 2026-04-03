#!/usr/bin/env bash
set -euo pipefail

matches="$(
  rg -n 'panic!|unreachable!|todo!|unimplemented!' \
    lang/src spl/src derive/src \
    --glob '!**/tests/**' || true
)"

violations=()

while IFS= read -r entry; do
  [[ -z "$entry" ]] && continue
  code="${entry#*:*:}"
  if [[ "$code" =~ ^[[:space:]]*// ]]; then
    continue
  fi
  case "$entry" in
    *'lang/src/lib.rs:'*'panic!("program aborted")'*)
      continue
      ;;
    *'lang/src/dynamic.rs:'*'panic!("dynamic account field contains invalid UTF-8")'*)
      continue
      ;;
  esac
  violations+=("$entry")
done <<<"$matches"

if ((${#violations[@]} > 0)); then
  echo "unexpected panic-style macro in runtime/derive code:" >&2
  printf '  %s\n' "${violations[@]}" >&2
  exit 1
fi
