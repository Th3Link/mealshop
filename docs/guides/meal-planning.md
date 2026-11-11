# Family meal planning

Enable household access as described in household.md. The executable links recipes and protected
planning automatically when `MEALSHOP_HOUSEHOLD_TOKEN` is configured. Embedding applications use
`family_router`; separate standalone household builders have no recipe store and planning returns
`recipe_store_unavailable`. Operator and reader roles are the same as F09. No tokens or profile
snapshots are stored in browser storage. Planning responses are `no-store` and data is not logged.

## Demo flow
Enter the household token, then Load family and saved plans. Choose date/meal, attendees,
recipe revision, target servings and available active/elapsed/advance seconds. Use the candidate
multi-select to request suggestions. The native Nextcloud client submits its whole recipe pool
(up to 1,000 unique recipes) and pages the coherent order in 20-result pages. A manual selection or suggested alternative
must be previewed before applying. Preview shows exclusions, selected members' current review
notes, duration estimates, scaled nutrition and signed changes in required shopping grams.
Blocked candidates cannot be applied. Every unblocked meal still requires explicit review of
unknown dietary coverage and batch-time uncertainty. Changing inputs invalidates the preview.

After acknowledgement, Apply saves the meal slot and refreshes aggregate shopping requirements
for all saved slots. Load a saved meal to change attendance/recipe/servings/time. Removal has its
own negative-quantity preview and operator-only apply. No shopping item is purchased or external
list modified. A profile/plan revision conflict requires reloading and a fresh preview; stale
choices are never silently applied. Profile/reference changes mark saved slots for renewed review.

## Read-only calendar import
The file input accepts an attendance JSON snapshot up to 32 KiB. Example:
```json
{"source":"Family calendar export","event_id":"event-42","date":"2026-11-09","member_ids":["member-id-from-household-export"]}
```
Consent must be checked after import; merely choosing a file does not consent. Check Use calendar
attendance to use its proposal; uncheck it and use the attendee selector to override. Source,
date and proposed IDs stay visible. Calendar date must match the meal date. Clear removes the
snapshot from the draft. Member IDs must already be mapped to this household; unmapped attendees
cannot silently become missing dietary constraints. No network calendar connection, recurrence
expansion or external calendar write occurs. This adapter consumes read-only snapshot data;
provider integrations remain separate deployment work.

## API
GET `/api/household/mealplan` returns schema/revision, saved meals, current profile revision,
slots requiring renewed review and derived `shopping_requirements`.
POST `/api/household/mealplan/preview` accepts:
```json
{"date":"2026-11-09","meal":"dinner","recipe":{"id":"recipe-00000001","revision":1},
 "attendees":["member-id"],"calendar":null,"servings":2,
 "active_seconds":1800,"elapsed_seconds":3600,"advance_seconds":0}
```
`attendees: null` uses `calendar.member_ids`; that calendar object additionally requires
`consent: true`. Explicit attendees override it. Targets are 1–1000000 integer servings.
Active/meal-day elapsed budgets are 0–86400 seconds; advance lead is 0–2592000 seconds.
Meal keys are lowercase ASCII letters/underscores, 1–40 bytes. Dates are validated, including
month lengths/leap years. Empty steps yield `preparation_time_unknown`; totals are sequential
recipe estimates and do not infer parallel work or batch-time scaling.

Preview returns `plan`, `profile_revision`, `plan_revision`, resolved attendees/source/override,
F09 review evidence, `feasible`, conflict codes, active/elapsed/advance timing, rationally scaled
nutrition and signed `shopping_delta`. No persistence occurs.

POST `/api/household/mealplan/suggestions` takes `{"template":{...plan...},"candidates":[
{"id":"recipe-00000001","revision":1}],"page":1}` (1–1,000 unique candidates).
The entire feasible pool is ranked once per request, with at most 20 full previews returned per page.
Responses include model/history versions, total/page and recommendation evidence; owner-controlled
learning and reset are described in [Cooking history](history-and-preferences.md).
Every rejected candidate includes conflicts/errors. No match returns `no_feasible_meal`;
preferences cannot relax exclusions.

POST `/api/household/mealplan/apply` takes `{"plan":{...},"expected_revision":0,
"profile_revision":1,"reviewed":true}`. Use the revisions from the preview. Matching date/meal
replaces one slot; recipe revisions stay pinned. A conflict returns 409; hard/time conflicts or
missing review return 422. Readers cannot apply (403). Acknowledgement means human review, not
an allergen-safety certificate. The server recomputes constraints; client `feasible` fields are
not accepted. GET subsequently derives shopping quantities from the saved pinned revisions.

POST `/api/household/mealplan/remove-preview` with the meal settings previews removing that
slot. Apply with `remove: true`, current revisions and review acknowledgement removes it.
The removal preview carries the correct negative quantities, including after constraints change.

## Storage/recovery
`mealplan.json` is a schema-1 snapshot in the protected household directory, at most 100 slots
and 1 MiB. The existing household process lock covers it. Writes use private `.mealplan.pending`,
file sync, rename and Unix directory sync. Ambiguous I/O failure disables further plan writes;
profiles remain independently accessible. Startup rejects malformed, oversized, duplicate-slot
or interrupted plans instead of silently creating an empty planner. Export JSON and backups can
contain attendance and calendar IDs; protect them like profiles. Removing profiles does not
rewrite existing plans; remove affected slots explicitly if their historical attendance must
also be removed. Previously exported copies/backups are outside server revocation.

For recovery, stop all writers and back up both recipe and household directories consistently.
Inspect any pending file and snapshot before removing a verified stale household lock. Work on
a recovery copy, preserve all recipe revisions referenced by plans, restore a validated complete
snapshot, then verify protected plan export, nutrition and shopping quantities. Do not directly
edit files while the application is running. Network filesystems and cross-platform crash
behaviour are not validated; Linux local filesystems are the initial target.

## Production and mealprep allocations (F31)

A saved plan's `servings` is the production count. Optional `allocations` contains date, meal and
positive whole serving count for each eating day. The total cannot exceed production; fewer
allocated portions leave the remainder unallocated. An explicit empty array means no eating days
are assigned. Omitting allocations preserves the existing single-slot behavior. The root date/meal remains the stable production identity during edits,
including when an eating day is moved. One shared attendee list applies conservatively to every
allocation; existing hard exclusions and profile revisions still apply.

Optional `preparation` contains a date and HH:MM time. Preparation must not follow an eating day.
The native calendar shows these in the same week as meals. Explicit preparation-only entries use
meal `preparation`, with no invented attendees; assigning actual eating days requires attendees
and the usual dietary review. Shopping requirements count production once. Selecting two eating
days of the same production exports one canonical source to shopping. Removing the entry removes
all its allocations; the native review explains the resulting shopping delta.

Opening cooking from a saved entry pins its recipe and production count and records a checked
`plan_slot` origin. Root stages inherit the chosen calendar preparation time; individual stage
anchors can be edited in the reviewed cooking draft. Saved active stage times appear in the
meal-plan calendar and link to their cooking session. Preparation-only metadata does not itself
promise notification delivery: create/review the durable cooking session and explicitly subscribe.
Changes to a meal plan do not silently rewrite an already reviewed cooking session.
