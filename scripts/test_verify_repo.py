#!/usr/bin/env python3
"""The privacy scanner's own regression test.

The scanner once scanned only a list of known text suffixes, and that was
a real false negative: a secret written into a `.sh`, `.env`, `.lock` or
`.py` file was not detected, because those suffixes were not on the list.
It also required a quoted value, so the usual unquoted `.env` form was
missed, and it required a bare key, so the quoted JSON form was missed.
It also passed quietly over any file that was too large to read, so an
oversized file could be added to the tree and disappear from verification.

This test builds a throwaway tree for each case and asserts what the
scanner does with it, so none of those gaps can come back.

The payload strings are assembled from fragments on purpose. The scanner
reads this file too, and a literal secret written here would be a finding
in the repository's own scan.

Standard library only, and it runs with a plain `python3` — no pytest and
no third-party package is needed. It writes into a temporary directory and
removes it afterwards.
"""

from __future__ import annotations

import os
import shutil
import stat
import sys
import tempfile
from pathlib import Path
from unittest.mock import patch

sys.path.insert(0, str(Path(__file__).resolve().parent))

import verify_repo  # noqa: E402
from verify_repo import scan  # noqa: E402

# The payloads, assembled so that no literal in this file matches a
# pattern the scanner looks for.
SECRET = "sk-" + "A" * 36
MAIL = "some" + "one" + "@" + "example" + ".net"
IP = ".".join(["192", "168", "1", "44"])
AUTH = "author" + "ization" + ": " + "Bear" + "er " + "abcdefghijklmnop"

PASSED = 0
FAILED = 0
SKIPPED = 0


def report(name: str, ok: bool, detail: str = "") -> None:
    global PASSED, FAILED
    if ok:
        PASSED += 1
        print(f"ok   {name}")
    else:
        FAILED += 1
        print(f"FAIL {name}{': ' + detail if detail else ''}")


def skip(name: str, reason: str) -> None:
    """A case that this environment cannot exercise.

    It is never counted as a pass: the count of skipped cases is
    reported on its own line.
    """
    global SKIPPED
    SKIPPED += 1
    print(f"SKIP {name}: {reason}")


def build(files: dict[str, bytes | str]) -> Path:
    """Build a throwaway tree holding the named files."""
    root = Path(tempfile.mkdtemp(prefix="agentops-scan-"))
    for name, content in files.items():
        target = root / name
        target.parent.mkdir(parents=True, exist_ok=True)
        if isinstance(content, bytes):
            target.write_bytes(content)
        else:
            target.write_text(content, encoding="utf-8")
    return root


def with_file(name: str, content: str, binary: bool = False):
    """Build a tree holding one file and scan it."""
    root = build({name: (content.encode("utf-8") + b"\x00\x01\x02") if binary else content})
    try:
        return scan(root)
    finally:
        shutil.rmtree(root, ignore_errors=True)


def expect_caught(name: str, content: str, binary: bool = False) -> None:
    result = with_file(name, content, binary=binary)
    report(f"catches {name}", bool(result.findings), "the scanner found nothing")


def expect_clean(name: str, content: str, binary: bool = False) -> None:
    result = with_file(name, content, binary=binary)
    report(f"allows {name}", not result.findings, f"unexpected findings: {result.findings[:2]}")


def the_oversize_cases() -> None:
    """The cases that need a file larger than the scanner's limit.

    The limit is lowered for these cases so the test does not have to
    write four megabytes to disk for each one. The code path is the same
    one a real oversized file takes, because `scan` reads the limit from
    this module at the moment it runs.
    """
    small_limit = 1024
    filler = ("an ordinary line of text\n" * 100)

    # 5. An oversized source file.
    root = build({"src/main.rs": filler})
    try:
        with patch.object(verify_repo, "MAX_BYTES", small_limit):
            result = scan(root)
        report(
            "an oversized source file needs review",
            len(result.oversize) == 1 and result.failed,
            f"the oversize list holds {len(result.oversize)} entries",
        )
        report(
            "an oversized source file is not counted as scanned",
            result.scanned == 0,
            f"the scan counted {result.scanned} files",
        )
    finally:
        shutil.rmtree(root, ignore_errors=True)

    # 6. An oversized configuration file.
    root = build({"config/settings.json": filler})
    try:
        with patch.object(verify_repo, "MAX_BYTES", small_limit):
            result = scan(root)
        report(
            "an oversized configuration file needs review",
            len(result.oversize) == 1 and result.failed,
            f"the oversize list holds {len(result.oversize)} entries",
        )
    finally:
        shutil.rmtree(root, ignore_errors=True)

    # 7. An oversized dependency lockfile: searched, not exempt.
    root = build(
        {
            "package-lock.json": (
                "{\n  \"packages\": {\n" + filler + f'    "apiKey": "{SECRET}"\n' + "  }\n}\n"
            )
        }
    )
    try:
        with patch.object(verify_repo, "MAX_BYTES", small_limit):
            result = scan(root)
        report(
            "an oversized dependency lockfile is searched, not exempt",
            result.scanned == 1 and not result.oversize,
            f"scanned {result.scanned}, oversize {len(result.oversize)}",
        )
        report(
            "a secret inside an oversized dependency lockfile is found",
            bool(result.findings),
            "the scanner found nothing in the lockfile",
        )
    finally:
        shutil.rmtree(root, ignore_errors=True)

    # The limit the repository actually uses.
    report(
        "the scanner's limit is four megabytes",
        verify_repo.MAX_BYTES == 4 * 1024 * 1024,
        f"the limit is {verify_repo.MAX_BYTES} bytes",
    )


