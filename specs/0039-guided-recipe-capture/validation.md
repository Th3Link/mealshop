# Validation

Local, 2026-11-27, Nextcloud 35.0.2 test instance with the locally built server and client.

| AC | Evidence |
|---|---|
| 2, 3, 6 | `clients/nextcloud/tests/recipe-text.test.ts` (6 tests) |
| 6 | `tests/recipes.rs::household_declared_unit_weight_needs_its_unit_but_no_source_url` |
| 7 | `feedback::tests::recipe_ratings_count_reported_values_without_learning` |
| 1–3, 6–10 | `clients/nextcloud/e2e/guided-capture.spec.ts` in embedded Chromium, incl. axe |
| 11 | `suggestions::tests` (2 tests), `tests/mcp.rs::suggestions_wait_unreviewed_in_the_inbox_and_become_imports_only_through_review` |
| 12 | `clients/nextcloud/tests/suggestion.test.ts` (2 tests) |
| 13 | `tests/test_mcp_stdio.py` (2 sender tests) |
| 11–14 | second test in `guided-capture.spec.ts` (2026-11-28): companion script → inbox → discard → review → rough estimate → saved import, incl. axe and 390 px |

Slice 2 was verified on 2026-11-28 on the same instance. After archiving the 20 synthetic fixture
households of this work (sole writer stopped; retained households and the allocation counter
untouched), `nutrition-ranges`, `unified-preparation`, `visual-shopping` and two of the three
`feedback` tests passed on the same day. `unified-preparation` now walks to its planned week instead
of assuming the run happens in it, and `feedback` rates by clicking the star.

Not verified: real microphone dictation (AC 5), picture upload through the new screen in the
browser suite (AC 4; the photo preview API itself is covered by `mobile-capture.spec.ts`),
Firefox/Safari, the customer's production instance. Tests that need a second (reader) account,
including `feedback` "reader roles stay read-only", cannot run locally; CI results belong in the PR.
AI structuring (AC 15) does not exist yet.
