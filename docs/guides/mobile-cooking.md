# Native cooking, recovery and device reminders

Status: F35 P4 implementation in progress; final acceptance is recorded in F35 verification and its PR.

## Existing domain, native actions

More → Cooking & baking and an owned recipe's cooking action use the existing authenticated household
domain. Food servings never become brew volume. Recipes/parts/instructions and exact ingredient
quantities remain pinned; the server distinguishes explicit step assignment, part assignment and an
unassigned recipe reference. Always-open steps show dependencies and independent actual timers.
Starting a completion-triggered waiting step reports finished work; a notification never starts it.
Explicit completion remains separate from elapsed time and does not imply physical readiness.

Create/replan reviews retain IANA zones, fold selection, dependencies, passive windows, uncertainty and
canonical old/new times. Linking a planned production is checked again at apply. Plan locks precede
cooking locks, matching inventory and avoiding an inverted lock order. Stock consumption and observed
prepared output remain separately reviewed inventory actions; completion alone books neither.

## Additive cooking protocol

All paths are under the existing `/api/households/{id}/domain` proxy, without a fixed hostname.

- `GET /api/household/cooking` adds `action_protocol:1` and `max_operation_age_seconds:604800`.
- Existing `POST /api/household/cooking` adds optional `idempotency_key` and `occurred_at` alongside
  `expected_revision` and `command`. Keyed requests receive stable `applied_revision`, `run_id` and
  `replayed`, plus the current domain revision. Legacy callers retain their existing response contract.
- Only a keyed timer command may supply `occurred_at`: the explicitly reported action's UTC second,
  no later than the current server clock and no older than seven days. Entry time is recorded
  separately. Current revision, source kind, dependencies and transition/time bounds remain mandatory.
- `POST /api/household/cooking/action-preview` accepts the same keyed timer envelope and shows
  canonical current/proposed run state without publication. A rejected retained action needs a new
  key/current revision and explicit review, preserving its original reported instant. An unknown result
  always retries its original key/body/revision. No automatic rebase.
- Cooking details add `parts`, `step_ingredients` with original index/scope/line indexes and resolved
  `local_timer` strings. These are additive projections, not a second scheduler or recipe mutation.

Receipts share the private atomic 4 MiB cooking transaction, are scoped by authenticated actor/key
and match the entire parsed command, expected revision and reported instant. Current write authority
is checked before replay. At most 2,000 food receipts are retained; capacity rejects before publication,
without dropping retry evidence. Normal reads never include their keys. Deleting sessions does not
free receipt capacity; an administrator must review a complete verified backup/retention policy.
Old snapshots default to empty receipts; populated P4 state requires a complete validated migration
to downgrade. An uncertain file publication fails closed.

## Protected offline scope

Up to five opened food/brew sessions and twenty ordered frozen cooking intents fit inside a bounded
245,760-byte feature record in the existing encrypted account/household vault. Writes finish before
network dispatch. Only cooking timer actions may be newly recorded offline; other domain mutations
remain online. Pending, uncertain, rejected and confirmed states are distinct. Source/quantity data
stays household-private; no new cache export or shared imported text is introduced.

The existing seven-day authorization/retention boundary applies. Membership/role revalidation closes
old rights; logout/scope changes clear local projections. Offline remote revocation cannot erase a
disconnected copy immediately. User-controlled offline-copy removal cancels its local reminders and
leaves the server session intact; unresolved actions must be reconciled first.

A server-clock anchor advances with Android elapsed realtime and boot identity, independent of wall
clock edits. After an unverified reboot, reconnect before recording an offline action. Retained absolute
deadlines remain readable, and Android reboot recovery does not invent a new duration.

## Local reminders are separate from Nextcloud delivery

