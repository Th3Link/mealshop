# Brewing in Nextcloud

Open **Brewing** in the household navigation. **Brew recipes** defines a reusable process;
**Equipment** contains actual vessels/tools; **Batches** pins a recipe revision, equipment and dates.
Current household owners/editors can write; readers can inspect all three views and saved readings.
Food recipe shares, cooking history and inventory consumption do not turn a brew into a food recipe.

## Choose and adapt a method

Sixteen source-linked starting templates cover beer and independently researched cider, white/red wine
and traditional mead. The twelve beer templates cover extract, extract with specialty-grain steeping,
partial mash, BIAB, Grainfather, Brew Monk, Speidel Braumeister and separate-tun/two-vessel/three-vessel/
HERMS/RIMS DIY setups. BIAB belongs to all-grain. Fermentable strategy, vessel layout, heating,
sparging and mash approach are independent recipe facts. Top-return baskets and upward-flow malt pipes
remain distinct. Read the actual model/version source before adapting a template to your equipment.
[Research and primary references](../../specs/0014-brewing-fermentation/research.md).

Templates supply originally authored phase prompts, not a ready-to-brew formulation or universal
mash temperature, water ratio, efficiency or fermentation duration. Enter the recipe's exact target
volume, ingredients and original quantities, phase instructions, dependencies and declared durations.
Optional quantities/temperatures are added with plus actions. Enter missing facts as unknown rather
than zero. All phases stay visible. Method references remain in every immutable recipe revision;
later edits leave existing batches pinned to their original recipe.

A phase can have active work, passive duration and shortest/longest passive bounds, a reminder lead,
target temperature, actual phase volume and uncertainty notes. Beer phase work/passive durations
support up to 365 days each; food-only duration policy is unchanged. Times are estimates, never a
fermentation/readiness model. A phase without duration runs manually until explicitly completed.

## Equipment and independent batches

Record each actual kettle, mash tun, hot-water tank, pump, cooler, fermenter or bag separately. Capacity
is a declared usable liquid volume, not an assumption derived from a manufacturer name. A batch freezes
its equipment details; subsequent edits/archive do not rewrite that record. Editing equipment checks
both the household revision and the original equipment details to prevent a stale rebase overwriting
another edit.

Choose a saved recipe revision, intended brew-day date/time/IANA zone and actual equipment per phase.
A late brew-day start projects unfinished work from the current server time. Optional recipe yeast
preparation is resolved by **calendar days**, keeping the intended local time across daylight-saving
transitions. It is product/recipe specific; choose the recipe reminder, skip it, or enter an explicit
alternative. A requested preparation reminder already in the past must be skipped or rescheduled.
A nonexistent civil time is rejected; a repeated time needs an earlier/later choice.

Review resolved phase dates, original quantities and equipment before saving. Known phase volumes are
compared to declared equipment capacity using exact ml/l conversion. Missing volume/capacity remains
unknown. Planned overlapping equipment reservations block a conflicting batch/replan atomically.
Unknown-duration phases cannot establish a reliable future occupied interval. Starting/resuming checks
actual equipment use again; running/paused work retains the physical equipment until explicit completion
or cancellation, even after its projected end. Unassigned equipment cannot be checked or reserved.
Independent equipment permits simultaneous batches. Cancelling/pausing/replanning one affects only it.

Start, pause/resume timed phases, adjust a pending start or actual timer deadline through review, and
complete phases yourself. An elapsed brewing/fermentation timer does **not** complete dependencies or release equipment.
Unknown-duration manual work has no fake countdown. Completing or cancelling releases its held equipment;
archiving is available after the batch finishes/cancels and preserves recipes, readings and receipts.

## Readings and corrections

Record the original value in °C, °F, specific gravity, °Plato, °Brix, ml or l. Decimal text retains up to
six places; localized display/input changes the decimal separator without floating-point arithmetic.
Measurement civil time/zone/fold and resolved instant are separate from authenticated actor/entry time.
Measurements cannot be future-dated. Manual input or named instrument attribution is explicit.
Instrument transcription does not configure a live connector. Authenticated API reporting can declare
instrument attribution, without claiming that Mealshop independently authenticated a physical sensor.

