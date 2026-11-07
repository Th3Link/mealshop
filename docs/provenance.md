# Bootstrap provenance

Retrieved 2026-11-07:
- [curvyo/curvyo](https://github.com/curvyo/curvyo/tree/9ca3ebba86e511cd24559424f13fa2183caeea57):
  AGENTS.md/CLAUDE.md conventions, specs/README.md lifecycle, docs/adr format and AGPL text.
  Mealshop rewrites project-specific guidance; no Curvyo product modules or agent setup copied.
- [Th3Link/homeautomation](https://github.com/Th3Link/homeautomation/tree/4c65a45d2046d795aaef5b79e3c311ae1f0a1847):
  CI quality gate, nextest/coverage, cargo-msrv, debug/release build artifacts, cargo-sbom,
  cargo-deny/audit, self-hosted Renovate and rustdoc/Pages publication patterns.
  ESP/QEMU/firmware jobs do not apply. Metrics measure host server binary bytes, not ESP memory.
  No upstream advisory exceptions or hardware-specific Renovate exclusions are carried over.

The initial customer request authorizes bootstrap and direct main publication. Future feature
specs remain Draft; they are not completed just because their documentation exists.