Local reminders default off and request notification permission only from explicit enable. A separate
optional precise-reminder action opens Android's special alarm settings. Exact scheduling is used only
when allowed; otherwise inexact alarms may be delayed. Revoking precise-alarm access can cancel alarms
until the app returns to reconcile. Notification denial leaves due work visible without claiming
delivery. Server subscriptions and accepted/unknown Nextcloud delivery remain separate personal state.

Stable opaque IDs include source, step generation and event. The encrypted projection contains only
bounded absolute deadlines, language and expiry; notification text is generic/private. Replan,
completion, opt-out and scope closure remove obsolete events. Boot/time changes restore the same
instants; an attempt is saved before notification dispatch to avoid blind duplicates. Attempted is
not proof of display. Opening a notification only opens the app, never acknowledges physical work.

The next 200 events within the seven-day authorization horizon are scheduled; a visible limitation
requires returning online for later actions. Missed events older than 24 hours are not sent as fresh
alarms. Force-stop, denied permission, Doze, reboot/unlock and manufacturer policies can delay/prevent
delivery. No mobile push provider, exact delivery guarantee or iOS reminder support is claimed.

Primary platform sources: [Android alarms](https://developer.android.com/develop/background-work/services/alarms),
[notification permission](https://developer.android.com/develop/ui/compose/notifications/notification-permission),
[elapsed realtime](https://developer.android.com/reference/android/os/SystemClock#elapsedRealtime()).

## Fermentation, agenda and actual cooking history

More → Brewing includes source-linked beer, cider, wine and mead templates, exact recipe quantities,
explicit unknown durations, original recipe language, equipment capacities and immutable revisions.
Batches pin recipe/equipment snapshots. Every mutation has a canonical before/after review; current
recipe/equipment changes require a fresh review before a new immutable envelope is created. Confirmed
brew requests use the existing brewing receipts through the same protected cooking queue; a lost
response is retried with the original key/body. Starting a brew timer uses server confirmation time;
these reviewed writes require connectivity. No offline instrument connector or chemistry conversion
is implied. Readings retain exact original units, measured civil time/fold, source/acquisition,
calibration or unknown calibration. Corrections preserve the original and require a reason. Archived
sources remain pinned and readable; batch equipment reservations continue to use the shared scheduler.

Today and Plan link pinned meals/preparations to cooking with their actual production servings and
source slot. The additive reader-authorized `GET /api/household/cooking/calendar` accepts Unix-second
`from`, exclusive `until` (up to seven days), `offset` and `limit` (1–200, default 100). It returns
`revision`, `server_now`, `total`, `offset`, `limit` and `entries`. Each entry identifies run/step, food
or brew kind, planned `start` or actual running `deadline`, UTC `at`, resolved `local`, IANA timezone
and optional plan slot. Completed/cancelled/archived processes, paused timers and untimed manual
phases have no fake deadlines. Ordering is stable by instant/run/step. Native day views select each
process's civil date and retain paging even when neighbouring time zones occupy a page.

More → Cooking history records explicit completed-session or manual cooks, attendees, optional actual
effort and 1–5 household/profile ratings. All household readers can see these reports; they are not
private authenticated votes. Blank effort/ratings stay unknown. A completed session is linked only
once while its history exists. Corrections, removed profiles, pinned recipe links and deletion use
the existing F11 API. History writes remain online: an ambiguous response blocks automatic resubmit
and asks the user to inspect saved history. They are not placed in an unreceipted offline outbox.
Owner-only learning scope/reset and bulk deletion are separate confirmed actions. Retained old ratings
are never silently replayed into a new learning period. Physical consumption/output still requires
explicit inventory review; neither a completed timer nor a history record moves stock.

Snapshot availability and local alarms expire after seven authorized days. Frozen uncertain request
envelopes survive expiry and wall-clock edits until exact reconciliation, safe explicit discard,
access removal or sign-out. Expired private snapshots are cleared; this bounded recovery envelope is
not extended offline authorization. Damaged recovery data blocks actions without overwriting the
stored unconfirmed requests. Native device reminders pause until recovery is resolved.
