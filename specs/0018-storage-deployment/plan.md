# Plan for Storage adapters, Nextcloud `ExApp` and Flutter

## Affected modules
`src/recipes.rs` and `src/recipe_api.rs` begin a bounded local file slice alongside F04.
No generic repository trait, SQL/Git adapter or placeholder crate is added.

## Tasks
- [ ] Design and implement the behaviour for AC 1; add acceptance evidence.
- [ ] Design and implement the behaviour for AC 2; add acceptance evidence.
- [ ] Design and implement the behaviour for AC 3; add acceptance evidence.
- [ ] Design and implement the behaviour for AC 4; add acceptance evidence.
- [ ] Design and implement the behaviour for AC 5; add acceptance evidence.

## Validation
Before implementation, derive acceptance tests from each numbered criterion. Include failure cases, household authorization and quantity/provenance checks where applicable. Record commands and results here at delivery.

Product web UI coordination: F22/F30; cross-client localization: F21/F29. The existing demo is not deployment UI.

Current local part of AC 5: exclusive lock, immutable revisions, expected-revision conflicts,
fail-closed startup and documented backup/recovery (ADR 0009). Tests in `tests/recipes.rs`
exercise these behaviours. This does not complete any full multi-adapter/client criterion.
