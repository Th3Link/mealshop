# Native household planning and physical preparation (P3)

P3 extends the same Nextcloud household and its owner/editor/reader roles. It activates Today/Plan
beside Shopping/Recipes/More. The phone uses real authenticated household domain routes; no fixed host,
alternate login, plan cache or new offline write queue. Readers can inspect plans and stock.

## Review a week

Plan opens a week strip and selected day's breakfast/lunch/snack/dinner. Today uses the current civil
day, ordered meals and dated preparation actions. Open a meal to change its pinned recipe, attendees,
production portions, budgets, eating allocations or preparation time. Uncheck skipped people. Review
constraints, exact/ranged/unknown nutrition and signed ingredient changes before saving/removing.
The same participant list conservatively protects every eating allocation. Unallocated planned portions
are not physical leftovers. A recipe's time estimate does not promise batch-time scaling or parallel work.

Regular attendance configures each person's weekdays/meals. Date exceptions replace a slot's list;
an empty list explicitly skips everyone. Personal hard availability remains an upper bound. Updating
attendance invalidates existing meal review markers. Existing web previews/apply enforce the schedule.

Plan a week chooses a civil start date, meals, up to 100 current recipe revisions, portions per attendee
and active/elapsed/advance budgets. The server preserves occupied meals and physical allocations. It
uses canonical dietary/time review, ranks feasible candidates deterministically, and explains each
unfilled slot. Dislikes cost 1000 points, each previous recipe use in the week costs 100, opted-in
history contributes its existing score clamped to ±100; optional active minutes add their count and
up to ten reused ingredients subtract two each. Stable recipe IDs break ties. This greedy heuristic
is not a global optimizer. Cost and nutrition are not optimization inputs; missing information never
means zero. Canonical per-recipe nutrition and sources remain available from inspection/meal review.

Confirm every feasible proposed slot. Impossible/skipped slots stay visibly unfilled. Applying writes
the accepted set atomically, without relaxing exclusions or changing shopping/stock. A profile, plan,
ingredient/allergen or candidate-head change requires fresh review. An ambiguous week response can
retry the same proposal; a retained applied receipt returns success without applying twice.

## Review the actual shopping change

Update shopping from plan defaults to currently exported productions plus this week's new unprepared
productions; retain other desired plan sources. Review the exact current list before confirmation.
This replaces only selected plan sources, counting each production once across multiple eating days.
Manual entries and purchased receipts remain. The preview displays remaining purchase amounts after
existing stock coverage, not just a blind ingredient difference. Concurrent changes reject a stale
request. A lost response is reconciled by refreshing and reviewing; there is no unadvertised sync replay.

## Record actual food and leftovers

Use a meal's action menu or More → Stock & preparation → Prepared portions. Record only an observed
physical yield, such as six whole portions. Optionally link its pinned planned production; a second
active output for that source is rejected. The observed yield may differ from planned production and
retains both counts. Linked physical output protects the production recipe/amount from silent edits.

Declare preparation/storage dates, location and ambient/chilled/frozen/other condition. A user date
is optional; there is no inferred expiry, safe shelf life or automatic thaw schedule. Allocate two
portions each to two later meals: six = four allocated + two available. Each allocation has its own
participants/current dietary and availability review. Over-allocation and duplicate participant/slot
holds are blocked. Recording physical output/allocation creates no new ingredient-shopping source.
Planned and physical rows share a presentation when they refer to the same production.

Record eating explicitly. Release unused holds, correct a mistaken eating record with a reason, or
reverse an output after all allocations are released and no use remains. Storage changes and reversals
remain in ordered history. Output booking never silently consumes raw ingredients: completed-cooking
consumption remains the separately reviewed F15/P4 workflow.

Ingredients & products provides exact lots, original quantities, location/batch/date, quantity
corrections, reservation/release, explicit purchase-receipt booking, history and reversal. Mass,
volume and count never cross implicitly; unmatched stock never matches a recipe by text. Offers retain
explicit shop/product/ingredient/package/price/currency/date/source, with review and current dietary
checks before adding packages to shopping. Existing [F15 rules](inventory.md) remain authoritative.

