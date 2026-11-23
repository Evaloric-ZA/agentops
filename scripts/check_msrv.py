#!/usr/bin/env python3
"""The minimum supported Rust check.

`src-tauri/Cargo.toml` declares a `rust-version`. That is a promise: the
crate builds on that compiler and no older one. Nothing in Cargo enforces
it — Cargo only warns, and a dependency that needs a newer compiler fails
with an obscure message like *"feature `edition2024` is required"*.

This check makes the promise verifiable. It reads the declared minimum,
asks Cargo for the resolved dependency graph, and compares the minimum
that every dependency itself declares against the one this crate claims.

It found a false claim in v0.1: the crate declared Rust 1.77 while
`tauri` itself requires 1.90.

Standard library only.
"""

from __future__ import annotations

import argparse
import json
import re
import subprocess
import sys
from pathlib import Path


def version_key(value: str) -> tuple[int, int, int]:
    """Sort a Rust version like 1.90.0, 1.9 or 1.90.0-alpha."""
    parts = value.split("-")[0].split(".")
    numbers = []
    for part in parts[:3]:
        numbers.append(int(part) if part.isdigit() else 0)
    while len(numbers) < 3:
        numbers.append(0)
    return numbers[0], numbers[1], numbers[2]


def declared_minimum(root: Path) -> str:
    text = (root / "src-tauri" / "Cargo.toml").read_text(encoding="utf-8")
    match = re.search(r'^rust-version\s*=\s*"([^"]+)"', text, re.M)
    if not match:
        raise ValueError("src-tauri/Cargo.toml declares no rust-version")
    return match.group(1)


def dependency_minimums(root: Path) -> dict[str, str]:
    """Every resolved package's own declared minimum Rust, by name."""
    manifest = root / "src-tauri" / "Cargo.toml"
    result = subprocess.run(
        ["cargo", "metadata", "--format-version", "1", "--all-features", "--manifest-path", str(manifest)],
        capture_output=True,
        text=True,
        check=False,
    )
    if result.returncode != 0:
        raise RuntimeError(f"cargo metadata failed: {result.stderr.strip()[:300]}")

    data = json.loads(result.stdout)
    minimums: dict[str, str] = {}
    for package in data.get("packages", []):
        rust_version = package.get("rust_version")
        if rust_version:
            minimums[package["name"]] = rust_version
    return minimums


def check(root: Path) -> list[str]:
    errors: list[str] = []

    try:
        declared = declared_minimum(root)
    except (OSError, ValueError) as error:
        return [str(error)]

    try:
        minimums = dependency_minimums(root)
    except (OSError, RuntimeError, json.JSONDecodeError) as error:
        return [str(error)]

    if not minimums:
        return ["no dependency declared a minimum Rust, which is not expected"]

    declared_key = version_key(declared)
    too_new = {
        name: minimum
        for name, minimum in minimums.items()
        if version_key(minimum) > declared_key
    }

    if too_new:
        worst = max(too_new.values(), key=version_key)
        names = sorted(name for name, value in too_new.items() if value == worst)
        errors.append(
            f"the crate declares rust-version {declared}, but {len(too_new)} "
            f"dependencies need more; the highest is {worst}, required by "
            f"{', '.join(names[:6])}"
        )
        return errors

    highest = max(minimums.values(), key=version_key)
    print(
        f"the declared minimum Rust is {declared}, and the highest any "
        f"dependency requires is {highest}."
    )
    return errors


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("root", type=Path)
    args = parser.parse_args()

    errors = check(args.root.resolve())
    if errors:
        print("The minimum supported Rust check failed:")
        for error in errors:
            print(f"- {error}")
        return 1
    return 0


if __name__ == "__main__":
    sys.exit(main())
