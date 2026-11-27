# Validation

Local, 2026-11-27, Nextcloud 35.0.2 test instance with the locally built server and client.

| AC | Evidence |
|---|---|
| 2, 3, 6 | `clients/nextcloud/tests/recipe-text.test.ts` (6 tests) |
| 6 | `tests/recipes.rs::household_declared_unit_weight_needs_its_unit_but_no_source_url` |
| 7 | `feedback::tests::recipe_ratings_count_reported_values_without_learning` |
| 1–3, 6–10 | `clients/nextcloud/e2e/guided-capture.spec.ts` in embedded Chromium, incl. axe |

Not verified: real microphone dictation (AC 5), picture upload through the new screen in the
browser suite (AC 4; the photo preview API itself is covered by `mobile-capture.spec.ts`),
Firefox/Safari, the customer's production instance. Specs that need a second (reader) account were
not run locally; CI results belong in the PR.