Calibration is explicitly unknown or declared with details, optional calibration time, reference
temperature and source link. No implicit SG/Plato/Brix conversion, temperature/alcohol correction,
ABV/IBU calculation or automatic readiness conclusion is made. Correct the latest reading with a reason;
the original stays saved. History loads in pages of 50 and retains the correction chain.

## Personal reminders, conflicts and recovery

The existing single durable household worker handles food and brewing/fermentation events. Reminders start off; each
current writer can opt into their own batch reminders. Browser permission is reported separately.
Prepare/start/active-ended/elapsed events link to the native batch. **Accepted** means Nextcloud accepted
the notification, not that a device displayed it. Interrupted attempts become **unknown** on restart;
resending an unknown result requires duplicate-risk acceptance. At most three attempts/backoff, personal
acknowledgements, cancelled obsolete events and missed events older than 24 hours retain the existing
cooking delivery semantics. Readers neither subscribe nor see another account's subscription/history.

Drafts survive background reads and rejected writes. Load the latest household state while keeping
entries, then review again; a concurrently changed recipe/equipment still needs explicit reconciliation.
A lost/uncertain save response retains the original immutable request/key; retrying it returns the same
operation, even after server restart, rather than creating another recipe, batch or reading. A stale
view is read-only until current server state is loaded. This is not an offline mutation queue.

Beer recipes, equipment snapshots, batches, observations and receipts share the existing private 0600
`household/cooking.json` transaction, fsync/rename recovery and 4 MiB publication limit. Recipe revisions
are immutable; archiving retains history. Limits are 200 recipe revisions, 100 equipment records,
100 batches/shared runs, 2,000 readings and 2,000 operation receipts; notification limits also apply.
An uncertain publication poisons writes/reads until verified recovery. Back up/restore the entire file
with the sole writer stopped; a pre-F14 binary cannot read populated F14 state. The delivered test
verification never copies, hashes or inspects retained household directories.

## API contract

Use the normal authenticated `AppAPI` household proxy prefix, without hardcoded hostnames:
`/api/households/{id}/domain` followed by:

| Method | Path | Result |
|---|---|---|
| GET | `/api/household/brewing/templates` | Validated method catalogue and primary source facts |
| GET | `/api/household/brewing` | Latest recipe heads, equipment, actor-filtered batch state and worker health |
| GET | `/api/household/brewing/recipes/{id}?revision=N` | Immutable recipe revision |
| GET | `/api/household/brewing/batches/{id}` | Pinned recipe/equipment, phase state, personal notices and first reading page |
| GET | `/api/household/brewing/batches/{id}/readings?before=ID` | Earlier batch readings, maximum 50 per page |
| POST | `/api/household/brewing/preview` | Validated before/after state without publication |
| POST | `/api/household/brewing` | Reviewed command, durable idempotency receipt and current state |

Mutation envelopes contain `expected_revision`, an opaque `idempotency_key`, `command` and `reviewed`.
Commands save/archive recipes/equipment, create/replan/operate/cancel/archive batches, manage the current
actor's subscription/notices, or append/correct a reading. `save_recipe` checks its pinned expected recipe
revision. Equipment edits include `expected_equipment`; new equipment omits it or supplies null.
Retries require identical parsed payload, expected revision, actor and key, plus current write authority.
Receipt keys never appear in normal reads. Reusing a key for a different action conflicts.

Independently sourced cider-from-juice, white wine, red wine and traditional mead templates include
their own phase order and process classification. White wine presses before fermentation; red wine
ferments on skins before pressing. They do not inherit grain/mash/sparge settings. All timings, readings,
additions and completion decisions remain explicitly recipe-specific. Automatic heater/
pump control, configured live device/vendor integrations, food-safety guarantees, automatic stock
consumption and non-beer chemistry are outside this delivered contract.
