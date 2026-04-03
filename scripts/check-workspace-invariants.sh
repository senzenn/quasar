#!/usr/bin/env bash
set -euo pipefail

require_executable() {
  local path="$1"
  if [[ ! -x "$path" ]]; then
    echo "expected executable script: $path" >&2
    exit 1
  fi
}

check_allowed_matches() {
  local description="$1"
  local pattern="$2"
  shift 2
  local allowed=("$@")
  local matches

  matches="$(rg -n "$pattern" cli/src || true)"
  while IFS= read -r entry; do
    [[ -z "$entry" ]] && continue
    local ok=0
    for prefix in "${allowed[@]}"; do
      if [[ "$entry" == "$prefix"* ]]; then
        ok=1
        break
      fi
    done
    if [[ "$ok" -eq 0 ]]; then
      echo "unexpected ${description}: $entry" >&2
      exit 1
    fi
  done <<<"$matches"
}

require_executable scripts/bench-tracked-programs.sh
require_executable scripts/check-runtime-panics.sh
require_executable scripts/check-workspace-invariants.sh
require_executable scripts/check-workspace-lints.sh

check_allowed_matches \
  "process::exit" \
  'std::process::exit|process::exit' \
  'cli/src/main.rs:' \
  'cli/src/init/banner.rs:'

check_allowed_matches \
  "polling watch loop sleep" \
  'std::thread::sleep\(std::time::Duration::from_secs\(1\)\)' \
  'cli/src/build_watch.rs:'

if rg -n 'split_whitespace\(' cli/src >/dev/null; then
  echo "cli command parsing must not use split_whitespace()" >&2
  rg -n 'split_whitespace\(' cli/src >&2
  exit 1
fi
