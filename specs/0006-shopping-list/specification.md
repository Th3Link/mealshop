# Shopping lists and recipe provenance

Status: Draft
Priority: Must
Origin: Customer
Milestone: M2

## User value
Shop from meal plans while keeping track of why each item is needed.

## Acceptance criteria
1. Given selected recipes and serving counts, a list aggregates compatible ingredient quantities without losing contributing recipe/plan references.
2. Given a listed item, the household can inspect contributing recipes and quantities; it can also add unrelated groceries or household goods.
3. Given removal or rescaling of a recipe, only its unpurchased contribution changes and manually added contributions remain.
4. Given shopping mode, large touch targets support fast checking/unchecking in a Bring-inspired interaction with a distinct visual design.

## Out of scope
Automatic shop ordering and barcode lookup, specified separately.

## UX notes
UI flows, accessibility, empty/error states and user review points must be specified before Ready.

## Links
Requirement: [F06](../../docs/requirements.md). Delivery tasks: [plan.md](plan.md). Decisions: [adrs.md](adrs.md).
