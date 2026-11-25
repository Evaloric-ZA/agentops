# The acceptance sweep, for Windows.
#
# Every check prints one PASS:, FAIL: or SKIP: line, and the script exits
# non-zero if any check fails. A check that cannot run is reported as a
# skip with the reason, never as a pass.
#
# The checks themselves live in scripts/verify.py, so this PowerShell
# entry point and the shell one validate the same requirements on the
# same versions of the same tools. This file only finds a Python and
# hands over. There is no Unix shell syntax here on purpose: the
# continuous integration job on Windows runs this file, not the shell.

$ErrorActionPreference = "Continue"

$root = Split-Path -Parent (Split-Path -Parent $MyInvocation.MyCommand.Path)
Set-Location $root

# `cargo tauri ...` invokes a binary named `cargo-tauri`, and Cargo looks
# for it on the PATH. The npm CLI installs the Tauri CLI into
# node_modules/.bin, which is not on the PATH by default — so the sweep
# puts it there itself rather than relying on the caller to have done it.
$bin = Join-Path $root "node_modules\.bin"
if (Test-Path $bin) {
    $env:PATH = "$bin;$env:PATH"
}

# The interpreter the checks are written for. The scripts use
# `from __future__ import annotations`, so an older Python runs them.
$python = $null
foreach ($candidate in @("python", "python3", "py")) {
    $found = Get-Command $candidate -ErrorAction SilentlyContinue
    if ($found) {
        $python = $found.Source
        break
    }
}

if (-not $python) {
    Write-Output "FAIL: the sweep (no Python interpreter was found)"
    exit 1
}

# Any arguments are passed on, so `scripts/verify.ps1 --release` runs the
# release verification rather than the developer one.
& $python (Join-Path $root "scripts\verify.py") $root $args
exit $LASTEXITCODE
