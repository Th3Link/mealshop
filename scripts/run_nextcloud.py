#!/usr/bin/env python3
"""Start the Nextcloud ExApp from a private, operator-owned JSON environment file."""
import argparse
import json
import os
from pathlib import Path

parser = argparse.ArgumentParser(description=__doc__)
parser.add_argument("config", type=Path)
args = parser.parse_args()
config = json.loads(args.config.read_text())
allowed = {"APP_SECRET", "APP_HOST", "APP_PORT", "NEXTCLOUD_URL", "APP_PERSISTENT_STORAGE", "MEALSHOP_ASSET_DIR", "HP_SHARED_KEY", "HP_FRP_ADDRESS", "HP_FRP_PORT"}
if not isinstance(config, dict) or not all(key in allowed and isinstance(value, str) for key, value in config.items()):
    parser.error("Config must contain only supported string environment values")
if args.config.stat().st_mode & 0o077:
    parser.error("Config contains credentials: restrict permissions to 0600")
root = Path(__file__).resolve().parent.parent
binary = root / "target" / "debug" / "mealshop-server"
if not binary.is_file():
    parser.error("Build the server with cargo build --locked first")
os.chdir(root)
os.execve(binary, [str(binary)], {**os.environ, **config})
