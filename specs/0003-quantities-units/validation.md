# Quantity acceptance evidence

## Completion evidence
AC coverage: recipe authoring/revisions/restart/conflicts and unknown data are covered by
`tests/recipes.rs`; original units, exact conversion and unavailable context by quantities
and recipe tests. All 43 curated portions are verified against frozen sources and round-trip
through saved recipes. Local display preferences do not alter stored input. Scaling tests
cover non-terminating ratios, repeated target changes, invalid counts, mass bounds, unknown
nutrition, original-yield retention and restart. Nutrition uses the original rational ratio;
gram strings alone are rounded for display, never fed back into nutrient computation.
No authenticated or cross-device preference claim is made. Existing de/en presentation is
retained; conversion hints identify required ingredient evidence and spoon standards.
Completion commands: import/check verifiers, locale tests, fmt, clippy, nextest, licenses,
audit, strict rustdoc and MSRV. Final outcomes and CI links are recorded in the completion PR.

Final local gate passed: 33 nextest tests, six locale tests, catalogue and conversion verifiers,
fmt, clippy, licenses, audit, strict rustdoc and MSRV 1.85.0.
