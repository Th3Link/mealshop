# Cooking history and online preference learning

Status: Done
Priority: Should
Origin: Customer
Milestone: M2

## User value
Improve suggestions from real cooking and family feedback.

## Acceptance criteria
1. Given a completed cook, history records recipe revision, attendees, date, actual effort and individual/household rating without counting merely planned meals.
2. Given new ratings or repeated cooking, suggestions update incrementally and explain relevant history, constraints and confidence.
3. Given cold-start users, transparent rules provide usable suggestions; new recipes remain discoverable rather than being excluded by sparse feedback.
4. Given model changes, evaluation compares constraint violations, acceptance, variety and user satisfaction against the baseline.
5. Given a reset or opt-out, learned preference state can be removed and suggestions fall back to rules; hard exclusions remain enforced.

## Out of scope
Mandatory cloud training or a chosen ML framework before evaluation.

## UX notes
UI flows, accessibility, empty/error states and user review points must be specified before Ready.

## Links
Requirement: [F11](../../docs/requirements.md). Delivery tasks: [plan.md](plan.md). Decisions: [adrs.md](adrs.md).

## Nextcloud delivery contract
The customer requested F11 after the merged F05 delivery. Use the existing protected household,
recipe-revision, cooking-session and planning boundaries. This contract enters Ready for the
requested implementation under the customer's autonomous-work instruction, then In progress.
No standalone/Flutter shell, remote training provider or new crate is introduced. The delivered
scope and reproducible acceptance evidence are recorded in [validation.md](validation.md).

- A history entry is an explicitly confirmed completed cook, never an automatically imported plan.
  It references a saved recipe/revision, servings, actual civil date, actual attendees, optional
  reported active effort in exact seconds, optional household rating and optional attendee ratings
  (1–5). Unknown effort/rating stays null. A completed session can seed an entry only after all
  its stages are completed. Manual recording supports cooking performed outside the timers.
- Pin the recipe title/language/revision and actual source on creation. Corrections retain their
  recorded actor/time and record the updating actor/time. Session-derived entries cannot duplicate
  a source session or change its recipe; historical entries survive session removal. Writers can
  correct/delete history, readers can browse. Stale writes fail without losing the client draft.
- Learning is off by default. Only the household owner manages the learning scope; household
  ratings/cooking frequency and optionally selected attendees' ratings stay inside that household.
  Ratings are household-reported for dietary profiles, not privately authenticated personal votes.
  All existing household readers can see the history. Explain that visibility before recording.
- Enabling/changing scope starts a fresh learning epoch. Only explicitly saved/corrected cooks in
  the current epoch contribute; no opt-in or restart silently processes old ratings. Opt-out/reset
  disables learning, erases aggregates and starts a new epoch. Raw history has separate confirmed
  single-entry/all-history deletion; reset must not silently delete it. No analytics requests leave
  the server and no health/rating data appears in application logs or public fixtures.
- Incrementally update bounded recipe/attendee counts and rating sums on save/correction/deletion.
  Validate persisted aggregates against current-epoch records at restart. Do not treat missing
  ratings as zero or timer estimates/elapsed wall time as measured effort.
- Reuse the planner's current dietary/time/quantity checks before ranking. Explicit exclusions,
  allergens, intolerance triggers, compositional closure and current plus frozen declarations are
  hard filters. Unknown allergy coverage continues to require review. Learning cannot make an
  infeasible recipe eligible. Recommendation selection retains the existing explicit plan review.
- The baseline orders feasible recipes by declared dislikes and stable recipe identity, with a
  deterministic discovery position for a recently added recipe. An opt-in
  versioned integer rule uses smoothed recorded ratings, actual cooking frequency and recent-cook
  variety; confidence describes evidence counts, not a probability or allergy/health assurance.
  Keep recipes without current learning evidence visible and reserve an early discovery position
  within equal dislike groups. Recipe-identity feedback can include older revisions; disclose their
  contribution count. The variety window uses the seven reported civil days ending on the plan day.
  Return reasons, sample counts, model/history versions and baseline position. Evaluate the whole
  bounded pool once and return at most 20 previews per page, rather than ranking separate client batches.
- Model-change evaluation is an offline, deterministic, versioned synthetic holdout benchmark
  against the baseline: hard violations, held-out acceptance, distinct recommendations and held-out
  rating utility. Acceptance/utility are explicitly labelled offline proxies, not measured human
  satisfaction or causal gains. Keep real-world effectiveness unknown until observed evidence exists.
  Constraint failures block the benchmark regardless of preference gains. No opaque ML framework
  is selected before this evidence. Publish no evaluation dataset derived from private households.

## Implementation and verification
Use `src/feedback/` for the bounded history/learning model, storage and protected routes; extend
`src/planning.rs` through shared candidate checks, and reuse completed cooking sources. Native
Vue views/contracts live in `clients/nextcloud/src/`; integration tests in `tests/feedback.rs`,
algorithm/storage regressions in the feature module, browser coverage under `clients/nextcloud/e2e/`.
Use typed tagged variants, exact integer counters/durations, stable keyed EN/DE messages and native
Nextcloud controls. Existing recipe files, grants, plans, authentication and CSRF remain intact.

Commands: `cargo nextest run --workspace --locked`; `cargo clippy --workspace --all-targets --locked -- -D warnings`;
`npm --prefix clients/nextcloud test`; `npm --prefix clients/nextcloud run build`; all repository gates.
Test planned/active/cancelled exclusion, completed/manual sources, duplicate/stale writes, scope/reset,
corrections/deletion, restart/corruption, household/role isolation, cold start/discovery, immutable
references, strong positive ratings against hard constraints and reproducible baseline evaluation.
Live checks use new ledger-recorded synthetic households only; never inspect/copy retained private
household directories for feature testing or cleanup. `AGENTS.md` remains unchanged.
