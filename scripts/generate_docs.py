"""Embed repository Markdown as linked rustdoc pages (no third-party dependencies)."""
from pathlib import Path
import json
import re
import sys

root = Path(__file__).resolve().parents[1]
files = [root / "README.md", *sorted((root / "docs").rglob("*.md")),
         *sorted((root / "specs").rglob("*.md"))]
names = {f.resolve(): "page_" + re.sub(r"[^a-z0-9_]", "_", str(f.relative_to(root)).lower()) for f in files}
lines = ["//! Product requirements, feature contracts and architecture decisions.", ""]
for f in files:
    def link(match):
        label, dest = match.groups()
        if "://" in dest or dest.startswith("#"):
            return match.group(0)
        target, sep, anchor = dest.partition("#")
        resolved = (f.parent / target).resolve()
        if resolved in names:
            url = "../" + names[resolved] + "/index.html" + ("#" + anchor if sep else "")
        else:
            url = "https://github.com/Th3Link/mealshop/blob/main/" + str(resolved.relative_to(root))
        return "[" + label + "](" + url + ")"
    content = re.sub(r"\[([^\]]+)\]\(([^)]+)\)", link, f.read_text())
    lines += ["#[doc = " + json.dumps(content, ensure_ascii=False) + "]", "pub mod " + names[f.resolve()] + " {}", ""]
output = "\n".join(lines)
path = root / "src/docs.rs"
if "--check" in sys.argv:
    if not path.exists() or path.read_text() != output:
        raise SystemExit("Run python3 scripts/generate_docs.py and commit src/docs.rs")
else:
    path.write_text(output)
