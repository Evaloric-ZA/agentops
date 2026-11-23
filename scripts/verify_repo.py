#!/usr/bin/env python3
"""The repository structure and privacy scan.

Two jobs:

1. Assert that the files a release needs are present.
2. Assert that no secret, email address or private network address is
   embedded in the tree.

The scan is deliberately extension-blind. An earlier version of this script
scanned only a list of known text suffixes, and that was a real false
negative: a secret written into a `.sh`, `.env`, `.lock` or `.py` file was
not detected, because those suffixes were not on the list. This version
reads every file in the tree and scans it if it decodes as UTF-8 and
contains no NUL byte, so a file's name cannot decide whether it is checked.

Standard library only. It makes no network request, writes nothing and
deletes nothing.
"""

from __future__ import annotations

import argparse
import re
import sys
from pathlib import Path

# The files and directories a release needs.
REQUIRED = [
    "README.md",
    "BUILD_STATE.md",
    "CHANGELOG.md",
    "LICENSE",
    "package.json",
    "src",
    "src-tauri",
    "src-tauri/Cargo.toml",
    "src-tauri/tauri.conf.json",
    "src-tauri/capabilities/default.json",
    "docs/ARCHITECTURE.md",
    "docs/SECURITY.md",
    "docs/TESTING.md",
    "docs/PACKAGING.md",
    "scripts/verify.sh",
    "scripts/verify_repo.py",
    "scripts/check_version.py",
    "scripts/check_msrv.py",
    "scripts/check_types.py",
    "scripts/test_verify_repo.py",
    ".github/workflows/ci.yml",
]

# Directories that hold generated output. They are not part of the source
# and are not scanned.
SKIP_DIRS = {"node_modules", "target", ".git", "dist", "gen", ".cache"}

# A file larger than this is not scanned. Nothing in this repository is
# close to the limit; the guard exists so the scan cannot be made to read
# an enormous file.
MAX_BYTES = 4 * 1024 * 1024

PATTERNS = {
    "email address": re.compile(r"\b[A-Z0-9._%+-]+@[A-Z0-9.-]+\.[A-Z]{2,}\b", re.I),
    "private IPv4 address": re.compile(
        r"\b(?:10\.\d{1,3}\.\d{1,3}\.\d{1,3}|192\.168\.\d{1,3}\.\d{1,3}"
        r"|172\.(?:1[6-9]|2\d|3[01])\.\d{1,3}\.\d{1,3})\b"
    ),
    "probable bearer token": re.compile(
        r"(?i)authorization\s*[:=]\s*bearer\s+[A-Za-z0-9._~-]{12,}"
    ),
    # The key may be quoted (the JSON form) or bare (the .env form), and
    # the value may be quoted or bare. Both forms were missed before.
    #
    # The key may also carry a prefix or a suffix, so SECRET_KEY,
    # DB_PASSWORD and PASSWORD_FILE are all recognised. The separators
    # are explicit, so a word that merely starts with one of the names —
    # tokenizer, for instance — is not.
    "probable secret assignment": re.compile(
        r"(?i)(?<![A-Za-z0-9])"
        r"(?:[A-Za-z0-9]+[_-])*"
        r"(?:api[_-]?key|secret|password|passwd|credential|token)"
        r"(?:[_-][A-Za-z0-9]+)*"
        r"\s*['\"]?\s*[:=]\s*['\"]?"
        r"(?!synthetic|example|placeholder)[A-Za-z0-9._~+/=-]{12,}"
    ),
}

# Values that are documented as safe and are not a leak.
ALLOWED = ("example.com", "example.org", "127.0.0.1", "localhost")


def read_if_text(path: Path) -> str | None:
    """Return the file's text, or None if it is not text worth scanning."""
    try:
        if path.stat().st_size > MAX_BYTES:
            return None
        raw = path.read_bytes()
    except OSError:
        return None

    # A NUL byte in the first block means this is binary.
    if b"\x00" in raw[:8192]:
        return None

    try:
        return raw.decode("utf-8")
    except UnicodeDecodeError:
        return None


def scan(root: Path) -> tuple[list[str], int]:
    """Scan the tree. Returns the findings and the number of files read."""
    errors: list[str] = []
    scanned = 0

    for path in sorted(root.rglob("*")):
        if not path.is_file():
            continue
        if any(part in SKIP_DIRS for part in path.parts):
            continue

        text = read_if_text(path)
        if text is None:
            continue
        scanned += 1

        for label, pattern in PATTERNS.items():
            for match in pattern.finditer(text):
                value = match.group(0)
                if any(allowed in value.lower() for allowed in ALLOWED):
                    continue
                errors.append(
                    f"{path.relative_to(root)}: possible {label}: {value[:100]}"
                )

    return errors, scanned


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("root", type=Path)
    args = parser.parse_args()
    root = args.root.resolve()

    errors: list[str] = []
    for item in REQUIRED:
        if not (root / item).exists():
            errors.append(f"missing required path: {item}")

    findings, scanned = scan(root)
    errors.extend(findings)

    if errors:
        print("Repository verification failed:")
        for error in errors:
            print(f"- {error}")
        return 1

    print(f"Repository structure and privacy scan passed. ({scanned} files scanned)")
    return 0


if __name__ == "__main__":
    sys.exit(main())
