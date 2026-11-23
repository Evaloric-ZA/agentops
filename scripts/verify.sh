#!/usr/bin/env bash
# The full acceptance sweep.
#
# Every command from the skill's acceptance-tests.md, in order, plus the
# privacy verifier. Each check prints one line; the script exits non-zero if
# any check fails.

set -u

cd "$(dirname "$0")/.." || exit 1

PASS=0
FAIL=0

run() {
  local label="$1"
  shift
  printf '\n=== %s ===\n' "$label"
  if "$@"; then
    printf 'PASS: %s\n' "$label"
    PASS=$((PASS + 1))
  else
    printf 'FAIL: %s\n' "$label"
    FAIL=$((FAIL + 1))
  fi
}

run "npm ci" npm ci --no-audit --no-fund

# `cargo tauri ...` invokes a binary named `cargo-tauri`, but the npm
# @tauri-apps/cli package installs one named `tauri`. npm ci wipes
# node_modules, so the bridge is recreated here.
if [ ! -x "node_modules/.bin/cargo-tauri" ]; then
  printf '#!/bin/sh\nDIR=$(CDPATH= cd -- "$(dirname -- "$0")" && pwd)\nif [ "$1" = "tauri" ]; then shift; fi\nexec "$DIR/tauri" "$@"\n' > node_modules/.bin/cargo-tauri
  chmod +x node_modules/.bin/cargo-tauri
fi

run "npm run check" npm run check
run "npm test" npm test -- --run
run "npm run build" npm run build
run "cargo fmt --check" cargo fmt --manifest-path src-tauri/Cargo.toml --all -- --check
run "cargo clippy" cargo clippy --manifest-path src-tauri/Cargo.toml --all-targets --all-features -- -D warnings
run "cargo test" cargo test --manifest-path src-tauri/Cargo.toml --all-features

printf '\n=== the privacy verifier ===\n'
if python3 scripts/verify_repo.py "$PWD"; then
  printf 'PASS: the privacy verifier\n'
  PASS=$((PASS + 1))
else
  printf 'FAIL: the privacy verifier\n'
  FAIL=$((FAIL + 1))
fi

printf '\n========================================\n'
printf 'passed: %d  failed: %d\n' "$PASS" "$FAIL"
[ "$FAIL" -eq 0 ] || exit 1
