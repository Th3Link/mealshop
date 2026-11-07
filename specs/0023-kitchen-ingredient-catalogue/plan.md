# Plan for the kitchen catalogue

- [x] Curate requested food identities/variants and classify exact source versus identity-only (AC 2–4).
- [x] Extract primary BLS/USDA source selections with immutable source hashes/references (AC 3, 6).
- [x] Implement deterministic import and independent provenance/unit/value verification (AC 3–6).
- [x] Add English-base/translations model, multilingual search and exact demo locale presentation (AC 1, 7).
- [x] Add separate available-carbohydrate/food-folate IDs and mixed-definition warnings (AC 5).
- [x] Complete focused tests, browser exercise and repository quality gate; prepare review PR.
- [ ] Customer review/acceptance; then mark Done.

## Validation
Source verifier checks every mapped value against frozen primary data within the documented
six-decimal import tolerance, checks units and exact record identities, and rejects unsourced
numeric values. API tests cover exact flour/dairy/protein variants, unknown product profiles,
translation identity/fallback, multilingual search and split carbohydrate definitions.
Locale tests cover resource IDs/placeholders, regional fallbacks, exact big-number display and
comma/point input. Reuse F02 arithmetic/provenance regressions. Record results at delivery.

Results: [delivery validation](validation.md). Customer acceptance remains pending.
