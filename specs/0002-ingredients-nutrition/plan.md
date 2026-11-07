# Plan for ingredient catalogue and nutrition

## Affected modules
`src/quantities.rs`: validated decimal amounts and gram quantities.
`src/nutrition/`: catalogue model/validation and pure nutrient calculation.
`src/api.rs`, `src/lib.rs`, `src/main.rs`: read-only catalogue and stateless calculation HTTP API.
`src/web/`: browser workbench. `data/`: reviewed catalogue and original source snapshots.
`tests/`: acceptance checks and fixtures. No new crate or storage abstraction.

## Tasks
- [x] Refine acceptance/UX notes and record implementation proposals before implementation.
- [x] Define failing acceptance tests for provenance, scaling, missing data and education (AC 1–4, 5–9).
- [x] Implement typed decimal quantities, validated catalogue, pure aggregation and rounding (AC 1–3, 6–9).
- [x] Freeze and verify primary-source starter records and evidence notes (AC 1, 4).
- [x] Expose read-only/search/detail API, stateless calculations and startup catalogue loading (AC 5–9).
- [x] Build and exercise the browser workbench including errors/unknowns (AC 3, 10).
- [x] Run quality/security/MSRV checks and prepare the reviewable feature branch/demo.

## Validation
Derive black-box HTTP tests from this specification before implementing the handlers. Check
source-record preservation, known arithmetic/portion scaling, exact decimal behaviour and
rounding boundaries; distinguish partial/unknown from zero; reject invalid/oversized input.
Verify the bundled values directly against the frozen USDA source records, not copied assertions.
Exercise custom-catalogue startup validation and existing health/startup regression tests.
Browser-check normal, unknown/partial, error and narrow-screen flows. Run the repository gate,
MSRV and CI; record exact outcomes before marking implementation review-ready.

## Evidence
See [validation.md](validation.md) and [the demo guide](../../docs/guides/nutrition.md).

## Acceptance
- [ ] Customer reviews and accepts the feature PR; only then mark the feature Done.