## New/extended API contracts

Paths are relative to the existing authenticated household domain. Responses are `no-store` and reuse
current JSON/CSRF/authorization/error boundaries. Inputs reject unknown fields; these are new P3 routes.

| Request | Body / result |
| --- | --- |
| `GET /api/household/mealplan` | Existing plan projection plus `attendance: {recurring, overrides}`. |
| `POST /api/household/mealplan/attendance` | `{expected_revision, attendance}`; recurring rows `{member_id, weekdays:[0..6], meals:[...]}`, overrides `{date, meal, member_ids}`. Max 100 people/366 date slots. |
| `POST /api/household/mealplan/week/preview` | `{start_date, meals, candidates:[{id,revision}], active_seconds, elapsed_seconds, advance_seconds, servings_per_person, prefer_quick, prefer_reuse}`. Returns server-owned `proposal` with identity, expiry, feasible plans, unfilled reasons/evidence, aggregate shopping delta and ranking limits. Writer-only because it retains a proposal; meals are unchanged. |
| `POST /api/household/mealplan/week/apply` | `{proposal_id, reviewed:true, confirmed_slots:[{date,meal}]}`. Exactly the feasible slot set, no client plans. Atomic apply/current-reference checks; applied proposal replay is harmless. |
| `POST /api/household/shopping/plan-preview` | `{expected_revision, plan_revision, slots:[{date,meal}]}`. Returns an immutable existing `sync_plan` command and changed before/after list rows. Authorized readers may preview; the existing write endpoint still requires writer rights. |
| `GET /api/household/inventory` | Existing view plus `prepared`, paginated `prepared_history` and `prepared_event_total`, using the existing history offset/limit. No private operation keys. |
| `POST /api/household/inventory[/preview]` | Existing actor-bound `{expected_revision, idempotency_key, reviewed, command}` with the new commands below. Preview never publishes. |

New stock commands use the existing Rust tagged enum:

- `record_prepared`: `{input:{recipe_id,revision,portions,produced_on,storage,source}}`, where source is
  null or `{date,meal}` of the pinned production. Storage is `{location,condition,declared_on,use_by}`.
- `store_prepared`: `{batch_id,storage}`; records a new explicit storage declaration.
- `allocate_prepared`: `{batch_id,date,meal,portions,member_ids}`.
- `release_prepared`, `use_prepared`: `{batch_id,allocation_id}`.
- `undo_prepared_use`: `{batch_id,allocation_id,reason}`; use becomes held again.
- `reverse_prepared`: `{batch_id,reason}`; requires no outstanding hold/use.

## Persistence, retry and recovery

The plan file retains five proposals for 30 minutes, at most 28 slots/100 candidates per proposal,
monotonic IDs, original snapshot limits (100 meals/1 MiB) and synced atomic replacement. Proposal
creation does not increment meal revision; attendance/apply do. Stale/expired proposals never bypass
current hard checks. Applied proposal receipts are retained until proposal expiry/pruning.

Prepared batches use the existing shared shopping/inventory snapshot, process lock, private atomic
publication, revision and actor-bound receipts: at most 500 batches/100 allocations each, within the
same 4 MiB and 2000-operation limits. Whole portions form their own dimension. Original output and
allocation references are checked against operation receipts on restart. Ambiguous publication fails
closed. Confirmed mobile stock retries retain the exact RAM-only key/body; leaving an uncertain review
warns that its retry record will be discarded. Check history before creating another intent.

Old snapshots remain readable with default-empty fields. After publishing P3 state, older binaries
cannot safely parse it. Back up/recover the matching complete household files with writers stopped;
never delete receipts or strip new fields to downgrade. No retained customer records are changed by
tests. See [ADR 0029](../adr/0029-mobile-week-and-prepared-stock.md) and F35 verification for actual evidence.
