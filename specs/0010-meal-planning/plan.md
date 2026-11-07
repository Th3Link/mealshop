# F10 family planning acceptance

- [x] Protected persisted date/meal slots with pinned recipe revisions, attendees and servings (AC 1).
- [x] Manual selection and constraint-compatible reviewed suggestions with rejected reasons (AC 1, 5).
- [x] Consented read-only calendar snapshot import, visible source and manual override (AC 2).
- [x] Active/elapsed/advance estimates and explicit unknown/batch-size timing uncertainty (AC 3).
- [x] Preview/apply replacement/removal with scaled nutrition and shopping requirement deltas (AC 4).
- [x] Profile/plan revision conflicts, renewed review markers, source revision retention (AC 4–5).
- [x] Acceptance tests for constraints, consent, overrides, time, conflicts, restart and failures.
- [x] Browser acceptance and full local repository gate; current-commit CI evidence in the PR.

Tests in tests/planning.rs map directly to the five acceptance criteria. Current local scope,
calendar import format and the distinction from live providers/F06 are documented in the guide
and ADR 0011. Source uncertainty is retained; unknown data never becomes safe automatically.

Status: Done for the documented local/import-based F10 contract after local and browser acceptance.
See [validation](validation.md). No live provider or F06 shopping execution is implied.
