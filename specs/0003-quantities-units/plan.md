# Plan for Precise quantities and configurable units

## Affected modules
`src/quantities.rs` and `src/api.rs` provide a typed, exact conversion endpoint. Recipe integration and user preferences remain open.

## Tasks
- [ ] Design and implement the behaviour for AC 1; add acceptance evidence.
- [ ] Design and implement the behaviour for AC 2; add acceptance evidence.
- [ ] Design and implement the behaviour for AC 3; add acceptance evidence.
- [ ] Design and implement the behaviour for AC 4; add acceptance evidence.
- [ ] Design and implement the behaviour for AC 5; add acceptance evidence.

## Validation
Before implementation, derive acceptance tests from each numbered criterion. Include failure cases, household authorization and quantity/provenance checks where applicable. Record commands and results here at delivery.

## First slice evidence
`cargo test --test quantities --locked` checks exact input retention, dimension-specific conversions, unavailable cross-dimension/spoon conversions and precision limits.
