# Cooking history and household preferences

## Record what actually happened
Open Cooking history in a household and record a completed cook, or use the action on a completed
cooking session. A plan, saved schedule, elapsed timer or cancelled session never creates a history
entry automatically. You confirm completion yourself. For a session, every step must already be
completed; the recorded recipe revision and serving count come from that session. A session can be
recorded only once while its history entry exists. Removing the session does not remove its history.

Manual recording covers cooking performed outside the timers. Select a saved recipe/revision,
servings, actual civil date/timezone and actual attendees. Effort and household/profile ratings are
optional. Blank means unknown, not zero. Active effort is explicitly reported whole seconds (the
form offers seconds/minutes); neither scheduled estimates nor elapsed wall time is presented as a
measurement. Ratings use a labelled 1–5 scale. Individual ratings are household reports for dietary
profiles, not privately authenticated votes from linked Nextcloud accounts.

All current household readers can read this history. Owners/editors can record, correct and delete
individual entries. The original recorder/time and last correcting actor/time are retained. Recipe
links open the pinned revision and return to history. Deleted dietary profiles remain referenced by
ID and appear as removed profiles; select current attendees before saving a correction. A future
completion date in its selected timezone, unknown attendees and stale profile/history revisions are
rejected without discarding the draft. A completed-session entry cannot be reassigned to another
recipe or serving count. Manual corrections are explicit source changes, not automatic revision updates.

## Control local learning
Learning is off by default. Only the household owner can enable it, choose which profile ratings may
contribute, reset/opt out, or bulk-delete history. Household cooking frequency and household ratings
form the base; the selected profile ratings are optional. Everything stays within this household;
no ratings, health data or training dataset are sent to an external learning service or logged.

Enabling or changing scope starts a new epoch and clears learned aggregates. Only cooks explicitly
saved or corrected afterward contribute. Saving an unchanged scope retains the epoch and contributions.
Old retained ratings are never replayed silently on opt-in,
reset, restart or re-enabling. Correcting an old entry is an explicit new contribution. Reset disables
learning, removes aggregates and retains raw history. Delete all cooking history is a separate confirmed
action; it removes records/contributions while leaving the chosen learning policy available for future
cooks. Individual deletion/correction subtracts the prior contribution before adding any replacement.

## Understand suggestions
The existing planner checks the entire bounded candidate pool against current attendees, declared
exclusions, allergens/intolerances, compositional closure, frozen plus current ingredient declarations,
time budgets and exact quantity limits **before** ranking. Unknown coverage still needs explicit review.
A positive rating cannot make a blocked recipe eligible. Selection still opens the normal plan review;
no recipe is automatically scheduled, and scheduling does not count it as cooked.

Rule mode orders by declared ingredient dislikes and stable recipe identity, reserving an early
position for a recently added candidate within each equal-dislike group. Learned mode keeps that
primary dislike grouping, then applies a versioned integer rule:

- Household rating points: `floor((sum + 6) × 100 / (count + 2)) − 300` (two neutral prior ratings).
- Profile points: the integer mean of those smoothed points for selected attendees with permitted
  rating evidence; missing ratings contribute no fabricated zero rating.
- Add `5 × min(completed cooks, 10)` for recorded familiarity.
- Subtract `30 × min(recent cooks, 3)` for variety. Recent means the seven reported civil days ending
  on the selected meal day; date-only records are not treated as measured physical timelines.
- Keep an early discovery position for a candidate without current-epoch cooking evidence. Each
  group retains all feasible candidates, accessible through 20-result pages. A fresh epoch can have
  no learning evidence even though raw history exists.

Feedback is pooled by stable recipe identity. The explanation explicitly counts cooks from earlier
recipe revisions; hard constraints are always checked for the target revision. Explanations show
observed counts and whether history is absent, limited (1–2 cooks) or repeated (3+). These are history
support labels, not probabilities, guarantees of enjoyment or allergy/health confidence.

Versions `rules-v1` and `history-v1` have a checked synthetic holdout comparison. See
[F11 evaluation](../../specs/0011-recommendation-feedback/evaluation.md). Its acceptance and rating
utility are offline proxies; real-world satisfaction and causal effectiveness remain unmeasured.

## API and recovery
Protected household routes use the existing `AppAPI` proxy below `/api/households/{home}/domain`:

| Methods | Path | Purpose |
|---|---|---|
| GET, POST, DELETE | `/api/household/history` | Paged history; explicitly record a completed cook; owner-only bulk clear |
| GET, PUT, DELETE | `/api/household/history/{id}` | Read, correct or delete an entry |
| GET, PUT | `/api/household/learning` | Current policy/epoch/counts; owner-only scope/reset |
| POST | `/api/household/mealplan/suggestions` | One coherent candidate pool, explained order and paged previews |

History saves supply `expected_revision`, `expected_profile_revision`, `confirmed_completed:true`,
`source` and `input`. Manual source is `{kind:"manual", recipe_id, revision, servings}`; session source
is `{kind:"session", run_id}`. Input contains `date`, IANA `timezone`, unique `attendees`, nullable
`active_seconds`/`household_rating`, and a `ratings` map restricted to attendees with values 1–5.
Deletes require `expected_revision` and `confirmed:true`. Policy updates require both expected
revisions and `{policy:{enabled,member_ids}}`; disabled policy uses an empty profile scope.

History reads support one-based `page` (20 results), `q` up to 200 bytes and optional session identity.
Suggestion input retains the existing plan `template`, accepts up to 1,000 unique recipe candidates
and optional one-based `page`. Responses include total/page, model version, learning epoch and history
revision; each suggestion retains its ordinary preview and adds explicit recommendation evidence.
No caller may forge the recorder or owner role: `AppAPI` supplies the current authenticated actor.
Existing CSRF, enabled state, household membership, read-only roles and sharing boundaries remain intact;
a recipe grant never conveys history or learning access.

`household/history.json` stores entries and derived counts/sums in one atomic private transaction.
It is bounded to 1,000 entries/4 MiB, uses mode 0600 and the existing household writer lock.
Counters, dates, scope, identity monotonicity and reconstructed current-epoch aggregates are checked
on load. A `.history.pending` marker or corrupted aggregate fails startup. An ambiguous write stops
mutations until verified recovery. Stop the writer and restore a verified complete household backup;
do not remove a pending marker blindly. No migration rewrites existing recipe, plan, cooking or grant
files. Retain source-sharing restrictions when implementing future clients or exports.
