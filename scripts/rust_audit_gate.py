#!/usr/bin/env python3
"""The Rust dependency audit gate.

The policy is applied here rather than being left to cargo-audit's own
exit code, because the exit code cannot tell a finding from a failure
and the policy has to be able to say which is which:

- A vulnerability fails the gate. It can only be accepted if it is
  listed in scripts/rust_audit_allowlist.json, and then only until the
  entry's review date.
- An informational advisory — an unmaintained crate, an unsound crate,
  a yanked crate — fails the gate for the same reason. It is not
  dismissed because it is not a vulnerability: an unsound crate is a
  defect whether or not anyone has exploited it.
- An entry whose review date has passed fails the gate. Nobody renews
  it automatically: somebody has to look at it and decide again.
- An entry for an advisory that no longer applies fails the release
  gate, because the file has to describe the world as it is.

The gate runs in two modes:

- developer (the default): a tool or an advisory database that is
  missing makes the gate report a skip and exit successfully, so a
  developer working offline can still run the rest of the sweep.
- release (--release): the same missing tool or database fails the
  gate. A release is not verified by a check that did not run.

Usage:
    python3 scripts/rust_audit_gate.py [--release] [repository-root]

The exit code is 0 when the gate passed, 1 when it failed, and 2 when
it could not run. The third one is deliberately distinct: a check that
did not run is not a check that passed, and the sweep needs to be able
to tell the two apart.

Standard library only.
"""

from __future__ import annotations

import argparse
import json
import shutil
import subprocess
import sys
from datetime import date
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parent))
from audit_policy import (  # noqa: E402
    RUST_REQUIRED,
    RUST_SCOPES,
    expired,
    load_allowlist,
)


def collect(cargo_audit: str, root: Path) -> tuple[dict | None, str]:
    """Run the audit and return its JSON, or the reason it has none."""
    try:
        finished = subprocess.run(
            [cargo_audit, "audit", "--json", "--file", "src-tauri/Cargo.lock"],
            cwd=root,
            stdout=subprocess.PIPE,
            stderr=subprocess.PIPE,
        )
    except (OSError, PermissionError) as error:
        return None, f"the audit could not be started: {error}"
    text = finished.stdout.decode("utf-8", errors="replace").strip()
    if not text:
        detail = finished.stderr.decode("utf-8", errors="replace").strip()
        return None, (
            "the audit produced no result, so the advisory database could not be "
            f"read or the tool failed: {detail or 'no output'}"
        )
    try:
        data = json.loads(text)
    except ValueError as error:
        return None, f"the audit did not produce JSON: {error}"
    if not isinstance(data, dict):
        return None, "the audit did not produce a JSON object"
    if "error" in data:
        error = data["error"]
        if isinstance(error, dict):
            error = error.get("summary") or error.get("detail") or error
        return None, f"the audit could not run: {error}"
    return data, ""


def found(data: dict) -> list[dict]:
    """Every advisory the audit found, as one flat list.

    The tool reports the informational advisories in a map keyed by
    their kind and the vulnerabilities in a list, so they are flattened
    into one shape here.
    """
    rows: list[dict] = []

    block = data.get("vulnerabilities")
    if isinstance(block, dict):
        for item in block.get("list", []) or []:
            advisory = item.get("advisory", {})
            package = item.get("package", {})
            rows.append(
                {
                    "id": advisory.get("id", "?"),
                    "package": package.get("name") or advisory.get("package", "?"),
                    "version": package.get("version", ""),
                    "kind": "vulnerability",
                    "title": advisory.get("title", ""),
                    "patched": (item.get("versions") or {}).get("patched") or [],
                }
            )

    warnings = data.get("warnings")
    if isinstance(warnings, dict):
        for kind, items in warnings.items():
            for item in items or []:
                advisory = item.get("advisory", {})
                package = item.get("package", {})
                rows.append(
                    {
                        "id": advisory.get("id", "?"),
                        "package": package.get("name") or advisory.get("package", "?"),
                        "version": package.get("version", ""),
                        "kind": kind,
                        "title": advisory.get("title", ""),
                        "patched": (item.get("versions") or {}).get("patched") or [],
                    }
                )
    rows.sort(key=lambda row: row["id"])
    return rows


