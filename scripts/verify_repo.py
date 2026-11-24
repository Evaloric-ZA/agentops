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

Every file the scan visits gets one of six verdicts, and the counts are
printed at the end:

    SCANNED                 the file was searched for patterns
    BINARY_SKIPPED          the file is not text, so there is nothing to read
    GENERATED_EXCLUDED      the file is generated output, not source
    OVERSIZE_REVIEW_REQUIRED the file is larger than the limit and is not a
                            known dependency lockfile, so it could not be
                            searched; a person has to look at it
    UNREADABLE              the file or its directory could not be read
    VIOLATION               a pattern matched

An earlier version of this script returned quietly for a file that was too
large, which meant an oversized file could be added to the tree and simply
disappear from verification. A file that is too large to search is now
reported and the verification fails, so it cannot be forgotten.

What this scanner is NOT: it is a pattern matcher over the working tree.
It does not read the Git history, it does not understand what a value is
for, and it cannot prove that no secret is present. A dedicated secret
scanning service is a different tool with a different job.

Standard library only. It makes no network request, writes nothing and
deletes nothing.
"""

from __future__ import annotations

import argparse
import os
import re
import sys
from dataclasses import dataclass, field
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
    "scripts/verify.ps1",
    "scripts/verify.py",
    "scripts/rust_audit_allowlist.json",
    "scripts/verify_repo.py",
    "scripts/check_version.py",
    "scripts/check_msrv.py",
    "scripts/check_types.py",
    "scripts/test_verify_repo.py",
    ".github/workflows/ci.yml",
    ".node-version",
]

# Directories that hold generated output. They are not part of the source
# and are not scanned.
SKIP_DIRS = {"node_modules", "target", ".git", "dist", "gen", ".cache"}

# A file larger than this is not searched in one piece. Nothing in this
# repository is close to the limit; the guard exists so the scan cannot be
# made to read an enormous file.
MAX_BYTES = 4 * 1024 * 1024

# Known dependency lockfiles are allowed to be larger than the limit, but
# they are NOT exempt: they are searched with the same patterns, read a
# line at a time so the whole file never has to be held in memory. This
# list names the lockfiles this repository uses; it is not a general
# exemption for any file called "a lock file".
CONTROLLED_LOCKFILES = {
    "package-lock.json",
    "npm-shrinkwrap.json",
    "yarn.lock",
    "pnpm-lock.yaml",
    "Cargo.lock",
    "bun.lockb",
    "composer.lock",
    "Gemfile.lock",
    "poetry.lock",
}

# The number of lines the streaming reader carries. A pattern can span a
# line break, because the separator in "api_key = value" may be written
# across two lines. Carrying the previous two lines means a match that
# spans up to two line breaks is still found. A match spanning three or
# more line breaks would not be, which is recorded in docs/SECURITY.md.
CARRY_LINES = 2

# The verdicts, in the order they are reported.
SCANNED = "scanned"
BINARY_SKIPPED = "binary_skipped"
GENERATED_EXCLUDED = "generated_excluded"
OVERSIZE_REVIEW_REQUIRED = "oversize_review_required"
UNREADABLE = "unreadable"
VIOLATION = "violations"

VERDICTS = (
    SCANNED,
    BINARY_SKIPPED,
    GENERATED_EXCLUDED,
    OVERSIZE_REVIEW_REQUIRED,
    UNREADABLE,
    VIOLATION,
)

PATTERNS = {
    # The image extensions are excluded because an icon file name such as
    # 128x128@2x.png is otherwise an email address. The conventional name
    # is the one the icon generator emits, and the tree has to carry it
    # for the icon set to be reproducible from its source.
    "email address": re.compile(
        r"\b[A-Z0-9._%+-]+@[A-Z0-9.-]+\."
        r"(?!png\b|jpe?g\b|gif\b|svg\b|webp\b|ico\b|icns\b|bmp\b|tiff?\b)[A-Z]{2,}\b",
        re.I,
    ),
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


@dataclass
class ScanResult:
    """What the scan found and what it decided about each file."""

    findings: list[str] = field(default_factory=list)
    violations: int = 0
    scanned: int = 0
    binary_skipped: int = 0
    generated_excluded: int = 0
    oversize: list[str] = field(default_factory=list)
    unreadable: list[str] = field(default_factory=list)

    @property
    def counts(self) -> dict[str, int]:
        return {
            SCANNED: self.scanned,
            BINARY_SKIPPED: self.binary_skipped,
            GENERATED_EXCLUDED: self.generated_excluded,
            OVERSIZE_REVIEW_REQUIRED: len(self.oversize),
            UNREADABLE: len(self.unreadable),
            VIOLATION: self.violations,
        }

    @property
    def failed(self) -> bool:
        """True when the verification must fail."""
        return bool(self.findings or self.oversize or self.unreadable)


def _walk(root: Path):
    """Yield every entry under root, with the reason it was skipped.

    `os.scandir` is used rather than `Path.rglob` because it reports the
    error when a directory cannot be listed, instead of passing over it.
    """
    seen: set[str] = set()
    stack = [root]
    while stack:
        directory = stack.pop()
        try:
            real = os.path.realpath(directory)
        except OSError:
            real = str(directory)
        if real in seen:
            # A symbolic link that points back up the tree. Following it
            # would never end.
            yield directory, "loop", None
            continue
        seen.add(real)

        try:
            entries = list(os.scandir(directory))
        except OSError as error:
            yield directory, "error", error
            continue

        for entry in entries:
            path = Path(entry.path)
            try:
                if entry.is_dir():
                    if entry.name in SKIP_DIRS:
                        yield path, "generated", None
                    else:
                        stack.append(path)
                    continue
                if not entry.is_file():
                    # A device, a socket or a symbolic link that points at
                    # nothing. It is not a readable regular file, so it is
                    # reported rather than quietly passed over.
                    yield path, "not-a-file", None
                    continue
            except OSError as error:
                yield path, "error", error
                continue
            yield path, "file", None


def _looks_binary(block: bytes) -> bool:
    """A NUL byte in the first block means this is not text."""
    return b"\x00" in block[:8192]


def _position(text: str, start: int) -> tuple[int, int]:
    """The line and the column of an offset, both counted from one."""
    line = text.count("\n", 0, start) + 1
    column = start - text.rfind("\n", 0, start)
    return line, column


def _findings_in(text: str, relative: str, base_line: int = 1) -> list[tuple[int, int, str]]:
    """Every pattern match in the text, as (line, column, label)."""
    found: list[tuple[int, int, str]] = []
    for label, pattern in PATTERNS.items():
        for match in pattern.finditer(text):
            value = match.group(0)
            if any(allowed in value.lower() for allowed in ALLOWED):
                continue
            line, column = _position(text, match.start())
            found.append((base_line + line - 1, column, label))
    return found


def _scan_stream(path: Path, relative: str) -> list[tuple[int, int, str]]:
    """Search a file a line at a time, without holding it all in memory.

    Used for a dependency lockfile that is larger than the limit. The
    previous two lines are carried, so a pattern that spans up to two line
    breaks is still found.
    """
    found: list[tuple[int, int, str]] = []
    reported: set[int] = set()
    carry: list[str] = []
    carry_base_line = 1
    offset = 0

    with path.open("r", encoding="utf-8", errors="replace") as handle:
        for line in handle:
            window = "".join(carry) + line
            window_base = offset - sum(len(part) for part in carry)
            for label, pattern in PATTERNS.items():
                for match in pattern.finditer(window):
                    absolute = window_base + match.start()
                    if absolute in reported:
                        continue
                    reported.add(absolute)
                    value = match.group(0)
                    if any(allowed in value.lower() for allowed in ALLOWED):
                        continue
                    line_no, column = _position(window, match.start())
                    found.append((carry_base_line + line_no - 1, column, label))
            offset += len(line)
            carry.append(line)
            if len(carry) > CARRY_LINES:
                carry.pop(0)
                carry_base_line += 1

    return found


def scan(root: Path) -> ScanResult:
    """Scan the tree and decide what happened to every file."""
    result = ScanResult()

    for path, kind, error in _walk(root):
        try:
            relative = str(path.relative_to(root))
        except ValueError:
            relative = str(path)

        if kind == "generated":
            result.generated_excluded += 1
            continue
        if kind == "loop":
            result.unreadable.append(f"{relative}: a symbolic link points back up the tree")
            continue
        if kind == "not-a-file":
            result.unreadable.append(f"{relative}: not a regular file")
            continue
        if kind == "error":
            result.unreadable.append(f"{relative}: {error}")
            continue

        try:
            size = path.stat().st_size
        except OSError as error:
            result.unreadable.append(f"{relative}: {error}")
            continue

        if size > MAX_BYTES:
            # Too large to read in one piece. The first block decides
            # whether it is text at all: a large image or archive is
            # binary and there is nothing in it to read.
            try:
                with path.open("rb") as handle:
                    block = handle.read(8192)
            except OSError as error:
                result.unreadable.append(f"{relative}: {error}")
                continue
            if _looks_binary(block):
                result.binary_skipped += 1
                continue
            if path.name in CONTROLLED_LOCKFILES:
                # A known dependency lockfile: searched a line at a time,
                # with the same patterns as every other file.
                try:
                    found = _scan_stream(path, relative)
                except OSError as error:
                    result.unreadable.append(f"{relative}: {error}")
                    continue
                result.scanned += 1
                for line, column, label in found:
                    result.violations += 1
                    result.findings.append(
                        f"{relative}:{line}:{column}: possible {label} (the value is not shown)"
                    )
                continue
            # A large text file that is not a known lockfile. It could be
            # source, a script, a configuration file, an environment file
            # or documentation, so it cannot be passed over: a person has
            # to look at it.
            result.oversize.append(
                f"{relative}: larger than {MAX_BYTES} bytes and not a known dependency "
                "lockfile, so it was not searched; review it by hand"
            )
            continue

        try:
            raw = path.read_bytes()
        except OSError as error:
            result.unreadable.append(f"{relative}: {error}")
            continue

        if _looks_binary(raw):
            result.binary_skipped += 1
            continue
        try:
            text = raw.decode("utf-8")
        except UnicodeDecodeError:
            result.binary_skipped += 1
            continue

        result.scanned += 1
        for line, column, label in _findings_in(text, relative):
            result.violations += 1
            result.findings.append(
                f"{relative}:{line}:{column}: possible {label} (the value is not shown)"
            )

    return result


def format_summary(result: ScanResult) -> list[str]:
    """The six lines that report what the scan decided."""
    counts = result.counts
    return [
        "Scanned: %d" % counts[SCANNED],
        "Binary skipped: %d" % counts[BINARY_SKIPPED],
        "Generated excluded: %d" % counts[GENERATED_EXCLUDED],
        "Oversize review required: %d" % counts[OVERSIZE_REVIEW_REQUIRED],
        "Unreadable: %d" % counts[UNREADABLE],
        "Violations: %d" % counts[VIOLATION],
    ]


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("root", type=Path)
    args = parser.parse_args()
    root = args.root.resolve()

    errors: list[str] = []
    for item in REQUIRED:
        if not (root / item).exists():
            errors.append(f"missing required path: {item}")

    result = scan(root)
    errors.extend(result.findings)
    errors.extend(result.oversize)
    errors.extend(result.unreadable)

    for line in format_summary(result):
        print(line)

    if errors:
        print()
        print("Repository verification failed:")
        for error in errors:
            print(f"- {error}")
        return 1

    print()
    print("Repository structure and privacy scan passed.")
    return 0


if __name__ == "__main__":
    sys.exit(main())
