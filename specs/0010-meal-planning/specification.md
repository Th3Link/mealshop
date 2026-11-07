# Family meal planning and calendar attendance

Status: Draft
Priority: Must
Origin: Customer
Milestone: M2

## User value
Choose feasible meals for the family and adapt when plans change.

## Acceptance criteria
1. Given a day/meal and attendees, a planner can select recipes manually or request constraint-compatible suggestions.
2. Given consented calendar access, attendance is proposed with a visible source and can be overridden without editing the external calendar.
3. Given per-day preparation windows, suggestions distinguish active work, total elapsed time and advance preparation.
4. Given a last-minute attendance/time change, the user can choose an alternative, rescale servings and preview shopping-list changes before applying them.
5. Given no feasible meal, the planner explains the conflicting constraints instead of silently violating intolerances.

## Out of scope
Autonomous calendar writes or purchases.

## UX notes
UI flows, accessibility, empty/error states and user review points must be specified before Ready.

## Links
Requirement: [F10](../../docs/requirements.md). Delivery tasks: [plan.md](plan.md). Decisions: [adrs.md](adrs.md).
