#!/usr/bin/env python3
"""Make `cargo tauri ...` work.

Cargo looks for a subcommand named `cargo-tauri`, but the npm
`@tauri-apps/cli` package installs one named `tauri`. A small bridge
connects the two.

The bridge has to be created after `npm ci` has run, because `npm ci`
replaces the whole of `node_modules` and would take the bridge with it.

Cargo passes the name of the subcommand to an external subcommand, so
the first argument the bridge receives is "tauri". The command line
interface wants the subcommand itself, not that name, so the name is
dropped.

This lives in its own file because two places need it: the sweep, and
the continuous integration job that builds the Windows application. One
implementation, so the two cannot drift apart.

Run it as a script to create the bridge:

    python scripts/ensure_cargo_tauri.py [the repository root]
"""

from __future__ import annotations

import os
import sys
from pathlib import Path


def ensure_cargo_tauri(root: Path) -> bool:
    """Create the bridge. Returns False when the Tauri CLI is not installed."""
    bin_dir = root / "node_modules" / ".bin"
    entry = root / "node_modules" / "@tauri-apps" / "cli" / "tauri.js"
    if not entry.exists():
        return False

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


def main(argv: list[str]) -> int:
    root = Path(argv[1]).resolve() if len(argv) > 1 else Path(__file__).resolve().parent.parent
    if not ensure_cargo_tauri(root):
        print(
            "the Tauri command line is not installed, so the bridge was not created",
            file=sys.stderr,
        )
        return 1
    print("the cargo tauri bridge is in place")
    return 0


if __name__ == "__main__":
    sys.exit(main(sys.argv))
