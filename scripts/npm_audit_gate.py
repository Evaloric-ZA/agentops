#!/usr/bin/env python3
"""The npm dependency audit gate.

The development toolchain is not less important than the shipped code
for this gate. A developer runs it and the continuous integration runs
it, so an advisory in it is an advisory in something that executes.

The policy is:

- Critical and High fail the gate.
- Moderate warns.
- Low and informational are reported and do not fail anything.
- An advisory that is listed in scripts/npm_audit_allowlist.json is
  accepted, but only until its review date. An expired exception
  fails the gate: it has to be looked at again, and nobody renews it
  automatically.
- An advisory that is not listed is never accepted, however new it is.

The gate runs in two modes:

- developer (the default): a tool or a network that is missing makes
  the gate report a skip and exit successfully, because a developer
  working offline still needs the rest of the sweep.
- release (--release): the same missing tool or network fails the
  gate. A release is not verified by a check that did not run.

Usage:
    python3 scripts/npm_audit_gate.py [--release] [repository-root]

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
    NPM_REQUIRED,
    NPM_SCOPES,
    expired,
    load_allowlist,
    severity_rank,
)

FAIL_AT = ("critical", "high")
WARN_AT = ("moderate",)


def the_program(name: str) -> str | None:
    """Find a program the way the platform finds one.

    On Windows an npm install puts `npm.cmd` on the path, not `npm`, and
    Python starts a program without a shell, so it does not apply the
    path extension rules the shell would. `shutil.which` applies them.
    """
    return shutil.which(name)


def advisory_id(url: object) -> str:
    """The advisory identifier out of its URL."""
    if not isinstance(url, str):
        return "?"
    return url.rstrip("/").rsplit("/", 1)[-1]


def collect(npm: str, root: Path) -> tuple[dict | None, str]:
    """Run the audit and return its JSON, or the reason it has none."""
    try:
        finished = subprocess.run(
            [npm, "audit", "--json"],
            cwd=root,
            stdout=subprocess.PIPE,
            stderr=subprocess.PIPE,
        )
    except (OSError, PermissionError) as error:
        return None, f"the audit could not be started: {error}"
    text = finished.stdout.decode("utf-8", errors="replace").strip()
    if not text:
        detail = finished.stderr.decode("utf-8", errors="replace").strip()
        return None, f"the audit produced no result: {detail or 'no output'}"
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


def findings(data: dict) -> list[dict]:
    """Every advisory in the audit result, one row per advisory.

    The severity of a row is the highest of the advisory's own severity
    and the severity npm rolled the package up to, so a package that
    npm considers high is never reported as something milder.
    """
    rows: list[dict] = []
    packages = data.get("vulnerabilities")
    if not isinstance(packages, dict):
        return rows
    for package, entry in packages.items():
        if not isinstance(entry, dict):
            continue
        package_severity = str(entry.get("severity", "")).lower()
        direct = [via for via in entry.get("via", []) if isinstance(via, dict)]
        if not direct:
            # A package that is only a dependent of a vulnerable package
            # carries no advisory of its own.
            continue
        for via in direct:
            severity = str(via.get("severity", "")).lower() or package_severity
            if severity_rank(severity) > severity_rank(package_severity):
                severity = package_severity
            rows.append(
                {
                    "package": package,
                    "advisory": advisory_id(via.get("url")),
                    "title": via.get("title") or "",
                    "severity": severity,
                    "direct": bool(entry.get("isDirect")),
                    "range": via.get("range") or entry.get("range") or "",
                    "fix": entry.get("fixAvailable"),
                }
            )
    rows.sort(key=lambda row: (severity_rank(row["severity"]), row["package"], row["advisory"]))
    return rows


def decide(
    rows: list[dict],
    allowed: dict[str, dict],
    release: bool,
    today: date | None = None,
) -> tuple[list[tuple[str, str]], list[str]]:
    """Apply the policy to the advisories that were found.

    Returns the lines to print and the failures. The failures are what
    makes the gate fail; the lines are what a person reads. The policy
    is kept here, away from the process handling, so that it can be
    tested with findings that no registry would ever produce.
    """
    outcomes: list[tuple[str, str]] = []
    failures: list[str] = []
    used: set[str] = set()
    for row in rows:
        advisory = row["advisory"]
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
                    f"{advisory} ({row['package']}) — accepted until "
                    f"{entry.get('review_by')}: {entry.get('reason')}",
                )
            )
            continue
        if row["severity"] in FAIL_AT:
            message = (
                f"{advisory} ({row['package']}) is {row['severity']} and is not accepted"
            )
            failures.append(message)
            outcomes.append(("FAIL", f"{message}: {row['title']}"))
        elif row["severity"] in WARN_AT:
            outcomes.append(
                ("WARN", f"{advisory} ({row['package']}) — {row['severity']}: {row['title']}")
            )
        else:
            outcomes.append(
                ("REPORT", f"{advisory} ({row['package']}) — {row['severity']}: {row['title']}")
            )

    # An exception for an advisory that no longer exists is untidy, and
    # untidy policy is how policy rots. It does not hide anything,
    # because an advisory identifier is unique, but the release gate
    # insists that the file describe the world as it actually is.
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

    allowlist_path = root / "scripts" / "npm_audit_allowlist.json"
    accepted, problems = load_allowlist(
        allowlist_path, NPM_REQUIRED, "the npm advisory allowlist", NPM_SCOPES
    )
    for problem in problems:
        print(f"FAIL: {problem}")
    if problems:
        return 1

    allowed = {entry["advisory"]: entry for entry in accepted}

    npm = the_program("npm")
    if npm is None:
        message = "npm is not installed, so the JavaScript dependencies were not audited"
        print(f"{'FAIL' if release else 'SKIP'}: the npm dependency audit ({message})")
        return 1 if release else 2

    data, reason = collect(npm, root)
    if data is None:
        print(f"{'FAIL' if release else 'SKIP'}: the npm dependency audit ({reason})")
        return 1 if release else 2

    rows = findings(data)
    counts = data.get("metadata", {}).get("vulnerabilities", {})
    if isinstance(counts, dict):
        summary = ", ".join(
            f"{counts.get(name, 0)} {name}"
            for name in ("critical", "high", "moderate", "low", "info")
            if counts.get(name)
        )
        print(f"the npm audit found: {summary or 'nothing'}")

    outcomes, failures = decide(rows, allowed, release)
    for kind, message in outcomes:
        print(f"{kind}: {message}")

    if failures:
        print(f"\nthe npm dependency audit: failed ({len(failures)} problems)")
        for failure in failures:
            print(f"  - {failure}")
        return 1
    print("the npm dependency audit: passed")
    return 0


if __name__ == "__main__":
    sys.exit(main())
