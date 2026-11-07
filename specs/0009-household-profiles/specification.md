# Household members and dietary constraints

Status: Draft
Priority: Must
Origin: Customer
Milestone: M1

## User value
Plan meals safely for everyone eating.

## Acceptance criteria
1. Given a household member, intolerances/allergies and disliked ingredients are recorded separately with explicit access controls.
2. Given a group meal, hard exclusions cover all attendees and known ingredient aliases; a dislike is a soft preference.
3. Given missing ingredient or allergen data, a candidate is flagged for review rather than declared safe.
4. Given member data, authorized people can correct, export and delete it; learning never relaxes a hard exclusion.

## Out of scope
Diagnosis or guaranteed absence of cross-contamination.

## UX notes
UI flows, accessibility, empty/error states and user review points must be specified before Ready.

## Links
Requirement: [F09](../../docs/requirements.md). Delivery tasks: [plan.md](plan.md). Decisions: [adrs.md](adrs.md).
