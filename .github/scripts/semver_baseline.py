"""Resolve a real published API baseline; never hide registry failures as a skip."""

import json
import os
from pathlib import Path
import re
import tomllib
from urllib.error import HTTPError
from urllib.request import Request, urlopen


def published_baseline(crate, opener=urlopen):
    request = Request(
        f"https://crates.io/api/v1/crates/{crate}",
        headers={"User-Agent": "spaceui-ci (https://github.com/lero-aom/spaceui)"},
    )
    try:
        with opener(request, timeout=30) as response:
            metadata = json.load(response)
    except HTTPError as error:
        if error.code == 404:
            return None
        raise
    # No usable release in a successful response requires maintainer investigation.
    versions = [version for version in metadata["versions"] if not version["yanked"]]
    stable = [version for version in versions if re.fullmatch(r"\d+\.\d+\.\d+", version["num"])]
    if stable:
        selected = max(stable, key=lambda v: tuple(map(int, v["num"].split("."))))
    elif versions:
        selected = max(versions, key=lambda v: v["created_at"])
    else:
        raise ValueError("Registry returned no non-yanked baseline; review the release state")
    version = selected["num"]
    if not re.fullmatch(r"\d+\.\d+\.\d+(?:-[0-9A-Za-z.-]+)?(?:\+[0-9A-Za-z.-]+)?", version):
        raise ValueError(f"Invalid version returned by registry: {version!r}")
    return version


def main():
    package = tomllib.loads(Path("Cargo.toml").read_text(encoding="utf-8"))["package"]
    version = published_baseline(package["name"])
    if version is None:
        message = (
            f"First publication: crates.io returned 404 for {package['name']}. "
            "There is no published API baseline. Compatibility checking starts automatically "
            "after the first version is published; this run does not establish compatibility."
        )
    else:
        message = f"Checking {package['name']} {package['version']} against published version {version}."
    print(message)
    if summary := os.environ.get("GITHUB_STEP_SUMMARY"):
        with open(summary, "a", encoding="utf-8") as stream:
            stream.write(message + "\n")
    if output := os.environ.get("GITHUB_OUTPUT"):
        with open(output, "a", encoding="utf-8") as stream:
            stream.write(f"version={version or ''}\n")


if __name__ == "__main__":
    main()
