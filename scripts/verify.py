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
import os
import shutil
import subprocess
import sys
from pathlib import Path

# The bridge that makes `cargo tauri ...` work lives in its own file,
# because the continuous integration job that builds the Windows
# application needs it too.
sys.path.insert(0, str(Path(__file__).resolve().parent))
from ensure_cargo_tauri import ensure_cargo_tauri  # noqa: E402


class Sweep:
    """Runs the checks and keeps the tally."""

    def __init__(self, root: Path, release: bool = False) -> None:
        self.root = root
        self.release = release
        self.passed = 0
        self.failed = 0
        self.skipped = 0

    def run(
        self,
        label: str,
        command: list[str],
        missing_reason: str = "",
        skip_codes: tuple[int, ...] = (),
    ) -> bool:
        """Run a check. Returns True when it passed.

        A check that exits with one of the skip codes reports that it
        could not run, rather than that it passed. The exit code 2 is
        what the two dependency audit gates use for it.
        """
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
        if code in skip_codes:
            self.skip(label, "the check reported that it could not run")
            return False
        self.fail(label, f"the command exited with {code}")
        return False

    def skip(self, label: str, reason: str) -> None:
        """Report a check that could not run.

        In release mode a check that could not run is a failure. A
        release is not verified by a control that was absent.
        """
        if self.release:
            self.failed += 1
            print(f"FAIL: {label} ({reason}) — a release check cannot be skipped", flush=True)
            return
        self.skipped += 1
        print(f"SKIP: {label} ({reason})", flush=True)

    def fail(self, label: str, reason: str) -> None:
        self.failed += 1
        print(f"FAIL: {label} ({reason})", flush=True)

    def note(self, label: str, reason: str) -> None:
        """A result that is reported but does not decide the sweep."""
        print(f"NOTE: {label} ({reason})", flush=True)


def the_program(name: str) -> list[str] | None:
    """Find a program the way the platform finds one.

    On Windows an npm install puts `npm.cmd` on the path, not `npm`.
    Python starts a program without a shell, so it does not apply the
    path extension rules the shell would, and a bare `npm` is not
    found. `shutil.which` applies them.

    This was not a theoretical problem: the first run on a Windows
    runner reported every npm check as skipped because npm is not
    installed, when npm was installed the whole time.
    """
    found = shutil.which(name)
    return [found] if found else None


def the_registry_is_reachable(root: Path, npm: list[str]) -> bool:
    """Whether the package registry can be reached at all."""
    try:
        code = subprocess.call(
            npm + ["ping", "--registry", "https://registry.npmjs.org/"],
            cwd=root,
            stdout=subprocess.DEVNULL,
            stderr=subprocess.DEVNULL,
        )
    except (FileNotFoundError, PermissionError):
        return False
    return code == 0


def main() -> int:
    parser = argparse.ArgumentParser(
        description="the acceptance sweep; --release turns every skip into a failure"
    )
    parser.add_argument("root", type=Path, nargs="?", default=Path.cwd())
    parser.add_argument(
        "--release",
        action="store_true",
        help="fail when a control is absent rather than skipping it",
    )
    args = parser.parse_args()
    root = args.root.resolve()
    if not root.is_dir():
        print(f"the repository root is not a directory: {root}")
        return 1
    os.chdir(root)

    release = args.release
    python = sys.executable
    sweep = Sweep(root, release=release)
    if release:
        print("RELEASE VERIFICATION: every check has to run, and every check has to pass")

    # The programs are resolved once. On Windows `npm` is `npm.cmd`, and
    # Python starts a program without a shell, so a bare `npm` is not
    # found. The fallback keeps the old behaviour: a missing program
    # raises, and the sweep reports it as a skip with the reason.
    npm = the_program("npm") or ["npm"]
    cargo = the_program("cargo") or ["cargo"]

    # The clean install first: every later check reads the tree it left.
    sweep.run("npm ci", npm + ["ci", "--no-audit", "--no-fund"], "npm is not installed")
    if not ensure_cargo_tauri(root):
        sweep.skip(
            "the native build",
            "the Tauri command line is not installed, so the application cannot be compiled",
        )

    sweep.run("npm run check", npm + ["run", "check"], "npm is not installed")
    sweep.run("npm test", npm + ["test", "--", "--run"], "npm is not installed")
    sweep.run("npm run build", npm + ["run", "build"], "npm is not installed")

    sweep.run(
        "cargo fmt --check",
        cargo + ["fmt", "--manifest-path", "src-tauri/Cargo.toml", "--all", "--", "--check"],
        "cargo is not installed",
    )
    sweep.run(
        "cargo clippy",
        cargo + [
            "clippy",
            "--manifest-path", "src-tauri/Cargo.toml",
            "--all-targets", "--all-features",
            "--", "-D", "warnings",
        ],
        "cargo is not installed",
    )
    sweep.run(
        "cargo test",
        cargo + ["test", "--manifest-path", "src-tauri/Cargo.toml", "--all-features"],
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

    # The two dependency gates' own regression test. It proves that the
    # gates fail when they should, without needing a registry or an
    # advisory database to produce a finding.
    sweep.run(
        "the audit policy's regression test",
        [python, "scripts/test_audit_policy.py"],
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
        # `npm run tauri -- ...` rather than `cargo tauri ...`. Cargo
        # looks for an external subcommand as a binary, and on Windows
        # it did not find the `.cmd` bridge: the first run on a Windows
        # runner failed with "no such command: `tauri`". npm resolves
        # the local command line interface itself, on every platform.
        sweep.run(
            "the native build",
            npm + ["run", "tauri", "--", "build", "--debug", "--no-bundle"],
            "npm is not installed",
        )

    # The dependency advisories. The two gates live in their own files,
    # because the continuous integration security job runs them on their
    # own and because they have to be testable without a registry or an
    # advisory database.
    #
    # The JavaScript gate audits the whole tree, the development
    # toolchain included: a developer runs those packages and the
    # continuous integration runs them, so an advisory in them is not
    # an advisory in something that never executes.
    gate = [python, "scripts/npm_audit_gate.py", str(root)]
    if release:
        gate.append("--release")
    if the_registry_is_reachable(root, npm):
        sweep.run(
            "the npm dependency audit",
            gate,
            "the Python interpreter is not available",
            skip_codes=(2,),
        )
    else:
        sweep.skip(
            "the npm dependency audit",
            "the package registry cannot be reached from here",
        )

    gate = [python, "scripts/rust_audit_gate.py", str(root)]
    if release:
        gate.append("--release")
    sweep.run(
        "the Rust dependency audit",
        gate,
        "the Python interpreter is not available",
        skip_codes=(2,),
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
    if release:
        print("RELEASE VERIFICATION: a skipped check is a failure, and the result above is the release result")
    return 1 if sweep.failed else 0


if __name__ == "__main__":
    sys.exit(main())
