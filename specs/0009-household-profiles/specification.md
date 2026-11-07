# Household members and dietary constraints

Status: In progress
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
See ux-notes.md.

## Links
Requirement: [F09](../../docs/requirements.md). Delivery tasks: [plan.md](plan.md). Decisions: [adrs.md](adrs.md).

## Current slice
Protected local single-household profiles with operator-token access, revisioned snapshot
correction/export/removal and group dietary review. Allergy/intolerance text requires human
review; only explicit ingredient exclusions are automatically enforced. Missing allergen
coverage always returns `review_required` rather than safe. Hosted identities are out of scope.
