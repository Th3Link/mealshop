# Plan for Precise quantities and configurable units

## Affected modules
`src/quantities.rs` and `src/api.rs` provide a typed, exact conversion endpoint. Recipe integration and local display preferences are implemented; cross-device synchronization is deferred.

## Tasks
- [x] Design and implement the behaviour for AC 1; add acceptance evidence.
- [x] Design and implement the behaviour for AC 2; add acceptance evidence.
- [x] Design and implement the behaviour for AC 3; add acceptance evidence.
- [x] Design and implement the behaviour for AC 4; add acceptance evidence.
- [x] Design and implement the behaviour for AC 5; add acceptance evidence.

## Validation
Before implementation, derive acceptance tests from each numbered criterion. Include failure cases, household authorization and quantity/provenance checks where applicable. Record commands and results here at delivery.

## First slice evidence
`cargo test --test quantities --locked` checks exact input retention, dimension-specific conversions, unavailable cross-dimension/spoon conversions and precision limits.

## Integration delivery
Recipe quantity/evidence persistence and legacy compatibility are implemented. Demo input units and local mass display preference are implemented. `tests/recipes.rs` checks kilogram input round trips across restart and declared piece-mass evidence with identity rejection. Full validation and review remain pending.

## Curated portions
43 exact USDA SR Legacy reference portions are selected from frozen source records, with original measure descriptions retained. Cross-device preferences are explicitly deferred to the end of the backlog at customer request. They are not part of this delivery.

## Completion
Customer requested feature completion and Done status. Acceptance evidence is in validation.md.
Authenticated ownership, Git/SQL storage and product clients belong to F09/F18/F22;
cross-device preference sync is deferred to the end of the backlog, not implemented here.
