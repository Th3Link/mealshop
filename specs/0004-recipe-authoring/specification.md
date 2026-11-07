# Recipe authoring and scaling

Status: Draft
Priority: Must
Origin: Customer
Milestone: M1

## User value
Create structured recipes that remain useful when portions change.

## Acceptance criteria
1. Given a recipe, an author can edit title, ingredients from the catalogue, quantities, serving count, instructions and active/passive durations.
2. Given a new serving count, quantities and per-serving nutrition update while the original yield remains recorded; invalid or zero servings are rejected.
3. Given ingredient substitutions or unknown ingredients, identity and nutrition gaps are visible instead of silently matched.
4. Given a saved revision, its ingredients, steps, source, author and calculation inputs can be retrieved consistently.

## Out of scope
Paid content delivery and external imports.

## UX notes
UI flows, accessibility, empty/error states and user review points must be specified before Ready.

## Links
Requirement: [F04](../../docs/requirements.md). Delivery tasks: [plan.md](plan.md). Decisions: [adrs.md](adrs.md).
