# ADR 0016: durable cooking instances and civil-time planning

Status: Accepted for the customer-requested F12/F13 implementation, 2026-11-10

## Context
Recipe revisions already contain ingredient/part references and exact active/passive step durations.
F12 needs independent recoverable timers; F13 adds reviewed dependency schedules across civil-time changes.
The customer requests both native Nextcloud frontend and backend in one PR, retaining existing data/auth.

## Alternatives
Browser-only countdowns cannot dispatch while closed. Treating local timestamps as UTC corrupts DST
transitions. Separate dough/timer engines duplicate deadlines and recovery. Guessing fermentation durations
or multiplying recipe time by servings invents information. A new scheduler service is unnecessary.

## Decision
Use one household cooking file under the existing writer lock. Instances pin recipe revision/servings;
stages retain their recipe-step or explicitly household-defined origin. Timers store absolute UTC seconds
and paused remaining durations. Reopening never resets duration. Completion is explicit, including overdue
phases; starting a step requires completed dependencies. Existing recipe parts seed an editable dependency
DAG. No automatic biological outcome, equipment control or nutrition changes.

Target-time planning uses backward duration bounds and forward dependency/actual-state constraints.
Repeated stages expand into named occurrences with explicit waits. Preview every target/delay/timer change;
retain the exact proposal in a short-lived actor-bound server cache and apply only its current domain
revision. Completed work remains fixed. Show active/equipment overlaps, missed targets and declared
fermentation/temperature uncertainty instead of silently resolving them.

Add the `jiff` dependency with standard-library support and an always-bundled timezone database. This
accepted feature needs IANA timezone resolution in minimal containers; fixed offsets and browser parsing
cannot validate folds/gaps. Reject nonexistent local times and require an explicit earlier/later choice
for repeated times. Absolute deadlines remain authoritative; display the chosen timezone and UTC offset.

A bounded Tokio worker uses internal typed request extensions, never browser-provided identity headers,
to claim/settle persisted notification attempts. Only explicitly subscribed current writers receive
Nextcloud reminders. Persist sending before requesting the documented `AppAPI` notification endpoint.
Accepted means accepted by Nextcloud, not confirmed on a device. Ambiguous outcomes become unknown and
are never automatically replayed; definitive connection failures may retry with bounded backoff. Startup
marks interrupted sending unknown. Disable/complete/cancel/replan cancels obsolete pending attempts.
Notification bookkeeping does not invalidate an otherwise unchanged user schedule preview.

## Consequences
Readers inspect but cannot operate timers or subscribe. Browser permission is device-specific and reported
as such; no guaranteed OS alarm or push delivery. Offline clients retain a stale read-only view and elapsed
clock display until reconnect. Notification failures and missed events stay inspectable on reopening.
No account credentials, household health data, recipe instructions or notification bodies are logged.
Preview cache loss requires review again. Storage and notification limits fail explicitly; backup/rollback
covers membership, recipes and all household files together. No new standalone or Flutter shell is added.

## Sources
[Nextcloud 35 notifications API](https://docs.nextcloud.com/server/stable/developer_manual/exapp_development/tech_details/api/notifications.html),
[Jiff civil-time ambiguity](https://docs.rs/jiff/latest/jiff/tz/struct.AmbiguousZoned.html).
