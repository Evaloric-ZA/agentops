#!/usr/bin/env python3
"""The shared policy for the two dependency audits.

Both audits answer the same question — "is this advisory allowed to
remain?" — and both answer it with the same rules, so the rules live
here rather than being written twice.

The rules are:

- An exception is a dated decision, not a permanent exemption. Every
  one of them carries the date it was added and the date it has to be
  looked at again.
- An exception that is past its review date has expired. An expired
  exception fails the release gate. Nothing renews it automatically:
  somebody has to look at it and decide again.
- An exception must say which advisory it is for, which package it
  covers, why it is accepted, what its scope is, and what would close
  it. An entry that does not say all of that is a malformed entry and
  fails the gate rather than being quietly ignored.
- An advisory that is not listed is not accepted. The gates enumerate
  what they found, so a new advisory cannot hide behind an old one.

Standard library only.
"""

from __future__ import annotations

import json
from datetime import date, datetime
from pathlib import Path

# The fields every npm exception must carry. The scope says whether the
# affected package ships in the desktop application or only runs in the
# development and continuous integration toolchain.
NPM_REQUIRED = (
    "advisory",
    "package",
    "reason",
    "scope",
    "added",
    "review_by",
    "remediation",
    "owner",
)

# The fields every Rust exception must carry. The upstream dependency
# says who is responsible for the crate, because that is the party who
# has to publish a fixed release.
RUST_REQUIRED = (
    "advisory",
    "crate",
    "reason",
    "scope",
    "added",
    "review_by",
    "remediation",
    "upstream_dependency",
    "status",
)

NPM_SCOPES = ("development", "runtime")
RUST_SCOPES = ("development", "runtime", "build-time", "platform-specific")

SEVERITIES = ("critical", "high", "moderate", "low", "info")


def parse_date(value: object, label: str, problems: list[str]) -> date | None:
    """Parse a YYYY-MM-DD date, recording a problem if it is not one."""
    if not isinstance(value, str):
        problems.append(f"{label}: the date {value!r} is not a string")
        return None
    try:
        return datetime.strptime(value, "%Y-%m-%d").date()
    except ValueError:
        problems.append(f"{label}: {value!r} is not a date in YYYY-MM-DD form")
        return None


def load_allowlist(
    path: Path,
    required: tuple[str, ...],
    label: str,
    scopes: tuple[str, ...] = (),
) -> tuple[list[dict], list[str]]:
    """Read an allowlist and check every entry against the schema.

    Returns the entries and the list of problems. A file that cannot be
    read, or an entry that does not carry the fields the policy
    requires, is a problem: the gate then fails closed, because a
    malformed allowlist must not turn into an empty allowlist.
    """
    problems: list[str] = []
    if not path.exists():
        problems.append(f"{label}: the file {path.name} does not exist")
        return [], problems
    try:
        raw = path.read_text(encoding="utf-8")
    except OSError as error:
        problems.append(f"{label}: the file could not be read: {error}")
        return [], problems
    try:
        data = json.loads(raw)
    except ValueError as error:
        problems.append(f"{label}: the file is not valid JSON: {error}")
        return [], problems
    if not isinstance(data, dict):
        problems.append(f"{label}: the file is not a JSON object")
        return [], problems
    entries = data.get("advisories")
    if entries is None:
        entries = []
    if not isinstance(entries, list):
        problems.append(f"{label}: the advisories key is not a list")
        return [], problems

    seen: set[str] = set()
    for entry in entries:
        if not isinstance(entry, dict):
            problems.append(f"{label}: an entry is not a JSON object")
            continue
        name = entry.get("advisory")
        name = name if isinstance(name, str) and name else "?"
        for field in required:
            if field not in entry:
                problems.append(f"{label}: {name} is missing {field}")
            elif not isinstance(entry[field], str) or not entry[field].strip():
                problems.append(f"{label}: {name}: {field} is empty")
        if scopes:
            scope = entry.get("scope")
            if isinstance(scope, str) and scope not in scopes:
                problems.append(
                    f"{label}: {name}: the scope {scope!r} is not one of {', '.join(scopes)}"
                )
        if isinstance(name, str) and name != "?":
            if name in seen:
                problems.append(f"{label}: {name} is listed more than once")
            seen.add(name)
        added = parse_date(entry.get("added"), f"{label}: {name}", problems)
        review = parse_date(entry.get("review_by"), f"{label}: {name}", problems)
        if added and review and review <= added:
            problems.append(
                f"{label}: {name}: the review date {review} is not after the date "
                f"it was added, {added}"
            )
    return entries, problems


def expired(entry: dict, when: date | None = None) -> bool:
    """Whether the entry is past its review date.

    The review date is the last day the exception may be relied on, so
    an exception is expired the day after it. Nothing here renews an
    expired exception: the gate has to fail and somebody has to decide
    again.
    """
    review = entry.get("review_by")
    if not isinstance(review, str):
        return True
    try:
        deadline = datetime.strptime(review, "%Y-%m-%d").date()
    except ValueError:
        return True
    return (when or date.today()) > deadline


def days_left(entry: dict, when: date | None = None) -> int | None:
    """How many days are left before the exception expires."""
    review = entry.get("review_by")
    if not isinstance(review, str):
        return None
    try:
        deadline = datetime.strptime(review, "%Y-%m-%d").date()
    except ValueError:
        return None
    return (deadline - (when or date.today())).days


def severity_rank(severity: str) -> int:
    """Order the severities, highest first. Unknown names rank lowest."""
    try:
        return SEVERITIES.index(severity.lower())
    except ValueError:
        return len(SEVERITIES)
