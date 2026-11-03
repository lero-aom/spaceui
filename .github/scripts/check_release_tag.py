"""Reject publishing from a branch or a tag for another manifest version."""

import os
from pathlib import Path
import tomllib

version = tomllib.loads(Path("Cargo.toml").read_text(encoding="utf-8"))["package"]["version"]
expected = f"refs/tags/v{version}"
actual = os.environ.get("GITHUB_REF")
if actual != expected:
    raise SystemExit(f"Publishing requires {expected}; selected ref was {actual!r}")
print(f"Release ref matches Cargo.toml: {expected}")
