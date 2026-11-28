# Plan for Interdata intake

Slice 1 is implemented and not deployed. Later slices stay unchecked proposals.

## Affected modules
`interdata/` (new workspace crate `mealshop-interdata`), root `Cargo.toml` (workspace),
`Dockerfile` and `.dockerignore` (server image builds its package only), `interdata/Dockerfile`,
`deploy/interdata/compose.yaml`, `docs/guides/interdata.md`, ADR 0040.

## Proposed slices
- [x] Slice 1 — intake only: service skeleton, upload admission (F43-ADMISSION), `POST`/`GET` of
      recipe and ingredient submissions, idempotency, bounds, image-link validation (AC 1–4, 13).
- [ ] Slice 2 — deterministic checks: exact ingredient matching, unit normalization, nutrition
      completeness, derived composition/nutrition tags, tag vocabulary, rule-based plausibility of declared
      condition tags (AC 5–8, 10).
- [ ] Slice 3 — review and publication: reviewer role, accept/reject/return, immutable published
      revisions, read API with tag filters (AC 9, 11, 12).
- [ ] Slice 4 — self-hosted micro model for what the rules leave open (AC 5, 6).
- [ ] Slice 5 — frontier escalation queue with moderator approval per item (AC 5).
- [x] Container image definition and compose file (F43-DOCKER).
- [ ] Deployment to `interdata.mealshop.luehr.dev` (DNS, TLS, reverse proxy, published image);
      done by the customer.

## Validation
Slice 1 evidence, `interdata/tests/api.rs` and unit tests in `interdata/src/model.rs`:

| AC | Tests |
|---|---|
| 1 | `upload_is_buffered_unchanged_and_readable`, `full_recipe_and_ingredient_are_accepted`, `upload_needs_the_secret`, `short_secret_and_bad_cdn_origin_refuse_to_start`, `invalid_recipes_are_rejected_with_a_reason` (including a missing or different licence) |
| 2 | `replay_returns_the_same_submission_and_changed_body_conflicts`, `replay_survives_restart`, `idempotency_key_is_required_and_safe` |
| 3 | `upload_is_buffered_unchanged_and_readable` (stored body equals the bytes sent), `decimals_are_exact_strings`, `ingredient_nutrients_need_a_source` |
| 4 | `invalid_recipes_are_rejected_with_a_reason` (foreign and look-alike hosts), `image_links_wait_for_a_cdn_origin`, `cdn_origin_rejects_paths_and_plain_http` |
| 13 | `oversized_and_non_json_uploads_are_refused`, `rate_limit_and_capacity_refuse_new_uploads_but_not_replays`, `unknown_submission_is_not_found`; no submitted URL is fetched and nothing is logged per request |

Commands and results are recorded in the pull request. Not covered: deployment, TLS, concurrent
instances on one directory, and the brief window in which a crash can duplicate a submission
(ADR 0040).

For later slices, derive acceptance tests from each numbered criterion before implementation,
including unknown-ingredient tag behaviour and incomplete nutrition.
