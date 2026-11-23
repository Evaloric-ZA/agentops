#!/usr/bin/env bash
# The full acceptance sweep.
#
# Every check prints one PASS: or FAIL: line, and the script exits
# non-zero if any check fails. A check that cannot run is a failure, not a
# skip: this script is the gate, so a silent skip would be a hole in it.
#
# The order matters: the cheap checks run first, so a broken tree is
# reported before the slow ones.

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

# `cargo tauri ...` invokes a binary named `cargo-tauri`, but the npm
# @tauri-apps/cli package installs one named `tauri`. The bridge is a
# three-line shim in node_modules/.bin, and `npm ci` wipes node_modules,
# so it has to be created after `npm ci` has run — creating it earlier
# means `npm ci` deletes it again.
ensure_cargo_tauri() {
  if [ ! -x "node_modules/.bin/cargo-tauri" ] && [ -x "node_modules/.bin/tauri" ]; then
    printf '#!/bin/sh\nDIR=$(CDPATH= cd -- "$(dirname -- "$0")" && pwd)\nif [ "$1" = "tauri" ]; then shift; fi\nexec "$DIR/tauri" "$@"\n' > node_modules/.bin/cargo-tauri
    chmod +x node_modules/.bin/cargo-tauri
  fi
}

run "npm ci" npm ci --no-audit --no-fund
ensure_cargo_tauri
run "npm run check" npm run check
run "npm test" npm test -- --run
run "npm run build" npm run build

run "cargo fmt --check" cargo fmt --manifest-path src-tauri/Cargo.toml --all -- --check
run "cargo clippy" cargo clippy --manifest-path src-tauri/Cargo.toml --all-targets --all-features -- -D warnings
run "cargo test" cargo test --manifest-path src-tauri/Cargo.toml --all-features

# The manifests must agree on the version. Nothing in the toolchain forces
# them to, and the application does not display its version, so a bump
# that misses one of them is invisible.
run "the version check" python3 scripts/check_version.py "$PWD"

# The privacy scanner's own regression test. The scanner once missed a
# secret in a .sh, .env, .lock or .py file; this proves it does not.
run "the privacy scanner's regression test" python3 scripts/test_verify_repo.py

# The native application. `--no-bundle` compiles and links the binary
# without the platform bundler, which needs an interactive session on
# macOS and downloads tooling on Linux. The .app and the installers are
# produced by a separate, documented step; see docs/PACKAGING.md.
ensure_cargo_tauri
run "the native build" cargo tauri build --debug --no-bundle

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
