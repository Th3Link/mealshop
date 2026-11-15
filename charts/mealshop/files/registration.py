#!/usr/bin/env python3
"""Combine a public registration template with a Kubernetes Secret read from stdin.

Pipe stdout directly to Nextcloud's registration command; it contains the installation secret.
"""
import base64
import json
from pathlib import Path
import re
import sys

if len(sys.argv) not in (2, 3):
    raise SystemExit("Usage: registration.py PUBLIC_TEMPLATE [SECRET_KEY] < kubernetes-secret.json")
try:
    template = json.loads(Path(sys.argv[1]).read_text())
    secret_object = json.load(sys.stdin)
    key = sys.argv[2] if len(sys.argv) == 3 else "APP_SECRET"
    secret = base64.b64decode(secret_object["data"][key], validate=True).decode("ascii")
    if not re.fullmatch(r"[A-Za-z0-9_-]{32,200}", secret):
        raise ValueError("Invalid secret")
    template["secret"] = secret
except (ValueError, KeyError, UnicodeError, OSError):
    raise SystemExit("Cannot prepare registration; check the public template and secret key") from None
json.dump(template, sys.stdout)
sys.stdout.write("\n")
