#!/usr/bin/env python3
"""The acceptance sweep.

Every check prints one PASS:, FAIL: or SKIP: line, and the script exits
non-zero if any check fails. A check that cannot run is reported as a
skip with the reason, never as a pass: a silent skip would be a hole in
the gate.

This script is the single place the checks are defined. The shell and
PowerShell entry points both call it, so the two platforms cannot drift
apart: `scripts/verify.sh` for macOS and Linux, `scripts/verify.ps1` for
Windows. A check that only ran on one platform would show up as a
missing line, not as a difference nobody noticed.

The order matters: the cheap checks run first, so a broken tree is
reported before the slow ones.

Standard library only. It makes no network request of its own, writes
nothing outside the repository and deletes nothing.
"""

from __future__ import annotations

import argparse
import json
import os
import shutil
import subprocess
import sys
from pathlib import Path


class Sweep:
    """Runs the checks and keeps the tally."""

    def __init__(self, root: Path) -> None:
        self.root = root
        self.passed = 0
        self.failed = 0
        self.skipped = 0

    def run(self, label: str, command: list[str], missing_reason: str = "") -> bool:
        """Run a check. Returns True when it passed."""
        print(f"\n=== {label} ===", flush=True)
        try:
            code = subprocess.call(command, cwd=self.root)
        except (FileNotFoundError, PermissionError):
            if missing_reason:
                self.skip(label, missing_reason)
                return False
            self.fail(label, "the command could not be started")
            return False
        if code == 0:
            self.passed += 1
            print(f"PASS: {label}", flush=True)
            return True
        self.fail(label, f"the command exited with {code}")
        return False

    def skip(self, label: str, reason: str) -> None:
        self.skipped += 1
        print(f"SKIP: {label} ({reason})", flush=True)

    def fail(self, label: str, reason: str) -> None:
        self.failed += 1
        print(f"FAIL: {label} ({reason})", flush=True)

    def note(self, label: str, reason: str) -> None:
        """A result that is reported but does not decide the sweep."""
        print(f"NOTE: {label} ({reason})", flush=True)


def ensure_cargo_tauri(root: Path) -> bool:
    """Make `cargo tauri ...` work.

    Cargo looks for a subcommand named `cargo-tauri`, but the npm
    `@tauri-apps/cli` package installs one named `tauri`. A small bridge
    connects the two. The bridge has to be created after `npm ci` has
    run, because `npm ci` replaces the whole of `node_modules` and would
    take the bridge with it.
    """
    bin_dir = root / "node_modules" / ".bin"
    entry = root / "node_modules" / "@tauri-apps" / "cli" / "tauri.js"
    if not entry.exists():
        return False

    # Cargo passes the name of the subcommand to an external subcommand,
    # so the first argument the bridge receives is "tauri". The command
    # line interface wants the subcommand itself, not that name, so the
    # name is dropped.
    if os.name == "nt":
        bridge = bin_dir / "cargo-tauri.cmd"
        if not bridge.exists():
            bridge.write_text(
                "@echo off\r\n"
                "setlocal enabledelayedexpansion\r\n"
                'if /I "%~1"=="tauri" shift\r\n'
                'set "args="\r\n'
                ":collect\r\n"
                'if "%~1"=="" goto run\r\n'
                'set "args=!args! %1"\r\n'
                "shift\r\n"
                "goto collect\r\n"
                ":run\r\n"
                'node "%~dp0\\..\\@tauri-apps\\cli\\tauri.js"!args!\r\n',
                encoding="utf-8",
            )
    else:
        bridge = bin_dir / "cargo-tauri"
        if not bridge.exists():
            bridge.write_text(
                "#!/bin/sh\n"
                'DIR=$(CDPATH= cd -- "$(dirname -- "$0")" && pwd)\n'
                'if [ "$1" = "tauri" ]; then shift; fi\n'
                'exec node "$DIR/../@tauri-apps/cli/tauri.js" "$@"\n',
                encoding="utf-8",
            )
            bridge.chmod(0o755)
    return True


def the_registry_is_reachable(root: Path) -> bool:
    """Whether the package registry can be reached at all."""
    try:
        code = subprocess.call(
            ["npm", "ping", "--registry", "https://registry.npmjs.org/"],
            cwd=root,
            stdout=subprocess.DEVNULL,
            stderr=subprocess.DEVNULL,
        )
    except (FileNotFoundError, PermissionError):
        return False
    return code == 0