def the_unreadable_case() -> None:
    """8. A file the scan cannot read."""
    root = build({"locked.txt": "ordinary text\n"})
    target = root / "locked.txt"
    os.chmod(target, 0)
    try:
        try:
            readable = target.read_text(encoding="utf-8")
        except OSError:
            readable = None
        if readable is not None:
            skip(
                "an unreadable file is reported",
                "this environment can read a file with no read permission",
            )
            return
        result = scan(root)
        report(
            "an unreadable file is reported",
            len(result.unreadable) == 1 and result.failed,
            f"the unreadable list holds {len(result.unreadable)} entries",
        )
    finally:
        os.chmod(target, stat.S_IRUSR | stat.S_IWUSR)
        shutil.rmtree(root, ignore_errors=True)


def the_end_of_file_case() -> None:
    """9. A secret near the end of a file that is an ordinary size."""
    padding = "an ordinary line\n" * 500
    root = build({"notes.txt": padding + f'api_key = "{SECRET}"\n' + "the last line\n"})
    try:
        result = scan(root)
        report("a secret near the end of a file is found", bool(result.findings))
        report(
            "the finding points at the line the secret is on",
            any(":501:" in finding for finding in result.findings),
            f"the findings are {result.findings[:2]}",
        )
    finally:
        shutil.rmtree(root, ignore_errors=True)


def the_redaction_case() -> None:
    """10. The detected value is not shown."""
    root = build({"config.env": f'api_key = "{SECRET}"\n'})
    try:
        result = scan(root)
        report("the secret is found", bool(result.findings))
        report(
            "the finding does not repeat the value",
            all(SECRET not in finding for finding in result.findings),
            "the finding printed the value it found",
        )
        report(
            "the finding still says where to look",
            any("config.env:" in finding for finding in result.findings),
            f"the findings are {result.findings[:2]}",
        )
    finally:
        shutil.rmtree(root, ignore_errors=True)


def the_summary_case() -> None:
    """11. The summary counts every verdict."""
    root = build(
        {
            "plain.txt": "ordinary text\n",
            "image.bin": b"\x00\x01\x02binary",
            "node_modules/thing/index.js": "generated\n",
        }
    )
    try:
        result = scan(root)
        counts = result.counts
        report(
            "the summary holds every verdict",
            set(counts) == set(verify_repo.VERDICTS),
            f"the summary holds {sorted(counts)}",
        )
        report("the summary counts the scanned file", counts["scanned"] == 1)
        report("the summary counts the skipped binary", counts["binary_skipped"] == 1)
        report(
            "the summary counts the generated directory",
            counts["generated_excluded"] == 1,
            f"the count is {counts['generated_excluded']}",
        )
        report("the summary counts the violations", counts["violations"] == 0)
        lines = verify_repo.format_summary(result)
        report(
            "the summary is six readable lines",
            len(lines) == 6
            and lines[0].startswith("Scanned:")
            and lines[5].startswith("Violations:"),
            f"the summary is {lines}",
        )
    finally:
        shutil.rmtree(root, ignore_errors=True)


