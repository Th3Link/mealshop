# ADR 0030: Native cooking recovery and Android reminders

Status: Accepted for the customer-authorized F35 P4 slice, 2026-11-14.

## Context

P3 is merged in PR #48. P4 delivers cooking, multi-day baking, fermentation and feedback in one
phone/backend PR. Rust already owns scheduling, civil-time/DST resolution and explicit actual-action
waits. Brewing has durable actor-bound operation receipts; food cooking has only expiring in-memory
schedule previews and revision guards. A phone must not retry an uncertain timer action as a new
action or restart a deadline from its current wall clock.

## Alternatives

- Keep cooking online/read-only after a failed response: simple, but cannot reconcile retained
  disconnected actual actions and does not fulfill the P4 recovery contract.
- Build another scheduler/sync engine in Dart: duplicates constraints and creates conflicting truth.
- Add a notification plugin: useful for broader platform distribution, but the accepted first device
  is Android and the existing native vault/channel already provides the required platform boundary.

## Decision

Extend existing cooking mutations with an optional opaque idempotency key and optional reported
timer-action instant. Store validated actor/body/revision/result receipts atomically with the action
inside the current private 4 MiB cooking file; bound receipt capacity and never prune retry evidence
silently. Replay still requires current write authority. Existing callers retain their old contract.
Only timer actions can carry reported instants, bounded to the retained seven-day offline window;
server scheduling/transition checks remain canonical. Distinguish reported action time from entry time.

The native client uses the existing encrypted account/household-scoped storage and seven-day retention
boundary for a bounded cooking read snapshot and frozen timer intents. Save before sending. Pending,
unknown, rejected and confirmed states remain distinct. Reconcile current state; do not silently rebase
confirmed/attempted requests. Recipe step ingredient scope is provided by the server, including explicit
unknown step assignment. Imported text remains household-private and is not newly exported/shared.

Use a focused Android framework adapter for notification permission, channels, `AlarmManager`, stable
immutable pending intents and boot/time-change receivers. No new package/crate. Schedule absolute server
deadlines, not new durations. Default off, generic private notification text, protected bounded schedule
projection, explicit cancellation on opt-out/scope closure. Local notification attempts, server
`AppAPI` delivery and acknowledged physical work are separate. The separate optional Android precise-alarm settings action enables exact alarms only after OS
authorization; otherwise inexact alarms may be delayed. Revoking precise-alarm access can cancel
alarms until foreground reconciliation. No alarm
delivery guarantee, physical readiness, stock movement or medical outcome follows from elapsed time.
Use elapsed-realtime/boot identity plus a server-clock anchor to detect wall-clock changes; after an
unverified reboot, refresh before recording offline actions. No timer starts from a notification.

## Consequences

The shared Rust scheduler and existing brewing/history rules remain authoritative. New state defaults
empty for old snapshots; populated P4 files require a complete validated migration to downgrade.
Offline remote revocation cannot erase disconnected copies or alarms before revalidation/retention;
force-stop, denied permission, Doze and manufacturer policies can delay/prevent reminders. iOS,
APNs/FCM, production signing and physical-handset evidence stay explicit separate limits. All new
flows need executable tests and synthetic Nextcloud/emulator evidence before the PR becomes ready.

## Primary platform references

- [Android alarm scheduling and reboot](https://developer.android.com/develop/background-work/services/alarms)
- [Android notification permission](https://developer.android.com/develop/ui/compose/notifications/notification-permission)
- [Android elapsed realtime](https://developer.android.com/reference/android/os/SystemClock#elapsedRealtime())
