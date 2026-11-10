# Guided cooking and preset timers

Status: Done
Priority: Must
Origin: Customer
Milestone: M2

## User value
Follow recipe steps without repeatedly setting timers.

## Acceptance criteria
1. Given a timed step, one action starts a correctly preset, named timer linked to that recipe instance.
2. Given concurrent steps, independent timers and phase reminders show the relevant next action and allow pause/reschedule/complete.
3. Given app/server restart or an offline interval, persisted deadlines recover without silently restarting the duration.
4. Given denied notifications or an unreachable device, the app displays the delivery state and upcoming actions when reopened.

## Out of scope
Guaranteed alarm delivery by a sleeping mobile operating system.

## UX notes
UI flows, accessibility, empty/error states and user review points must be specified before Ready.

## Links
Requirement: [F12](../../docs/requirements.md). Delivery tasks: [plan.md](plan.md). Decisions: [adrs.md](adrs.md).

Acceptance evidence: [validation.md](validation.md).
