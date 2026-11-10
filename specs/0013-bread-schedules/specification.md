# Multi-day bread, rolls and pizza scheduling

Status: Done
Priority: Should
Origin: Customer
Milestone: M3

## User value
Prepare long-fermentation dough around a target serving time.

## Acceptance criteria
1. Given bread/roll/pizza/baguette steps across several days, dependencies and active/passive time windows produce a reviewable schedule.
2. Given preheating, stretch-and-fold cycles and fermentation stages, reminders name the action and required equipment before it is due.
3. Given a delayed step or changed bake time, dependent reminders are recalculated with a preview and temperature/fermentation uncertainty is explicit.
4. Given a multi-day schedule crossing a daylight-saving transition, local-time presentation and stored deadlines remain consistent.

## Out of scope
Guaranteed fermentation outcomes or copying a third-party recipe collection.

## UX notes
UI flows, accessibility, empty/error states and user review points must be specified before Ready.

## Links
Requirement: [F13](../../docs/requirements.md). Delivery tasks: [plan.md](plan.md). Decisions: [adrs.md](adrs.md).

Acceptance evidence: [validation.md](validation.md).
