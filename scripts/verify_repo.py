#!/usr/bin/env python3
import argparse
import re
import sys
from pathlib import Path

REQUIRED = [
    "README.md", "BUILD_STATE.md", "package.json", "src", "src-tauri",
    "src-tauri/Cargo.toml", "src-tauri/tauri.conf.json", "docs/ARCHITECTURE.md",
    "docs/SECURITY.md", "docs/TESTING.md", "docs/PACKAGING.md"
]
TEXT_SUFFIXES = {".rs", ".ts", ".js", ".svelte", ".json", ".toml", ".md", ".css", ".html", ".sql", ".yaml", ".yml"}
PATTERNS = {
    "email address": re.compile(r"\b[A-Z0-9._%+-]+@[A-Z0-9.-]+\.[A-Z]{2,}\b", re.I),
    "private IPv4 address": re.compile(r"\b(?:10\.\d{1,3}\.\d{1,3}\.\d{1,3}|192\.168\.\d{1,3}\.\d{1,3}|172\.(?:1[6-9]|2\d|3[01])\.\d{1,3}\.\d{1,3})\b"),
    "probable bearer token": re.compile(r"(?i)authorization\s*[:=]\s*bearer\s+[A-Za-z0-9._~-]{12,}"),
    "probable secret assignment": re.compile(r"(?i)\b(?:api[_-]?key|secret|password|token)\s*[:=]\s*['\"](?!synthetic|example|placeholder)[^'\"]{8,}['\"]"),
}


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("root", type=Path)
    args = parser.parse_args()
    root = args.root.resolve()
    errors = []
    for item in REQUIRED:
        if not (root / item).exists():
            errors.append(f"missing required path: {item}")

    for path in root.rglob("*"):
        if not path.is_file() or path.suffix.lower() not in TEXT_SUFFIXES:
            continue
        if any(part in {"node_modules", "target", ".git"} for part in path.parts):
            continue
        try:
            text = path.read_text(encoding="utf-8")
        except UnicodeDecodeError:
            continue
        for label, pattern in PATTERNS.items():
            for match in pattern.finditer(text):
                value = match.group(0)
                if "example.com" in value.lower() or "127.0.0.1" in value:
                    continue
                errors.append(f"{path.relative_to(root)}: possible {label}: {value[:100]}")

    if errors:
        print("Repository verification failed:")
        for error in errors:
            print(f"- {error}")
        return 1
    print("Repository structure and privacy scan passed.")
    return 0

if __name__ == "__main__":
    sys.exit(main())
