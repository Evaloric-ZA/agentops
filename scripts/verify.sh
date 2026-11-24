#!/usr/bin/env bash
# The acceptance sweep, for macOS and Linux.
#
# Every check prints one PASS:, FAIL: or SKIP: line, and the script exits
# non-zero if any check fails. A check that cannot run is reported as a
# skip with the reason, never as a pass.
#
# The checks themselves live in scripts/verify.py, so this shell entry
# point and the PowerShell one validate the same requirements on the
# same versions of the same tools. This file only finds a Python and
# hands over.

set -u

cd "$(dirname "$0")/.." || exit 1

# `cargo tauri ...` invokes a binary named `cargo-tauri`, and Cargo looks
# for it on the PATH. The npm CLI installs the Tauri CLI into
# node_modules/.bin, which is not on the PATH by default — so the sweep
# puts it there itself rather than relying on the caller to have done it.
export PATH="$PWD/node_modules/.bin:$PATH"

# The interpreter the checks are written for. The scripts use
# `from __future__ import annotations`, so an older Python runs them.
PYTHON=""
for candidate in python3 python; do
  if command -v "$candidate" > /dev/null 2>&1; then
    PYTHON="$candidate"
    break
  fi
done

if [ -z "$PYTHON" ]; then
  echo "FAIL: the sweep (no Python interpreter was found)"
  exit 1
fi

exec "$PYTHON" scripts/verify.py "$PWD"