def decide(
    rows: list[dict],
    allowed: dict[str, dict],
    release: bool,
    today: date | None = None,
) -> tuple[list[tuple[str, str]], list[str]]:
    """Apply the policy to the advisories that were found.

    Returns the lines to print and the failures. The policy is kept
    here, away from the process handling, so that it can be tested with
    findings that no advisory database would ever produce.
    """
    outcomes: list[tuple[str, str]] = []
    failures: list[str] = []
    used: set[str] = set()
    for row in rows:
        advisory = row["id"]
        entry = allowed.get(advisory)
        if entry is not None:
            used.add(advisory)
            if expired(entry, today):
                message = (
                    f"{advisory} ({row['package']}) is accepted but its review date "
                    f"{entry.get('review_by')} has passed; it has to be reviewed, not renewed"
                )
                if release:
                    failures.append(message)
                    outcomes.append(("FAIL", message))
                else:
                    # A developer is told, but a developer's sweep is not
                    # held up by a policy decision that an operator has
                    # to make. The release gate is what refuses it.
                    outcomes.append(("WARN", message))
                continue
            outcomes.append(
                (
                    "NOTE",
                    f"{advisory} ({row['package']}, {row['kind']}) — accepted until "
                    f"{entry.get('review_by')}: {entry.get('reason')}",
                )
            )
            continue
        message = (
            f"{advisory} ({row['package']}, {row['kind']}) is not accepted: {row['title']}"
        )
        failures.append(message)
        outcomes.append(("FAIL", message))
        if row.get("patched"):
            outcomes.append(
                ("NOTE", f"  a patched version exists: {', '.join(row['patched'])}")
            )

    for advisory in sorted(set(allowed) - used):
        message = f"{advisory} is accepted but the audit did not find it"
        if release:
            failures.append(message)
            outcomes.append(("FAIL", f"{message}; remove the exception"))
        else:
            outcomes.append(("NOTE", f"{message}; remove the exception"))
    return outcomes, failures


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("root", type=Path, nargs="?", default=Path.cwd())
    parser.add_argument(
        "--release",
        action="store_true",
        help="fail when a control is absent rather than skipping it",
    )
    args = parser.parse_args()
    root = args.root.resolve()
    release = args.release

    allowlist_path = root / "scripts" / "rust_audit_allowlist.json"
    accepted, problems = load_allowlist(
        allowlist_path, RUST_REQUIRED, "the Rust advisory allowlist", RUST_SCOPES
    )
    for problem in problems:
        print(f"FAIL: {problem}")
    if problems:
        return 1
    allowed = {entry["advisory"]: entry for entry in accepted}

    cargo_audit = shutil.which("cargo-audit")
    if cargo_audit is None:
        message = "cargo-audit is not installed, so the Rust advisories were not scanned"
        print(f"{'FAIL' if release else 'SKIP'}: the Rust dependency audit ({message})")
        return 1 if release else 2

    data, reason = collect(cargo_audit, root)
    if data is None:
        print(f"{'FAIL' if release else 'SKIP'}: the Rust dependency audit ({reason})")
        return 1 if release else 2

    rows = found(data)
    vulnerabilities = [row for row in rows if row["kind"] == "vulnerability"]
    informational = [row for row in rows if row["kind"] != "vulnerability"]
    print(
        f"the Rust audit found {len(vulnerabilities)} vulnerabilities and "
        f"{len(informational)} informational advisories"
    )

    outcomes, failures = decide(rows, allowed, release)
    for kind, message in outcomes:
        print(f"{kind}: {message}")

    if failures:
        print(f"\nthe Rust dependency audit: failed ({len(failures)} problems)")
        for failure in failures:
            print(f"  - {failure}")
        return 1
    print("the Rust dependency audit: passed")
    return 0


if __name__ == "__main__":
    sys.exit(main())
