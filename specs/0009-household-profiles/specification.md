# Household members and dietary constraints

Status: Done
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

## Completion scope
Protected local single-household profiles with operator and optional read-only credentials,
revision-aware correction/export/removal, structured hard allergen groups, separate intolerance
and allergy notes, explicit ingredient exclusions and soft dislikes. Positive food-identity
classifications apply only to matching catalogue records/states. Group evaluation unions all
attendees' hard constraints and known ingredient aliases. Unknown or incomplete allergen data
always requires review; no result establishes cross-contamination safety. Learning has no
write path to profiles. Hosted accounts/roles remain a separate deployment feature.

Completed evidence is recorded in validation.md. Data coverage can grow independently of this
feature; absent classifications never mean allergen absence.
