# F02 validation

Date: 2026-11-07. Status: implementation verified locally; customer acceptance pending.

## Acceptance traceability
| Criteria | Evidence |
|---|---|
| AC 1, 5 | `catalogue_search_and_detail_retain_provenance_and_evidence`; original USDA snapshots; source verifier |
| AC 2, 6 | `recipe_and_serving_values_are_scaled_from_source_values`; exact portion/rounding tests |
| AC 3, 7–8 | `unknown_and_partial_values_are_never_reported_as_complete_zero`; browser partial/preparation flow |
| AC 4 | evidence conditions/limitations checked in catalogue/detail test and browser notebook/results |
| AC 6, 9 | bounds/repeated-line tests; invalid payloads; custom catalogue, duplicate keys and startup validation |
| AC 10 | normal calculation, partial/unknown, invalid grams preserving input and 390 px responsive view exercised in browser |

The first five API acceptance tests were run against the bootstrap before handlers existed
and failed as expected. They pass with the implementation; the suite now has 15 tests total.

## Local gate results
- `python3 scripts/generate_docs.py --check`: PASS.
- `python3 scripts/verify_catalogue.py`: PASS, 26 values match frozen USDA responses exactly.
- `node --check src/web/app.js`: PASS.
- `cargo fmt --all --check`: PASS.
- `cargo clippy --workspace --all-targets --locked -- -D warnings`: PASS.
- `cargo nextest run --workspace --locked`: 15 passed, 0 skipped.
- `cargo build --release --locked`: PASS.
- `cargo deny check licenses`: PASS.
- `cargo audit`: PASS, no advisory ignores.
- `RUSTDOCFLAGS="-D warnings" cargo doc --workspace --no-deps --locked`: PASS.
- `cargo msrv verify -- cargo check --all-targets --locked`: PASS, Rust 1.85.0.

## Browser observations
- 100 g oats + 25 g chia, two servings: 255.25 kcal and 10.51 g protein per serving.
- Switching to psyllium leaves totals unknown and labels the known subtotals separately.
- Unmodelled cooking shows the lack of retention/yield correction.
- Negative gram input shows an error, preserves the input and hides stale results.
- At a 390 px viewport the document has no horizontal overflow; controls and the two-column
  macro-card layout are readable. The full nutrient table scrolls within its own wrapper.
- Source/evidence notes remain separate from measured component values; beta-glucan and
  soluble fibre remain unknown, never extrapolated from total fibre.

## Review scope and remaining work
This PR provides read-only public reference data, calculations and a browser review surface.
A local JSON file can replace the catalogue; startup validation is not a persistence adapter.
There are no stored recipes, household records, general unit preferences or Flutter shell.
The 15-definition vocabulary is a starter subset, not exhaustive micronutrient coverage.
Source refresh, further verified foods and cooking-retention models require later work.
F02 stays In progress until customer review/acceptance. GitHub CI outcomes are checked on the
feature PR and reported with the delivery, not inferred from local test success.
