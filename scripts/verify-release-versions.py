#!/usr/bin/env python3

"""Verify release metadata versions across Cargo, npm, and Go wrapper files."""

import json
import os
import re
import sys
from pathlib import Path


def fail(message: str) -> None:
    print(message)
    sys.exit(1)


def parse_cargo_version() -> str:
    text = Path("Cargo.toml").read_text(encoding="utf-8")
    match = re.search(r'^version = "([^"]+)"', text, re.MULTILINE)
    if not match:
        fail("Could not parse version from Cargo.toml")
    return match.group(1)


def parse_npm_version() -> str:
    pkg = json.loads(Path("npm/package.json").read_text(encoding="utf-8"))
    return pkg.get("version", "")


def parse_go_version() -> tuple[str, str]:
    preferred = [
        Path("go/internal/installer/installer.go"),
        Path("go/cmd/infynon/main.go"),
        Path("go/main.go"),
    ]

    pattern = re.compile(r'^\s*version\s*=\s*"([^"]+)"', re.MULTILINE)

    for path in preferred:
        if not path.exists():
            continue
        text = path.read_text(encoding="utf-8")
        match = pattern.search(text)
        if match:
            return path.as_posix(), match.group(1)

    discovered = []
    for path in sorted(Path("go").rglob("*.go")):
        text = path.read_text(encoding="utf-8")
        match = pattern.search(text)
        if match:
            discovered.append((path.as_posix(), match.group(1)))

    if not discovered:
        fail("Could not parse version from Go wrapper sources")

    versions = {value for _, value in discovered}
    if len(versions) != 1:
        fail(f"Go wrapper has conflicting versions: {discovered}")

    return discovered[0]


def main(argv: list[str]) -> int:
    tag = " ".join(argv).strip()
    if not tag:
        tag = os.getenv("GITHUB_REF_NAME", "")
    if not tag:
        fail("Expected release tag argument or GITHUB_REF_NAME env var")

    expected = tag[1:] if tag.startswith("v") else tag

    versions = {
        "Cargo.toml": parse_cargo_version(),
        "npm/package.json": parse_npm_version(),
    }
    go_source, go_version = parse_go_version()
    versions[f"Go wrapper ({go_source})"] = go_version

    mismatches = {name: value for name, value in versions.items() if value != expected}
    if mismatches:
        print(f"Release metadata mismatch for tag {expected}:")
        for name, value in mismatches.items():
            print(f"  {name}: found {value}")
        print("Update release metadata before creating the release tag.")
        return 1

    print(f"Verified release metadata version {expected}")
    return 0


if __name__ == "__main__":
    sys.exit(main(sys.argv[1:]))
