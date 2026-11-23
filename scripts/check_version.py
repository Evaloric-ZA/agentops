#!/usr/bin/env python3
"""The version consistency check.

The application version is written in four places: `package.json`,
`src-tauri/Cargo.toml`, `src-tauri/tauri.conf.json` and the `agentops`
entry in `src-tauri/Cargo.lock`. Nothing in the toolchain forces them to
agree, so a bump that misses one of them produces a release whose
manifests disagree — and the application does not display its version
anywhere, so the drift is invisible to the operator and to anyone reading
a bug report.

This check makes the agreement a property of the build rather than a
convention. Standard library only.
"""

from __future__ import annotations

import argparse
import json
import re
import sys
from pathlib import Path

SEMVER = re.compile(r"^\d+\.\d+\.\d+(?:-[0-9A-Za-z.-]+)?(?:\+[0-9A-Za-z.-]+)?$")


def package_json_version(root: Path) -> str:
    data = json.loads((root / "package.json").read_text(encoding="utf-8"))
    return str(data["version"])


def cargo_toml_version(root: Path) -> str:
    text = (root / "src-tauri" / "Cargo.toml").read_text(encoding="utf-8")
    # The [package] section's version, not any other version key.
    in_package = False
    for line in text.splitlines():
        stripped = line.strip()
        if stripped.startswith("["):
            in_package = stripped == "[package]"
            continue
        if in_package and stripped.startswith("version"):
            _, _, value = stripped.partition("=")
            return value.strip().strip('"').strip("'")
    raise ValueError("no version in the [package] section of src-tauri/Cargo.toml")


def tauri_conf_version(root: Path) -> str:
    data = json.loads((root / "src-tauri" / "tauri.conf.json").read_text(encoding="utf-8"))
    return str(data["version"])


def cargo_lock_version(root: Path) -> str:
    text = (root / "src-tauri" / "Cargo.lock").read_text(encoding="utf-8")
    blocks = text.split("[[package]]")
    for block in blocks:
        name = re.search(r'^name = "([^"]+)"', block, re.M)
        if name and name.group(1) == "agentops":
            version = re.search(r'^version = "([^"]+)"', block, re.M)
            if version:
                return version.group(1)
    raise ValueError("no agentops package in src-tauri/Cargo.lock")


def check(root: Path) -> list[str]:
    sources = {
        "package.json": package_json_version,
        "src-tauri/Cargo.toml": cargo_toml_version,
        "src-tauri/tauri.conf.json": tauri_conf_version,
        "src-tauri/Cargo.lock": cargo_lock_version,
    }

    versions: dict[str, str] = {}
    errors: list[str] = []

    for name, reader in sources.items():
        try:
            versions[name] = reader(root)
        except (OSError, ValueError, KeyError, json.JSONDecodeError) as error:
            errors.append(f"{name}: could not read the version: {error}")

    if errors:
        return errors

    distinct = set(versions.values())
    if len(distinct) > 1:
        detail = ", ".join(f"{name} = {value}" for name, value in versions.items())
        errors.append(f"the version differs between the manifests: {detail}")
        return errors

    version = distinct.pop()
    if not SEMVER.match(version):
        errors.append(f"the version {version!r} is not a semantic version")
        return errors

    print(f"the version is {version} in every manifest.")
    return errors


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("root", type=Path)
    args = parser.parse_args()

    errors = check(args.root.resolve())
    if errors:
        print("The version check failed:")
        for error in errors:
            print(f"- {error}")
        return 1
    return 0


if __name__ == "__main__":
    sys.exit(main())
