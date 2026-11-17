# Verification evidence

## Guard behavior and tool compatibility

- `python3 -m unittest discover -s tests -p 'test_licence_inventories.py'`: eight tests cover
  version changes/removals, repeatability, new/changed identities, preservation of both inventories,
  multiple native versions, command restrictions, pinned runtimes and portable Flutter SDK discovery.
- Removing the identity guard makes the regression suite fail. Restoring it passes all eight tests.
- The original SDK directory heuristic fails the Containerbase-layout regression; ancestor-based
  discovery passes it and preserves the existing project-local SDK inventory identities.
- Rendered runner configuration passes `renovate-config-validator --strict` in the actual official
  `ghcr.io/renovatebot/renovate:44` image. Actionlint passes the workflow changes.
- An isolated public-source fixture runs the refresh in that same image, with Containerbase-installed
  Python 3.15.0, Rust 1.100.0 and Flutter 3.50.0. It refreshes all 68 Dart/229 native records without
  an Android SDK or emulator. Repeating the command preserves both inventories byte for byte.

## PR #65 and delivery

The [original Android failure](https://github.com/Th3Link/mealshop/actions/runs/44086998085)
is reproduced after locked Pub resolution. The new guarded refresh repairs the actual `material_ui`
1.11.0 → 1.11.1 record; its reviewed BSD-3-Clause licence hash is unchanged. The independent native,
Dart, capture/OCR hash and OSV checks pass after refresh. Current main is retained.

[PR #65](https://github.com/Th3Link/mealshop/pull/65) records final repository gates and current-head
CI, including Android integration and uploaded APKs, before handoff. Regular self-hosted updates
use this policy after merge; no live write-enabled Renovate run against other PRs is part of testing.
