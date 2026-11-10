# Guided cooking and multi-day baking

The native Nextcloud **Cooking & baking** view uses the household's existing writer/reader roles.
Open it from the navigation, or choose **Start cooking or baking** in a recipe. Select a saved recipe
revision and serving count, review its preparation steps, then save a separate cooking session.
Ingredient quantities are scaled with the existing exact decimal arithmetic. Later recipe changes
leave the source revision and serving count pinned in the session.

## Prepare and review

Cooking starts when you choose a step. Bread, rolls, pizza and baguette plans also require a desired
finish and IANA timezone. Steps contain active work, expected passive time, shortest/longest passive
bounds, dependencies, equipment, reminder lead time and optional repetitions/waits. Added steps require
a source/reason. Repeated occurrences are independent named timers; a later occurrence depends on the
previous one. Recipe parts seed dependencies; review any edits yourself.

Planning works backward from the desired finish, then respects dependencies, explicit start delays and
actual running/completed work. Review every target, delay or timer adjustment before applying it.
The review shows old/new times, windows, equipment conflicts, overlapping active work and missed targets.
Times include the timezone and UTC offset. Nonexistent spring-transition times are rejected; a repeated
autumn time requires the first or second occurrence. The server's bundled timezone data resolves times;
absolute UTC seconds, rather than browser-local arithmetic, drive deadlines.

Durations and passive windows are user/recipe estimates, not a fermentation model. Record conditions
and uncertainty where known; unknown temperature/fermentation conditions remain unknown. No result,
readiness, food-safety or health outcome is inferred from a countdown.

## Cook and recover

One action starts a named timer with the saved active/passive duration. Concurrent timers remain
independent. Pause stores the remaining duration; resume preserves it. A deadline becoming overdue
does not restart a timer or complete the step. Complete steps yourself to unlock dependent steps.
The planned timeline stays as reviewed until you explicitly review an adjustment; timer controls
show the actual deadline separately. An explicit step delay survives later target changes.

Saved deadlines survive server restart. The browser advances its last server clock with monotonic
elapsed time and refreshes while visible. If disconnected, its retained view becomes read-only with a
stale-state message; reconnect loads current server state. This is not an offline mutation queue.

## Personal Nextcloud reminders

Reminders are off by default. Each current owner/editor can opt into their own reminders for a session;
readers cannot change timers or subscribe. Browser permission is reported separately and does not
establish device delivery. Denied/unsupported desktop notifications leave due actions visible here.

A server worker checks saved sessions every two seconds and attempts at most eight deliveries per
cycle, rotating households fairly. It names the action and equipment for preparation/start/active-phase/
elapsed reminders through Nextcloud's `AppAPI` notification endpoint. **Accepted** means Nextcloud accepted
it; **unknown** means acceptance could not be established. Neither means a device displayed an alarm.
Interrupted sending becomes unknown after restart and is never automatically replayed. Connection
failures retry at most three attempts with backoff. Retrying an unknown result needs explicit duplicate
risk acceptance. Events more than 24 hours late are listed as missed rather than sent as fresh alarms.
Acknowledgement stops a queued reminder. Cancelling/completing/replanning or opting out cancels obsolete
queued reminders; a request already in flight cannot be recalled. Each account sees only its own
subscription/delivery history.

Sleeping operating systems and unreachable devices do not provide guaranteed alarms. Check the due
actions and delivery history when returning. A stale worker check or storage failure is shown explicitly.

## Transport and operations

Protected routes live below `/api/households/{id}/domain` in `AppAPI` mode:

| Method | Path | Purpose |
|---|---|---|
| GET | `/api/household/cooking` | Session summaries, domain revision, clock and worker health |
| GET | `/api/household/cooking/template` | Recipe ID/revision/servings/timezone → editable steps |
| POST | `/api/household/cooking/preview` | Actor-bound create/replan review; no durable mutation |
| GET | `/api/household/cooking/{id}` | Session, exact ingredients and actor's delivery history |
| POST | `/api/household/cooking` | Reviewed apply, timer, cancellation, subscription and notice commands |

Every mutation supplies `expected_revision`. Preview apply also requires `reviewed:true`; the exact
cached review expires after ten minutes or replacement by a newer review for that actor. Restart
invalidates cached reviews. A stale revision is rejected rather than silently overwriting changes.
Delivery bookkeeping has a separate revision and cannot invalidate an unchanged schedule preview.
`/api/household/cooking/internal` additionally requires an unforgeable internal Rust extension;
browser headers or an owner bearer token cannot invoke the worker.

Set optional `MEALSHOP_PUBLIC_NEXTCLOUD_URL` to the browser-visible Nextcloud base URL, including an
installation prefix, when `NEXTCLOUD_URL` is an internal container hostname. Both are operator-trusted
HTTP(S) URLs without credentials, query or fragment; browser requests cannot override them. No hostname
is compiled into the client. Without the public override, notification links use `NEXTCLOUD_URL`.

`household/cooking.json` is a private, bounded, versioned file under the existing household writer lock.
Atomic pending-file writes sync data and directory. Corrupt/interrupted writes fail closed; restore a
verified complete household backup. Back up membership, recipe revisions and all household files together.
Limits: 100 saved sessions, 100 input stages/200 expanded occurrences per session, 2,000 reminder records
per household, 4 MiB store, 20 actor-bound cached reviews. Remove completed/cancelled sessions and their
reminder history to free capacity. No standalone/Flutter shell or external calendar integration is added.

## Calendar anchors and completion-triggered waits (F31)

Stages optionally have `start_at` as a local date/time, IANA timezone and DST fold choice.
The server resolves and preserves the calendar anchor; ambiguous or nonexistent local times
cannot silently become different instants. Calendar reminders do not prevent an earlier actual
completion. Explicit reviewed delays still retain their separate `not_before` constraint. Multiple independent stages can share an anchor.
New recipe-derived stages with waiting time carry `start_after_work: true`: the completion
action starts only their passive timer from the actual click. Active work remains an estimate
for calendar planning, so 2 minutes of estimated mixing plus 45 minutes waiting still means a
07:08 completion starts a timer until 07:53. Legacy/custom stages default to the existing timed
active-and-passive behavior. Starting the next step acknowledges elapsed waiting timers;
paused timers and timers still counting active work require their existing explicit transitions.
This acknowledges elapsed time, not physical readiness or food safety. No timer is automatically
started by a scheduled notification, and no separate sourdough assessment is required.
