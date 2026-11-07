# Cooking history and online preference learning

Status: Draft
Priority: Should
Origin: Customer
Milestone: M2

## User value
Improve suggestions from real cooking and family feedback.

## Acceptance criteria
1. Given a completed cook, history records recipe revision, attendees, date, actual effort and individual/household rating without counting merely planned meals.
2. Given new ratings or repeated cooking, suggestions update incrementally and explain relevant history, constraints and confidence.
3. Given cold-start users, transparent rules provide usable suggestions; new recipes remain discoverable rather than being excluded by sparse feedback.
4. Given model changes, evaluation compares constraint violations, acceptance, variety and user satisfaction against the baseline.
5. Given a reset or opt-out, learned preference state can be removed and suggestions fall back to rules; hard exclusions remain enforced.

## Out of scope
Mandatory cloud training or a chosen ML framework before evaluation.

## UX notes
UI flows, accessibility, empty/error states and user review points must be specified before Ready.

## Links
Requirement: [F11](../../docs/requirements.md). Delivery tasks: [plan.md](plan.md). Decisions: [adrs.md](adrs.md).
