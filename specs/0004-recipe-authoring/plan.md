# Plan for recipe authoring and scaling

Status: Done for the documented local feature scope.

## Current slice
- [x] Specify local scope, persistence decision and pragmatic demo flow (AC 1, 3–4).
- [x] Implement exact ingredient/step/metadata validation and pinned source snapshots (AC 1, 3–4).
- [x] Add immutable file revisions, exclusive locking and stale-write detection (AC 4).
- [x] Add create/list/detail/update API and English-base de/en demo authoring (AC 1, 4).
- [x] Add restart/source-change/conflict/invalid-input/I/O-failure tests (AC 2 validation, 3–4).
- [x] Complete browser exercise and full repository gate; prepare review PR.
- [x] Customer review and acceptance of this slice.

## Scaling slice (customer request after PR #6)
- [x] Define a scaling preview from the immutable original revision and its original servings.
- [x] Scale each original gram quantity by target servings / original servings using exact
  integer arithmetic; document precision and rounding, and reject invalid or unrepresentable amounts.
- [x] Return original and target servings and retain the pinned source snapshot for nutrition.
- [x] Add localized demo controls without overwriting the saved revision.
- [x] Test non-terminating ratios, repeated changes, boundaries, unknown nutrients and restart.
- [x] Run the repository gate and prepare a reviewable PR.

## Remaining feature work
- [x] Proportional ingredient scaling, original-yield semantics and precision behaviour (AC 2).
- [ ] Deferred to F09/F18/F22: authenticated household/author ownership and product-client integration (F09/F18/F22).
- [ ] Deferred to F18: Git/database adapters and validated cross-adapter migration (F18).

## Validation
`tests/recipes.rs` covers AC 1/4 round trips with exact input/steps, restart and pinned source
retrieval; AC 2 invalid serving rejection and same-batch portion division; AC 3 unknown IDs and
composition; AC 4 concurrent revision conflicts, corrupt/locked/interrupted stores and I/O
failures without overwrites. Reuse nutrition arithmetic/provenance and locale regression tests.
Record final commands/results and browser evidence in validation.md at delivery.

Delivery evidence: [validation](validation.md).

## Completion
Customer requested feature completion and Done status. Acceptance evidence is in validation.md.
Authenticated ownership, Git/SQL storage and product clients belong to F09/F18/F22;
cross-device preference sync is deferred to the end of the backlog, not implemented here.
