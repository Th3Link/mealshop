#!/usr/bin/env python3
"""Validate release identities and derive public AppAPI routes without third-party packages."""
import argparse
import json
from pathlib import Path
import re
import tomllib
import xml.etree.ElementTree as ET

ROOT = Path(__file__).resolve().parents[1]
VERSION = r"(0|[1-9][0-9]*)\.(0|[1-9][0-9]*)\.(0|[1-9][0-9]*)"


def routes(manifest):
    result = []
    for route in manifest.findall("./external-app/routes/route"):
        entry = {field.tag: field.text for field in route}
        entry["access_level"] = {"PUBLIC": 0, "USER": 1, "ADMIN": 2}[entry["access_level"]]
        if "headers_to_exclude" in entry:
            entry["headers_to_exclude"] = json.loads(entry["headers_to_exclude"])
        result.append(entry)
    return result


def validate(root, tag=None):
    manifest = ET.parse(root / "appinfo/info.xml").getroot()
    version = tomllib.loads((root / "Cargo.toml").read_text())["package"]["version"]
    if not re.fullmatch(VERSION, version):
        raise ValueError("Cargo version must be a canonical stable X.Y.Z version")
    if tag is not None and tag != "v" + version:
        raise ValueError(f"Expected tag v{version}; update all release metadata before tagging")
    lock = tomllib.loads((root / "Cargo.lock").read_text())
    locked = next(p["version"] for p in lock["package"] if p["name"] == "mealshop-server")
    chart = (root / "charts/mealshop/Chart.yaml").read_text()
    versions = {key: re.search(rf'^{key}:\s*[\"\']?([0-9.]+)[\"\']?\s*$', chart, re.M)
                for key in ("version", "appVersion")}
    if any(value is None or value[1] != version for value in versions.values()):
        raise ValueError("Chart version/appVersion must match Cargo.toml")
    if locked != version or manifest.findtext("version") != version:
        raise ValueError("Cargo.lock and appinfo/info.xml must match Cargo.toml")
    if manifest.findtext("./external-app/docker-install/image-tag") != version:
        raise ValueError("ExApp image-tag must match Cargo.toml")
    if manifest.findtext("id") != "mealshop":
        raise ValueError("Unexpected ExApp identity")
    projected = json.loads((root / "charts/mealshop/files/routes.json").read_text())
    if projected != routes(manifest):
        raise ValueError("Run python3 scripts/release.py --sync-routes after changing ExApp routes")
    return version


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--tag", help="Expected pushed stable tag vX.Y.Z")
    parser.add_argument("--sync-routes", action="store_true", help="Refresh the public chart route projection")
    args = parser.parse_args()
    if args.sync_routes:
        manifest = ET.parse(ROOT / "appinfo/info.xml").getroot()
        (ROOT / "charts/mealshop/files/routes.json").write_text(json.dumps(routes(manifest), indent=2) + "\n")
    try:
        version = validate(ROOT, args.tag)
    except (ValueError, KeyError, StopIteration) as error:
        parser.exit(1, f"Release metadata error: {error}\n")
    print(version)


if __name__ == "__main__":
    main()
