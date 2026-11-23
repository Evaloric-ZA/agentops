#!/usr/bin/env python3
"""The privacy scanner's own regression test.

The scanner once scanned only a list of known text suffixes, and that was
a real false negative: a secret written into a `.sh`, `.env`, `.lock` or
`.py` file was not detected, because those suffixes were not on the list.
It also required a quoted value, so the usual unquoted `.env` form was
missed, and it required a bare key, so the quoted JSON form was missed.
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

import shutil
import sys
import tempfile
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parent))

from verify_repo import scan  # noqa: E402

# The payloads, assembled so that no literal in this file matches a
# pattern the scanner looks for.
SECRET = "sk-" + "A" * 36
MAIL = "some" + "one" + "@" + "example" + ".net"
IP = ".".join(["192", "168", "1", "44"])
AUTH = "author" + "ization" + ": " + "Bear" + "er " + "abcdefghijklmnop"

PASSED = 0
FAILED = 0


def report(name: str, ok: bool, detail: str = "") -> None:
    global PASSED, FAILED
    if ok:
        PASSED += 1
        print(f"ok   {name}")
    else:
        FAILED += 1
        print(f"FAIL {name}{': ' + detail if detail else ''}")


def with_file(name: str, content: str, binary: bool = False) -> list[str]:
    """Build a tree holding one file and scan it."""
    root = Path(tempfile.mkdtemp(prefix="agentops-scan-"))
    try:
        target = root / name
        target.parent.mkdir(parents=True, exist_ok=True)
        if binary:
            target.write_bytes(content.encode("utf-8") + b"\x00\x01\x02")
        else:
            target.write_text(content, encoding="utf-8")
        findings, _ = scan(root)
        return findings
    finally:
        shutil.rmtree(root, ignore_errors=True)


def expect_caught(name: str, content: str, binary: bool = False) -> None:
    findings = with_file(name, content, binary=binary)
    report(f"catches {name}", bool(findings), f"the scanner found nothing in {name}")


def expect_clean(name: str, content: str, binary: bool = False) -> None:
    findings = with_file(name, content, binary=binary)
    report(f"allows {name}", not findings, f"unexpected findings: {findings[:2]}")


def main() -> int:
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

    print()
    print(f"{PASSED} passed, {FAILED} failed")
    return 1 if FAILED else 0


if __name__ == "__main__":
    sys.exit(main())