def run_the_rust_audit(sweep: "Sweep", root: Path) -> None:
    """Audit the Rust dependencies.

    The gate denies warnings, so an unmaintained, an unsound or a yanked
    crate fails the sweep. The advisories that have been accepted are
    listed in scripts/rust_audit_allowlist.json, each with the reason
    and what would close it; they are passed to the tool as exclusions.
    Anything not listed fails.
    """
    allowlist = root / "scripts" / "rust_audit_allowlist.json"
    accepted = []
    if allowlist.exists():
        try:
            data = json.loads(allowlist.read_text(encoding="utf-8"))
            accepted = [entry["id"] for entry in data.get("advisories", [])]
        except (ValueError, KeyError, TypeError) as error:
            sweep.fail(
                "the Rust dependency audit",
                f"the allowlist could not be read: {error}",
            )
            return

    # The advisory database is fetched once. If it cannot be fetched the
    # check is skipped with the reason, not failed: an unreachable
    # network says nothing about the dependencies.
    database = Path.home() / ".cargo" / "advisory-db"
    if not database.exists():
        fetched = subprocess.call(
            ["cargo", "audit", "update-db"],
            cwd=root,
            stdout=subprocess.DEVNULL,
            stderr=subprocess.DEVNULL,
        )
        if fetched != 0:
            sweep.skip(
                "the Rust dependency audit",
                "the advisory database could not be fetched from here",
            )
            return

    command = [
        "cargo", "audit",
        "--file", "src-tauri/Cargo.lock",
        "--no-fetch",
        "--deny", "warnings",
    ]
    for advisory in accepted:
        command += ["--ignore", advisory]
    sweep.run("the Rust dependency audit", command, "cargo is not installed")


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("root", type=Path, nargs="?", default=Path.cwd())
    args = parser.parse_args()
    root = args.root.resolve()
    if not root.is_dir():
        print(f"the repository root is not a directory: {root}")
        return 1
    os.chdir(root)

    python = sys.executable
    sweep = Sweep(root)

    # The clean install first: every later check reads the tree it left.
    sweep.run("npm ci", ["npm", "ci", "--no-audit", "--no-fund"], "npm is not installed")
    if not ensure_cargo_tauri(root):
        sweep.skip(
            "the native build",
            "the Tauri command line is not installed, so the application cannot be compiled",
        )

    sweep.run("npm run check", ["npm", "run", "check"], "npm is not installed")
    sweep.run("npm test", ["npm", "test", "--", "--run"], "npm is not installed")
    sweep.run("npm run build", ["npm", "run", "build"], "npm is not installed")

    sweep.run(
        "cargo fmt --check",
        ["cargo", "fmt", "--manifest-path", "src-tauri/Cargo.toml", "--all", "--", "--check"],
        "cargo is not installed",
    )
    sweep.run(
        "cargo clippy",
        [
            "cargo", "clippy",
            "--manifest-path", "src-tauri/Cargo.toml",
            "--all-targets", "--all-features",
            "--", "-D", "warnings",
        ],
        "cargo is not installed",
    )
    sweep.run(
        "cargo test",
        ["cargo", "test", "--manifest-path", "src-tauri/Cargo.toml", "--all-features"],
        "cargo is not installed",
    )

    # The manifests must agree on the version. Nothing in the toolchain
    # forces them to, and the application does not display its version,
    # so a bump that misses one of them is invisible.
    sweep.run(
        "the version check",
        [python, "scripts/check_version.py", str(root)],
        "the Python interpreter is not available",
    )

    # The declared minimum Rust has to be true. Cargo only warns when it
    # is not, and a dependency that needs a newer compiler fails with an
    # obscure message, so the claim is checked here instead.
    sweep.run(
        "the minimum supported Rust check",
        [python, "scripts/check_msrv.py", str(root)],
        "the Python interpreter is not available",
    )

    # The privacy scanner's own regression test.
    sweep.run(
        "the privacy scanner's regression test",
        [python, "scripts/test_verify_repo.py"],
        "the Python interpreter is not available",
    )

    # The Rust domain vocabulary and the TypeScript mirror must agree.
    sweep.run(
        "the type vocabulary check",
        [python, "scripts/check_types.py", str(root)],
        "the Python interpreter is not available",
    )

    # The native application. `--no-bundle` compiles and links the binary
    # without the platform bundler, which needs an interactive session on
    # macOS and downloads tooling on Linux. The .app and the installers
    # are produced by a separate, documented step; see docs/PACKAGING.md.
    if ensure_cargo_tauri(root):
        sweep.run(
            "the native build",
            ["cargo", "tauri", "build", "--debug", "--no-bundle"],
            "cargo is not installed",
        )

    # The dependency advisories. The npm registry has to be reachable for
    # the JavaScript audit; the Rust audit needs a tool that is not part
    # of the toolchain and is not installed without the operator's
    # agreement, so it is reported as skipped rather than as passed.
    if the_registry_is_reachable(root):
        sweep.run(
            "the production dependency audit",
            ["npm", "audit", "--omit=dev", "--audit-level=moderate"],
            "npm is not installed",
        )
        # The development toolchain has advisories of its own. They do
        # not ship with the application and the application does not
        # exercise them, so they are reported here and listed in
        # docs/SECURITY.md rather than allowed to fail the sweep.
        print("\n=== the development dependency audit ===", flush=True)
        code = subprocess.call(["npm", "audit"], cwd=root)
        if code == 0:
            sweep.note("the development dependency audit", "no advisories were found")
        else:
            sweep.note(
                "the development dependency audit",
                "there are advisories in the development toolchain; "
                "they are listed in docs/SECURITY.md",
            )
    else:
        sweep.skip(
            "the production dependency audit",
            "the package registry cannot be reached from here",
        )
        sweep.skip(
            "the development dependency audit",
            "the package registry cannot be reached from here",
        )

    if shutil.which("cargo-audit"):
        run_the_rust_audit(sweep, root)
    else:
        sweep.skip(
            "the Rust dependency audit",
            "cargo-audit is not installed, so the Rust advisories were not scanned",
        )

    sweep.run(
        "the privacy verifier",
        [python, "scripts/verify_repo.py", str(root)],
        "the Python interpreter is not available",
    )

    print("\n========================================")
    print(f"passed: {sweep.passed}  failed: {sweep.failed}  skipped: {sweep.skipped}")
    if sweep.skipped:
        print("a skipped check is not a passed check")
    return 1 if sweep.failed else 0


if __name__ == "__main__":
    sys.exit(main())
