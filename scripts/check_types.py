"""Check that the Rust domain vocabulary and the TypeScript mirror agree.

The frontend mirrors a handful of Rust enums as TypeScript string unions.
A value that is added on one side and forgotten on the other is not a
compile error: the Rust side would send a value the TypeScript union does
not contain, and the frontend would treat it as an unknown member. This
reads both sides and compares them.

    python3 scripts/check_types.py [repository root]
"""

from __future__ import annotations

import re
import sys
from pathlib import Path

# The Rust enums and the TypeScript unions that mirror them.
MIRRORED = [
    ("AgentStatus", "AgentStatus"),
    ("RunStatus", "RunStatus"),
    ("ApprovalState", "ApprovalState"),
    ("Severity", "Severity"),
]


def rust_values(source: str, enum: str) -> list[str]:
    """The wire values of a Rust enum, from its as_str method."""
    arm = re.search(
        r"impl " + re.escape(enum) + r" \{(.*?)\n\}", source, re.S
    )
    if arm is None:
        raise ValueError(f"no implementation block for the {enum} enum")
    body = arm.group(1)
    fn = re.search(r"pub fn as_str.*?\{(.*?)\n    \}", body, re.S)
    if fn is None:
        raise ValueError(f"the {enum} enum has no as_str method")
    values = re.findall(r'"([a-z_]+)"', fn.group(1))
    if not values:
        raise ValueError(f"the {enum} enum has no wire values")
    return values


def typescript_values(source: str, name: str) -> list[str]:
    """The members of a TypeScript string union."""
    union = re.search(
        r"export type " + re.escape(name) + r"\s*=\s*([^;]+);", source
    )
    if union is None:
        raise ValueError(f"no type alias named {name}")
    values = re.findall(r'"([^"]+)"', union.group(1))
    if not values:
        raise ValueError(f"the {name} type is not a string union")
    return values


def main(argv: list[str]) -> int:
    root = Path(argv[1]) if len(argv) > 1 else Path(__file__).resolve().parent.parent
    rust_path = root / "src-tauri" / "src" / "domain" / "mod.rs"
    types_path = root / "src" / "domain" / "types.ts"

    for path in (rust_path, types_path):
        if not path.exists():
            print(f"the type check failed: {path} does not exist")
            return 1

    rust_source = rust_path.read_text(encoding="utf-8")
    types_source = types_path.read_text(encoding="utf-8")

    problems: list[str] = []
    for rust_enum, ts_name in MIRRORED:
        try:
            rust = rust_values(rust_source, rust_enum)
            types = typescript_values(types_source, ts_name)
        except ValueError as error:
            problems.append(str(error))
            continue
        if set(rust) != set(types):
            only_rust = sorted(set(rust) - set(types))
            only_types = sorted(set(types) - set(rust))
            problems.append(
                f"{rust_enum} and {ts_name} disagree"
                + (f"; only in Rust: {only_rust}" if only_rust else "")
                + (f"; only in TypeScript: {only_types}" if only_types else "")
            )

    if problems:
        print("The type check failed:")
        for problem in problems:
            print(f"- {problem}")
        return 1

    print(f"The Rust and TypeScript vocabularies agree ({len(MIRRORED)} types).")
    return 0


if __name__ == "__main__":
    sys.exit(main(sys.argv))
