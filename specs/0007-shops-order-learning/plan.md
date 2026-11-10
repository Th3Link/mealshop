# Delivery plan: shops and optional order learning

- [x] AC1: persistent named shops, per-ingredient first/second choices and keyboard/touch manual order.
- [x] AC2: explicit opt-in completed sessions; corrections invalidate session observations.
- [x] AC3: at least three valid multi-item sessions before learned order; manual overrides win.
- [x] AC4: assignment/fallback changes only metadata, preserving quantities and source snapshots.
- [x] AC5: inspect/discard/reset/disable sessions independently of shopping data.

## Evidence
See [combined validation](../0006-shopping-list/validation.md). Backend acceptance contracts are in
`tests/shopping.rs`, provider/GTIN boundary tests in `src/shopping/lookup.rs`, native browser flows
in `clients/nextcloud/e2e/shopping.spec.ts`.
