# Family meal planning and calendar attendance

Status: Done
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
See ux-notes.md.

## Links
Requirement: [F10](../../docs/requirements.md). Delivery tasks: [plan.md](plan.md). Decisions: [adrs.md](adrs.md).

## Implementation contract
Local protected planning uses F09 operator/reader roles and immutable recipe revisions. Calendar
attendance is supplied through a consented read-only snapshot import with source/date/event and
mapped household IDs; no specific live provider is implied. Preparation is estimated from ordered
recipe steps; scaled batch-time uncertainty is shown for review. Preview/apply revisions prevent
stale profile/plan decisions, and known hard/time conflicts cannot be acknowledged away.
Shopping changes are plan-derived mass requirements, not completion of F06 purchase workflows.
Unknown allergen data remains review-required. Hosted provider adapters and process scheduling
belong to their own features; F10 does not autonomously write external calendars or make purchases.
