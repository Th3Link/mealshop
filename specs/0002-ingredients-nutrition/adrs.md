# Decisions for ingredient catalogue and nutrition

- [ADR 0002](../../docs/adr/0002-quantities-and-nutrition.md): preserve quantity dimensions,
  provenance and unknowns; perform no implicit volume/mass conversions.

## Feature-local decisions
2026-11-07 (Proposal, reversible within this slice):
- Use bounded scaled integers (six decimal places) and checked rational arithmetic through
  final display rounding. JSON amounts are strings; original decimal spelling is retained.
  `serde`/`serde_json` (MIT OR Apache-2.0) provide typed API
  serialization already present transitively; declare them directly now that they are used.
- Retain a fixed per-100-g edible-portion basis and a named nutrient registry with explicit
  units; serving counts are positive integers. F03 will extend user-facing quantity units.
- Read a validated immutable catalogue at startup, using bundled public-domain/CC0 USDA SR
  Legacy snapshots by default. An optional local JSON file replaces the whole catalogue.
  This does not decide the persistent household storage architecture of F18.
- A stateless estimate endpoint takes ingredient IDs/grams/servings/context and returns
  complete or explicitly partial totals, retained provenance and preparation warnings.
- Supply a minimal same-origin browser workbench instead of starting the full Flutter shell.
  Only public operator-provided catalogue records are readable; no user data is persisted.
- Reviewed educational notes carry source, applicability and limitations separately from
  measured composition. No dose/benefit is inferred when component data is absent.

Provider curation/refresh, a complete nutrient vocabulary and general cooking-retention models
remain future refinements. The starter catalogue is deliberately small, not an authoritative
food/allergen database. Customer authorization to start F02 is not acceptance of a finished PR.