def the_basic_cases() -> None:
    # 1. An ordinary small text file.
    result = with_file("notes.txt", "an ordinary file\n")
    report(
        "an ordinary small text file is scanned",
        result.scanned == 1 and not result.findings,
        f"scanned {result.scanned}",
    )

    # 2. A small binary file.
    result = with_file("image.bin", "some bytes", binary=True)
    report(
        "a small binary file is skipped",
        result.binary_skipped == 1 and result.scanned == 0,
        f"skipped {result.binary_skipped}, scanned {result.scanned}",
    )

    # 3. A generated directory is excluded.
    root = build({"node_modules/thing/index.js": "generated output\n"})
    try:
        result = scan(root)
        report(
            "a generated directory is excluded",
            result.generated_excluded == 1 and result.scanned == 0,
            f"excluded {result.generated_excluded}, scanned {result.scanned}",
        )
    finally:
        shutil.rmtree(root, ignore_errors=True)

    # 4. A readable file with no extension.
    result = with_file("README", "an ordinary file\n")
    report(
        "a readable file with no extension is scanned",
        result.scanned == 1 and not result.findings,
        f"scanned {result.scanned}",
    )


def main() -> int:
    the_basic_cases()
    the_oversize_cases()
    the_unreadable_case()
    the_end_of_file_case()
    the_redaction_case()
    the_summary_case()

    # The extension false negatives that motivated the extension-blind scan.
    expect_caught("a secret in a .sh file", f'export API_KEY="{SECRET}"\n')
    expect_caught("a secret in a .env file", f"API_KEY={SECRET}\n")
    expect_caught("a secret in a .lock file", f'secret = "{SECRET}"\n')
    expect_caught("a secret in a .py file", f'TOKEN = "{SECRET}"\n')
    expect_caught("a secret in a .json file", f'{{"api_key": "{SECRET}"}}\n')
    expect_caught("a secret in a .md file", f'The key is `api_key = "{SECRET}"`.\n')
    expect_caught("a secret in a file with no extension", f'api_key = "{SECRET}"\n')
    expect_caught("a secret in a .svg file", f'<text>api_key="{SECRET}"</text>\n')
    expect_caught("a secret in a .yml file", f'  api_key: "{SECRET}"\n')

    # The value forms that were missed: bare, and a quoted key.
    expect_caught("a bare secret value", f"credential={SECRET}\n")
    expect_caught("a quoted secret key", f'{{"password": "{SECRET}"}}\n')
    expect_caught("a colon separated secret", f"token: {SECRET}\n")

    # The key names that were missed because a word boundary cannot sit
    # next to an underscore.
    expect_caught("an uppercase underscored secret", f"SECRET_KEY={SECRET}\n")
    expect_caught("a prefixed password", f"DB_PASSWORD={SECRET}\n")
    expect_caught("a suffixed password", f'PASSWORD_FILE = "{SECRET}"\n')
    expect_caught("a lowercase underscored secret", f"secret_key={SECRET}\n")
    expect_caught("a prefixed api key", f"MY_API_KEY={SECRET}\n")
    expect_caught("a camel case api key", f"apiKey={SECRET}\n")

    # A word that merely starts with one of the names is not a secret key.
    expect_clean("a counter that is not a secret", "tokenizer_count = 42\n")
    expect_clean("a short value", "secret = 42\n")

    # The other patterns.
    expect_caught("an email address", f"contact {MAIL} about it\n")
    expect_caught("a private IPv4 address", f"the host is {IP}\n")
    expect_caught("a bearer token", AUTH + "\n")

    # Values that are documented as safe.
    expect_clean("a loopback address", "the server is on 127.0.0.1\n")
    expect_clean("the example domain", "see https://example.com/docs\n")
    expect_clean("a placeholder secret", 'api_key = "synthetic-placeholder"\n')
    expect_clean("an ordinary shell script", "#!/bin/sh\necho hello\n")
    expect_clean("a binary file", "some bytes", binary=True)

    # An empty source file is usually an edit that went to the wrong
    # path: the file was created and the content landed somewhere else.
    # The compiler never sees it, so nothing else would notice it.
    #
    # The paths are compared with forward slashes, which is what the
    # function returns on every platform: the first version of this
    # case compared a Windows path with a slash in it and failed only
    # on the Windows runner.
    empty_root = build({"src/empty.rs": "", "src/full.rs": "fn main() {}\n"})
    empty = verify_repo.empty_source_files(empty_root)
    report("an empty source file is found", empty == ["src/empty.rs"], str(empty))
    report(
        "a source file with content is not flagged",
        verify_repo.empty_source_files(build({"src/full.rs": "fn main() {}\n"})) == [],
    )
    report(
        "an empty file outside the source directories is not flagged",
        verify_repo.empty_source_files(build({"docs/notes.md": ""})) == [],
    )

    print()
    print(f"{PASSED} passed, {FAILED} failed, {SKIPPED} skipped")
    return 1 if FAILED else 0


if __name__ == "__main__":
    sys.exit(main())
