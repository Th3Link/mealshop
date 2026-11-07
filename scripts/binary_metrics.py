"""Record host binary size; values are observations, not performance budgets."""
import json
import os
from pathlib import Path
import sys

profile = sys.argv[1]
if profile not in ("debug", "release"):
    raise SystemExit("Expected debug or release")
binary = Path("target") / profile / "mealshop-server"
record = {"schema_version": 1, "profile": profile, "bytes": binary.stat().st_size,
          "sha": os.environ.get("GITHUB_SHA", "local"), "target": "linux-x86_64"}
Path("metrics").mkdir(exist_ok=True)
(Path("metrics") / (profile + ".json")).write_text(json.dumps(record, indent=2) + "\n")
