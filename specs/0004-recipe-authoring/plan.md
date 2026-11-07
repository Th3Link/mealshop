# Plan for recipe authoring and scaling

Status: In progress; first local authoring slice, customer acceptance pending.

## Current slice
- [x] Specify local scope, persistence decision and pragmatic demo flow (AC 1, 3–4).
- [x] Implement exact ingredient/step/metadata validation and pinned source snapshots (AC 1, 3–4).
- [x] Add immutable file revisions, exclusive locking and stale-write detection (AC 4).
- [x] Add create/list/detail/update API and English-base de/en demo authoring (AC 1, 4).
- [x] Add restart/source-change/conflict/invalid-input/I/O-failure tests (AC 2 validation, 3–4).
- [x] Complete browser exercise and full repository gate; prepare review PR.
- [ ] Customer review and acceptance of this slice.

## Remaining feature work
- [ ] Proportional ingredient scaling, original-yield semantics and precision behaviour (AC 2).
- [ ] Authenticated household/author ownership and product-client integration (F09/F18/F22).
- [ ] Git/database adapters and validated cross-adapter migration (F18).

## Validation
`tests/recipes.rs` covers AC 1/4 round trips with exact input/steps, restart and pinned source
retrieval; AC 2 invalid serving rejection and same-batch portion division; AC 3 unknown IDs and
composition; AC 4 concurrent revision conflicts, corrupt/locked/interrupted stores and I/O
failures without overwrites. Reuse nutrition arithmetic/provenance and locale regression tests.
Record final commands/results and browser evidence in validation.md at delivery.

Delivery evidence: [validation](validation.md).
